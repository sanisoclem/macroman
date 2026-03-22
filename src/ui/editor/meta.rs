use crate::{
    app::{App, EditField, Pane},
    colors::*,
    model::MacroStyle,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    // editor is part of app, so we can't pass unwrapped
    // TODO: maybe we should break apart app?
    let ed = app
        .editor
        .as_ref()
        .expect("editor should always have a value");
    let in_editor = app.active_pane == Pane::Editor;

    let cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(area);

    let name_focused = in_editor && ed.focused_field == EditField::Name;
    let name_text = if ed.editing_name {
        // fake editing,
        // TODO: need a vim like editor?
        format!("{}▌", ed.name_buf)
    } else {
        ed.name_buf.clone()
    };
    frame.render_widget(
        Paragraph::new(Span::styled(name_text, Style::default().fg(Color::White))).block(
            Block::default()
                .title(Span::styled(" Name ", Style::default().fg(DIM)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color(name_focused))),
        ),
        cols[0],
    );

    let trig_focused = in_editor && ed.focused_field == EditField::TriggerKey;
    let trig_text = if ed.editing_trigger {
        // same fakeness
        format!("{}▌", ed.trigger_buf)
    } else {
        ed.trigger_buf.clone()
    };
    frame.render_widget(
        Paragraph::new(Span::styled(trig_text, Style::default().fg(KEY_COLOR))).block(
            Block::default()
                .title(Span::styled(" Trigger Key ", Style::default().fg(DIM)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color(trig_focused))),
        ),
        cols[1],
    );

    let style_focused = in_editor && ed.focused_field == EditField::Style;
    let (a, b) = match ed.style {
        MacroStyle::Trigger => ("▶ Trigger", "  Toggle Loop"),
        MacroStyle::ToggleLoop => ("  Trigger", "▶ Toggle Loop"),
    };
    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::from(Span::styled(a, Style::default().fg(TEXT))),
            Line::from(Span::styled(b, Style::default().fg(TEXT))),
        ]))
        .block(
            Block::default()
                .title(Span::styled(" Style ", Style::default().fg(DIM)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color(style_focused))),
        ),
        cols[2],
    );
}
