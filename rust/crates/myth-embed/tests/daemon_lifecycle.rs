//! End-to-end lifecycle without the real model: accept → serve N
//! requests → Shutdown → server loop exits. Uses an inline accept loop
//! that mimics `daemon::run` but skips model loading.

use std::sync::Arc;
use std::time::Duration;

use myth_embed::daemon::idle::IdleTracker;
use myth_embed::daemon::stats::Stats;
use myth_embed::protocol::{
    read_message, write_message, Op, OpResult, Request, Response, PROTOCOL_VERSION,
};
use tempfile::tempdir;
use tokio::net::{UnixListener, UnixStream};

async fn serve_without_model(
    listener: UnixListener,
    stats: Arc<Stats>,
    idle: Arc<IdleTracker>,
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
    mut shutdown_rx: tokio::sync::mpsc::Receiver<()>,
) {
    loop {
        tokio::select! {
            accept = listener.accept() => {
                let Ok((mut stream, _)) = accept else { continue };
                let stats = stats.clone();
                let idle = idle.clone();
                let tx = shutdown_tx.clone();
                tokio::spawn(async move {
                    let req: Request = match read_message(&mut stream).await {
                        Ok(r) => r,
                        Err(_) => return,
                    };
                    idle.bump();
                    let result = match req.op {
                        Op::Ping => OpResult::Pong {
                            uptime_secs: stats.uptime_secs(),
                            request_count: stats.request_count(),
                            rss_bytes: 0,
                            model_name: "test".into(),
                        },
                        Op::Shutdown => {
                            let _ = tx.send(()).await;
                            OpResult::ShuttingDown
                        }
                        Op::Embed { .. } => OpResult::Error {
                            code: 999,
                            message: "lifecycle test has no model loaded".into(),
                        },
                    };
                    let resp = Response {
                        version: PROTOCOL_VERSION,
                        id: req.id,
                        result,
                    };
                    let _ = write_message(&mut stream, &resp).await;
                });
            }
            _ = shutdown_rx.recv() => break,
        }
    }
}

#[tokio::test]
async fn ping_then_shutdown_cycles_cleanly() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("lifecycle.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind");

    let stats = Arc::new(Stats::new());
    let idle = Arc::new(IdleTracker::new(Duration::from_secs(60)));
    let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    let tx_clone = shutdown_tx.clone();
    let server = tokio::spawn(async move {
        serve_without_model(listener, stats, idle, tx_clone, shutdown_rx).await;
    });

    // 1. Three successive Pings — all succeed.
    for _ in 0..3 {
        let mut c = UnixStream::connect(&socket_path).await.expect("connect");
        let req = Request::new(Op::Ping);
        write_message(&mut c, &req).await.unwrap();
        let resp: Response = read_message(&mut c).await.unwrap();
        assert!(matches!(resp.result, OpResult::Pong { .. }));
    }

    // 2. Shutdown request. Server replies ShuttingDown and exits loop.
    let mut c = UnixStream::connect(&socket_path).await.expect("connect");
    let req = Request::new(Op::Shutdown);
    write_message(&mut c, &req).await.unwrap();
    let resp: Response = read_message(&mut c).await.unwrap();
    assert!(matches!(resp.result, OpResult::ShuttingDown));

    // 3. Server task finishes within 5s.
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("server should exit after Shutdown")
        .expect("server task");
}

#[tokio::test]
async fn idle_bump_prevents_timeout() {
    // Tiny timeout, ensure bump() resets it.
    let idle = IdleTracker::new(Duration::from_millis(200));
    let idle = Arc::new(idle);

    let idle_clone = idle.clone();
    let timeout_task = tokio::spawn(async move {
        idle_clone.wait_for_timeout().await;
    });

    // Bump 3 times over 400ms — each bump should reset the timer.
    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(80)).await;
        idle.bump();
    }

    // Timeout task should still be running after 240ms of bumps.
    assert!(
        !timeout_task.is_finished(),
        "timeout fired despite bumps"
    );

    // Now stop bumping; within 400ms the timeout fires.
    tokio::time::timeout(Duration::from_millis(400), timeout_task)
        .await
        .expect("timeout eventually fires")
        .unwrap();
}
