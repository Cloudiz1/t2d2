//! Domain types: tasks, the store, edit state, mode.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub note: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoEntry {
    pub task: Task,
    pub original_index: usize,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub tasks: Vec<Task>,
    pub undo: Vec<UndoEntry>,
    pub cursor: usize,
    pub next_id: u64,
}

impl Store {
    pub fn empty() -> Self {
        Self {
            tasks: Vec::new(),
            undo: Vec::new(),
            cursor: 0,
            next_id: 1,
        }
    }

    pub fn normalize_cursor(&mut self) {
        if self.tasks.is_empty() {
            self.cursor = 0;
        } else if self.cursor >= self.tasks.len() {
            self.cursor = self.tasks.len() - 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert { field: EditField },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditField {
    Title {
        task_id: u64,
        cursor: usize,
        buffer: String,
    },
    Note {
        task_id: u64,
        cursor_row: usize,
        cursor_col: usize,
        buffer: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    DeleteAll,
}