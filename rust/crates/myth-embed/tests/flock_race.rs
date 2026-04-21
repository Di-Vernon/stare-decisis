//! Proves the daemon-lock behaviour: only one process can hold the
//! exclusive flock at a time. Uses `libc::fork` so we exercise the
//! real OS-level flock semantics (not just Rust Mutex).

#![cfg(unix)]

use std::fs::OpenOptions;

use fs4::fs_std::FileExt;
use tempfile::tempdir;

#[test]
fn child_cannot_acquire_while_parent_holds() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("race.lock");

    // Parent grabs the lock first.
    let parent_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .expect("parent open");
    let got = parent_file
        .try_lock_exclusive()
        .expect("parent try_lock_exclusive syscall should succeed");
    assert!(got, "parent should have acquired the exclusive lock");

    // Fork. Child opens the same path (fresh fd → fresh OFD) and
    // attempts `try_lock_exclusive`. flock is associated with open
    // file descriptions, so the child's attempt must fail.
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0, "fork() failed");

    if pid == 0 {
        // Child
        let child_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .expect("child open");
        let code = match child_file.try_lock_exclusive() {
            Ok(true) => 1,  // unexpected — both held
            Ok(false) => 0, // expected — contention, parent blocks us
            Err(_) => 2,    // IO error (distinct from "parent holds")
        };
        unsafe {
            libc::_exit(code);
        }
    }

    // Parent: wait for the child to report.
    let mut status: i32 = 0;
    unsafe {
        libc::waitpid(pid, &mut status as *mut i32, 0);
    }
    assert!(libc::WIFEXITED(status), "child did not exit normally");
    assert_eq!(
        libc::WEXITSTATUS(status),
        0,
        "child exited with {}; expected 0 (meaning parent's lock blocked it)",
        libc::WEXITSTATUS(status)
    );
}

#[test]
fn release_on_drop_lets_another_acquire() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("release.lock");

    {
        let first = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .unwrap();
        let got = first.try_lock_exclusive().expect("first syscall ok");
        assert!(got, "first should acquire lock");
        // first drops at end of scope → flock released.
    }

    let second = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap();
    let got = second
        .try_lock_exclusive()
        .expect("second syscall ok");
    assert!(got, "second should acquire after first dropped");
}
