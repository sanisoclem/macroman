use serde::{Deserialize, Serialize};
use uuid::Uuid;

const STEP_KINDS: &[&str] = &["PressKey", "ReleaseKey", "Wait"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyCombo {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl KeyCombo {
    pub fn empty() -> Self {
        Self {
            modifiers: vec![],
            key: String::new(),
        }
    }

    pub fn display(&self) -> String {
        if self.key.is_empty() {
            return "(unset)".into();
        }
        let mut parts = self.modifiers.clone();
        parts.push(self.key.clone());
        parts.join("+")
    }

    pub fn parse(s: &str) -> Self {
        let parts: Vec<String> = s
            .split('+')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
        if parts.is_empty() {
            return Self::empty();
        }
        Self {
            modifiers: parts[..parts.len() - 1].to_vec(),
            key: parts.last().cloned().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroStyle {
    Trigger,
    ToggleLoop,
}

impl MacroStyle {
    pub fn label(&self) -> &'static str {
        match self {
            MacroStyle::Trigger => "Trigger",
            MacroStyle::ToggleLoop => "Toggle Loop",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepAction {
    PressKey(String),
    ReleaseKey(String),
    Wait(u64),
}

impl StepAction {
    pub fn display(&self) -> String {
        match self {
            // TODO: how to handle vertical alignment with adjustable column widths?
            StepAction::PressKey(k) => format!("Press    {k}"),
            StepAction::ReleaseKey(k) => format!("Release  {k}"),
            StepAction::Wait(ms) => format!("Wait     {ms} ms"),
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            StepAction::PressKey(_) => "PressKey",
            StepAction::ReleaseKey(_) => "ReleaseKey",
            StepAction::Wait(_) => "Wait",
        }
    }

    pub fn value_string(&self) -> String {
        match self {
            StepAction::PressKey(k) => k.clone(),
            StepAction::ReleaseKey(k) => k.clone(),
            StepAction::Wait(ms) => ms.to_string(),
        }
    }

    pub fn build(kind: &str, value: &str) -> Self {
        match kind {
            "PressKey" => StepAction::PressKey(value.trim().to_string()),
            "ReleaseKey" => StepAction::ReleaseKey(value.trim().to_string()),
            "Wait" => StepAction::Wait(value.trim().parse().unwrap_or(100)),
            _ => StepAction::Wait(0),
        }
    }

    pub fn default_value_for_kind(kind: &str) -> &'static str {
        match kind {
            "PressKey" | "ReleaseKey" => "a",
            _ => "100",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDef {
    pub id: String,
    pub name: String,
    pub trigger: KeyCombo,
    pub style: MacroStyle,
    pub steps: Vec<StepAction>,
}

impl MacroDef {
    pub fn new_empty(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            trigger: KeyCombo::empty(),
            style: MacroStyle::Trigger,
            steps: vec![],
        }
    }
}
