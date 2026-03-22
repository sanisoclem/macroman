mod editor;
mod footer;
mod header;
mod list;

use ratatui::prelude::*;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let root = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(4),
    ])
    .split(area);

    header::draw(frame, root[0]);

    let body =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).split(root[1]);

    list::draw(frame, app, body[0]);
    editor::draw(frame, app, body[1]);
    footer::draw(frame, app, root[2]);
}
