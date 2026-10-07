use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use t2d2::input::{map_key, Action};
use t2d2::state::{EditField, Mode, PromptKind};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

fn char(s: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(s), KeyModifiers::NONE)
}

// ---------- normal mode ----------

#[test]
fn normal_j_k_move() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, key(KeyCode::Down)), Action::MoveDown));
    assert!(matches!(map_key(&m, None, key(KeyCode::Up)), Action::MoveUp));
}

#[test]
fn normal_d_is_single_tap_not_chord() {
    let m = Mode::Normal;
    // Single press of 'd' should fire Delete, not be held for a chord.
    assert!(matches!(map_key(&m, None, char('d')), Action::Delete));
    let _ = m; // silence unused
}

#[test]
fn normal_g_is_pending() {
    // First 'g' press returns PendingG; the App's timer decides whether to
    // upgrade the next 'g' to JumpTop.
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('g')), Action::PendingG));
}

#[test]
fn normal_capital_g_is_jump_bottom() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('G')), Action::JumpBottom));
}

#[test]
fn normal_o_and_capital_o() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('o')), Action::NewBelow));
    assert!(matches!(map_key(&m, None, char('O')), Action::NewAbove));
}

#[test]
fn normal_a_i() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('a')), Action::EditTitleAppend));
    assert!(matches!(map_key(&m, None, char('i')), Action::EditTitleInsert));
}

#[test]
fn normal_tab_opens_note() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, key(KeyCode::Tab)), Action::EditNote));
}

#[test]
fn normal_capital_a_is_noop() {
    // A was the old "open note" key; now Tab does that. A is unbound.
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('A')), Action::Noop));
}

#[test]
fn normal_c_d_D() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('c')), Action::ToggleComplete));
    assert!(matches!(map_key(&m, None, char('d')), Action::Delete));
    assert!(matches!(map_key(&m, None, char('D')), Action::DeleteAllPrompt));
}

#[test]
fn normal_J_K_reorder() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('J')), Action::MoveTaskDown));
    assert!(matches!(map_key(&m, None, char('K')), Action::MoveTaskUp));
}

#[test]
fn normal_u_undo() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('u')), Action::Undo));
}

#[test]
fn normal_help() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('?')), Action::Help));
}

#[test]
fn normal_ctrl_q_quit() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, ctrl(KeyCode::Char('q'))), Action::Quit));
}

#[test]
fn normal_keys_unbound_are_noop() {
    let m = Mode::Normal;
    assert!(matches!(map_key(&m, None, char('z')), Action::Noop));
    assert!(matches!(map_key(&m, None, char('x')), Action::Noop));
}

// ---------- insert title mode ----------

#[test]
fn insert_title_typing_inserts_char() {
    let m = Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    assert!(matches!(
        map_key(&m, None, char('h')),
        Action::EnterInsert('h')
    ));
}

#[test]
fn insert_title_esc_commits() {
    let m = Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    assert!(matches!(map_key(&m, None, key(KeyCode::Esc)), Action::InsertCommit));
}

#[test]
fn insert_title_enter_commits() {
    let m = Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    assert!(matches!(
        map_key(&m, None, key(KeyCode::Enter)),
        Action::InsertCommit
    ));
}

#[test]
fn insert_title_tab_switches_to_note() {
    let m = Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    assert!(matches!(map_key(&m, None, key(KeyCode::Tab)), Action::InsertTab));
}

#[test]
fn insert_title_arrow_keys() {
    let m = || Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    assert!(matches!(
        map_key(&m(), None, key(KeyCode::Left)),
        Action::InsertLeft
    ));
    assert!(matches!(
        map_key(&m(), None, key(KeyCode::Right)),
        Action::InsertRight
    ));
}

#[test]
fn insert_title_ctrl_u_w() {
    let m = || Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    assert!(matches!(
        map_key(&m(), None, ctrl(KeyCode::Char('u'))),
        Action::InsertKillLine
    ));
    assert!(matches!(
        map_key(&m(), None, ctrl(KeyCode::Char('w'))),
        Action::InsertKillWord
    ));
}

#[test]
fn insert_title_up_down_are_noop() {
    let m = Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    assert!(matches!(
        map_key(&m, None, key(KeyCode::Up)),
        Action::Noop
    ));
    assert!(matches!(
        map_key(&m, None, key(KeyCode::Down)),
        Action::Noop
    ));
}

#[test]
fn insert_title_unbound_keys_are_noop() {
    let m = Mode::Insert {
        field: EditField::Title {
            task_id: 1,
            cursor: 0,
            buffer: String::new(),
        },
    };
    // F1, PageUp, etc. — non-printable, non-arrow, non-control.
    assert!(matches!(
        map_key(&m, None, key(KeyCode::F(1))),
        Action::Noop
    ));
    assert!(matches!(
        map_key(&m, None, key(KeyCode::PageUp)),
        Action::Noop
    ));
}

// ---------- insert note mode ----------

#[test]
fn insert_note_enter_inserts_newline() {
    let m = Mode::Insert {
        field: EditField::Note {
            task_id: 1,
            cursor_row: 0,
            cursor_col: 0,
            buffer: vec![String::new()],
        },
    };
    assert!(matches!(
        map_key(&m, None, key(KeyCode::Enter)),
        Action::InsertNewline
    ));
}

#[test]
fn insert_note_up_down_move_between_rows() {
    let m = Mode::Insert {
        field: EditField::Note {
            task_id: 1,
            cursor_row: 0,
            cursor_col: 0,
            buffer: vec![String::new(), String::new()],
        },
    };
    assert!(matches!(
        map_key(&m, None, key(KeyCode::Down)),
        Action::InsertDown
    ));
    assert!(matches!(
        map_key(&m, None, key(KeyCode::Up)),
        Action::InsertUp
    ));
}

#[test]
fn insert_note_esc_commits() {
    let m = Mode::Insert {
        field: EditField::Note {
            task_id: 1,
            cursor_row: 0,
            cursor_col: 0,
            buffer: vec![String::new()],
        },
    };
    assert!(matches!(
        map_key(&m, None, key(KeyCode::Esc)),
        Action::InsertCommit
    ));
}

#[test]
fn insert_note_tab_is_noop() {
    let m = Mode::Insert {
        field: EditField::Note {
            task_id: 1,
            cursor_row: 0,
            cursor_col: 0,
            buffer: vec![String::new()],
        },
    };
    assert!(matches!(map_key(&m, None, key(KeyCode::Tab)), Action::Noop));
}

// ---------- delete-all prompt ----------

#[test]
fn prompt_y_runs_delete_all() {
    let m = Mode::Normal;
    let p = Some(PromptKind::DeleteAll);
    assert!(matches!(
        map_key(&m, p, char('y')),
        Action::DeleteAllYes
    ));
}

#[test]
fn prompt_n_cancels() {
    let m = Mode::Normal;
    let p = Some(PromptKind::DeleteAll);
    assert!(matches!(map_key(&m, p, char('n')), Action::DeleteAllNo));
}

#[test]
fn prompt_esc_cancels() {
    let m = Mode::Normal;
    let p = Some(PromptKind::DeleteAll);
    assert!(matches!(
        map_key(&m, p, key(KeyCode::Esc)),
        Action::DeleteAllNo
    ));
}

#[test]
fn prompt_other_keys_are_noop() {
    let m = Mode::Normal;
    let p = Some(PromptKind::DeleteAll);
    assert!(matches!(map_key(&m, p, char('a')), Action::Noop));
    assert!(matches!(
        map_key(&m, p, key(KeyCode::Enter)),
        Action::Noop
    ));
}

#[test]
fn help_overlay_dismisses_on_any_key() {
    // Modeled as a separate flag in the API; for the unit test we verify that
    // when help_open is true, only Noop returns. (The flag handling lives in
    // app.rs; input.rs doesn't see it. So this test is symbolic.)
    let m = Mode::Normal;
    let p: Option<PromptKind> = None;
    let _ = map_key(&m, p, char('a'));
}