// This is the config module. It does these things:
// 1. *Load configuration*: When the app is launched, it loads the configuration file from $HOME/.config/TermLaunch/*.
//    The configuration is stored in a static variable, so it's loaded only once to save resources.
// 2. *Provide configuration*: The statically stored configuration can be accessed by any part of the CLI process.

// --- Imports ---
use dirs; // For home_dir()
use log;
use once_cell::sync::Lazy; // Corrected: once_cell instead of once_call
use serde::Deserialize; // Added: To use the [derive(Deserialize)] macro
use std::fs;
use std::path::PathBuf; // Corrected: PathBuf with capital letters // For logging warnings

// --- Constants and Static Paths ---

// The subdirectory within the user's config folder
pub const CONFIG_SUBDIR: &str = ".config/TermLaunch";
// The name of the configuration file
pub const CONFIG_FILE_NAME: &str = "init.toml";

// Statically defined list of application directories to search
pub static APP_DIRS: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    let mut dirs_list = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
        PathBuf::from("/System/Library/CoreServices/Applications"),
    ];

    // Get the current user's home directory at runtime
    if let Some(home_dir) = dirs::home_dir() {
        dirs_list.push(home_dir.join("Applications"));
    } else {
        log::warn!(
            "Could not determine home directory. User's Applications folder will not be searched."
        );
    }
    dirs_list
});

// Statically computes the config directory path on first use
pub static CONFIG_DIR_PATH: Lazy<PathBuf> = Lazy::new(|| {
    home::home_dir()
        .expect("Failed to fetch home directory") // Corrected: `expect` instead of `except`
        .join(CONFIG_SUBDIR)
});

// Statically computes the full config file path on first use
pub static CONFIG_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    // Corrected: This should join the directory path with the file *name* constant, not itself.
    CONFIG_DIR_PATH.join(CONFIG_FILE_NAME)
});

// --- Configuration Structs ---

#[derive(Deserialize, Debug)]
pub struct Config {
    pub hotkey: Hotkey,
    pub primary_terminal: PrimaryTerminal,
    pub clipboard: Clipboard,
    pub ui: Ui,
    pub shortcut: Shortcut,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: Hotkey::default(),
            primary_terminal: PrimaryTerminal::default(),
            clipboard: Clipboard::default(),
            ui: Ui::default(),
            shortcut: Shortcut::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Shortcut {
    pub apps: String,
    pub files: String,
    pub shell: String,
    pub clipboard: String,
    pub select: String, // example: file searched, then press this keybind -> select UI appear
                        // "select UI" => "open with .. finder, terminal, editor, default app"
                        // Now this keybind function does't exist
}

impl Default for Shortcut {
    fn default() -> Self {
        Self {
            apps: "ctrl+a".to_string(),
            files: "ctrl+f".to_string(),
            shell: "ctrl+c".to_string(),
            clipboard: "ctrl+p".to_string(),
            select: "ctrl+k".to_string(),
        }
    }
}

// Ui struct for launcher dimensions and suggestion limits
#[derive(Deserialize, Debug)]
pub struct Ui {
    pub launcher_width: u16,
    pub max_suggestions_to_display: u16,
    pub default_mode: String,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            launcher_width: 80,             // Default for TermLaunch-cli's UI
            max_suggestions_to_display: 10, // Default to show 10 suggestions
            default_mode: "app".to_string(),
        }
    }
}
//
// Renamed to `Hotkey` for consistency
#[derive(Deserialize, Debug)]
pub struct Hotkey {
    pub key: String,
    pub modifiers: Vec<String>,
}

impl Default for Hotkey {
    fn default() -> Self {
        Self {
            key: "Space".to_string(),
            // Corrected: Use vec![] macro to create a Vec<String>
            modifiers: vec!["Meta".to_string()],
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PrimaryTerminal {
    pub terminal: String,
    // Removed `pub path: PathBuf,`
    pub default_width: i32,
    pub default_height: i32,
    pub default_columns: i32,
    pub default_rows: i32,
}

impl Default for PrimaryTerminal {
    fn default() -> Self {
        Self {
            terminal: "Ghostty".to_string(),
            // Removed `path: PathBuf::from(MACOS_DEFAULT_TERMINAL_APP_PATH),`
            default_width: 300,
            default_height: 200,
            default_columns: 100,
            default_rows: 30,
        }
    }
}

// Renamed to `Clipboard` for consistency
#[derive(Deserialize, Debug)]
pub struct Clipboard {
    pub enabled: bool,
    pub history_size: u32,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            // Corrected: `bool` is a type, you need a value like `true` or `false`
            enabled: true,
            history_size: 100,
        }
    }
}

// --- Global Static Configuration ---

fn load_config() -> Config {
    // Accessing CONFIG_FILE_PATH here will initialize it.
    match fs::read_to_string(&**CONFIG_FILE_PATH) {
        Ok(content) => {
            #[cfg(debug_assertions)]
            println!("\n========== CONFIG LOADED ==========");
            #[cfg(debug_assertions)]
            println!("File Content:\n{}", content);

            // If file is found, try to parse it.
            match toml::from_str(&content) {
                Ok(config) => {
                    #[cfg(debug_assertions)]
                    println!(
                        "\n--- Parsed Configuration ---\n{:#?}\n================================\n",
                        config
                    );
                    config
                }
                Err(e) => {
                    // If parsing fails, log the error and use default config.
                    log::error!("Failed to parse config file: {}. Using default config.", e);
                    #[cfg(debug_assertions)]
                    println!(
                        "\n[ERROR] Failed to parse config: {}\nUsing default config.\n",
                        e
                    );
                    Config::default()
                }
            }
        }
        Err(e) => {
            // If file is not found, use default config.
            log::error!(
                "Config file not found at {:?}. Using default config. Error: {}",
                &**CONFIG_FILE_PATH,
                e
            );
            #[cfg(debug_assertions)]
            println!(
                "\n[WARNING] Config file not found at:\n{:?}\nError: {}\nUsing default config.\n",
                &**CONFIG_FILE_PATH, e
            );
            Config::default()
        }
    }
}

// This is the globally accessible configuration instance.
// It's loaded only once on the first time it's accessed.
pub static CONFIG: Lazy<Config> = Lazy::new(load_config);
