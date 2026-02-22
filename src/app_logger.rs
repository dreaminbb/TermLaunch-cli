// src/cli/log.rs
use dirs;
use log::{LevelFilter, info, warn, error}; // Added warn, error
use simplelog::{Config as SimplelogConfig, WriteLogger};
use std::env;
use std::fs::{File, create_dir_all};

pub fn init_logger(binary_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = if cfg!(debug_assertions) {
        // Check if debug build
        let current_dir = env::current_dir()?;
        current_dir.join("logs")
    } else {
        if let Some(home_dir) = dirs::home_dir() {
            home_dir.join("Library/Logs/TermLaunch")
        } else {
            return Err("Could not find home directory for logging.".into());
        }
    };

    create_dir_all(&log_dir)?; // Create the log directory if it doesn't exist

    let log_file_name = format!("{}.log", binary_name);
    let log_file_path = log_dir.join(log_file_name);
    let log_file = File::create(&log_file_path)?;

    WriteLogger::init(
        LevelFilter::Info, // Log Info level and above
        SimplelogConfig::default(),
        log_file,
    )?;
    info!(
        "Logger initialized for {} at {:?}",
        binary_name, log_file_path
    );
    Ok(())
}
