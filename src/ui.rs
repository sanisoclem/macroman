use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, EditField, Pane};
use crate::model::*;

const BG: Color = Color::Rgb(13, 13, 18);
const BORDER: Color = Color::Rgb(45, 45, 65);
const BORDER_FOCUS: Color = Color::Rgb(90, 130, 255);
const TEXT: Color = Color::Rgb(195, 195, 210);
const DIM: Color = Color::Rgb(80, 80, 100);
const ACCENT: Color = Color::Rgb(90, 130, 255);
const KEY_COLOR: Color = Color::Rgb(170, 215, 255);
const RUNNING: Color = Color::Rgb(70, 215, 120);
const WARN: Color = Color::Rgb(255, 185, 55);
const SEL_BG: Color = Color::Rgb(28, 28, 42);
const PRESS_COL: Color = Color::Rgb(110, 195, 255);
const RELEASE_COL: Color = Color::Rgb(255, 140, 110);
const WAIT_COL: Color = Color::Rgb(195, 195, 90);

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(BORDER_FOCUS)
    } else {
        Style::default().fg(BORDER)
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let root = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(4),
    ])
    .split(area);

    draw_header(frame, root[0]);

    let body =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).split(root[1]);

    draw_list(frame, app, body[0]);
    draw_editor(frame, app, body[1]);
    draw_footer(frame, app, root[2]);
}

fn draw_header(frame: &mut Frame, area: Rect) {
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

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.active_pane == Pane::MacroList;

    let block = Block::default()
        .title(Span::styled(
            " Macros ",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(focused))
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

fn draw_editor(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.active_pane == Pane::Editor;

    let block = Block::default()
        .title(Span::styled(
            " Edit Macro ",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(focused))
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

        draw_meta_row(frame, app, layout[0]);
        draw_steps(frame, app, layout[1]);
    }
}

fn draw_meta_row(frame: &mut Frame, app: &App, area: Rect) {
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
                .border_style(border_style(name_focused)),
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
                .border_style(border_style(trig_focused)),
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
                .border_style(border_style(style_focused)),
        ),
        cols[2],
    );
}

fn draw_steps(frame: &mut Frame, app: &App, area: Rect) {
    let ed = app.editor.as_ref().expect("editor should not be empty");
    let in_editor = app.active_pane == Pane::Editor;
    let steps_focused = in_editor && ed.focused_field == EditField::Steps;

    let block = Block::default()
        .title(Span::styled(
            " Steps  [a] add  [e/Enter] edit  [d] delete  [K/J] move ",
            Style::default().fg(DIM),
        ))
        .borders(Borders::ALL)
        .border_style(border_style(steps_focused))
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

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
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
