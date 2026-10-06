//! ratatui rendering for the main view.
//!
//! Layout:
//!   header  ── "t2d2"
//!   list    ── one or two rows per task, scrollable
//!   status  ── mode + counts + abbreviated hints

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::{EditField, Mode, Store, Task};

const GREY: Color = Color::DarkGray;

pub fn render(frame: &mut Frame, store: &Store, mode: &Mode, status_hint: &str) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(1),    // list
            Constraint::Length(1), // statusline
        ])
        .split(area);

    render_header(frame, chunks[0]);
    render_list(frame, chunks[1], store, mode);
    render_statusline(frame, chunks[2], store, mode, status_hint);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::raw("t2d2"));
    frame.render_widget(Paragraph::new(line), area);
}

fn render_list(frame: &mut Frame, area: Rect, store: &Store, mode: &Mode) {
    if store.tasks.is_empty() {
        let hint = Line::from(Span::raw("── no tasks — press o to add ──"));
        let p = Paragraph::new(hint)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default());
        // Center vertically inside the area.
        let vcenter = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(50), Constraint::Length(1), Constraint::Percentage(50)],
        )
        .split(area);
        frame.render_widget(p, vcenter[1]);
        return;
    }

    // Compute total visible rows so we can scroll.
    let rows = build_rows(store, mode);
    let cursor_row = rows.iter().position(|(_, is_cursor, _)| *is_cursor).unwrap_or(0);

    let visible_height = area.height as usize;
    let scroll = if cursor_row >= visible_height {
        cursor_row + 1 - visible_height
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (text, is_cursor, is_completed) in rows.iter().skip(scroll).take(visible_height) {
        let mut style = Style::default();
        if *is_completed {
            style = style
                .fg(GREY)
                .add_modifier(Modifier::CROSSED_OUT);
        }
        if *is_cursor {
            style = style.add_modifier(Modifier::REVERSED);
        }
        lines.push(Line::from(Span::styled(text.clone(), style)));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Build the (text, is_cursor, is_completed) tuples that the list renders.
/// One tuple per visual row.
fn build_rows(store: &Store, mode: &Mode) -> Vec<(String, bool, bool)> {
    let mut out = Vec::new();

    // The task currently being edited in note-mode, if any.
    let editing_note_id = match mode {
        Mode::Insert {
            field: EditField::Note { task_id, .. },
        } => Some(*task_id),
        _ => None,
    };

    for (i, task) in store.tasks.iter().enumerate() {
        let is_cursor = i == store.cursor;

        // Title row.
        let marker = if is_cursor { '>' } else { ' ' };
        let title = if task.title.is_empty() {
            " ".to_string()
        } else {
            task.title.clone()
        };
        out.push((format!("{marker} {title}"), is_cursor, task.completed));

        // Note row(s).
        if Some(task.id) == editing_note_id {
            // Expand the note into the editor's rows.
            if let Mode::Insert {
                field: EditField::Note { buffer, .. },
            } = mode
            {
                for line in buffer {
                    out.push((format!("  - {line}"), is_cursor, task.completed));
                }
            }
        } else if !task.note.is_empty() {
            let preview = task.note.lines().next().unwrap_or("");
            out.push((format!("  - {preview}"), is_cursor, task.completed));
        }
    }

    out
}

fn render_statusline(frame: &mut Frame, area: Rect, store: &Store, mode: &Mode, hint: &str) {
    let completed = store.tasks.iter().filter(|t| t.completed).count();
    let total = store.tasks.len();
    let count = format!("{total} tasks ({completed} done)");
    let mode_str = match mode {
        Mode::Normal => "-- NORMAL --",
        Mode::Insert {
            field: EditField::Title { .. },
        } => "-- INSERT (title) --",
        Mode::Insert {
            field: EditField::Note { .. },
        } => "-- INSERT (note) --",
    };
    let line = Line::from(vec![
        Span::raw(mode_str),
        Span::raw(" "),
        Span::raw(count),
        Span::raw(" "),
        Span::raw(hint),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

// Suppress unused warning for the Wrap import; kept for future use.
#[allow(dead_code)]
fn _unused() {
    let _: Wrap = Wrap::default();
    let _: Task = Task {
        id: 0,
        title: String::new(),
        note: String::new(),
        completed: false,
    };
}