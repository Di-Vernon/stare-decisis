//! Type-safe ID newtypes around `uuid::Uuid`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! impl_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn short(&self) -> String {
                self.0.to_string().chars().take(8).collect()
            }

            pub fn as_bytes(&self) -> &[u8; 16] {
                self.0.as_bytes()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

impl_id!(LessonId);
impl_id!(SessionId);
impl_id!(ReminderId);
