//! Protocol v1 — wire types (see ~/myth/PROTOCOL.md §4).
//!
//! These struct/enum shapes are frozen for myth v1. No field additions,
//! removals, or reorderings without a v2 migration plan.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub version: u8,
    pub id: [u8; 16],
    pub op: Op,
}

impl Request {
    pub fn new(op: Op) -> Self {
        Self {
            version: super::PROTOCOL_VERSION,
            id: *uuid::Uuid::new_v4().as_bytes(),
            op,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Op {
    Embed { text: String },
    Ping,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub version: u8,
    pub id: [u8; 16],
    pub result: OpResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpResult {
    Embedded {
        /// Always length 384 for multilingual-e5-small.
        vector: Vec<f32>,
    },
    Pong {
        uptime_secs: u64,
        request_count: u64,
        rss_bytes: u64,
        model_name: String,
    },
    ShuttingDown,
    Error {
        code: u16,
        message: String,
    },
}

/// Wire error codes. Transported as u16 inside `OpResult::Error` so the
/// protocol stays compatible with non-Rust clients.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Ok = 0,
    InvalidVersion = 1,
    InvalidOperation = 2,
    TextTooLong = 3,
    ModelNotLoaded = 4,
    InternalError = 5,
}
