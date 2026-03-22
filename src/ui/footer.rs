use crate::{app::App, colors::*};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let running = app.running_ids.len();
    let mut spans = vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.status_msg, Style::default().fg(TEXT)),
        Span::styled(" ", Style::default()),
    ];
    //  TODO: separate into multiple sections?
    if running > 0 {
        spans.push(Span::styled(
            format!("● {running} running"),
            Style::default().fg(RUNNING),
        ));
    }
    let p = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER)),
        )
        .style(Style::default().bg(BG));
    frame.render_widget(p, area);
}
