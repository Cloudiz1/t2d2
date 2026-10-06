use std::fs;
use std::path::PathBuf;

use t2d2::{persistence, ops, Store, Task};

fn unique_temp_path(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("t2d2-test-{}-{}-{}.json", tag, std::process::id(), stamp));
    p
}

fn task_in_store(s: &mut Store, title: &str) -> u64 {
    ops::add_below(s, title)
}

#[test]
fn save_and_load_roundtrip() {
    let path = unique_temp_path("roundtrip");

    let mut s = Store::empty();
    task_in_store(&mut s, "first");
    task_in_store(&mut s, "second");
    s.tasks[1].note = "with a note".to_string();
    persistence::save_to(&path, &s).unwrap();

    let loaded = persistence::load_from(&path).unwrap();
    assert_eq!(loaded.tasks.len(), 2);
    assert_eq!(loaded.tasks[0].title, "first");
    assert_eq!(loaded.tasks[1].title, "second");
    assert_eq!(loaded.tasks[1].note, "with a note");
    // next_id must resume above max id.
    assert!(loaded.next_id > 2);

    let _ = fs::remove_file(&path);
}

#[test]
fn load_missing_file_returns_empty_store() {
    let path = unique_temp_path("missing");
    let _ = fs::remove_file(&path); // ensure it doesn't exist
    let s = persistence::load_from(&path).unwrap();
    assert!(s.tasks.is_empty());
    assert_eq!(s.next_id, 1);
    assert_eq!(s.cursor, 0);
}

#[test]
fn load_corrupt_file_returns_empty_store_and_warns() {
    let path = unique_temp_path("corrupt");
    fs::write(&path, "this is not valid json {").unwrap();
    let s = persistence::load_from(&path).unwrap();
    assert!(s.tasks.is_empty());
    assert_eq!(s.next_id, 1);
    let _ = fs::remove_file(&path);
}

#[test]
fn save_creates_parent_dir_if_missing() {
    let mut path = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("t2d2-test-nested-{}-{}", std::process::id(), stamp));
    path.push("subdir");
    path.push("tasks.json");

    // Make sure the parent dirs don't exist.
    let _ = fs::remove_dir_all(path.parent().unwrap());

    let mut s = Store::empty();
    task_in_store(&mut s, "alpha");
    persistence::save_to(&path, &s).unwrap();
    assert!(path.exists());

    let loaded = persistence::load_from(&path).unwrap();
    assert_eq!(loaded.tasks.len(), 1);
    assert_eq!(loaded.tasks[0].title, "alpha");

    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn id_counter_resumes_above_max_after_load() {
    let path = unique_temp_path("idresume");

    let mut s = Store::empty();
    let id1 = ops::add_below(&mut s, "first");
    let id2 = ops::add_below(&mut s, "second");
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    persistence::save_to(&path, &s).unwrap();

    let mut loaded = persistence::load_from(&path).unwrap();
    let id3 = ops::add_below(&mut loaded, "third");
    assert!(id3 > id2, "next id must resume above the previous max");

    let _ = fs::remove_file(&path);
}

#[test]
fn load_preserves_completed_flag() {
    let path = unique_temp_path("completed");
    let mut s = Store::empty();
    task_in_store(&mut s, "x");
    s.tasks[0].completed = true;
    persistence::save_to(&path, &s).unwrap();

    let loaded = persistence::load_from(&path).unwrap();
    assert!(loaded.tasks[0].completed);
    let _ = fs::remove_file(&path);
}

#[test]
fn empty_list_roundtrip() {
    let path = unique_temp_path("empty");
    let s = Store::empty();
    persistence::save_to(&path, &s).unwrap();
    let loaded = persistence::load_from(&path).unwrap();
    assert!(loaded.tasks.is_empty());
    assert_eq!(loaded.next_id, 1);
    let _ = fs::remove_file(&path);
}

#[test]
fn task_in_store_unused_compiles() {
    // Keep the unused-helper from rustc complaining if a future test stops using it.
    let _ = task_in_store;
    let _: Vec<Task> = Vec::new();
}