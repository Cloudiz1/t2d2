//! Pure keymap dispatch: `key + mode + prompt → Action`.
//!
//! The state machine lives in `app.rs` (it owns the pending-g timer, the help
//! overlay flag, the prompt, etc.). This module knows nothing about timing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::{EditField, Mode, PromptKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Noop,
    Quit,
    MoveUp,
    MoveDown,
    JumpTop,
    JumpBottom,
    NewBelow,
    NewAbove,
    EditTitleAppend,
    EditTitleInsert,
    EditNote,
    ToggleComplete,
    Delete,
    DeleteAllPrompt,
    MoveTaskDown,
    MoveTaskUp,
    Undo,
    Help,
    EnterInsert(char),
    InsertBackspace,
    InsertLeft,
    InsertRight,
    InsertUp,        // note only
    InsertDown,      // note only
    InsertKillLine,
    InsertKillWord,
    InsertNewline,
    InsertTab,       // title only
    InsertCommit,
    DeleteAllYes,
    DeleteAllNo,
    /// First 'g' press in normal mode. Caller decides whether to time it out
    /// or follow with JumpTop on the next 'g'.
    PendingG,
}

pub fn map_key(mode: &Mode, prompt: Option<PromptKind>, key: KeyEvent) -> Action {
    // The prompt absorbs all keys except its specific bindings.
    if let Some(p) = prompt {
        return match p {
            PromptKind::DeleteAll => match key.code {
                KeyCode::Char('y') => Action::DeleteAllYes,
                KeyCode::Char('n') => Action::DeleteAllNo,
                KeyCode::Esc => Action::DeleteAllNo,
                _ => Action::Noop,
            },
        };
    }

    match mode {
        Mode::Normal => normal(key),
        Mode::Insert { field } => insert(field.clone(), key),
    }
}

fn normal(key: KeyEvent) -> Action {
    // Ctrl-modified keys.
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('q') => Action::Quit,
            _ => Action::Noop,
        };
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::PendingG,
        KeyCode::Char('G') => Action::JumpBottom,
        KeyCode::Char('o') => Action::NewBelow,
        KeyCode::Char('O') => Action::NewAbove,
        KeyCode::Char('a') => Action::EditTitleAppend,
        KeyCode::Char('i') => Action::EditTitleInsert,
        KeyCode::Tab => Action::EditNote,
        KeyCode::Char('c') => Action::ToggleComplete,
        KeyCode::Char('d') => Action::Delete,
        KeyCode::Char('D') => Action::DeleteAllPrompt,
        KeyCode::Char('J') => Action::MoveTaskDown,
        KeyCode::Char('K') => Action::MoveTaskUp,
        KeyCode::Char('u') => Action::Undo,
        KeyCode::Char('?') => Action::Help,
        _ => Action::Noop,
    }
}

fn insert(field: EditField, key: KeyEvent) -> Action {
    // Ctrl-modified.
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('u') => Action::InsertKillLine,
            KeyCode::Char('w') => Action::InsertKillWord,
            _ => Action::Noop,
        };
    }

    match field {
        EditField::Title { .. } => match key.code {
            KeyCode::Char(c) => Action::EnterInsert(c),
            KeyCode::Backspace => Action::InsertBackspace,
            KeyCode::Left => Action::InsertLeft,
            KeyCode::Right => Action::InsertRight,
            KeyCode::Enter => Action::InsertCommit,
            KeyCode::Tab => Action::InsertTab,
            KeyCode::Esc => Action::InsertCommit,
            _ => Action::Noop,
        },
        EditField::Note { .. } => match key.code {
            KeyCode::Char(c) => Action::EnterInsert(c),
            KeyCode::Backspace => Action::InsertBackspace,
            KeyCode::Left => Action::InsertLeft,
            KeyCode::Right => Action::InsertRight,
            KeyCode::Up => Action::InsertUp,
            KeyCode::Down => Action::InsertDown,
            KeyCode::Enter => Action::InsertNewline,
            KeyCode::Esc => Action::InsertCommit,
            _ => Action::Noop,
        },
    }
}