//! Multiple concurrent clients against a single accept loop. Ensures
//! handle-per-connection spawning works under load and individual
//! requests don't interleave on the wire.

use std::sync::Arc;
use std::time::Duration;

use myth_embed::daemon::idle::IdleTracker;
use myth_embed::daemon::stats::Stats;
use myth_embed::protocol::{
    read_message, write_message, Op, OpResult, Request, Response, PROTOCOL_VERSION,
};
use tempfile::tempdir;
use tokio::net::{UnixListener, UnixStream};

#[tokio::test]
async fn ten_clients_all_receive_pong() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("concurrent.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind");

    let stats = Arc::new(Stats::new());
    let idle = Arc::new(IdleTracker::new(Duration::from_secs(60)));

    let stats_clone = stats.clone();
    let idle_clone = idle.clone();
    let server = tokio::spawn(async move {
        let mut handled = 0usize;
        while handled < 10 {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let stats = stats_clone.clone();
            let idle = idle_clone.clone();
            tokio::spawn(async move {
                let req: Request = read_message(&mut stream).await.expect("read");
                idle.bump();
                let result = OpResult::Pong {
                    uptime_secs: stats.uptime_secs(),
                    request_count: stats.request_count(),
                    rss_bytes: 0,
                    model_name: format!("client-{:?}", &req.id[..4]),
                };
                let resp = Response {
                    version: PROTOCOL_VERSION,
                    id: req.id,
                    result,
                };
                write_message(&mut stream, &resp).await.expect("write");
            });
            handled += 1;
        }
    });

    let mut tasks = Vec::new();
    for _ in 0..10 {
        let path = socket_path.clone();
        tasks.push(tokio::spawn(async move {
            let mut c = UnixStream::connect(&path).await.expect("connect");
            let req = Request::new(Op::Ping);
            let id = req.id;
            write_message(&mut c, &req).await.expect("write");
            let resp: Response = read_message(&mut c).await.expect("read");
            assert_eq!(resp.id, id, "each response must echo its own id");
            assert!(matches!(resp.result, OpResult::Pong { .. }));
        }));
    }

    for t in tasks {
        t.await.expect("client task");
    }
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("server exits")
        .expect("server task");
}
