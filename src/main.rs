use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
                if self.selected_index < self.suggestions.len() - 1 {
                    self.selected_index += 1;
                }
            }
            _ => {}
        }
    }

    fn update_suggestions(&mut self) {
        self.suggestions.clear();
        let apps = get_applications();
        for app in apps {
            if app.name.to_lowercase().contains(&self.input.to_lowercase()) {
                self.suggestions.push(Suggestion::App(app));
            }
        }

        // Only add calculation if there are no app suggestions
        if self.suggestions.is_empty() && !self.input.is_empty() {
            if let Ok(result) = meval::eval_str(&self.input) {
                self.suggestions.push(Suggestion::Calc(result.to_string()));
            }
        }

        self.selected_index = 0;
        if !self.suggestions.is_empty() {
            self.selected_index = 0;
        }
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
                                icon: "".to_string(), // Generic app icon
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
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Enter => {
                    if let Some(suggestion) = app.suggestions.get(app.selected_index) {
                        if let Suggestion::App(selected_app) = suggestion {
                            let status_result = Command::new("open")
                                .arg(&selected_app.path)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .status();

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
    let parent_area = f.area();

    // Define the maximum desired fixed size for the launcher content
    let max_width = 80;
    let max_height = 20;

    // The actual width/height will be the smaller of the max size and the terminal size
    let actual_width = parent_area.width.min(max_width);
    let actual_height = parent_area.height.min(max_height);

    // Calculate the top-left corner to center the desired area
    let x = (parent_area.width.saturating_sub(actual_width)) / 2;
    let y = (parent_area.height.saturating_sub(actual_height)) / 2;

    let centered_area = Rect::new(x, y, actual_width, actual_height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // For input
                Constraint::Min(0),    // For suggestions
            ]
            .as_ref(),
        )
        .split(centered_area);

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[0]);

    let suggestions: Vec<ListItem> = app
        .suggestions
        .iter()
        .enumerate()
        .map(|(i, suggestion)| {
            let (icon, text) = match suggestion {
                Suggestion::App(app) => (app.icon.clone(), app.name.clone()),
                Suggestion::Calc(res) => ("".to_string(), format!("= {}", res)),
            };

            let content = Line::from(vec![Span::raw(icon), Span::raw(" "), Span::raw(text)]);
            let mut list_item = ListItem::new(content);

            if i == app.selected_index {
                list_item = list_item.style(Style::default().fg(Color::Black).bg(Color::White));
            }
            list_item
        })
        .collect();

    let suggestions_list = List::new(suggestions)
        .block(Block::default().borders(Borders::ALL).title("Suggestions"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )
        .highlight_symbol("> ");

    f.render_widget(suggestions_list, chunks[1]);
}
