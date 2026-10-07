//! ratatui rendering for the main view.
//!
//! Layout:
//!   header  ── "t2d2"
//!   list    ── one or two rows per task, scrollable
//!   status  ── mode + counts + abbreviated hints

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::{EditField, Mode, PromptKind, Store, Task};

const GREY: Color = Color::DarkGray;

pub fn render(
    frame: &mut Frame,
    store: &Store,
    mode: &Mode,
    prompt: Option<PromptKind>,
    status_hint: &str,
) {
    let area = frame.area();

    // Reserve the last row for the statusline (outside the bordered block).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // bordered block
            Constraint::Length(1), // statusline (outside the border)
        ])
        .split(area);

    let block_area = chunks[0];
    let status_area = chunks[1];

    // Bordered block with inner padding: 1 top/bottom, 2 left/right.
    let block = Block::default()
        .borders(Borders::ALL)
        .title("t2d2")
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(block_area);
    frame.render_widget(block, block_area);

    render_list(frame, inner, store, mode);
    render_statusline(frame, status_area, store, mode, prompt, status_hint);

    place_cursor(frame, inner, store, mode);
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
    let cursor_row = rows
        .iter()
        .position(|line| line.spans.first().map(|s| s.content == "> ").unwrap_or(false))
        .unwrap_or(0);

    let visible_height = area.height as usize;
    let scroll = if cursor_row >= visible_height {
        cursor_row + 1 - visible_height
    } else {
        0
    };

    let lines: Vec<Line> = rows.into_iter().skip(scroll).take(visible_height).collect();
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Build the rendered lines for the list. Each line is a sequence of spans:
/// the cursor marker (if any), then the title or note content. The cursor
/// marker is never struck through, even when the task is completed.
fn note_preview(note: &str) -> String {
    let mut lines = note.split('\n');
    let first = lines.next().unwrap_or("");
    let has_more = lines.next().is_some();
    if has_more {
        format!("{first}…")
    } else {
        first.to_string()
    }
}

fn build_rows<'a>(store: &'a Store, mode: &'a Mode) -> Vec<Line<'a>> {
    let mut out = Vec::new();

    // The task currently being edited in note-mode, if any.
    let editing_note_id: Option<u64> = match mode {
        Mode::Insert {
            field: EditField::Note { task_id, .. },
        } => Some(*task_id),
        _ => None,
    };

    // The buffer for the task being edited in title-mode, if any.
    let editing_title: Option<(u64, String)> = match mode {
        Mode::Insert {
            field: EditField::Title {
                task_id, buffer, ..
            },
        } => Some((*task_id, buffer.clone())),
        _ => None,
    };

    let body_style_for = |completed: bool| -> Style {
        if completed {
            Style::default().fg(GREY).add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default()
        }
    };

    let title_style_for = |completed: bool, is_cursor: bool| -> Style {
        match (completed, is_cursor) {
            (false, false) => Style::default(),
            (false, true) => Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
            (true, false) => Style::default()
                .fg(GREY)
                .add_modifier(Modifier::CROSSED_OUT),
            (true, true) => Style::default()
                .fg(GREY)
                .add_modifier(Modifier::CROSSED_OUT)
                .add_modifier(Modifier::BOLD),
        }
    };

    for (i, task) in store.tasks.iter().enumerate() {
        let is_cursor = i == store.cursor;
        let marker: String = if is_cursor { "> ".to_string() } else { "  ".to_string() };
        let body = body_style_for(task.completed);
        let title_style = title_style_for(task.completed, is_cursor);

        // Title row.
        let title_text = if let Some((id, buf)) = &editing_title {
            if *id == task.id {
                buf.clone()
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
        out.push(Line::from(vec![
            Span::styled(marker, Style::default()),
            Span::styled(title, title_style),
        ]));

        // Note row(s).
        if Some(task.id) == editing_note_id {
            if let Mode::Insert {
                field: EditField::Note { buffer, .. },
            } = mode
            {
                for line in buffer {
                    out.push(Line::from(vec![
                        Span::styled("  ".to_string(), Style::default()),
                        Span::styled("- ".to_string(), Style::default()),
                        Span::styled(line.clone(), body),
                    ]));
                }
            }
        } else if !task.note.is_empty() {
            let preview = note_preview(&task.note);
            out.push(Line::from(vec![
                Span::styled("  ".to_string(), Style::default()),
                Span::styled("- ".to_string(), Style::default()),
                Span::styled(preview, body),
            ]));
        }
    }

    out
}

fn mode_label_style(mode: &Mode) -> Style {
    let bg = match mode {
        Mode::Normal => Color::LightGreen,
        Mode::Insert { .. } => Color::LightBlue,
    };
    Style::default().bg(bg).fg(Color::Black)
}

fn mode_label(mode: &Mode) -> &'static str {
    match mode {
        Mode::Normal => " NORMAL ",
        Mode::Insert { .. } => " INSERT ",
    }
}

fn prompt_label_style() -> Style {
    Style::default().bg(Color::Yellow).fg(Color::Black)
}

fn prompt_label(p: PromptKind) -> &'static str {
    match p {
        PromptKind::DeleteAll => " DELETE ",
    }
}

fn render_statusline(
    frame: &mut Frame,
    area: Rect,
    store: &Store,
    mode: &Mode,
    prompt: Option<PromptKind>,
    hint: &str,
) {
    let completed = store.tasks.iter().filter(|t| t.completed).count();
    let total = store.tasks.len();
    let count = format!("{total} tasks ({completed} done)");
    let pill_text = match prompt {
        Some(p) => prompt_label(p),
        None => mode_label(mode),
    };
    let pill_style = match prompt {
        Some(_) => prompt_label_style(),
        None => mode_label_style(mode),
    };
    let line = Line::from(vec![
        Span::styled(pill_text, pill_style),
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
