mod meta;
mod steps;

use crate::{
    app::{App, Pane},
    colors::*,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.active_pane == Pane::Editor;

    let block = Block::default()
        .title(Span::styled(
            " Edit Macro ",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color(focused)))
        .style(Style::default().bg(BG));

    if app.editor.is_none() {
        let p = Paragraph::new(Line::from(vec![
            Span::styled("Select a macro and press ", Style::default().fg(DIM)),
            Span::styled("[Enter]", Style::default().fg(ACCENT)),
            Span::styled(" to edit", Style::default().fg(DIM)),
        ]))
        .block(block);
        frame.render_widget(p, area);
    } else {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).split(inner);

        meta::draw(frame, app, layout[0]);
        steps::draw(frame, app, layout[1]);
    }
}
