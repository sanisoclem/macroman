use crate::{
    app::{App, Pane},
    colors::*,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.active_pane == Pane::MacroList;

    let block = Block::default()
        .title(Span::styled(
            " Macros ",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color(focused)))
        .style(Style::default().bg(BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(inner);

    let items: Vec<ListItem> = app
        .macros
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let selected = i == app.list_cursor;
            let running = app.running_ids.contains(&m.id);

            let dot = if running {
                Span::styled(" ● ", Style::default().fg(RUNNING))
            } else {
                Span::styled(" ○ ", Style::default().fg(DIM))
            };

            let name = Span::styled(
                &m.name,
                if selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(TEXT)
                },
            );

            let meta = Span::styled(
                format!("  {} · {}", m.trigger.display(), m.style.label()),
                Style::default().fg(if selected { KEY_COLOR } else { DIM }),
            );

            ListItem::new(Text::from(vec![
                Line::from(vec![dot, name]),
                Line::from(meta),
            ]))
            .style(if selected {
                Style::default().bg(SEL_BG)
            } else {
                Style::default()
            })
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.list_cursor));
    frame.render_stateful_widget(List::new(items), layout[0], &mut state);

    let cols = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(layout[1]);

    for (col, (key, label)) in cols
        .iter()
        .zip([("[n]", "Add"), ("[d]", "Del"), ("[u]", "Dup")])
    {
        let p = Paragraph::new(Line::from(vec![
            Span::styled(key, Style::default().fg(ACCENT)),
            Span::styled(format!(" {label}"), Style::default().fg(TEXT)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER)),
        )
        .style(Style::default().bg(BG));
        frame.render_widget(p, *col);
    }
}
