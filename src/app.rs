//! Top-level state machine: owns the Store, the mode, the prompt, the help
//! overlay, the pending-g timer, and the edit buffers. Calls into `ops` for
//! the actual mutations.

use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;

use crate::input::{map_key, Action};
use crate::ops;
use crate::state::{EditField, Mode, PromptKind, Store, Task};

const GG_WINDOW: Duration = Duration::from_millis(500);

pub struct App {
    pub store: Store,
    pub mode: Mode,
    pub prompt: Option<PromptKind>,
    pub help_open: bool,
    pub should_quit: bool,
    pub dirty: bool,
    pending_g: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    None,
    Save,
    Quit,
}

impl App {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            mode: Mode::Normal,
            prompt: None,
            help_open: false,
            should_quit: false,
            dirty: false,
            pending_g: None,
        }
    }

    pub fn step(&mut self, key: KeyEvent) -> Effect {
        // Help overlay absorbs any key (including quit).
        if self.help_open {
            self.help_open = false;
            return Effect::None;
        }

        // Pending-g timer expiry.
        if let Some(t) = self.pending_g {
            if t.elapsed() > GG_WINDOW {
                self.pending_g = None;
            }
        }

        let action = map_key(&self.mode, self.prompt, key);

        // Special-case: PendingG → second 'g' becomes JumpTop.
        let action = match action {
            Action::PendingG => {
                if self.pending_g.is_some() {
                    self.pending_g = None;
                    Action::JumpTop
                } else {
                    self.pending_g = Some(Instant::now());
                    return Effect::None;
                }
            }
            other => {
                self.pending_g = None;
                other
            }
        };

        self.apply(action)
    }

    fn apply(&mut self, action: Action) -> Effect {
        match action {
            Action::Noop => Effect::None,

            Action::Quit => {
                self.should_quit = true;
                Effect::Quit
            }

            Action::MoveUp => {
                if self.store.cursor > 0 {
                    self.store.cursor -= 1;
                }
                Effect::None
            }
            Action::MoveDown => {
                if !self.store.tasks.is_empty()
                    && self.store.cursor + 1 < self.store.tasks.len()
                {
                    self.store.cursor += 1;
                }
                Effect::None
            }

            Action::JumpTop => {
                self.store.cursor = 0;
                Effect::None
            }
            Action::JumpBottom => {
                if !self.store.tasks.is_empty() {
                    self.store.cursor = self.store.tasks.len() - 1;
                }
                Effect::None
            }

            Action::NewBelow => {
                let title = String::new();
                let id = ops::add_below(&mut self.store, &title);
                self.enter_title_insert(id, /*append=*/ false);
                self.dirty = true;
                Effect::None
            }
            Action::NewAbove => {
                let title = String::new();
                let id = ops::add_above(&mut self.store, &title);
                self.store.cursor = self
                    .store
                    .tasks
                    .iter()
                    .position(|t| t.id == id)
                    .unwrap_or(self.store.cursor);
                self.enter_title_insert(id, false);
                self.dirty = true;
                Effect::None
            }

            Action::EditTitleAppend => {
                if let Some(task) = current_task(&self.store) {
                    let id = task.id;
                    self.enter_title_insert(id, /*append=*/ true);
                }
                Effect::None
            }
            Action::EditTitleInsert => {
                if let Some(task) = current_task(&self.store) {
                    let id = task.id;
                    self.enter_title_insert(id, /*append=*/ false);
                }
                Effect::None
            }
            Action::EditNote => {
                if let Some(task) = current_task(&self.store) {
                    let id = task.id;
                    let buffer = if task.note.is_empty() {
                        vec![String::new()]
                    } else {
                        task.note.split('\n').map(String::from).collect()
                    };
                    let cursor_col = buffer
                        .last()
                        .map(|l| l.chars().count())
                        .unwrap_or(0);
                    let cursor_row = buffer.len() - 1;
                    self.mode = Mode::Insert {
                        field: EditField::Note {
                            task_id: id,
                            cursor_row,
                            cursor_col,
                            buffer,
                        },
                    };
                }
                Effect::None
            }

            Action::ToggleComplete => {
                ops::toggle_complete(&mut self.store);
                self.dirty = true;
                Effect::None
            }
            Action::Delete => {
                ops::delete_at_cursor(&mut self.store);
                self.dirty = true;
                Effect::None
            }
            Action::DeleteAllPrompt => {
                self.prompt = Some(PromptKind::DeleteAll);
                Effect::None
            }
            Action::DeleteAllYes => {
                ops::delete_all(&mut self.store);
                self.prompt = None;
                self.dirty = true;
                Effect::None
            }
            Action::DeleteAllNo => {
                self.prompt = None;
                Effect::None
            }

            Action::MoveTaskDown => {
                ops::move_down(&mut self.store);
                self.dirty = true;
                Effect::None
            }
            Action::MoveTaskUp => {
                ops::move_up(&mut self.store);
                self.dirty = true;
                Effect::None
            }

            Action::Undo => {
                if ops::undo(&mut self.store) {
                    self.dirty = true;
                }
                Effect::None
            }

            Action::Help => {
                self.help_open = true;
                Effect::None
            }

            Action::EnterInsert(c) => {
                self.insert_char(c);
                Effect::None
            }
            Action::InsertBackspace => {
                self.insert_backspace();
                Effect::None
            }
            Action::InsertLeft => {
                self.insert_left();
                Effect::None
            }
            Action::InsertRight => {
                self.insert_right();
                Effect::None
            }
            Action::InsertUp => {
                self.insert_move_row(-1);
                Effect::None
            }
            Action::InsertDown => {
                self.insert_move_row(1);
                Effect::None
            }
            Action::InsertKillLine => {
                self.insert_kill_line();
                Effect::None
            }
            Action::InsertKillWord => {
                self.insert_kill_word();
                Effect::None
            }
            Action::InsertNewline => {
                self.insert_newline();
                Effect::None
            }
            Action::InsertTab => {
                self.insert_tab_to_note();
                Effect::None
            }
            Action::InsertCommit => {
                self.commit_insert();
                Effect::None
            }

            // Defensive: PendingG should have been resolved before reaching apply.
            Action::PendingG => Effect::None,
        }
    }

    fn enter_title_insert(&mut self, task_id: u64, append: bool) {
        let task = match self.store.tasks.iter().find(|t| t.id == task_id) {
            Some(t) => t.clone(),
            None => return,
        };
        let cursor = if append {
            task.title.chars().count()
        } else {
            0
        };
        self.mode = Mode::Insert {
            field: EditField::Title {
                task_id,
                cursor,
                buffer: task.title.clone(),
            },
        };
    }

    fn commit_insert(&mut self) {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        match mode {
            Mode::Insert {
                field: EditField::Title {
                    task_id,
                    buffer,
                    ..
                },
            } => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    // Cancel: if it's a brand-new task (empty on creation), drop it.
                    let idx = self.store.tasks.iter().position(|t| t.id == task_id);
                    if let Some(i) = idx {
                        if self.store.tasks[i].title.is_empty() {
                            self.store.tasks.remove(i);
                            self.store.normalize_cursor();
                        }
                    }
                    return;
                }
                if let Some(t) = self.store.tasks.iter_mut().find(|t| t.id == task_id) {
                    t.title = trimmed.to_string();
                }
                self.dirty = true;
            }
            Mode::Insert {
                field: EditField::Note {
                    task_id, buffer, ..
                },
            } => {
                if let Some(t) = self.store.tasks.iter_mut().find(|t| t.id == task_id) {
                    t.note = buffer.join("\n");
                }
                self.dirty = true;
            }
            Mode::Normal => {} // shouldn't happen
        }
    }

    fn insert_char(&mut self, c: char) {
        match &mut self.mode {
            Mode::Insert {
                field: EditField::Title { buffer, cursor, .. },
            } => {
                let byte_idx = char_to_byte(buffer, *cursor);
                buffer.insert(byte_idx, c);
                *cursor += 1;
            }
            Mode::Insert {
                field:
                    EditField::Note {
                        buffer,
                        cursor_row,
                        cursor_col,
                        ..
                    },
                ..
            } => {
                let line = &mut buffer[*cursor_row];
                let byte_idx = char_to_byte(line, *cursor_col);
                line.insert(byte_idx, c);
                *cursor_col += 1;
            }
            Mode::Normal => {}
        }
    }

    fn insert_backspace(&mut self) {
        match &mut self.mode {
            Mode::Insert {
                field: EditField::Title { buffer, cursor, .. },
            } => {
                if *cursor == 0 {
                    return;
                }
                let byte_idx = char_to_byte(buffer, *cursor - 1);
                buffer.remove(byte_idx);
                *cursor -= 1;
            }
            Mode::Insert {
                field:
                    EditField::Note {
                        buffer,
                        cursor_row,
                        cursor_col,
                        ..
                    },
                ..
            } => {
                if *cursor_col > 0 {
                    let line = &mut buffer[*cursor_row];
                    let byte_idx = char_to_byte(line, *cursor_col - 1);
                    line.remove(byte_idx);
                    *cursor_col -= 1;
                } else if *cursor_row > 0 {
                    // Join with previous line.
                    let current = buffer.remove(*cursor_row);
                    let prev_row = *cursor_row - 1;
                    let prev_len_chars = buffer[prev_row].chars().count();
                    buffer[prev_row].push_str(&current);
                    *cursor_row = prev_row;
                    *cursor_col = prev_len_chars;
                }
            }
            Mode::Normal => {}
        }
    }

    fn insert_left(&mut self) {
        match &mut self.mode {
            Mode::Insert {
                field: EditField::Title { cursor, .. },
            } => {
                if *cursor > 0 {
                    *cursor -= 1;
                }
            }
            Mode::Insert {
                field:
                    EditField::Note {
                        buffer,
                        cursor_row,
                        cursor_col,
                        ..
                    },
                ..
            } => {
                if *cursor_col > 0 {
                    *cursor_col -= 1;
                } else if *cursor_row > 0 {
                    *cursor_row -= 1;
                    *cursor_col = buffer[*cursor_row].chars().count();
                }
            }
            Mode::Normal => {}
        }
    }

    fn insert_right(&mut self) {
        match &mut self.mode {
            Mode::Insert {
                field: EditField::Title { buffer, cursor, .. },
            } => {
                if *cursor < buffer.chars().count() {
                    *cursor += 1;
                }
            }
            Mode::Insert {
                field:
                    EditField::Note {
                        buffer,
                        cursor_row,
                        cursor_col,
                        ..
                    },
                ..
            } => {
                let len = buffer[*cursor_row].chars().count();
                if *cursor_col < len {
                    *cursor_col += 1;
                } else if *cursor_row + 1 < buffer.len() {
                    *cursor_row += 1;
                    *cursor_col = 0;
                }
            }
            Mode::Normal => {}
        }
    }

    fn insert_move_row(&mut self, delta: i32) {
        if let Mode::Insert {
            field:
                EditField::Note {
                    buffer,
                    cursor_row,
                    cursor_col,
                    ..
                },
        } = &mut self.mode
        {
            let new_row = (*cursor_row as i32 + delta).max(0).min(buffer.len() as i32 - 1);
            let new_row = new_row as usize;
            *cursor_row = new_row;
            let len = buffer[new_row].chars().count();
            if *cursor_col > len {
                *cursor_col = len;
            }
        }
    }

    fn insert_kill_line(&mut self) {
        match &mut self.mode {
            Mode::Insert {
                field: EditField::Title { buffer, cursor, .. },
            } => {
                let byte_idx = char_to_byte(buffer, *cursor);
                buffer.truncate(byte_idx);
            }
            Mode::Insert {
                field:
                    EditField::Note {
                        buffer,
                        cursor_row,
                        cursor_col,
                        ..
                    },
                ..
            } => {
                let byte_idx = char_to_byte(&buffer[*cursor_row], *cursor_col);
                buffer[*cursor_row].truncate(byte_idx);
            }
            Mode::Normal => {}
        }
    }

    fn insert_kill_word(&mut self) {
        match &mut self.mode {
            Mode::Insert {
                field: EditField::Title { buffer, cursor, .. },
            } => {
                let byte_idx = char_to_byte(buffer, *cursor);
                let prefix = &buffer[..byte_idx];
                let cut = last_word_boundary(prefix);
                buffer.truncate(cut);
                *cursor = buffer.chars().count();
            }
            Mode::Insert {
                field:
                    EditField::Note {
                        buffer,
                        cursor_row,
                        cursor_col,
                        ..
                    },
                ..
            } => {
                let line = &mut buffer[*cursor_row];
                let byte_idx = char_to_byte(line, *cursor_col);
                let prefix = &line[..byte_idx].to_string();
                let cut = last_word_boundary(&prefix);
                line.truncate(cut);
                *cursor_col = line.chars().count();
            }
            Mode::Normal => {}
        }
    }

    fn insert_newline(&mut self) {
        if let Mode::Insert {
            field:
                EditField::Note {
                    buffer,
                    cursor_row,
                    cursor_col,
                    ..
                },
        } = &mut self.mode
        {
            let line = &mut buffer[*cursor_row];
            let byte_idx = char_to_byte(line, *cursor_col);
            let tail = line[byte_idx..].to_string();
            line.truncate(byte_idx);
            buffer.insert(*cursor_row + 1, tail);
            *cursor_row += 1;
            *cursor_col = 0;
        }
    }

    fn insert_tab_to_note(&mut self) {
        if let Mode::Insert {
            field: EditField::Title { task_id, .. },
        } = &self.mode
        {
            let task_id = *task_id;
            self.commit_insert();
            // Now re-enter note insert for the same task.
            if let Some(task) = self.store.tasks.iter().find(|t| t.id == task_id).cloned() {
                let buffer = if task.note.is_empty() {
                    vec![String::new()]
                } else {
                    task.note.split('\n').map(String::from).collect()
                };
                self.mode = Mode::Insert {
                    field: EditField::Note {
                        task_id,
                        cursor_row: buffer.len() - 1,
                        cursor_col: buffer.last().map(|l| l.chars().count()).unwrap_or(0),
                        buffer,
                    },
                };
            }
        }
    }
}

fn current_task(store: &Store) -> Option<Task> {
    store.tasks.get(store.cursor).cloned()
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(b, _)| b).unwrap_or(s.len())
}

/// Find the byte index just after the last word-boundary char in `s`.
/// A "word" is a run of non-whitespace; kill-word deletes back to the
/// previous whitespace (or to start).
fn last_word_boundary(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = bytes.len();
    // Skip trailing whitespace.
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    // Skip word chars.
    while i > 0 && !bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    i
}