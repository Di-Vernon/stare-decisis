//! In-memory vector store with on-disk `vectors.bin` persistence.
//!
//! File layout (MYEV magic, 07-STATE.md §벡터 바이너리):
//!
//!   offset  size  field
//!   ------  ----  ---------------------------------
//!   0x00    4     Magic: b"MYEV"
//!   0x04    2     Version: u16 LE = 1
//!   0x06    2     Dimension: u16 LE = 384
//!   0x08    4     Count: u32 LE
//!   0x0C    8     Generation: u64 LE
//!   0x14    12    Reserved (zero-filled)
//!   0x20    ...   Count * 384 * 4 bytes (f32 LE, row-major)
//!
//! Day-1 interim: vectors are kept as `Vec<[f32; 384]>` instead of
//! live mmap. The file is still written atomically (tmp + rename) on
//! every mutation so crash recovery works. mmap switch is a Milestone
//! B optimisation and has no effect on the trait surface.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use anyhow::{anyhow, Context};
use myth_common::LessonId;

use super::{Embedding, IntegrityReport, VectorStore, EMBEDDING_DIM};

const MAGIC: [u8; 4] = *b"MYEV";
const HEADER_VERSION: u16 = 1;
const HEADER_SIZE: usize = 0x20;

pub struct InMemoryStore {
    path: PathBuf,
    inner: RwLock<Inner>,
}

struct Inner {
    vectors: Vec<Embedding>,
    id_to_row: HashMap<LessonId, usize>,
    generation: u64,
}

impl InMemoryStore {
    /// Create a fresh store at `path`, overwriting any existing file
    /// with an empty vectors.bin.
    pub fn create(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let store = Self {
            path: path.clone(),
            inner: RwLock::new(Inner {
                vectors: Vec::new(),
                id_to_row: HashMap::new(),
                generation: 0,
            }),
        };
        write_file(&store.path, &[], 0).context("writing empty vectors.bin")?;
        Ok(store)
    }

    /// Open an existing store. `id_to_row` starts empty — callers are
    /// expected to hydrate the mapping from the `vector_metadata`
    /// table (see `load_index`).
    pub fn open(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let (vectors, generation) = read_file(&path)
            .with_context(|| format!("reading {:?}", path))?;
        Ok(Self {
            path,
            inner: RwLock::new(Inner {
                vectors,
                id_to_row: HashMap::new(),
                generation,
            }),
        })
    }

    /// Replace the id→row index (for example, after hydrating from
    /// `state.db::vector_metadata`).
    pub fn load_index(&self, index: HashMap<LessonId, usize>) -> anyhow::Result<()> {
        let mut inner = self.inner.write().map_err(|_| anyhow!("rwlock poisoned"))?;
        inner.id_to_row = index;
        Ok(())
    }

    pub fn generation(&self) -> u64 {
        self.inner.read().map(|i| i.generation).unwrap_or(0)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl VectorStore for InMemoryStore {
    fn upsert(&self, id: LessonId, vec: &Embedding) -> anyhow::Result<()> {
        let mut inner = self.inner.write().map_err(|_| anyhow!("rwlock poisoned"))?;
        let new_gen = inner.generation + 1;
        match inner.id_to_row.get(&id).copied() {
            Some(row) => {
                inner.vectors[row] = *vec;
            }
            None => {
                let row = inner.vectors.len();
                inner.vectors.push(*vec);
                inner.id_to_row.insert(id, row);
            }
        }
        inner.generation = new_gen;
        write_file(&self.path, &inner.vectors, inner.generation)
            .context("persisting vectors.bin")?;
        Ok(())
    }

    fn knn(&self, query: &Embedding, k: usize) -> anyhow::Result<Vec<(LessonId, f32)>> {
        let inner = self.inner.read().map_err(|_| anyhow!("rwlock poisoned"))?;

        // Build a row→id reverse lookup once per query.
        let mut row_to_id: Vec<Option<LessonId>> = vec![None; inner.vectors.len()];
        for (id, &row) in &inner.id_to_row {
            if row < row_to_id.len() {
                row_to_id[row] = Some(*id);
            }
        }

        let mut scored: Vec<(LessonId, f32)> = inner
            .vectors
            .iter()
            .enumerate()
            .filter_map(|(row, v)| {
                let id = row_to_id[row]?;
                Some((id, cosine_distance(query, v)))
            })
            .collect();
        scored.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        Ok(scored)
    }

    fn delete(&self, id: LessonId) -> anyhow::Result<()> {
        let mut inner = self.inner.write().map_err(|_| anyhow!("rwlock poisoned"))?;
        let Some(row) = inner.id_to_row.remove(&id) else {
            return Ok(());
        };

        let last = inner.vectors.len() - 1;
        if row != last {
            inner.vectors.swap(row, last);
            // Any id that was pointing at `last` now points at `row`.
            let moved = inner
                .id_to_row
                .iter()
                .find_map(|(other_id, &other_row)| {
                    if other_row == last {
                        Some(*other_id)
                    } else {
                        None
                    }
                });
            if let Some(mid) = moved {
                inner.id_to_row.insert(mid, row);
            }
        }
        inner.vectors.pop();
        inner.generation += 1;
        write_file(&self.path, &inner.vectors, inner.generation)
            .context("persisting vectors.bin after delete")?;
        Ok(())
    }

    fn len(&self) -> usize {
        self.inner.read().map(|i| i.vectors.len()).unwrap_or(0)
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn integrity_check(&self) -> anyhow::Result<IntegrityReport> {
        let inner = self.inner.read().map_err(|_| anyhow!("rwlock poisoned"))?;
        let total_vectors = inner.vectors.len();

        let index_consistent = inner
            .id_to_row
            .values()
            .all(|&r| r < total_vectors);

        let (_, file_gen) = read_file(&self.path).unwrap_or_else(|_| (Vec::new(), 0));
        let generation_match = file_gen == inner.generation;

        // multilingual-e5-small outputs normalised vectors — any norm
        // far from 1.0 is suspicious. We allow 0.9..=1.1 to absorb
        // f32 rounding.
        let norm_anomalies = inner
            .vectors
            .iter()
            .filter(|v| {
                let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                !(0.9..=1.1).contains(&n)
            })
            .count();

        Ok(IntegrityReport {
            total_vectors,
            index_consistent,
            generation_match,
            norm_anomalies,
        })
    }
}

fn cosine_distance(a: &Embedding, b: &Embedding) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 2.0;
    }
    1.0 - (dot / (na * nb))
}

fn write_file(path: &Path, vectors: &[Embedding], generation: u64) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let tmp = path.with_extension("bin.tmp");
    {
        let mut file = File::create(&tmp).with_context(|| format!("creating {:?}", tmp))?;
        file.write_all(&MAGIC)?;
        file.write_all(&HEADER_VERSION.to_le_bytes())?;
        file.write_all(&(EMBEDDING_DIM as u16).to_le_bytes())?;
        file.write_all(&(vectors.len() as u32).to_le_bytes())?;
        file.write_all(&generation.to_le_bytes())?;
        file.write_all(&[0u8; 12])?; // reserved
        for v in vectors {
            for f in v.iter() {
                file.write_all(&f.to_le_bytes())?;
            }
        }
        file.sync_all().context("fsync tmp")?;
    }
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {:?} -> {:?}", tmp, path))?;
    Ok(())
}

fn read_file(path: &Path) -> anyhow::Result<(Vec<Embedding>, u64)> {
    let mut file = File::open(path).with_context(|| format!("opening {:?}", path))?;
    let mut header = [0u8; HEADER_SIZE];
    file.read_exact(&mut header).context("reading header")?;

    if header[0..4] != MAGIC {
        return Err(anyhow!("invalid magic bytes in {:?}", path));
    }
    let version = u16::from_le_bytes([header[4], header[5]]);
    if version != HEADER_VERSION {
        return Err(anyhow!("unsupported vectors.bin version: {}", version));
    }
    let dim = u16::from_le_bytes([header[6], header[7]]) as usize;
    if dim != EMBEDDING_DIM {
        return Err(anyhow!(
            "dimension mismatch: expected {}, got {}",
            EMBEDDING_DIM,
            dim
        ));
    }
    let count = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as usize;
    let generation = u64::from_le_bytes([
        header[12], header[13], header[14], header[15], header[16], header[17], header[18],
        header[19],
    ]);

    let expected_bytes = count * EMBEDDING_DIM * 4;
    let mut buf = vec![0u8; expected_bytes];
    file.read_exact(&mut buf).context("reading vector payload")?;

    let mut vectors = Vec::with_capacity(count);
    for i in 0..count {
        let mut v = [0f32; EMBEDDING_DIM];
        let row = &buf[i * EMBEDDING_DIM * 4..(i + 1) * EMBEDDING_DIM * 4];
        for (slot, chunk) in v.iter_mut().zip(row.chunks_exact(4)) {
            *slot = f32::from_le_bytes(chunk.try_into().expect("4-byte chunk"));
        }
        vectors.push(v);
    }
    Ok((vectors, generation))
}
