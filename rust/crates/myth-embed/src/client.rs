//! Embedding client.
//!
//! `EmbedClient` is the primary in-process API for other myth crates
//! (e.g. `myth-identity`'s tier-2 matcher). It wraps a single-shot
//! Unix-socket request/response and auto-spawns the daemon on
//! `ECONNREFUSED` / `ENOENT`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context};
use tokio::net::UnixStream;

use crate::protocol::{
    read_message, write_message, Op, OpResult, Request, Response, PROTOCOL_VERSION,
};
use crate::spawn;

// 30s envelope for cold-start retries. Rationale: the first probe on
// a fresh machine downloads ~465 MB of multilingual-e5-small from
// HuggingFace and loads ONNX; measured ~18.5s on WSL2. Hot-path
// requests never enter this loop (try_once succeeds on first try), so
// the generous envelope only matters for cold boot and for
// MYTH_NO_EMBED_DAEMON bypass paths.
const SPAWN_RETRY_ATTEMPTS: u32 = 300;
const SPAWN_RETRY_INTERVAL: Duration = Duration::from_millis(100);

pub struct EmbedClient {
    socket_path: PathBuf,
}

impl EmbedClient {
    pub fn new() -> Self {
        Self {
            socket_path: myth_common::embed_socket_path(),
        }
    }

    pub fn with_socket_path(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    /// Synchronous embed — spins up a single-thread tokio runtime so
    /// non-async callers (hook binaries) can use it directly.
    pub fn embed(&self, text: &str) -> anyhow::Result<[f32; 384]> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("building tokio runtime")?;
        let path = self.socket_path.clone();
        let text = text.to_string();
        rt.block_on(async move {
            let req = Request::new(Op::Embed { text });
            let resp = query(&path, req).await?;
            vector_from_response(resp)
        })
    }

    pub async fn embed_async(&self, text: &str) -> anyhow::Result<[f32; 384]> {
        let req = Request::new(Op::Embed {
            text: text.to_string(),
        });
        let resp = query(&self.socket_path, req).await?;
        vector_from_response(resp)
    }

    /// Send an arbitrary request (Ping, Shutdown, or Embed).
    pub async fn query_async(&self, req: Request) -> anyhow::Result<Response> {
        query(&self.socket_path, req).await
    }
}

impl Default for EmbedClient {
    fn default() -> Self {
        Self::new()
    }
}

fn vector_from_response(resp: Response) -> anyhow::Result<[f32; 384]> {
    match resp.result {
        OpResult::Embedded { vector } => {
            let len = vector.len();
            <[f32; 384]>::try_from(vector)
                .map_err(|_| anyhow!("expected 384-dim vector, got {}", len))
        }
        OpResult::Error { code, message } => Err(anyhow!("embed error {}: {}", code, message)),
        other => Err(anyhow!("unexpected result variant: {:?}", other)),
    }
}

async fn query(socket_path: &Path, req: Request) -> anyhow::Result<Response> {
    // Attempt 1: the daemon might already be up.
    match try_once(socket_path, &req).await {
        Ok(r) => return Ok(r),
        Err(e) => tracing::debug!(error = %e, "first connect failed — will try to auto-spawn"),
    }

    if std::env::var_os("MYTH_NO_EMBED_DAEMON").is_some() {
        return Err(anyhow!(
            "daemon unreachable and MYTH_NO_EMBED_DAEMON is set"
        ));
    }

    // Attempt 2: self-spawn the daemon and poll the socket.
    spawn::spawn_daemon().context("spawning daemon")?;
    for _ in 0..SPAWN_RETRY_ATTEMPTS {
        tokio::time::sleep(SPAWN_RETRY_INTERVAL).await;
        if let Ok(r) = try_once(socket_path, &req).await {
            return Ok(r);
        }
    }
    Err(anyhow!(
        "daemon did not respond within {}ms after spawn",
        (SPAWN_RETRY_ATTEMPTS * SPAWN_RETRY_INTERVAL.as_millis() as u32)
    ))
}

async fn try_once(socket_path: &Path, req: &Request) -> anyhow::Result<Response> {
    let mut stream = UnixStream::connect(socket_path).await?;
    write_message(&mut stream, req).await?;
    let resp: Response = read_message(&mut stream).await?;
    if resp.version != PROTOCOL_VERSION {
        return Err(anyhow!(
            "response carries unsupported version {} (expected {})",
            resp.version,
            PROTOCOL_VERSION
        ));
    }
    Ok(resp)
}
