//! myth-embed — self-daemonizing embedding service.
//!
//! Library exports for in-process consumers (primarily `myth-identity`'s
//! tier-2 matcher). The binary (see `main.rs`) reuses the same types to
//! serve the daemon side over a Unix socket.

pub mod client;
pub mod cli;
pub mod daemon;
pub mod lock;
pub mod protocol;
pub mod spawn;

pub use client::EmbedClient;
pub use protocol::{ErrorCode, Op, OpResult, Request, Response};
