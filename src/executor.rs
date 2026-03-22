use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot::error::TryRecvError;

use crate::model::{MacroDef, MacroStyle, StepAction};
use crate::platform::InputDriver;
use crate::platform::PlatformDriver as DriverImpl;

pub enum ExecutorCommand {
    Trigger(MacroDef),
    StopAll,
}

pub enum ExecutorEvent {
    MacroStarted(String),
    MacroStopped(String),
    MacroError { id: String, msg: String },
    CommandThreadTerminated,
}

struct RunHandle {
    stop_tx: tokio::sync::oneshot::Sender<()>,
}

pub struct Executor {
    cmd_tx: mpsc::UnboundedSender<ExecutorCommand>,
}

impl Executor {
    pub fn new(event_tx: mpsc::UnboundedSender<ExecutorEvent>) -> Self {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<ExecutorCommand>();

        tokio::spawn(async move {
            let mut running: HashMap<String, RunHandle> = HashMap::new();

            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    ExecutorCommand::Trigger(m) => {
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
                    ExecutorCommand::StopAll => {
                        for (_, h) in running.drain() {
                            let _ = h.stop_tx.send(());
                        }
                        break;
                    }
                }
            }

            let _ = event_tx.send(ExecutorEvent::CommandThreadTerminated);
        });

        Self { cmd_tx }
    }

    pub fn send(&self, cmd: ExecutorCommand) {
        let _ = self.cmd_tx.send(cmd);
    }
}

fn spawn_macro(
    macro_def: MacroDef,
    event_tx: mpsc::UnboundedSender<ExecutorEvent>,
    looping: bool,
) -> RunHandle {
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    let id = macro_def.id.clone();
    let _ = event_tx.send(ExecutorEvent::MacroStarted(id.clone()));

    std::thread::spawn(move || {
        let mut driver = match DriverImpl::create() {
            Ok(d) => d,
            Err(e) => {
                let _ = event_tx.send(ExecutorEvent::MacroError {
                    id: id.clone(),
                    msg: e.to_string(),
                });
                return;
            }
        };

        'wow: loop {
            for step in &macro_def.steps {
                // break when signal received or tx is closed
                match stop_rx.try_recv() {
                    Err(TryRecvError::Closed) | Ok(_) => break 'wow,
                    _ => {}
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
                    let _ = event_tx.send(ExecutorEvent::MacroError {
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

        let _ = event_tx.send(ExecutorEvent::MacroStopped(id));
    });

    RunHandle { stop_tx }
}
