use core_graphics::display::CGDisplay;
use rdev::{Event, EventType, Key, listen};
use std::collections::HashSet;
use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[path = "../config.rs"]
mod config;
use crate::config::CONFIG; // Now CONFIG is correctly imported

fn main() {
    // Print loaded config in debug builds
    #[cfg(debug_assertions)]
    {
        println!("[DEBUG] Loaded Daemon Config: {:#?}", *CONFIG);
    }

    println!("Starting TermLaunch daemon to listen for hotkey...");
    println!(
        "Configured hotkey: {} + {}",
        CONFIG.hotkey.modifiers.join(" + "),
        CONFIG.hotkey.key
    );
    println!("Please ensure Accessibility permissions are granted.");

    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));
    let pressed_keys_clone = Arc::clone(&pressed_keys);

    if let Err(error) = listen(move |event: Event| {
        let mut keys = pressed_keys_clone.lock().unwrap();
        match event.event_type {
            EventType::KeyPress(key) => {
                keys.insert(key);
                check_hotkey(&keys);
            }
            EventType::KeyRelease(key) => {
                keys.remove(&key);
            }
            _ => (),
        }
    }) {
        eprintln!("Error listening for events: {:?}", error);
    }
}

// Helper function to map string modifier to rdev::Key
fn map_modifier_str_to_key(modifier_str: &str) -> Option<Key> {
    match modifier_str {
        "Meta" => Some(Key::MetaLeft), // macOS Command key
        "Control" => Some(Key::ControlLeft),
        "Shift" => Some(Key::ShiftLeft),
        "Alt" => Some(Key::Alt), // Option key
        _ => None,
    }
}

// Helper function to map string key to rdev::Key
fn map_main_key_str_to_key(key_str: &str) -> Option<Key> {
    match key_str {
        "Space" => Some(Key::Space),
        "Return" => Some(Key::Return),
        "Escape" => Some(Key::Escape),
        // Add more keys as needed from rdev::Key enum
        _ => None,
    }
}

fn check_hotkey(pressed_keys: &HashSet<Key>) {
    let mut all_modifiers_are_pressed = true;
    for modifier_str in &CONFIG.hotkey.modifiers {
        if let Some(rdev_key) = map_modifier_str_to_key(modifier_str) {
            if !pressed_keys.contains(&rdev_key) {
                all_modifiers_are_pressed = false;
                break;
            }
        } else {
            eprintln!("Unknown modifier key in config: {}", modifier_str);
            all_modifiers_are_pressed = false;
            break;
        }
    }

    let main_key_is_pressed = if let Some(rdev_key) = map_main_key_str_to_key(&CONFIG.hotkey.key) {
        pressed_keys.contains(&rdev_key)
    } else {
        eprintln!("Unknown main hotkey in config: {}", CONFIG.hotkey.key);
        false
    };

    if all_modifiers_are_pressed && main_key_is_pressed {
        println!("Hotkey detected! Launching TermLaunch-cli in configured terminal...");

        if let Ok(mut current_exe) = env::current_exe() {
            current_exe.pop(); // Navigate to parent dir (/target/debug/ or /target/release/)
            let tui_path = current_exe.join("TermLaunch-cli");
            if let Some(path_str) = tui_path.to_str() {
                open_in_configured_terminal(path_str);
            } else {
                eprintln!("Failed to convert TUI path to string.");
            }
        } else {
            eprintln!("Could not determine the path of the current executable.");
        }
    }
}

fn open_in_configured_terminal(command_path: &str) {
    let main_display = CGDisplay::main();
    let main_display_bounds = main_display.bounds();

    let screen_width = main_display_bounds.size.width;
    let screen_height = main_display_bounds.size.height;

    // Use configured terminal dimensions
    let terminal_width = CONFIG.primary_terminal.default_width as f64;
    let terminal_height = CONFIG.primary_terminal.default_height as f64;
    let terminal_columns = CONFIG.primary_terminal.default_columns;
    let terminal_rows = CONFIG.primary_terminal.default_rows;

    let pos_x = (screen_width - terminal_width) / 2.0;
    let pos_y = (screen_height - terminal_height) / 2.0;

    // TODO: Change method of opening terminal depends on terminal app. It works only ghostty

    let status = Command::new("open")
        .arg("-a")
        .arg(&CONFIG.primary_terminal.terminal) // Use configured terminal name (e.g., "Ghostty", "Terminal")
        .arg("-n") // Open a new instance
        .arg("--args")
        .arg(format!("--window-position-x={}", pos_x as i32))
        .arg(format!("--window-position-y={}", pos_y as i32))
        .arg(format!("--window-width={}", terminal_columns))
        .arg(format!("--window-height={}", terminal_rows))
        .arg("-e") // Assume -e flag for executing a command
        .arg(command_path)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!(
                "Successfully launched {} with TermLaunch-cli.",
                CONFIG.primary_terminal.terminal
            );
        }
        Ok(status) => {
            eprintln!(
                "{} process exited with status {}.",
                CONFIG.primary_terminal.terminal, status
            );
        }
        Err(e) => {
            eprintln!(
                "Failed to execute 'open' command for {}: {}",
                CONFIG.primary_terminal.terminal, e
            );
        }
    }
}
