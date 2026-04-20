//! Daemon entry point — `--daemon` dispatch target in `main.rs`.

pub mod idle;
pub mod model;
pub mod server;
pub mod stats;

use std::os::unix::fs::PermissionsExt;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context};
use tokio::net::UnixListener;

use crate::daemon::idle::IdleTracker;
use crate::daemon::model::Model;
use crate::daemon::stats::Stats;
use crate::lock;

pub const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

pub async fn run() -> anyhow::Result<ExitCode> {
    let _lock = lock::try_acquire()?
        .ok_or_else(|| anyhow!("another myth-embed daemon is already holding the flock"))?;

    let socket_path = myth_common::embed_socket_path();
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).context("creating runtime dir")?;
        let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
    }
    // Remove any stale socket left by a previous crashed instance.
    let _ = std::fs::remove_file(&socket_path);

    tracing::info!("loading multilingual-e5-small");
    let model = Model::load().await.context("loading embedding model")?;
    tracing::info!("model loaded");

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("binding unix socket at {:?}", socket_path))?;
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
        .context("setting socket 0600 permissions")?;

    tracing::info!(socket = ?socket_path, "myth-embed listening");

    let stats = Arc::new(Stats::new());
    let idle = Arc::new(IdleTracker::new(IDLE_TIMEOUT));
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _)) => {
                        let model = model.clone();
                        let stats = stats.clone();
                        let idle = idle.clone();
                        let tx = shutdown_tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = server::handle_client(stream, model, stats, idle, tx).await {
                                tracing::error!(error = %e, "client handler error");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "accept failed");
                    }
                }
            }
            _ = idle.wait_for_timeout() => {
                tracing::info!("idle timeout — shutting down");
                break;
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("shutdown signal — exiting");
                break;
            }
        }
    }

    drop(listener);
    let _ = std::fs::remove_file(&socket_path);
    Ok(ExitCode::SUCCESS)
}
