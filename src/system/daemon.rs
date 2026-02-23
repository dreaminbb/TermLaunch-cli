use core_graphics::display::CGDisplay;
use rdev::{Event, EventType, Key, listen};
use std::collections::HashSet;
use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex};
// Removed local fs imports, as they are now in cli::log
// use std::fs::{File, create_dir_all};

#[path = "../config.rs"]
mod config;
use crate::config::CONFIG; // Now CONFIG is correctly imported

use std::sync::atomic::{AtomicBool, Ordering};

// Removed local dirs, log, simplelog imports, as they are now in cli::log
// use dirs;
// use log::{error, info, warn};
// use simplelog::{Config as SimplelogConfig, LevelFilter, WriteLogger};

// Add shared logger module
#[path = "../app_logger.rs"] // Corrected path
mod app_logger;

// Removed init_logger function as it is now shared in cli::log

fn main() {
    app_logger::init_logger("daemon").expect("Failed to initialize logger"); // Call shared logger

    // Print loaded config in debug builds
    #[cfg(debug_assertions)]
    {
        log::info!("[DEBUG] Loaded Daemon Config: {:#?}", *CONFIG); // Changed to log::info!
        log::info!("Starting TermLaunch daemon to listen for hotkey...");
        log::info!("Please ensure Accessibility permissions are granted.");
        log::info!(
            "Configured hotkey: {} + {}",
            CONFIG.hotkey.modifiers.join(" + "),
            CONFIG.hotkey.key
        );
    }

    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));
    let is_tui_running = Arc::new(AtomicBool::new(false)); // Lock to prevent multiple instances

    if let Err(err) = listen(move |event: Event| {
        let mut keys = pressed_keys.lock().unwrap();
        match event.event_type {
            EventType::KeyPress(key) => {
                keys.insert(key);
                check_hotkey(&keys, Arc::clone(&is_tui_running));
            }
            EventType::KeyRelease(key) => {
                keys.remove(&key);
            }
            _ => (),
        }
    }) {
        log::error!("Error listening for events: {:?}", err);
        log::info!("Please ensure Accessibility permissions are granted.");
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

fn check_hotkey(pressed_keys: &HashSet<Key>, is_tui_running: Arc<AtomicBool>) {
    let mut all_modifiers_are_pressed = true;
    for modifier_str in &CONFIG.hotkey.modifiers {
        if let Some(rdev_key) = map_modifier_str_to_key(modifier_str) {
            // Check for both left and right variants of the modifier key
            let (left_variant, right_variant) = match rdev_key {
                Key::MetaLeft => (Key::MetaLeft, Key::MetaRight),
                Key::ControlLeft => (Key::ControlLeft, Key::ControlRight),
                Key::ShiftLeft => (Key::ShiftLeft, Key::ShiftRight),
                Key::Alt => (Key::Alt, Key::AltGr),
                _ => (rdev_key, rdev_key),
            };
            if !pressed_keys.contains(&left_variant) && !pressed_keys.contains(&right_variant) {
                all_modifiers_are_pressed = false;
                break;
            }
        } else {
            log::warn!("Unknown modifier key in config: {}", modifier_str);
            all_modifiers_are_pressed = false;
            break;
        }
    }

    let main_key_is_pressed = if let Some(rdev_key) = map_main_key_str_to_key(&CONFIG.hotkey.key) {
        pressed_keys.contains(&rdev_key)
    } else {
        log::warn!("Unknown main hotkey in config: {}", CONFIG.hotkey.key);
        false
    };

    if all_modifiers_are_pressed && main_key_is_pressed {
        if is_tui_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            log::info!("Hotkey detected! Launching TermLaunch-cli in configured terminal...");

            if let Ok(mut current_exe) = env::current_exe() {
                current_exe.pop(); // Navigate to parent dir (/target/debug/ or /target/release/)
                let tui_path = current_exe.join("TermLaunch-cli");
                if let Some(path_str) = tui_path.to_str() {
                    open_in_configured_terminal(path_str);
                } else {
                    log::error!("Failed to convert TUI path to string.");
                }
            } else {
                log::error!("Could not determine the path of the current executable.");
            }

            // After the terminal process has finished, release the lock.
            is_tui_running.store(false, Ordering::SeqCst);
            log::info!("TUI closed. Ready for next hotkey.");
        } else {
            log::info!("[INFO] TermLaunch is already running. Ignoring hotkey.");
        }
    }
}

// --- Terminal Opening Logic ---

fn open_with_ghostty(command_path: &str) -> std::io::Result<std::process::ExitStatus> {
    let main_display = CGDisplay::main();
    let main_display_bounds = main_display.bounds();

    let screen_width = main_display_bounds.size.width;
    let screen_height = main_display_bounds.size.height;

    let terminal_width = CONFIG.primary_terminal.default_width as f64;
    let terminal_height = CONFIG.primary_terminal.default_height as f64;
    let terminal_columns = CONFIG.primary_terminal.default_columns;
    let terminal_rows = CONFIG.primary_terminal.default_rows;

    let pos_x = (screen_width - terminal_width) / 2.5;
    let pos_y = (screen_height - terminal_height) / 2.5;

    // If you use yabai wm, you should add this config to avoid changing windows size
    // yabai -m rule --add app="^Ghostty$" title="TermLaunch" manage=off
    Command::new("open")
        .arg("-a")
        .arg("Ghostty")
        .arg("-n") // Open a new instance
        .arg("--args")
        .arg(format!("--window-position-x={}", pos_x as i32))
        .arg(format!("--window-position-y={}", pos_y as i32))
        .arg(format!("--window-width={}", terminal_columns))
        .arg(format!("--window-height={}", terminal_rows))
        .arg("--title=TermLaunch")
        .arg("-e") // Tell Ghostty to execute a shell
        .arg("sh")
        .arg("-c")
        .arg(format!("\"{}\"; exit", command_path)) // The shell runs the command, then exits
        .status()
}

fn open_with_default_terminal(command_path: &str) -> std::io::Result<std::process::ExitStatus> {
    let main_display = CGDisplay::main();
    let main_display_bounds = main_display.bounds();

    let screen_width = main_display_bounds.size.width;
    let screen_height = main_display_bounds.size.height;

    // Preserving user's custom size calculations as per their feedback.
    let terminal_width = CONFIG.primary_terminal.default_width as f64 * 3.0;
    let terminal_height = CONFIG.primary_terminal.default_height as f64 * 2.5;

    let x1 = (screen_width - terminal_width) / 2.0;
    let y1 = (screen_height - terminal_height) / 2.0;
    let x2 = x1 + terminal_width;
    let y2 = y1 + terminal_height;

    // A more robust AppleScript that creates the window first, then activates and modifies it.
    let script = format!(
        r#"
        tell application "Terminal"
            -- 1. Create the window by running the script. This returns a reference to the tab.
            set term_tab to do script quoted form of "{}"
            
            -- 2. Activate the application to bring it to the front.
            activate
            
            -- 3. Now that it's frontmost, set its bounds.
            tell front window to set its bounds to {{ {}, {}, {}, {} }}
            
            -- 4. Poll until the command in the tab is no longer busy.
            repeat while busy of term_tab
                delay 0.2
            end repeat
            
            -- 5. Once the command is done, close the window.
            close front window
        end tell
        "#,
        command_path, x1 as i32, y1 as i32, x2 as i32, y2 as i32,
    );

    Command::new("osascript").arg("-e").arg(&script).status()
}

fn open_in_configured_terminal(command_path: &str) {
    let terminal_name = &CONFIG.primary_terminal.terminal;

    let status_result = match terminal_name.as_str() {
        "Ghostty" => open_with_ghostty(command_path),
        "Terminal" => open_with_default_terminal(command_path),
        unsupported => {
            log::warn!(
                "Unsupported terminal in config: '{}'. Falling back to default Terminal.app.",
                unsupported
            );
            open_with_default_terminal(command_path)
        }
    };

    match status_result {
        Ok(status) if status.success() => {
            log::info!(
                "Successfully launched {} with TermLaunch-cli.",
                terminal_name
            );
        }
        Ok(status) => {
            log::error!("{} process exited with status {}.", terminal_name, status);
        }
        Err(e) => {
            log::error!("Failed to launch terminal '{}': {}", terminal_name, e);
        }
    }
}
