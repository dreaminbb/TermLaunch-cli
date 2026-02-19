use crossterm::event::KeyCode;

pub struct App {
    pub input: String,
    pub suggestions: Vec<Suggestion>,
    pub selected_index: usize,
}

#[derive(Clone)]
pub enum Suggestion {
    App(Application),
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
        App {
            input: String::new(),
            suggestions: Vec::new(),
            selected_index: 0,
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

    pub fn update_suggestions(&mut self) {
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
