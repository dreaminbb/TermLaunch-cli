use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

// Removed direct APP_DIRS import here, will use `crate::config::APP_DIRS` directly
// Removed `#[path = "../config.rs"] mod config;`
// Removed `use config::fetch_app_dirs;`

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    AppLauncher,
    FileSearch,
    ShellExecution,
    ClipboardHistory,
}

pub struct App {
    pub input: String,
    pub suggestions: Vec<Suggestion>,
    pub selected_index: usize,
    pub current_mode: AppMode,
}

#[derive(Clone)]
pub enum Suggestion {
    App(Application),
    File(String),
    Shell(String),
    Clipboard(String),
    Calc(String),
}

#[derive(Clone)]
pub struct Application {
    pub name: String,
    pub icon: String,
    pub path: String,
}

impl App {
    pub fn new() -> App {
        let default_mode = match crate::config::CONFIG.ui.default_mode.as_str() {
            "file" => AppMode::FileSearch,
            "shell" => AppMode::ShellExecution,
            "clipboard" => AppMode::ClipboardHistory,
            _ => AppMode::AppLauncher,
        };

        App {
            input: String::new(),
            suggestions: Vec::new(),
            selected_index: 0,
            current_mode: default_mode,
        }
    }

    pub fn get_mode_color(&self) -> Color {
        match self.current_mode {
            AppMode::AppLauncher => Color::Rgb(0x95, 0x7f, 0xb8), // oniViolet
            AppMode::FileSearch => Color::Rgb(0x76, 0x94, 0x6a),  // autumnGreen
            AppMode::ShellExecution => Color::Rgb(0xc3, 0x40, 0x43), // autumnRed
            AppMode::ClipboardHistory => Color::Rgb(0xc0, 0xa3, 0x6e), // carpYellow
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
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

    fn is_shortcut(event: KeyEvent, shortcut_str: &str) -> bool {
        let parts: Vec<&str> = shortcut_str.split('+').collect();
        if parts.len() != 2 {
            return false;
        }

        let modifier = match parts[0].to_lowercase().as_str() {
            "ctrl" => KeyModifiers::CONTROL,
            "alt" => KeyModifiers::ALT,
            "shift" => KeyModifiers::SHIFT,
            _ => return false,
        };

        let key_code = if parts[1].len() == 1 {
            KeyCode::Char(parts[1].chars().next().unwrap().to_ascii_lowercase())
        } else {
            return false;
        };

        // Normalize character case for comparison
        let normalized_event_code = match event.code {
            KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
            other => other,
        };

        event.modifiers.contains(modifier) && normalized_event_code == key_code
    }

    pub fn change_mode(&mut self, event: KeyEvent) -> bool {
        let shortcuts = &crate::config::CONFIG.shortcut;

        let new_mode = if Self::is_shortcut(event, &shortcuts.apps) {
            #[cfg(debug_assertions)]
            println!("→ Mode changed to: AppLauncher\n");
            Some(AppMode::AppLauncher)
        } else if Self::is_shortcut(event, &shortcuts.files) {
            #[cfg(debug_assertions)]
            println!("→ Mode changed to: FileSearch\n");
            Some(AppMode::FileSearch)
        } else if Self::is_shortcut(event, &shortcuts.shell) {
            #[cfg(debug_assertions)]
            println!("→ Mode changed to: ShellExecution\n");
            Some(AppMode::ShellExecution)
        } else if Self::is_shortcut(event, &shortcuts.clipboard) {
            #[cfg(debug_assertions)]
            println!("→ Mode changed to: ClipboardHistory\n");
            Some(AppMode::ClipboardHistory)
        } else {
            None
        };

        if let Some(mode) = new_mode {
            self.current_mode = mode;
            self.input.clear();
            self.update_suggestions();
            return true;
        }
        false
    }

    pub fn update_suggestions(&mut self) {
        self.suggestions.clear();
        self.selected_index = 0;

        match self.current_mode {
            AppMode::AppLauncher => self.update_app_suggestions(),
            AppMode::FileSearch => self.update_file_suggestions(),
            AppMode::ShellExecution => self.update_shell_suggestions(),
            AppMode::ClipboardHistory => self.update_clipboard_suggestions(),
        }

        // Calculation is a global fallback or secondary feature
        if self.suggestions.is_empty() && !self.input.is_empty() {
            if let Ok(result) = meval::eval_str(&self.input) {
                if result.is_finite() {
                    self.suggestions.push(Suggestion::Calc(result.to_string()));
                }
            }
        }
    }

    fn update_app_suggestions(&mut self) {
        if self.input.is_empty() {
            return;
        }
        let apps = get_applications();
        for app in apps {
            if app.name.to_lowercase().contains(&self.input.to_lowercase()) {
                self.suggestions.push(Suggestion::App(app));
            }
        }
    }

    fn update_file_suggestions(&mut self) {
        if self.input.is_empty() {
            return;
        }
        // Placeholder: File search logic will go here
        self.suggestions
            .push(Suggestion::File(format!("Search file: {}", self.input)));
    }

    fn update_shell_suggestions(&mut self) {
        if self.input.is_empty() {
            return;
        }
        // Placeholder: Shell command logic will go here
        self.suggestions
            .push(Suggestion::Shell(format!("Run: {}", self.input)));
    }

    fn update_clipboard_suggestions(&mut self) {
        // Placeholder: Clipboard history logic will go here
        self.suggestions
            .push(Suggestion::Clipboard("Last copied text...".to_string()));
    }
}

fn get_applications() -> Vec<Application> {
    let mut apps = Vec::new();
    // Use the global APP_DIRS directly
    let app_dirs = &*crate::config::APP_DIRS;

    for dir in app_dirs.iter() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "app") {
                    if let Some(app_name_os) = path.file_stem() {
                        let app_name = app_name_os.to_string_lossy().into_owned();
                        if let Some(path_str) = path.to_str() {
                            apps.push(Application {
                                name: app_name.clone(),
                                icon: "󰣆".to_string(), // Nerd Font icon for desktop
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
