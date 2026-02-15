# TermLaunch TUI (Rust Edition)

A fast, lightweight, and extensible TUI launcher inspired by Raycast, written in Rust.

## Features

-   **Application Launcher:** Quickly find and launch your applications.
-   **Calculator:** Perform simple mathematical calculations.
-   **Nerd Font Support:** Rich UI with icons.
-   **Performant:** Built in Rust for maximum speed and safety.
-   **Single Binary:** Compiles down to a single, statically-linked executable.

## Prerequisites

-   **Rust:** You'll need the Rust toolchain (including `cargo`) installed. You can get it from [rust-lang.org](https://www.rust-lang.org/).
-   **Nerd Font:** A Nerd Font must be installed and configured in your terminal to correctly display icons.

## Build and Run

1.  **Clone the repository** (if you haven't already).
2.  **Navigate to the project directory.**
3.  **Build and run the application:**

    ```bash
    cargo run
    ```
4.  **For a release build (optimized):**
    ```bash
    cargo run --release
    ```

## How to Use

-   Start typing to search for applications or enter a mathematical expression.
-   Use the **Up** and **Down** arrow keys to navigate the suggestion list.
-   Press **Enter** to launch the selected application.
-   Press **q** to quit.

## Customization

You can customize the available applications by editing the `get_applications` function in `src/main.rs`:

```rust
fn get_applications<'a>() -> Vec<Application<'a>> {
    vec![
        Application { name: "Firefox", command: "firefox", icon: "" },
        Application { name: "Visual Studio Code", command: "code", icon: "" },
        // Add your own applications here
    ]
}
```
