#![cfg(unix)]

use myth_db::JsonlWriter;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Debug)]
struct Rec {
    from: String,
    n: u32,
}

#[test]
fn fork_two_processes_concurrent_append() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("log.jsonl");

    // Fork a child that appends 100 records.
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0, "fork() failed");

    if pid == 0 {
        // Child process.
        let writer = JsonlWriter::new(&path);
        for i in 0..100 {
            writer
                .append(&Rec {
                    from: "child".into(),
                    n: i,
                })
                .expect("child append");
        }
        unsafe {
            libc::_exit(0);
        }
    }

    // Parent appends 100 records concurrently.
    let writer = JsonlWriter::new(&path);
    for i in 0..100 {
        writer
            .append(&Rec {
                from: "parent".into(),
                n: i,
            })
            .expect("parent append");
    }

    // Wait for the child.
    let mut status: i32 = 0;
    unsafe {
        libc::waitpid(pid, &mut status as *mut i32, 0);
    }
    assert!(libc::WIFEXITED(status), "child did not exit cleanly");
    assert_eq!(libc::WEXITSTATUS(status), 0, "child exit status");

    // Verify: total 200 records, all parse cleanly, 100 parent + 100 child,
    // no truncation or interleaving that corrupts a line.
    let reader = JsonlWriter::new(&path);
    let records: Vec<Rec> = reader
        .iter::<Rec>()
        .unwrap()
        .collect::<anyhow::Result<_>>()
        .expect("parse JSONL");

    assert_eq!(records.len(), 200, "expected 200 records total");
    let parents = records.iter().filter(|r| r.from == "parent").count();
    let children = records.iter().filter(|r| r.from == "child").count();
    assert_eq!(parents, 100);
    assert_eq!(children, 100);
}
