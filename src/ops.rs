//! Pure mutations on `Store`. No I/O, no rendering.

use crate::state::{Store, Task, UndoEntry};

pub const UNDO_BOUND: usize = 32;

pub fn add_below(store: &mut Store, title: &str) -> u64 {
    let id = store.next_id;
    store.next_id += 1;
    let task = Task {
        id,
        title: title.to_string(),
        note: String::new(),
        completed: false,
    };
    let insert_at = (store.cursor + 1).min(store.tasks.len());
    store.tasks.insert(insert_at, task);
    store.cursor = insert_at;
    id
}

pub fn add_above(store: &mut Store, title: &str) -> u64 {
    let id = store.next_id;
    store.next_id += 1;
    let task = Task {
        id,
        title: title.to_string(),
        note: String::new(),
        completed: false,
    };
    let insert_at = store.cursor.min(store.tasks.len());
    store.tasks.insert(insert_at, task);
    // Cursor stays on the new task.
    id
}

pub fn delete_at_cursor(store: &mut Store) {
    if store.tasks.is_empty() {
        return;
    }
    let idx = store.cursor.min(store.tasks.len() - 1);
    let removed = store.tasks.remove(idx);
    store.undo.push(UndoEntry {
        task: removed,
        original_index: idx,
    });
    if store.undo.len() > UNDO_BOUND {
        store.undo.remove(0);
    }
    store.normalize_cursor();
}

pub fn undo(store: &mut Store) -> bool {
    let Some(entry) = store.undo.pop() else {
        return false;
    };
    let idx = entry.original_index.min(store.tasks.len());
    store.tasks.insert(idx, entry.task.clone());
    store.cursor = idx;
    true
}

pub fn toggle_complete(store: &mut Store) {
    if store.tasks.is_empty() {
        return;
    }
    let idx = store.cursor.min(store.tasks.len() - 1);
    let was_completed = store.tasks[idx].completed;
    store.tasks[idx].completed = !was_completed;

    if !was_completed {
        // Now completed — find last completed task after removal and insert after it;
        // if none, append.
        let task = store.tasks.remove(idx);
        let mut insert_at = store.tasks.len();
        for (i, t) in store.tasks.iter().enumerate().rev() {
            if t.completed {
                insert_at = i + 1;
                break;
            }
        }
        store.tasks.insert(insert_at, task);
        store.cursor = insert_at;
    }
    // Toggling to incomplete: leave position alone.
}

pub fn move_down(store: &mut Store) {
    if store.cursor + 1 < store.tasks.len() {
        store.tasks.swap(store.cursor, store.cursor + 1);
        store.cursor += 1;
    }
}

pub fn move_up(store: &mut Store) {
    if store.cursor > 0 && !store.tasks.is_empty() {
        store.tasks.swap(store.cursor, store.cursor - 1);
        store.cursor -= 1;
    }
}

pub fn delete_all(store: &mut Store) {
    store.tasks.clear();
    store.undo.clear();
    store.cursor = 0;
}