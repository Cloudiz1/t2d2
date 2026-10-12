//! Integration tests for the single-instance lock.
//!
//! These exercise the *mechanism* (exclusive flock on a file) using the
//! same `fs2` API the binary uses. The binary itself is harder to test in
//! a forked process without a TTY, so we test the lock primitives
//! directly and trust the wiring in `main.rs`.

use std::fs;
use std::path::PathBuf;

use fs2::FileExt;

fn unique_lock_path() -> PathBuf {
    let mut p = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!(
        "t2d2-test-lock-{}-{}.lock",
        std::process::id(),
        stamp
    ));
    p
}

fn open(path: &PathBuf) -> fs::File {
    fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .expect("open lock file")
}

#[test]
fn second_exclusive_lock_fails() {
    let path = unique_lock_path();
    let a = open(&path);
    let b = open(&path);

    assert!(a.try_lock_exclusive().is_ok(), "first lock should succeed");
    assert!(
        b.try_lock_exclusive().is_err(),
        "second lock on same file must fail"
    );

    // Drop `a` (close the fd) — kernel releases the flock. The second
    // attempt should now succeed, which is the stale-lock recovery path.
    drop(a);
    assert!(
        b.try_lock_exclusive().is_ok(),
        "lock should be re-acquirable after holder is dropped"
    );

    let _ = fs::remove_file(&path);
}

#[test]
fn lock_is_per_file() {
    // Two different paths should not contend. Sanity check that we're
    // not accidentally using a global lock.
    let p1 = unique_lock_path();
    let p2 = unique_lock_path();
    let a = open(&p1);
    let b = open(&p2);
    assert!(a.try_lock_exclusive().is_ok());
    assert!(b.try_lock_exclusive().is_ok());
    let _ = fs::remove_file(&p1);
    let _ = fs::remove_file(&p2);
}
