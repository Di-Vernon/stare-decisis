//! Self-fork-exec daemon spawner.
//!
//! Uses `std::process::Command::pre_exec` + `libc::setsid` so the spawned
//! daemon detaches from the controlling terminal / parent process group.
//! stdout/stderr are redirected to the daemon log file.

use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use anyhow::Context;

pub fn spawn_daemon() -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("locating current executable")?;

    let log_path = myth_common::myth_state().join("embed-daemon.log");
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("opening daemon log {:?}", log_path))?;

    let mut cmd = Command::new(&exe);
    cmd.arg("--daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log_file);

    // SAFETY: pre_exec runs in the forked child between fork() and exec().
    // setsid() is async-signal-safe; we only call that single function
    // here and surface any failure via io::Error.
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    cmd.spawn().context("spawning daemon process")?;
    // Parent returns immediately; the child is now detached. The client
    // polls the socket to decide when the daemon is actually listening.
    Ok(())
}
