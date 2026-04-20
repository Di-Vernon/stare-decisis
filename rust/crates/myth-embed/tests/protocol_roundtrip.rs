//! bincode round-trip for every Request/Response variant. Ensures the
//! frozen wire format (PROTOCOL.md §4) serialises and deserialises
//! without data loss.

use myth_embed::{
    protocol::{MAX_PAYLOAD_BYTES, PROTOCOL_VERSION},
    ErrorCode, Op, OpResult, Request, Response,
};

fn roundtrip_request(op: Op) {
    let req = Request {
        version: PROTOCOL_VERSION,
        id: *uuid::Uuid::new_v4().as_bytes(),
        op,
    };
    let bytes = bincode::serialize(&req).expect("serialize");
    let decoded: Request = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(decoded, req);
}

fn roundtrip_response(result: OpResult) {
    let resp = Response {
        version: PROTOCOL_VERSION,
        id: *uuid::Uuid::new_v4().as_bytes(),
        result,
    };
    let bytes = bincode::serialize(&resp).expect("serialize");
    let decoded: Response = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(decoded, resp);
}

#[test]
fn roundtrip_embed_request() {
    roundtrip_request(Op::Embed {
        text: "the quick brown fox".into(),
    });
}

#[test]
fn roundtrip_ping_request() {
    roundtrip_request(Op::Ping);
}

#[test]
fn roundtrip_shutdown_request() {
    roundtrip_request(Op::Shutdown);
}

#[test]
fn roundtrip_embed_response() {
    let vec: Vec<f32> = (0..384).map(|i| i as f32 * 0.01).collect();
    roundtrip_response(OpResult::Embedded { vector: vec });
}

#[test]
fn roundtrip_pong_response() {
    roundtrip_response(OpResult::Pong {
        uptime_secs: 42,
        request_count: 1234,
        rss_bytes: 150 * 1024 * 1024,
        model_name: "multilingual-e5-small".into(),
    });
}

#[test]
fn roundtrip_shutting_down_response() {
    roundtrip_response(OpResult::ShuttingDown);
}

#[test]
fn roundtrip_error_response() {
    roundtrip_response(OpResult::Error {
        code: ErrorCode::TextTooLong as u16,
        message: "payload exceeds 1MB".into(),
    });
}

#[test]
fn empty_string_embed_is_valid() {
    roundtrip_request(Op::Embed {
        text: String::new(),
    });
}

#[test]
fn utf8_text_survives_roundtrip() {
    roundtrip_request(Op::Embed {
        text: "한국어 テスト 中文 🔥".into(),
    });
}

#[test]
fn max_payload_constant_is_one_million() {
    assert_eq!(MAX_PAYLOAD_BYTES, 1_000_000);
}

#[test]
fn protocol_version_constant_is_one() {
    assert_eq!(PROTOCOL_VERSION, 1);
}
