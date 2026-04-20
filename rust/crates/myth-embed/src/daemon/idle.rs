//! Idle tracker — fires `wait_for_timeout` when no activity has happened
//! for `timeout` duration. Activity is recorded via `bump`.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tokio::sync::Notify;

pub struct IdleTracker {
    last_activity: Mutex<Instant>,
    timeout: Duration,
    notify: Notify,
}

impl IdleTracker {
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_activity: Mutex::new(Instant::now()),
            timeout,
            notify: Notify::new(),
        }
    }

    pub fn bump(&self) {
        *self.last_activity.lock().expect("idle mutex poisoned") = Instant::now();
        self.notify.notify_waiters();
    }

    pub async fn wait_for_timeout(&self) {
        loop {
            let last = *self.last_activity.lock().expect("idle mutex poisoned");
            let elapsed = last.elapsed();
            if elapsed >= self.timeout {
                return;
            }
            let remaining = self.timeout - elapsed;
            tokio::select! {
                _ = tokio::time::sleep(remaining) => {}
                _ = self.notify.notified() => {
                    // woken up by bump — recompute remaining time
                }
            }
        }
    }
}
