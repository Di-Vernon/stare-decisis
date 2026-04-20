//! Tamper-evident audit log with blake3 Merkle chain.

pub mod entry;
pub mod hasher;

pub use entry::{AuditEntry, AuditEvent};

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Context;

use crate::jsonl::JsonlWriter;

pub struct AuditLog {
    writer: JsonlWriter,
    last_hash: Mutex<[u8; 32]>,
}

impl AuditLog {
    /// Open (or initialise) an audit log file. Reads any existing entries
    /// to resume the chain; otherwise seeds `last_hash` to all zeros.
    pub fn open(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let writer = JsonlWriter::new(path);
        let last_hash = if writer.path().exists() {
            let mut last = [0u8; 32];
            for res in writer.iter::<AuditEntry>()? {
                let entry = res?;
                last = entry.hash;
            }
            last
        } else {
            [0u8; 32]
        };
        Ok(Self {
            writer,
            last_hash: Mutex::new(last_hash),
        })
    }

    pub fn append(&self, event: AuditEvent) -> anyhow::Result<AuditEntry> {
        let mut last = self.last_hash.lock().expect("audit mutex poisoned");
        let entry = AuditEntry::new(event, *last);
        self.writer.append(&entry).context("appending audit entry")?;
        *last = entry.hash;
        Ok(entry)
    }

    /// Verify the full chain: each entry's `prev_hash` equals the
    /// preceding entry's `hash`, and each entry's `hash` matches a fresh
    /// blake3 computation over `(ts, event, prev_hash)`.
    pub fn verify(&self) -> anyhow::Result<bool> {
        if !self.writer.path().exists() {
            return Ok(true);
        }
        let mut prev = [0u8; 32];
        for res in self.writer.iter::<AuditEntry>()? {
            let entry = res?;
            if entry.prev_hash != prev {
                return Ok(false);
            }
            if entry.compute_hash() != entry.hash {
                return Ok(false);
            }
            prev = entry.hash;
        }
        Ok(true)
    }
}
