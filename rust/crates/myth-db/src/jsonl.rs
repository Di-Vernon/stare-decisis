//! Append-only JSONL writer with advisory file locking.
//!
//! Design: open+append+lock+write+flush+drop for each record. This keeps
//! the writer stateless so different hook binaries can share a file
//! safely — multiple processes coordinating through `fs4::FileExt`
//! (backed by `fcntl` flock on Linux).

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use fs4::fs_std::FileExt;
use serde::{de::DeserializeOwned, Serialize};

pub struct JsonlWriter {
    path: PathBuf,
}

impl JsonlWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append<T: Serialize>(&self, record: &T) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let line = serde_json::to_string(record).context("serializing JSONL record")?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("opening {:?} for append", self.path))?;

        file.lock_exclusive().context("acquiring file lock")?;

        let res = (|| -> anyhow::Result<()> {
            writeln!(file, "{}", line).context("writing JSONL line")?;
            file.flush().context("flushing JSONL line")?;
            Ok(())
        })();

        // Always release the lock, even on write error.
        let _ = FileExt::unlock(&file);
        res
    }

    pub fn iter<T: DeserializeOwned>(
        &self,
    ) -> anyhow::Result<impl Iterator<Item = anyhow::Result<T>>> {
        let file = std::fs::File::open(&self.path)
            .with_context(|| format!("opening {:?} for read", self.path))?;
        let reader = BufReader::new(file);
        Ok(reader.lines().map(|line| {
            let line = line.context("reading JSONL line")?;
            let record: T = serde_json::from_str(&line).context("parsing JSONL record")?;
            Ok(record)
        }))
    }

    pub fn count_lines(&self) -> anyhow::Result<usize> {
        if !self.path.exists() {
            return Ok(0);
        }
        let file = std::fs::File::open(&self.path)
            .with_context(|| format!("opening {:?}", self.path))?;
        Ok(BufReader::new(file).lines().count())
    }
}
