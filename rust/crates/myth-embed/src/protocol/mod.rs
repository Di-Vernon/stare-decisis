//! Wire protocol v1 — see `~/myth/PROTOCOL.md`.
//!
//! This module's wire format is **frozen for myth v1**. Any breaking
//! change requires a v2 migration (new socket path, version bump).
//! Do not modify Request/Response/Op/OpResult structure without that
//! process.

pub mod types;
pub mod wire;

pub use types::{ErrorCode, Op, OpResult, Request, Response};
pub use wire::{read_message, write_message, MAX_PAYLOAD_BYTES, PROTOCOL_VERSION};
