mod app;
mod executor;
mod input;
mod model;
mod platform;
mod ui;

use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use app::App;
use executor::{ExecCmd, ExecEvent, Executor};

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let result = run(&mut term).await;

    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    if let Err(e) = &result {
        eprintln!("macropan error: {e}");
    }
    result
}

async fn run(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    let (ev_tx, mut ev_rx) = mpsc::unbounded_channel::<ExecEvent>();
    let executor = Executor::new(ev_tx);

    loop {
        term.draw(|f| ui::draw(f, &app))?;

        while let Ok(ev) = ev_rx.try_recv() {
            match ev {
                ExecEvent::MacroStarted(id) => {
                    app.running_ids.insert(id);
                    app.set_status("Macro started");
                }
                ExecEvent::MacroStopped(id) => {
                    app.running_ids.remove(&id);
                    app.set_status("Macro stopped");
                }
                ExecEvent::MacroError { id, msg } => {
                    // TODO: toasts
                    app.running_ids.remove(&id);
                    app.set_status(format!("Error: {msg}"));
                }
                ExecEvent::Info(msg) => {
                    app.set_status(msg.clone());
                }
            }
        }

        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;

            if let Event::Key(k) = &ev {
                if k.code == crossterm::event::KeyCode::F(5) {
                    if let Some(m) = app.macros.get(app.list_cursor) {
                        executor.send(ExecCmd::Trigger(m.clone()));
                        app.set_status(format!("Triggered '{}'", m.name));
                    }
                    continue;
                }
            }

            input::handle_event(&mut app, ev);
        }

        if app.should_quit {
            executor.send(ExecCmd::StopAll);
            break;
        }
    }

    Ok(())
}
