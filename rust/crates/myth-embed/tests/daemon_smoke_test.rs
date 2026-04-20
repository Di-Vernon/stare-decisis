//! Smoke test — daemon accept loop + Ping/Pong roundtrip without
//! loading the actual embedding model. Ensures the protocol plumbing
//! (bind, accept, read_message, write_message) is sound before we
//! commit further modules to the chain.

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
async fn daemon_responds_to_ping_without_model() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("embed.sock");

    let listener = UnixListener::bind(&socket_path).expect("bind");
    let stats = Arc::new(Stats::new());
    let idle = Arc::new(IdleTracker::new(Duration::from_secs(60)));

    // Background accept: handles a single Ping/Shutdown round without
    // needing a loaded model.
    let stats_clone = stats.clone();
    let idle_clone = idle.clone();
    let accept_task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let req: Request = read_message(&mut stream).await.expect("read request");
        idle_clone.bump();
        let result = match req.op {
            Op::Ping => OpResult::Pong {
                uptime_secs: stats_clone.uptime_secs(),
                request_count: stats_clone.request_count(),
                rss_bytes: 0,
                model_name: "multilingual-e5-small".into(),
            },
            other => OpResult::Error {
                code: 999,
                message: format!("smoke test only handles Ping, got {:?}", other),
            },
        };
        let resp = Response {
            version: PROTOCOL_VERSION,
            id: req.id,
            result,
        };
        write_message(&mut stream, &resp).await.expect("write response");
    });

    // Client: connect + send Ping + read Pong.
    let mut client = UnixStream::connect(&socket_path)
        .await
        .expect("connect");
    let req = Request::new(Op::Ping);
    let expected_id = req.id;
    write_message(&mut client, &req).await.expect("write ping");
    let resp: Response = read_message(&mut client).await.expect("read pong");

    assert_eq!(resp.version, PROTOCOL_VERSION);
    assert_eq!(resp.id, expected_id, "response must echo the request id");
    match resp.result {
        OpResult::Pong {
            model_name,
            rss_bytes,
            ..
        } => {
            assert_eq!(model_name, "multilingual-e5-small");
            assert_eq!(rss_bytes, 0); // smoke test hardcoded value
        }
        other => panic!("expected Pong, got {:?}", other),
    }

    accept_task.await.expect("accept task");
}

#[tokio::test]
async fn version_mismatch_surfaces_as_error() {
    // Construct a Request with a deliberately-wrong version and check
    // that the server-side code path (duplicated inline here) produces
    // an InvalidVersion error. Exercises the protocol-level error
    // envelope without needing the full daemon.
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("bad.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind");

    let accept_task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let req: Request = read_message(&mut stream).await.expect("read");
        // Emulate handle_client's version gate.
        let resp = if req.version != PROTOCOL_VERSION {
            Response {
                version: PROTOCOL_VERSION,
                id: req.id,
                result: OpResult::Error {
                    code: myth_embed::ErrorCode::InvalidVersion as u16,
                    message: format!("unsupported version: {}", req.version),
                },
            }
        } else {
            unreachable!("test only sends wrong version")
        };
        write_message(&mut stream, &resp).await.expect("write");
    });

    let bad_req = Request {
        version: 99,
        id: *uuid::Uuid::new_v4().as_bytes(),
        op: Op::Ping,
    };

    let mut client = UnixStream::connect(&socket_path).await.expect("connect");
    write_message(&mut client, &bad_req).await.expect("write");
    let resp: Response = read_message(&mut client).await.expect("read");

    match resp.result {
        OpResult::Error { code, .. } => {
            assert_eq!(code, myth_embed::ErrorCode::InvalidVersion as u16);
        }
        other => panic!("expected Error InvalidVersion, got {:?}", other),
    }

    accept_task.await.expect("task");
}
