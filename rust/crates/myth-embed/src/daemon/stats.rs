//! Process statistics — request counter, uptime, RSS.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct Stats {
    start: Instant,
    requests: AtomicU64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            requests: AtomicU64::new(0),
        }
    }

    pub fn inc_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    pub fn request_count(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

/// Current process RSS in bytes via `/proc/self/status` (Linux).
/// Returns 0 on non-Linux or parse failure — Pong is observational only.
pub fn rss_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        let status = match std::fs::read_to_string("/proc/self/status") {
            Ok(s) => s,
            Err(_) => return 0,
        };
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb_str = rest.split_whitespace().next().unwrap_or("0");
                if let Ok(kb) = kb_str.parse::<u64>() {
                    return kb * 1024;
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}
