# TermLaunch: A Terminal-Based Application Launcher

## Project Overview

TermLaunch is a terminal-based application launcher for macOS, inspired by tools like Alfred and Raycast. It is designed to be fast, keyboard-driven, and developer-friendly.

It has two primary components:
1.  **Daemon (`TermLaunch-daemon`):** A background process that listens for a global hotkey to launch the TUI.
2.  **TUI (`TermLaunch-cli`):** A Text User Interface that runs inside a terminal window, providing the core launcher functionality.

Instead of a traditional GUI, TermLaunch uses your installed terminal emulator (e.g., Ghostty) as its interface.

- **Language:** Rust
- **Platform:** macOS

## Core Features

- Application launching
- Built-in calculator
- File search
- Open URLs in a browser
- Clipboard history search

## Configuration

- Configuration files are located in `~/.config/termlaunch/`.
- This allows for easy customization and backup of settings.

## Core Crates

- **ratatui:** For building the Terminal User Interface.
- **crossterm:** The terminal backend for `ratatui`.
- **meval:** For the calculator feature.
- **rdev:** For global hotkey listening in the daemon.
- **core-graphics:** For interacting with macOS-specific APIs.

## Build and Run Commands

- **Build for production:**
  ```bash
  cargo build --release
  ```
- **Run the daemon (for hotkey):**
  ```bash
  cargo run --bin TermLaunch-daemon
  ```
- **Run the TUI directly:**
  ```bash
  cargo run --bin TermLaunch-cli
  ```
- **Linting:**
  ```bash
  cargo clippy --all-targets
  ```
- **Formatting:**
  ```bash
  cargo fmt
  ```

## Useful Documentation

- **Target Terminal:** Currently supports [Ghostty](https://ghostty.org/docs/features), with plans to support others.
- **Rust Language:** [The Rust Book](https://doc.rust-lang.org/book/)

## Output rules
- **Strict No-File Policy**: Do not generate any file outputs (e.g., code blocks meant to be saved) unless I explicitly use the keyword "WRITE".
- **Guiding Principle**: When I ask "I want to implement this...", do not provide the full source code. Instead:
    1. Explain the **logic and workflow**.
    2. Provide **pseudocode** or small, isolated snippets for complex parts only.
    3. List **relevant libraries or API functions** I should look into.
    4. Offer **pro-tips** regarding performance or common pitfalls.
