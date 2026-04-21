//! Claude Code 버전 파싱 및 호환성 검증.

use anyhow::{anyhow, Result};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaudeVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ClaudeVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// `claude --version` 출력에서 세 자리 버전 튜플 추출.
    ///
    /// 허용 포맷 예: `"claude 2.1.109"`, `"claude-code 2.1.109 (extra)"`,
    /// `"2.1.109\n"`.
    pub fn parse(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        for token in trimmed.split_whitespace() {
            if let Some(v) = try_parse_triplet(token) {
                return Ok(v);
            }
        }
        Err(anyhow!(
            "could not parse Claude Code version from: {:?}",
            trimmed
        ))
    }

    pub fn validate_compatible(&self) -> Result<()> {
        if self.major != 2 || self.minor < 1 {
            return Err(anyhow!(
                "Claude Code version {} not supported by myth v1. \
                 Expected 2.1.x or later (with PostToolUseFailure support).",
                self
            ));
        }

        if self.minor == 1 && self.patch < 27 {
            tracing::warn!(
                "Claude Code 2.1.{} < 2.1.27. PostToolUseFailure may not be available. \
                 myth will fall back to PostToolUse for all cases.",
                self.patch
            );
        }

        Ok(())
    }
}

impl fmt::Display for ClaudeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn try_parse_triplet(token: &str) -> Option<ClaudeVersion> {
    let core = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
    let mut parts = core.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    let patch_raw = parts.next()?;
    let patch: u32 = patch_raw
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(ClaudeVersion { major, minor, patch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain() {
        let v = ClaudeVersion::parse("claude 2.1.109").unwrap();
        assert_eq!(v, ClaudeVersion::new(2, 1, 109));
    }

    #[test]
    fn parse_bare_triplet() {
        let v = ClaudeVersion::parse("2.1.27").unwrap();
        assert_eq!(v, ClaudeVersion::new(2, 1, 27));
    }

    #[test]
    fn parse_with_trailing() {
        let v = ClaudeVersion::parse("claude-code 2.1.109 (extra)").unwrap();
        assert_eq!(v, ClaudeVersion::new(2, 1, 109));
    }

    #[test]
    fn parse_fails_on_garbage() {
        assert!(ClaudeVersion::parse("unknown output").is_err());
    }

    #[test]
    fn compatible_min() {
        ClaudeVersion::new(2, 1, 27).validate_compatible().unwrap();
    }

    #[test]
    fn compatible_with_warn() {
        // < 2.1.27: warns but returns Ok
        ClaudeVersion::new(2, 1, 10).validate_compatible().unwrap();
    }

    #[test]
    fn incompatible_major() {
        assert!(ClaudeVersion::new(1, 9, 0).validate_compatible().is_err());
        assert!(ClaudeVersion::new(3, 0, 0).validate_compatible().is_err());
    }

    #[test]
    fn incompatible_minor() {
        assert!(ClaudeVersion::new(2, 0, 99).validate_compatible().is_err());
    }

    #[test]
    fn display() {
        assert_eq!(ClaudeVersion::new(2, 1, 109).to_string(), "2.1.109");
    }
}
