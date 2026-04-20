//! Merkle-chain audit entry.
//!
//! Each entry binds `(ts, event, prev_hash)` through blake3. The
//! `hash` field is excluded from the hash input so the serialised form
//! encodes exactly what the hash covers.

use myth_common::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEvent {
    Genesis,
    BedrockRuleModified {
        rule_id: String,
        user: String,
    },
    FoundationRuleModified {
        rule_id: String,
        user: String,
    },
    AppealGranted {
        lesson_id: String,
        resolver: String,
    },
    AppealDenied {
        lesson_id: String,
        resolver: String,
    },
    GridOverride {
        level: u8,
        recurrence: u8,
        enforcement: String,
        rationale: String,
    },
    RetrialResult {
        lesson_id: String,
        outcome: String,
    },
    LessonInvalidated {
        lesson_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub ts: Timestamp,
    pub event: AuditEvent,
    #[serde(with = "hex_bytes32")]
    pub prev_hash: [u8; 32],
    #[serde(with = "hex_bytes32")]
    pub hash: [u8; 32],
}

#[derive(Serialize)]
struct HashInput<'a> {
    ts: &'a Timestamp,
    event: &'a AuditEvent,
    #[serde(with = "hex_bytes32")]
    prev_hash: &'a [u8; 32],
}

impl AuditEntry {
    pub fn new(event: AuditEvent, prev_hash: [u8; 32]) -> Self {
        let ts = myth_common::now();
        let input = HashInput {
            ts: &ts,
            event: &event,
            prev_hash: &prev_hash,
        };
        let bytes = serde_json::to_vec(&input).expect("audit input serializable");
        let hash = super::hasher::hash_bytes(&bytes);
        Self {
            ts,
            event,
            prev_hash,
            hash,
        }
    }

    /// Recompute the hash covering `(ts, event, prev_hash)`. Used by
    /// `AuditLog::verify` to detect tampering.
    pub fn compute_hash(&self) -> [u8; 32] {
        let input = HashInput {
            ts: &self.ts,
            event: &self.event,
            prev_hash: &self.prev_hash,
        };
        let bytes = serde_json::to_vec(&input).expect("audit input serializable");
        super::hasher::hash_bytes(&bytes)
    }
}

mod hex_bytes32 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use std::fmt::Write;
        let mut s = String::with_capacity(64);
        for b in bytes {
            write!(s, "{:02x}", b).unwrap();
        }
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() != 64 {
            return Err(serde::de::Error::custom(format!(
                "expected 64 hex chars, got {}",
                s.len()
            )));
        }
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
                .map_err(serde::de::Error::custom)?;
        }
        Ok(out)
    }
}
