//! Core shared type definitions.

use serde::{Deserialize, Serialize};

/// Severity level 1–5 (Constitution Part V — Rubric).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Level {
    Info = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    Critical = 5,
}

impl Level {
    pub fn label(&self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Low => "LOW",
            Level::Medium => "MEDIUM",
            Level::High => "HIGH",
            Level::Critical => "CRITICAL",
        }
    }
}

/// Recurrence bucket I–VI (Constitution Part VI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Recurrence {
    I = 1,
    II = 2,
    III = 3,
    IV = 4,
    V = 5,
    VI = 6,
}

impl Recurrence {
    pub fn from_count(count: f64) -> Self {
        match count {
            c if c < 1.0 => Self::I,
            c if c < 2.0 => Self::II,
            c if c < 4.0 => Self::III,
            c if c < 7.0 => Self::IV,
            c if c < 12.0 => Self::V,
            _ => Self::VI,
        }
    }
}

/// Enforcement action (Constitution Article 5, Part VII — Sentencing Grid).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Enforcement {
    Dismiss,
    Note,
    Advisory,
    Caution,
    Warn,
    Strike,
    Seal,
}

impl Enforcement {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Warn | Self::Strike | Self::Seal)
    }

    pub fn label_ko(&self) -> &'static str {
        match self {
            Self::Dismiss => "기각",
            Self::Note => "기록",
            Self::Advisory => "권고",
            Self::Caution => "주의",
            Self::Warn => "경고",
            Self::Strike => "차단",
            Self::Seal => "봉인",
        }
    }
}

/// Failure/lesson category (Constitution Part VI.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Security,
    Correctness,
    Process,
    DataSafety,
    Temporal,
}

/// Composite identity hash spanning tier 1 (SHA1), tier 2 (embedding),
/// and an optional tier 3 LLM-resolved flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityHash {
    pub tier1_sha1: [u8; 20],
    // tier 2 embedding is persisted out-of-band (vectors.bin) and the DB
    // only stores tier1. These fields are runtime-only composition; skip
    // them from serde to avoid the [f32; 384] size limit on `serde` derive.
    #[serde(skip)]
    pub tier2_embedding: Option<[f32; 384]>,
    #[serde(skip)]
    pub tier3_resolved: Option<bool>,
}

impl IdentityHash {
    pub fn tier1_hex(&self) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(40);
        for b in &self.tier1_sha1 {
            write!(s, "{:02x}", b).unwrap();
        }
        s
    }
}
