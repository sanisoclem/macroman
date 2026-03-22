use crate::{
    app::{App, EditField, Pane},
    colors::*,
    model::StepAction,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let ed = app.editor.as_ref().expect("editor should not be empty");
    let in_editor = app.active_pane == Pane::Editor;
    let steps_focused = in_editor && ed.focused_field == EditField::Steps;

    let block = Block::default()
        .title(Span::styled(
            " Steps  [a] add  [e/Enter] edit  [d] delete  [K/J] move ",
            Style::default().fg(DIM),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color(steps_focused)))
        .style(Style::default().bg(BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([Constraint::Min(0), Constraint::Length(30)]).split(inner);

    let items: Vec<ListItem> = if ed.steps.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  No steps yet — press [a] to add one",
            Style::default().fg(DIM),
        )))]
    } else {
        ed.steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let is_cur = i == ed.step_cursor && steps_focused;
                let editing = ed.step_edit.as_ref().filter(|_| is_cur);

                let prefix = if is_cur { "▶" } else { " " };
                let num =
                    Span::styled(format!("{prefix} {:>2}. ", i + 1), Style::default().fg(DIM));

                let kind_color = match step {
                    StepAction::PressKey(_) => PRESS_COL,
                    StepAction::ReleaseKey(_) => RELEASE_COL,
                    StepAction::Wait(_) => WAIT_COL,
                };
                let (body, color) = if let Some(se) = editing {
                    let temp = StepAction::build(&se.kind_buf, &se.value_buf);
                    (temp.display(), WARN)
                } else {
                    (step.display(), kind_color)
                };

                ListItem::new(Line::from(vec![
                    num,
                    Span::styled(body, Style::default().fg(color)),
                ]))
                .style(if is_cur {
                    Style::default().bg(SEL_BG)
                } else {
                    Style::default()
                })
            })
            .collect()
    };

    let mut state = ListState::default();
    if steps_focused && !ed.steps.is_empty() {
        state.select(Some(ed.step_cursor));
    }
    frame.render_stateful_widget(List::new(items), cols[0], &mut state);

    // TODO: how can we centralized management of keybinds
    let hints: &[(&str, &str)] = if ed.step_edit.is_some() {
        &[("TODO", "editing keybind hints")]
    } else {
        &[("TODO", "focus keybind hints")]
    };

    let lines: Vec<Line> = hints
        .iter()
        .map(|(k, v)| {
            if k.is_empty() {
                Line::from("")
            } else {
                // TODO: find better way to deal with vertical alignment
                Line::from(vec![
                    Span::styled(format!("{k:<6}"), Style::default().fg(ACCENT)),
                    Span::styled(*v, Style::default().fg(DIM)),
                ])
            }
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(Span::styled(" Keys ", Style::default().fg(DIM)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER)),
        ),
        cols[1],
    );
}
