mod theme;

use crate::theme::{BG, BORDER_ACTIVE, FG, SELECTION_BG, SELECTION_FG};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};
use std::{
    io::{self, Stdout, stdout},
    process::{Command, Stdio},
};

struct App {
    input: String,
    suggestions: Vec<Suggestion>,
    selected_index: usize,
}

#[derive(Clone)]
enum Suggestion {
    App(Application),
    Calc(String),
}

#[derive(Clone)]
struct Application {
    name: String,
    icon: String,
    path: String,
}

impl App {
    fn new() -> App {
        App {
            input: String::new(),
            suggestions: Vec::new(),
            selected_index: 0,
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.input.push(c);
                self.update_suggestions();
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.update_suggestions();
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if !self.suggestions.is_empty() && self.selected_index < self.suggestions.len() - 1
                {
                    self.selected_index += 1;
                }
            }
            _ => {}
        }
    }

    fn update_suggestions(&mut self) {
        self.suggestions.clear();
        if self.input.is_empty() {
            self.selected_index = 0;
            return;
        }

        let apps = get_applications();
        for app in apps {
            if app.name.to_lowercase().contains(&self.input.to_lowercase()) {
                self.suggestions.push(Suggestion::App(app));
            }
        }

        // Only add calculation if there are no app suggestions
        if self.suggestions.is_empty() {
            if let Ok(result) = meval::eval_str(&self.input) {
                if result.is_finite() {
                    self.suggestions.push(Suggestion::Calc(result.to_string()));
                }
            }
        }

        self.selected_index = 0;
    }
}

fn get_applications() -> Vec<Application> {
    let mut apps = Vec::new();
    let home_apps_path = format!("{}/Applications", env!("HOME"));
    let app_dirs = [
        "/Applications",
        home_apps_path.as_str(),
        "/System/Applications",
        "/System/Library/CoreServices/Applications",
    ];

    for dir in &app_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "app") {
                    if let Some(app_name_os) = path.file_stem() {
                        let app_name = app_name_os.to_string_lossy().into_owned();
                        if let Some(path_str) = path.to_str() {
                            apps.push(Application {
                                name: app_name.clone(),
                                icon: "".to_string(), // Nerd Font icon for desktop
                                path: path_str.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    apps
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    run_app(&mut terminal, &mut app)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
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

fn ui(f: &mut Frame, app: &App) {
    let area = f.area(); // Corrected: f.size() is deprecated, use f.area()

    // Define a centered area for the launcher
    // default widht is 600 height is 400
    // It's also written in src/daemon.rs
    let launcher_width = 100;
    let suggestions_height = 100;
    let launcher_height = 3 + suggestions_height; // 1 for border, 1 for input, 1 for border, + suggestions

    let area = centered_rect(launcher_width, launcher_height, area);

    // Clear the area before drawing to handle dynamic height
    f.render_widget(Clear, area);

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_ACTIVE))
        .style(Style::default().bg(BG));

    let inner_area = main_block.inner(area);
    f.render_widget(main_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(inner_area);

    // --- Input Line ---
    let input_line = Line::from(vec![
        Span::styled("❯ ", Style::default().fg(BORDER_ACTIVE)),
        Span::styled(app.input.as_str(), Style::default().fg(FG)),
    ]);
    let input_paragraph = Paragraph::new(input_line);
    f.render_widget(input_paragraph, chunks[0]);

    // --- Suggestions List ---
    let suggestions: Vec<ListItem> = app
        .suggestions
        .iter()
        .enumerate()
        .map(|(i, suggestion)| {
            // Corrected: Ensure both match arms return the same type (&str, &str)
            let (icon, text): (&str, &str) = match suggestion {
                Suggestion::App(app) => (app.icon.as_str(), app.name.as_str()),
                Suggestion::Calc(res) => ("", res.as_str()),
            };

            let content = Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(SELECTION_FG)),
                // Corrected: `text` is now a `&str`, no clone needed.
                Span::raw(text),
            ]);

            let mut list_item = ListItem::new(content).style(Style::default().fg(FG));

            if i == app.selected_index {
                list_item = list_item.style(
                    Style::default()
                        .bg(SELECTION_BG)
                        .fg(FG)
                        .add_modifier(Modifier::BOLD),
                );
            }
            list_item
        })
        .collect();

    let suggestions_list = List::new(suggestions);
    f.render_widget(suggestions_list, chunks[1]);
}

/// Helper function to create a centered rect with a max width and dynamic height
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15), // 15% from top
            Constraint::Min(height),
            Constraint::Max(
                r.height
                    .saturating_sub(height)
                    .saturating_sub(r.height / 10),
            ), // flexible bottom margin
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
