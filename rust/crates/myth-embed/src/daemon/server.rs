//! Per-connection request handler — reads one Request, writes one
//! Response, closes.

use std::sync::Arc;

use tokio::net::UnixStream;
use tokio::sync::mpsc;

use crate::daemon::idle::IdleTracker;
use crate::daemon::model::Model;
use crate::daemon::stats::{self, Stats};
use crate::protocol::{
    read_message, write_message, ErrorCode, Op, OpResult, Request, Response,
    MAX_PAYLOAD_BYTES, PROTOCOL_VERSION,
};

pub async fn handle_client(
    mut stream: UnixStream,
    model: Arc<Model>,
    stats: Arc<Stats>,
    idle: Arc<IdleTracker>,
    shutdown_tx: mpsc::Sender<()>,
) -> anyhow::Result<()> {
    let req: Request = read_message(&mut stream).await?;
    idle.bump();

    if req.version != PROTOCOL_VERSION {
        let resp = Response {
            version: PROTOCOL_VERSION,
            id: req.id,
            result: OpResult::Error {
                code: ErrorCode::InvalidVersion as u16,
                message: format!("unsupported protocol version: {}", req.version),
            },
        };
        write_message(&mut stream, &resp).await?;
        return Ok(());
    }

    let start = std::time::Instant::now();
    let result = match req.op {
        Op::Embed { text } => {
            if text.len() > MAX_PAYLOAD_BYTES as usize {
                OpResult::Error {
                    code: ErrorCode::TextTooLong as u16,
                    message: format!(
                        "text is {} bytes (cap {})",
                        text.len(),
                        MAX_PAYLOAD_BYTES
                    ),
                }
            } else {
                stats.inc_request();
                match model.embed(&text).await {
                    Ok(vec) => OpResult::Embedded {
                        vector: vec.to_vec(),
                    },
                    Err(e) => OpResult::Error {
                        code: ErrorCode::InternalError as u16,
                        message: format!("{:#}", e),
                    },
                }
            }
        }
        Op::Ping => OpResult::Pong {
            uptime_secs: stats.uptime_secs(),
            request_count: stats.request_count(),
            rss_bytes: stats::rss_bytes(),
            model_name: "multilingual-e5-small".into(),
        },
        Op::Shutdown => {
            let _ = shutdown_tx.send(()).await;
            OpResult::ShuttingDown
        }
    };

    let resp = Response {
        version: PROTOCOL_VERSION,
        id: req.id,
        result,
    };
    write_message(&mut stream, &resp).await?;

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    tracing::debug!(elapsed_ms, "request handled");
    Ok(())
}
