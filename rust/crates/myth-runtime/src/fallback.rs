//! Extra Usage / quota 소진 경로. Day-1 stub.
//!
//! Max quota가 바닥나면 Claude Code stderr에 특정 문구가 나온다.
//! 여기서는 detect_quota_exhausted만 동작하고, 정책 실행은 향후 wave에서
//! ccusage 통합과 함께 확장한다.

use std::time::Duration;

/// stderr 텍스트에 quota 소진 관련 단서가 있는지 확인.
pub fn detect_quota_exhausted(stderr: &str) -> bool {
    let lc = stderr.to_lowercase();
    lc.contains("rate limit")
        || lc.contains("quota exceeded")
        || lc.contains("upgrade to continue")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuotaPolicy {
    Wait,
    UseExtraUsage,
    #[default]
    Abort,
}

#[derive(Debug, Clone)]
pub enum QuotaAction {
    Wait { retry_after: Option<Duration> },
    EnableExtraUsage,
    Abort,
}

/// Day-1 stub: policy 그대로 QuotaAction 반환. ccusage 연동은 향후 wave.
pub fn action_for(policy: QuotaPolicy) -> QuotaAction {
    match policy {
        QuotaPolicy::Wait => QuotaAction::Wait { retry_after: None },
        QuotaPolicy::UseExtraUsage => QuotaAction::EnableExtraUsage,
        QuotaPolicy::Abort => QuotaAction::Abort,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rate_limit() {
        assert!(detect_quota_exhausted("Error: rate limit reached"));
    }

    #[test]
    fn detects_quota_exceeded_case_insensitive() {
        assert!(detect_quota_exhausted("QUOTA EXCEEDED"));
    }

    #[test]
    fn detects_upgrade_msg() {
        assert!(detect_quota_exhausted(
            "You have used all messages. Upgrade to continue."
        ));
    }

    #[test]
    fn does_not_flag_unrelated() {
        assert!(!detect_quota_exhausted("Connection reset by peer"));
        assert!(!detect_quota_exhausted(""));
    }

    #[test]
    fn policy_mapping() {
        assert!(matches!(action_for(QuotaPolicy::Abort), QuotaAction::Abort));
        assert!(matches!(
            action_for(QuotaPolicy::UseExtraUsage),
            QuotaAction::EnableExtraUsage
        ));
        assert!(matches!(
            action_for(QuotaPolicy::Wait),
            QuotaAction::Wait { retry_after: None }
        ));
    }
}
