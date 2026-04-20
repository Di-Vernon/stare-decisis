//! Advisory file lock used to serialise daemon spawn attempts.
//!
//! The lock file lives next to the socket in
//! `$XDG_RUNTIME_DIR/myth/embed.lock`. Holding an exclusive flock on
//! that file is the contract for "I am the daemon". Dropping the
//! `DaemonLock` releases it (via File drop).

use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use anyhow::Context;
use fs2::FileExt;

pub struct DaemonLock {
    _file: File,
}

fn lock_path() -> PathBuf {
    myth_common::myth_runtime().join("embed.lock")
}

/// Try to acquire the daemon lock without blocking. Returns:
///   `Ok(Some(lock))`  — we hold it, caller can proceed as the daemon
///   `Ok(None)`        — another process holds it (not us)
///   `Err(e)`          — IO failure opening/creating the lock file
pub fn try_acquire() -> anyhow::Result<Option<DaemonLock>> {
    let path = lock_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("creating runtime dir")?;
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("opening {:?}", path))?;

    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(DaemonLock { _file: file })),
        Err(_) => Ok(None),
    }
}

/// Block until the daemon lock is ours. Used in tests or when we're
/// sure no other daemon is (or should be) running.
pub fn acquire_blocking() -> anyhow::Result<DaemonLock> {
    let path = lock_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("creating runtime dir")?;
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("opening {:?}", path))?;
    file.lock_exclusive().context("lock_exclusive")?;
    Ok(DaemonLock { _file: file })
}
