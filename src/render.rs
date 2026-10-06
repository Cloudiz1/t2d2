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

use crate::state::{EditField, Mode, PromptKind, Store, Task};
use crate::ui::prompt;

const GREY: Color = Color::DarkGray;

pub fn render(
    frame: &mut Frame,
    store: &Store,
    mode: &Mode,
    prompt: Option<PromptKind>,
    status_hint: &str,
) {
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
    if prompt.is_some() {
        prompt::render(frame, chunks[2]);
    } else {
        render_statusline(frame, chunks[2], store, mode, status_hint);
    }
    place_cursor(frame, chunks[1], store, mode);
}

/// Place the terminal caret at the cursor position when in insert mode.
fn place_cursor(frame: &mut Frame, list_area: Rect, store: &Store, mode: &Mode) {
    let Mode::Insert { field } = mode else { return };

    // Find the visual row of the current task, accounting for any rows above it.
    let editing_id = match field {
        EditField::Title { task_id, .. } => *task_id,
        EditField::Note { task_id, .. } => *task_id,
    };

    let Some(cursor_pos) = store.tasks.iter().position(|t| t.id == editing_id) else {
        return;
    };

    // Compute the visual-row offset: 1 for the title row, plus 1 if a note row
    // exists or the note is being edited.
    let mut visual_row_offset: u16 = 0;
    for (i, task) in store.tasks.iter().enumerate() {
        if i == cursor_pos {
            break;
        }
        visual_row_offset += 1;
        if !task.note.is_empty() {
            visual_row_offset += 1;
        }
    }

    let visual_col_in_list: u16 = match field {
        EditField::Title { cursor, .. } => {
            // Title row: "> {title}". Cursor sits at the character position
            // after "> ".
            2 + *cursor as u16
        }
        EditField::Note {
            cursor_row,
            cursor_col,
            ..
        } => {
            // The title row is one, then note rows start.
            visual_row_offset += 1;
            visual_row_offset += *cursor_row as u16;
            // Note row prefix is "  - ".
            4 + *cursor_col as u16
        }
    };

    let y = list_area.y + visual_row_offset;
    let x = list_area.x + visual_col_in_list;
    if y < list_area.y + list_area.height {
        frame.set_cursor_position(ratatui::layout::Position { x, y });
    }
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

    // The buffer + cursor for the task being edited in title-mode, if any.
    let editing_title: Option<(u64, &str, usize)> = match mode {
        Mode::Insert {
            field: EditField::Title {
                task_id,
                buffer,
                cursor,
            },
        } => Some((*task_id, buffer.as_str(), *cursor)),
        _ => None,
    };

    for (i, task) in store.tasks.iter().enumerate() {
        let is_cursor = i == store.cursor;

        // Title row.
        let marker = if is_cursor { '>' } else { ' ' };
        let title_text = if let Some((id, buf, _)) = editing_title {
            if id == task.id {
                buf.to_string()
            } else {
                task.title.clone()
            }
        } else {
            task.title.clone()
        };
        let title = if title_text.is_empty() {
            " ".to_string()
        } else {
            title_text
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