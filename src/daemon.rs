use core_graphics::display::CGDisplay;
use rdev::{Event, EventType, Key, listen};
use std::collections::HashSet;
use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex};

const GHOSTTY_DEFAULT_WIDTH: f64 = 600.0;
const GHOSTTY_DEFAULT_HEIGHT: f64 = 400.0;
const GHOSTTY_DEFAULT_COLUMNS: i32 = 100;
const GHOSTTY_DEFAULT_ROWS: i32 = 30;

fn main() {
    println!("Starting TermLaunch daemon to listen for Cmd+Space...");
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
        println!("Error: {:?}", error);
    }
}

fn check_hotkey(pressed_keys: &HashSet<Key>) {
    let cmd_pressed =
        pressed_keys.contains(&Key::MetaLeft) || pressed_keys.contains(&Key::MetaRight);
    let space_pressed = pressed_keys.contains(&Key::Space);

    if cmd_pressed && space_pressed {
        println!("Cmd+Space detected! Launching TermLaunch-cli in ghostty...");

        if let Ok(mut current_exe) = env::current_exe() {
            current_exe.pop(); // Navigate to parent dir (/target/debug/ or /target/release/)
            let tui_path = current_exe.join("TermLaunch-cli");
            if let Some(path_str) = tui_path.to_str() {
                open_in_ghostty(path_str);
            } else {
                eprintln!("Failed to convert TUI path to string.");
            }
        } else {
            eprintln!("Could not determine the path of the current executable.");
        }
    }
}

fn open_in_ghostty(command_path: &str) {
    let main_display = CGDisplay::main();
    let main_display_bounds = main_display.bounds();

    let screen_width = main_display_bounds.size.width;
    let screen_height = main_display_bounds.size.height;

    let pos_x = (screen_width - GHOSTTY_DEFAULT_WIDTH) / 2.0;
    let pos_y = (screen_height - GHOSTTY_DEFAULT_HEIGHT) / 2.0;

    let status = Command::new("open")
        .arg("-a")
        .arg("ghostty")
        .arg("-n") // Open a new instance
        .arg("--args")
        .arg(format!("--window-position-x={}", pos_x as i32))
        .arg(format!("--window-position-y={}", pos_y as i32))
        .arg(format!("--window-width={}", GHOSTTY_DEFAULT_COLUMNS))
        .arg(format!("--window-height={}", GHOSTTY_DEFAULT_ROWS))
        .arg("-e") // Assume -e flag for executing a command
        .arg(command_path)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("Successfully launched ghostty with TermLaunch-cli.");
        }
        Ok(status) => {
            eprintln!("Ghostty process exited with status {}.", status);
        }
        Err(e) => {
            eprintln!("Failed to execute 'open' command for ghostty: {}", e);
        }
    }
}
