//! Full-screen help overlay.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const KEYMAP: &[(&str, &str)] = &[
    ("NORMAL", ""),
    ("  j / k", "move cursor"),
    ("  gg / G", "first / last task"),
    ("  o / O", "new task below / above"),
    ("  a / i", "edit title (append / insert)"),
    ("  Tab", "edit note"),
    ("  c", "toggle complete"),
    ("  d", "delete"),
    ("  D", "delete all"),
    ("  J / K", "move task down / up"),
    ("  u", "undo"),
    ("  ?", "this help"),
    ("  Ctrl+Q", "quit"),
    ("", ""),
    ("INSERT (title)", ""),
    ("  Esc / Enter", "save & exit"),
    ("  Tab", "switch to note"),
    ("  Ctrl+U", "kill line"),
    ("  Ctrl+W", "kill word"),
    ("", ""),
    ("INSERT (note)", ""),
    ("  Esc", "save & exit"),
    ("  Enter", "newline"),
    ("  Ctrl+U", "kill line"),
    ("  Ctrl+W", "kill word"),
    ("  Up / Down", "move between lines"),
    ("", ""),
    ("any key dismisses", ""),
];

pub fn render(frame: &mut Frame) {
    let area = centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title("t2d2 — keymap");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for (k, v) in KEYMAP {
        if k.is_empty() && v.is_empty() {
            lines.push(Line::raw(""));
        } else if v.is_empty() {
            lines.push(Line::from(Span::styled(
                *k,
                Style::default().add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled(*k, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::raw(*v),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}