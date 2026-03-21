use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, EditField, Pane, StepEditField, StepEditState, STEP_KINDS};
use crate::model::*;

// TODO: need a way to centrally manage keybinds
//  - need to be easy to change
//  - has API to make it configurable
//  - inspectable so we can show dynamic help text

// TODO: refactor and move logic to app
// input module should only dispatch commands and data to app

pub fn handle_event(app: &mut App, event: Event) -> bool {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Resize(..) => true,
        _ => false,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    let text_mode = is_editing_text(app);

    if !text_mode {
        // not typing, handle global commands
        if let Some(retval) = handle_global(app, &key) {
            return retval;
        }
    }

    match app.active_pane.clone() {
        Pane::MacroList => handle_list(app, key),
        Pane::Editor => handle_editor(app, key),
    }
}

fn is_editing_text(app: &App) -> bool {
    let Some(ed) = &app.editor else { return false };
    if ed.editing_name || ed.editing_trigger {
        return true;
    }
    if let Some(se) = &ed.step_edit {
        return se.field == StepEditField::Value;
    }
    false
}

fn handle_global(app: &mut App, key: &KeyEvent) -> Option<bool> {
    match key {
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.should_quit = true;
            Some(true)
        }
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.active_pane = Pane::MacroList;
            Some(true)
        }
        KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.active_pane = Pane::Editor;
            Some(true)
        }
        KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            ..
        } => match (&app.active_pane, app.editor.as_mut()) {
            (Pane::Editor, Some(ed)) => {
                ed.focused_field = match ed.focused_field {
                    EditField::Name => EditField::TriggerKey,
                    EditField::TriggerKey => EditField::Style,
                    EditField::Style => EditField::Steps,
                    EditField::Steps => EditField::Name,
                };
                Some(true)
            }
            _ => None,
        },
        _ => None,
    }
}

fn handle_list(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.list_up(),
        KeyCode::Down | KeyCode::Char('j') => app.list_down(),
        KeyCode::Enter => app.open_editor(),
        KeyCode::Char('n') => {
            app.add_macro();
            app.set_status("New macro created — press Enter to edit");
        }
        KeyCode::Char('d') => {
            if !app.macros.is_empty() {
                let name = app.macros[app.list_cursor].name.clone();
                app.remove_macro();
                app.set_status(format!("Deleted '{name}'"));
            }
        }
        KeyCode::Char('u') => {
            app.duplicate_macro();
            app.set_status("Macro duplicated");
        }
        _ => return false,
    }
    true
}

fn handle_editor(app: &mut App, key: KeyEvent) -> bool {
    {
        let Some(ed) = &app.editor else { return false };
        if ed.editing_name {
            return edit_name(app, key);
        }
        if ed.editing_trigger {
            return edit_trigger(app, key);
        }
        if ed.step_edit.is_some() {
            return edit_step(app, key);
        }
    }

    let Some(ed) = app.editor.as_mut() else {
        return false;
    };
    match ed.focused_field.clone() {
        EditField::Name => {
            if key.code == KeyCode::Enter {
                ed.editing_name = true;
                true
            } else {
                false
            }
        }
        EditField::TriggerKey => {
            if key.code == KeyCode::Enter {
                ed.editing_trigger = true;
                true
            } else {
                false
            }
        }
        EditField::Style => match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') => {
                ed.style = match ed.style {
                    MacroStyle::Trigger => MacroStyle::ToggleLoop,
                    MacroStyle::ToggleLoop => MacroStyle::Trigger,
                };
                app.commit_editor();
                true
            }
            _ => false,
        },
        EditField::Steps => steps_nav(app, key),
    }
}

fn edit_name(app: &mut App, key: KeyEvent) -> bool {
    let ed = app.editor.as_mut().expect("Editor should not be none");
    match key.code {
        KeyCode::Char(c) => ed.name_buf.push(c),
        KeyCode::Backspace => {
            ed.name_buf.pop();
        }
        KeyCode::Enter | KeyCode::Esc => {
            ed.editing_name = false;
            app.commit_editor();
            app.set_status("Name saved");
        }
        _ => {}
    }
    true
}

fn edit_trigger(app: &mut App, key: KeyEvent) -> bool {
    let ed = app.editor.as_mut().expect("Editor should not be none");
    match key.code {
        KeyCode::Char(c) => ed.trigger_buf.push(c),
        KeyCode::Backspace => {
            ed.trigger_buf.pop();
        }
        KeyCode::Enter | KeyCode::Esc => {
            ed.editing_trigger = false;
            app.commit_editor();
            app.set_status("Trigger key saved — format: [mod+]key  e.g. ctrl+F5");
        }
        _ => {}
    }
    true
}

fn steps_nav(app: &mut App, key: KeyEvent) -> bool {
    let ed = app.editor.as_mut().expect("Editor should not be none");
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if ed.step_cursor > 0 {
                ed.step_cursor -= 1;
            }
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !ed.steps.is_empty() && ed.step_cursor + 1 < ed.steps.len() {
                ed.step_cursor += 1;
            }
            true
        }
        KeyCode::Char('a') => {
            let at = if ed.steps.is_empty() {
                0
            } else {
                ed.step_cursor + 1
            };
            ed.steps.insert(at, StepAction::Wait(100));
            ed.step_cursor = at;
            let kind = ed.steps[at].kind_label().to_string();
            let val = ed.steps[at].value_string();
            ed.step_edit = Some(StepEditState {
                field: StepEditField::Kind,
                kind_buf: kind,
                value_buf: val,
            });
            app.set_status("[←/→] change type  [Space] edit value  [Enter] confirm  [Esc] cancel");
            true
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            if !ed.steps.is_empty() {
                let i = ed.step_cursor;
                let kind = ed.steps[i].kind_label().to_string();
                let val = ed.steps[i].value_string();
                ed.step_edit = Some(StepEditState {
                    field: StepEditField::Kind,
                    kind_buf: kind,
                    value_buf: val,
                });
            }
            true
        }
        KeyCode::Char('d') => {
            if !ed.steps.is_empty() {
                ed.steps.remove(ed.step_cursor);
                if ed.step_cursor > 0 && ed.step_cursor >= ed.steps.len() {
                    ed.step_cursor -= 1;
                }
                app.commit_editor();
                app.set_status("Step deleted");
            }
            true
        }
        KeyCode::Char('K') => {
            let i = ed.step_cursor;
            if i > 0 {
                ed.steps.swap(i, i - 1);
                ed.step_cursor -= 1;
                app.commit_editor();
            }
            true
        }
        KeyCode::Char('J') => {
            let i = ed.step_cursor;
            if i + 1 < ed.steps.len() {
                ed.steps.swap(i, i + 1);
                ed.step_cursor += 1;
                app.commit_editor();
            }
            true
        }
        _ => false,
    }
}

fn edit_step(app: &mut App, key: KeyEvent) -> bool {
    let ed = app.editor.as_mut().expect("Editor should not be none");
    let se = ed
        .step_edit
        .as_mut()
        .expect("stepd edit state should not be none");

    match se.field {
        StepEditField::Kind => match key {
            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                se.kind_buf = cycle_kind(&se.kind_buf, -1);
                se.value_buf = StepAction::default_value_for_kind(&se.kind_buf).to_string();
            }
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                se.kind_buf = cycle_kind(&se.kind_buf, 1);
                se.value_buf = StepAction::default_value_for_kind(&se.kind_buf).to_string();
            }
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
                ..
            } => se.field = StepEditField::Value,
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                commit_step(app);
                app.set_status("Step saved");
            }
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => cancel_step(app),
            _ => {}
        },
        StepEditField::Value => match key {
            KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => se.value_buf.push(c),
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                se.value_buf.pop();
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                commit_step(app);
                app.set_status("Step saved");
            }
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => cancel_step(app),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
                ..
            } => se.field = StepEditField::Kind,
            _ => {}
        },
    }
    true
}

fn commit_step(app: &mut App) {
    let ed = app.editor.as_mut().unwrap();
    if let Some(se) = ed.step_edit.take() {
        let step = StepAction::build(&se.kind_buf, &se.value_buf);
        if ed.step_cursor < ed.steps.len() {
            ed.steps[ed.step_cursor] = step;
        }
        app.commit_editor();
    }
}

fn cancel_step(app: &mut App) {
    let ed = app.editor.as_mut().unwrap();
    ed.step_edit = None;
    app.commit_editor();
}

fn cycle_kind(current: &str, delta: i32) -> String {
    // TODO: make a dropdown?
    let pos = STEP_KINDS.iter().position(|&k| k == current).unwrap_or(0) as i32;
    let len = STEP_KINDS.len() as i32;
    STEP_KINDS[((pos + delta).rem_euclid(len)) as usize].to_string()
}
