use t2d2::{ops, Store, Task};

fn store_with(tasks: Vec<Task>, cursor: usize, next_id: u64) -> Store {
    Store {
        tasks,
        undo: Vec::new(),
        cursor,
        next_id,
    }
}

fn task(id: u64, title: &str, completed: bool) -> Task {
    Task {
        id,
        title: title.to_string(),
        note: String::new(),
        completed,
    }
}

// ---------- add_below / add_above ----------

#[test]
fn add_below_inserts_at_cursor_plus_one() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 0, 3);
    let id = ops::add_below(&mut s, "x");
    assert_eq!(id, 3);
    assert_eq!(s.tasks.len(), 3);
    assert_eq!(s.tasks[1].title, "x");
    assert_eq!(s.cursor, 1);
}

#[test]
fn add_below_at_last_appends() {
    let mut s = store_with(vec![task(1, "a", false)], 0, 2);
    let id = ops::add_below(&mut s, "b");
    assert_eq!(id, 2);
    assert_eq!(s.tasks.len(), 2);
    assert_eq!(s.tasks[1].title, "b");
    assert_eq!(s.cursor, 1);
}

#[test]
fn add_above_inserts_at_cursor() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 1, 3);
    let id = ops::add_above(&mut s, "x");
    assert_eq!(id, 3);
    assert_eq!(s.tasks.len(), 3);
    assert_eq!(s.tasks[1].title, "x");
    assert_eq!(s.tasks[2].title, "b");
    assert_eq!(s.cursor, 1);
}

#[test]
fn add_below_increments_next_id() {
    let mut s = store_with(vec![task(1, "a", false)], 0, 5);
    let id = ops::add_below(&mut s, "b");
    assert_eq!(id, 5);
    assert_eq!(s.next_id, 6);
}

// ---------- delete ----------

#[test]
fn delete_pushes_to_undo_stack() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 0, 3);
    ops::delete_at_cursor(&mut s);
    assert_eq!(s.tasks.len(), 1);
    assert_eq!(s.tasks[0].title, "b");
    assert_eq!(s.undo.len(), 1);
    assert_eq!(s.undo[0].task.title, "a");
    assert_eq!(s.undo[0].original_index, 0);
}

#[test]
fn delete_on_empty_store_is_noop() {
    let mut s = store_with(vec![], 0, 1);
    ops::delete_at_cursor(&mut s);
    assert!(s.tasks.is_empty());
    assert!(s.undo.is_empty());
}

// ---------- undo ----------

#[test]
fn undo_restores_at_original_index() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false), task(3, "c", false)], 1, 4);
    ops::delete_at_cursor(&mut s); // removes "b" at index 1
    assert_eq!(s.tasks.len(), 2);

    let ok = ops::undo(&mut s);
    assert!(ok);
    assert_eq!(s.tasks.len(), 3);
    assert_eq!(s.tasks[1].title, "b");
    assert!(s.undo.is_empty());
}

#[test]
fn undo_restores_with_cursor_on_restored_task() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 0, 3);
    ops::delete_at_cursor(&mut s); // removes "a"
    // After delete, cursor normalizes to 0 (now points at "b").
    ops::undo(&mut s);
    assert_eq!(s.cursor, 0); // original index of restored task
}

#[test]
fn undo_on_empty_stack_returns_false() {
    let mut s = store_with(vec![task(1, "a", false)], 0, 2);
    let ok = ops::undo(&mut s);
    assert!(!ok);
    assert_eq!(s.tasks.len(), 1);
}

// ---------- toggle_complete ----------

#[test]
fn toggle_complete_to_true_sinks_to_bottom_of_completed_section() {
    let mut s = store_with(
        vec![
            task(1, "a", false),
            task(2, "b", true), // already done
            task(3, "c", false),
            task(4, "d", true), // already done, last done
        ],
        0, // cursor on "a"
        5,
    );
    ops::toggle_complete(&mut s);
    // "a" should be inserted after the last completed task (d).
    // Other tasks keep their relative order; only "a" moves to the end.
    let titles: Vec<&str> = s.tasks.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(titles, vec!["b", "c", "d", "a"]);
    assert!(s.tasks[3].completed);
    assert!(s.tasks[0].completed);
    assert!(s.tasks[2].completed);
    assert!(!s.tasks[1].completed);
}

#[test]
fn toggle_complete_to_true_when_no_completed_appends() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 0, 3);
    ops::toggle_complete(&mut s);
    assert_eq!(s.tasks[0].title, "b");
    assert_eq!(s.tasks[1].title, "a");
    assert!(s.tasks[1].completed);
}

#[test]
fn toggle_complete_to_false_does_not_move_task() {
    let mut s = store_with(vec![task(1, "a", true), task(2, "b", false)], 0, 3);
    ops::toggle_complete(&mut s);
    assert_eq!(s.tasks[0].title, "a");
    assert!(!s.tasks[0].completed);
    assert_eq!(s.tasks[1].title, "b");
    assert!(!s.tasks[1].completed);
}

#[test]
fn toggle_complete_to_true_keeps_cursor_in_place() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 0, 3);
    ops::toggle_complete(&mut s);
    // "a" sinks to the bottom of the completed section (end of list).
    // Cursor stays where it was (index 0), now pointing at "b".
    assert_eq!(s.cursor, 0);
    assert_eq!(s.tasks[s.cursor].title, "b");
    assert_eq!(s.tasks[1].title, "a");
    assert!(s.tasks[1].completed);
}

// ---------- move_down / move_up ----------

#[test]
fn move_down_swaps_with_next_when_not_last() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 0, 3);
    ops::move_down(&mut s);
    assert_eq!(s.tasks[0].title, "b");
    assert_eq!(s.tasks[1].title, "a");
    assert_eq!(s.cursor, 1);
}

#[test]
fn move_down_at_last_is_noop() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 1, 3);
    ops::move_down(&mut s);
    assert_eq!(s.tasks[0].title, "a");
    assert_eq!(s.tasks[1].title, "b");
    assert_eq!(s.cursor, 1);
}

#[test]
fn move_up_swaps_with_prev_when_not_first() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 1, 3);
    ops::move_up(&mut s);
    assert_eq!(s.tasks[0].title, "b");
    assert_eq!(s.tasks[1].title, "a");
    assert_eq!(s.cursor, 0);
}

#[test]
fn move_up_at_first_is_noop() {
    let mut s = store_with(vec![task(1, "a", false), task(2, "b", false)], 0, 3);
    ops::move_up(&mut s);
    assert_eq!(s.tasks[0].title, "a");
    assert_eq!(s.tasks[1].title, "b");
    assert_eq!(s.cursor, 0);
}

// ---------- delete_all ----------

#[test]
fn delete_all_empties_tasks_and_clears_undo() {
    let mut s = store_with(
        vec![task(1, "a", false), task(2, "b", false)],
        0,
        3,
    );
    ops::delete_at_cursor(&mut s);
    assert!(!s.undo.is_empty());

    ops::delete_all(&mut s);
    assert!(s.tasks.is_empty());
    assert!(s.undo.is_empty());
    assert_eq!(s.cursor, 0);
}

// ---------- id counter ----------

#[test]
fn id_counter_strictly_monotonic_after_many_adds() {
    let mut s = Store::empty();
    let mut last = 0u64;
    for _ in 0..50 {
        let id = ops::add_below(&mut s, "x");
        assert!(id > last);
        last = id;
    }
}