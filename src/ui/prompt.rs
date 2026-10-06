//! The "delete all?" confirm bar. Rendered in the statusline area when active.

use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect) {
    let line = Line::from(Span::raw("-- delete all? (y/n) --"));
    frame.render_widget(Paragraph::new(line), area);
}