use crate::model::*;
use std::collections::HashSet;
use uuid::Uuid;

pub const STEP_KINDS: &[&str] = &["PressKey", "ReleaseKey", "Wait"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditField {
    Name,
    TriggerKey,
    Style,
    Steps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepEditField {
    Kind,
    Value,
}

#[derive(Debug, Clone)]
pub struct StepEditState {
    pub field: StepEditField,
    pub kind_buf: String,
    pub value_buf: String,
}

#[derive(Debug, Clone)]
pub struct EditorState {
    pub macro_id: String,
    pub name_buf: String,
    pub trigger_buf: String,
    pub style: MacroStyle,
    pub steps: Vec<StepAction>,
    pub focused_field: EditField,
    pub step_cursor: usize,
    pub step_edit: Option<StepEditState>,
    pub editing_name: bool,
    pub editing_trigger: bool,
}

impl EditorState {
    pub fn from_macro(m: &MacroDef) -> Self {
        Self {
            macro_id: m.id.clone(),
            name_buf: m.name.clone(),
            trigger_buf: m.trigger.display(),
            style: m.style.clone(),
            steps: m.steps.clone(),
            focused_field: EditField::Name,
            step_cursor: 0,
            step_edit: None,
            editing_name: false,
            editing_trigger: false,
        }
    }

    pub fn apply_to(&self, m: &mut MacroDef) {
        m.name = self.name_buf.clone();
        m.style = self.style.clone();
        m.steps = self.steps.clone();
        m.trigger = KeyCombo::parse(&self.trigger_buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pane {
    MacroList,
    Editor,
}

// TODO: add application modes
//  - text editing mode
//  - selection mode
//  - ??
pub struct App {
    pub macros: Vec<MacroDef>,
    pub list_cursor: usize,
    pub active_pane: Pane,
    pub editor: Option<EditorState>,
    pub running_ids: HashSet<String>,
    pub status_msg: String,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            macros: vec![],
            list_cursor: 0,
            active_pane: Pane::MacroList,
            editor: None,
            running_ids: HashSet::new(),
            status_msg: "Ready — [n] new  [Enter] edit  [F5] run  [q] quit".into(),
            should_quit: false,
        };

        let mut m1 = MacroDef::new_empty("Test 1");
        m1.trigger = KeyCombo::parse("ctrl+F1");
        m1.style = MacroStyle::Trigger;
        m1.steps = vec![
            StepAction::PressKey("ctrl".into()),
            StepAction::PressKey("c".into()),
            StepAction::Wait(50),
            StepAction::ReleaseKey("c".into()),
            StepAction::ReleaseKey("ctrl".into()),
        ];
        app.macros.push(m1);

        let mut m2 = MacroDef::new_empty("Test 2");
        m2.trigger = KeyCombo::parse("F2");
        m2.style = MacroStyle::ToggleLoop;
        m2.steps = vec![
            StepAction::PressKey("num_0".into()),
            StepAction::Wait(30),
            StepAction::ReleaseKey("num_0".into()),
            StepAction::Wait(30),
        ];
        app.macros.push(m2);

        app
    }

    pub fn add_macro(&mut self) {
        let n = self.macros.len() + 1;
        let m = MacroDef::new_empty(format!("Macro {n}"));
        self.macros.push(m);
        self.list_cursor = self.macros.len() - 1;
        self.open_editor();
    }

    pub fn remove_macro(&mut self) {
        if self.macros.is_empty() {
            return;
        }
        let id = self.macros[self.list_cursor].id.clone();
        self.macros.remove(self.list_cursor);
        if self.list_cursor > 0 && self.list_cursor >= self.macros.len() {
            self.list_cursor = self.macros.len().saturating_sub(1);
        }
        if matches!(&self.editor, Some(ed) if ed.macro_id == id) {
            self.editor = None;
        }
    }

    pub fn duplicate_macro(&mut self) {
        if self.macros.is_empty() {
            return;
        }
        let mut m = self.macros[self.list_cursor].clone();
        m.id = Uuid::new_v4().to_string();
        m.name = format!("{} (copy)", m.name);
        let at = self.list_cursor + 1;
        self.macros.insert(at, m);
        self.list_cursor = at;
    }

    pub fn open_editor(&mut self) {
        if let Some(m) = self.macros.get(self.list_cursor) {
            self.editor = Some(EditorState::from_macro(m));
            self.active_pane = Pane::Editor;
        }
    }

    pub fn commit_editor(&mut self) {
        let Some(ed) = self.editor.clone() else {
            return;
        };
        let Some(m) = self.macros.iter_mut().find(|m| m.id == ed.macro_id) else {
            return;
        };
        ed.apply_to(m);
    }

    pub fn list_up(&mut self) {
        if self.list_cursor > 0 {
            self.list_cursor -= 1;
        }
    }

    pub fn list_down(&mut self) {
        if !self.macros.is_empty() && self.list_cursor < self.macros.len() - 1 {
            self.list_cursor += 1;
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = msg.into();
    }
}
