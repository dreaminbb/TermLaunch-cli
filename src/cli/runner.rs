use crate::cli::{
    app::{App, Suggestion},
    terminal::restore_terminal,
    ui::ui,
};
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use std::{
    io::{self, Stdout},
    process::{Command, Stdio},
};

pub fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != event::KeyEventKind::Press {
                continue;
            }
            if app.change_mode(key) {
                continue;
            }
            match key.code {
                // I thought quit qpp with q key is good idea.
                // But in case there is a program name start with q, you can't open it
                // So the key only can quit is ESC key
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => {
                    if let Some(suggestion) = app.suggestions.get(app.selected_index) {
                        if let Suggestion::App(selected_app) = suggestion {
                            // Before exiting, restore the terminal to a clean state
                            restore_terminal(terminal)?;

                            Command::new("open")
                                .arg(&selected_app.path)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .status()?; // Use status() to wait for command to be issued

                            // We exit the TUI app after launching the selected application.
                            return Ok(());
                        }
                    }
                }
                _ => app.on_key(key.code),
            }
        }
    }
}
