use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::model::{MacroDef, MacroStyle, StepAction};
use crate::platform::InputDriver;

use crate::platform::PlatformDriver as DriverImpl;

pub enum ExecCmd {
    Trigger(MacroDef),
    Stop(String),
    StopAll,
}

pub enum ExecEvent {
    MacroStarted(String),
    MacroStopped(String),
    MacroError { id: String, msg: String },
    Info(String),
}

struct RunHandle {
    stop_tx: tokio::sync::oneshot::Sender<()>,
}

pub struct Executor {
    cmd_tx: mpsc::UnboundedSender<ExecCmd>,
}

impl Executor {
    pub fn new(event_tx: mpsc::UnboundedSender<ExecEvent>) -> Self {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<ExecCmd>();

        tokio::spawn(async move {
            let mut running: HashMap<String, RunHandle> = HashMap::new();

            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    ExecCmd::Trigger(m) => {
                        let id = m.id.clone();
                        match m.style {
                            MacroStyle::Trigger => {
                                if running.contains_key(&id) {
                                    // macro already running
                                    continue;
                                }
                                let handle = spawn_macro(m, event_tx.clone(), false);
                                running.insert(id, handle);
                            }
                            MacroStyle::ToggleLoop => {
                                if let Some(h) = running.remove(&id) {
                                    let _ = h.stop_tx.send(());
                                } else {
                                    let handle = spawn_macro(m, event_tx.clone(), true);
                                    running.insert(id, handle);
                                }
                            }
                        }
                    }
                    ExecCmd::Stop(id) => {
                        if let Some(h) = running.remove(&id) {
                            let _ = h.stop_tx.send(());
                        }
                    }
                    ExecCmd::StopAll => {
                        for (_, h) in running.drain() {
                            let _ = h.stop_tx.send(());
                        }
                        break;
                    }
                }
            }
        });

        Self { cmd_tx }
    }

    pub fn send(&self, cmd: ExecCmd) {
        let _ = self.cmd_tx.send(cmd);
    }
}

fn spawn_macro(
    macro_def: MacroDef,
    event_tx: mpsc::UnboundedSender<ExecEvent>,
    looping: bool,
) -> RunHandle {
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    let id = macro_def.id.clone();
    let _ = event_tx.send(ExecEvent::MacroStarted(id.clone()));

    std::thread::spawn(move || {
        let mut driver = match DriverImpl::create() {
            Ok(d) => d,
            Err(e) => {
                let _ = event_tx.send(ExecEvent::MacroError {
                    id: id.clone(),
                    msg: e.to_string(),
                });
                return;
            }
        };

        'wow: loop {
            for step in &macro_def.steps {
                if stop_rx.try_recv().is_ok() {
                    break 'wow;
                }

                let result = match step {
                    StepAction::PressKey(k) => driver.press_key(k),
                    StepAction::ReleaseKey(k) => driver.release_key(k),
                    StepAction::Wait(ms) => {
                        driver.wait_ms(*ms);
                        Ok(())
                    }
                };

                if let Err(e) = result {
                    let _ = event_tx.send(ExecEvent::MacroError {
                        id: id.clone(),
                        msg: e.to_string(),
                    });
                    return;
                }
            }

            if !looping {
                break;
            }
        }

        let _ = event_tx.send(ExecEvent::MacroStopped(id));
    });

    RunHandle { stop_tx }
}
