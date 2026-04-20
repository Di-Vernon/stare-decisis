//! myth-common — Layer 0.
//!
//! Shared types, IDs, error, timestamps, XDG paths, and tracing
//! initialisation. No other myth crate is a dependency of this one;
//! every upstream crate may depend on it.

pub mod error;
pub mod ids;
pub mod logging;
pub mod paths;
pub mod time;
pub mod types;

pub use error::{MythError, Result};
pub use ids::{LessonId, ReminderId, SessionId};
pub use paths::*;
pub use time::{format_iso, now, Timestamp};
pub use types::{Category, Enforcement, IdentityHash, Level, Recurrence};
