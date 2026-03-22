use crate::colors::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(frame: &mut Frame, area: Rect) {
    let p = Paragraph::new(Line::from(vec![
        Span::styled(
            "  MACROMAN ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("·", Style::default().fg(DIM)),
        Span::styled(" macro manager", Style::default().fg(DIM)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER)),
    )
    .style(Style::default().bg(BG));
    frame.render_widget(p, area);
}
