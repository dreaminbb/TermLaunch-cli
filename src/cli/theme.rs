//! theme.rs
//!
//! This module defines the color palette for the application, based on the Kanagawa theme.
//! https://github.com/rebelot/kanagawa.nvim

use ratatui::style::Color;

// These are constants for the Kanagawa color theme's "wave" palette.
// We are defining them as `Color` instances directly for use with ratatui.

// Backgrounds
pub const BG: Color = Color::Rgb(0x1f, 0x1f, 0x28); // sumiInk1

// Foregrounds
pub const FG: Color = Color::Rgb(0xdc, 0xd7, 0xba); // fujiWhite

// Accents
pub const BORDER_ACTIVE: Color = Color::Rgb(0x95, 0x7f, 0xb8); // oniViolet

pub const SELECTION_BG: Color = Color::Rgb(0x4d, 0x4d, 0x69); // A mix for selection background
pub const SELECTION_FG: Color = Color::Rgb(0xc0, 0xa3, 0x6e); // carpYellow
