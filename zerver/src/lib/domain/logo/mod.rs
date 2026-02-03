//! ASCII art branding for CLI applications.
//!
//! This module provides ASCII art logos for the ZWIPE project's various components,
//! used for CLI startup branding and visual identity.
//!
//! # Components
//!
//! - **ZWIPE**: Main application logo
//! - **ZERVER**: Backend server logo
//! - **ZERVICE**: Background service logo
//! - **ZWIPER**: Frontend application logo
//!
//! # Usage
//!
//! Each logo can be printed to stdout or accessed as a string constant:
//!
//! ```rust,ignore
//! use zwipe::domain::logo::Zerver;
//!
//! // Print to stdout
//! Zerver::print();
//!
//! // Or use the string constant
//! println!("{}", zwipe::domain::logo::ZERVER);
//! ```

#![allow(clippy::print_stdout)]

/// ASCII art logo for ZWIPE (main application).
pub const ZWIPE: &str = include_str!("zwipe.txt");

/// ZWIPE logo printer.
///
/// # Example
///
/// ```rust,ignore
/// Zwipe::print(); // Prints logo to stdout
/// ```
pub struct Zwipe;

impl Zwipe {
    /// Prints the ZWIPE ASCII art logo to stdout.
    pub fn print() {
        println!("{ZWIPE}");
    }
}

/// ASCII art logo for ZERVER (backend server).
pub const ZERVER: &str = include_str!("zerver.txt");

/// ZERVER logo printer.
///
/// # Example
///
/// ```rust,ignore
/// Zerver::print(); // Prints logo to stdout
/// ```
pub struct Zerver;

impl Zerver {
    /// Prints the ZERVER ASCII art logo to stdout.
    pub fn print() {
        println!("{ZERVER}");
    }
}

/// ASCII art logo for ZERVICE (background service).
pub const ZERVICE: &str = include_str!("zervice.txt");

/// ZERVICE logo printer.
///
/// # Example
///
/// ```rust,ignore
/// Zervice::print(); // Prints logo to stdout
/// ```
pub struct Zervice;

impl Zervice {
    /// Prints the ZERVICE ASCII art logo to stdout.
    pub fn print() {
        println!("{ZERVICE}");
    }
}

/// ASCII art logo for ZWIPER (frontend application).
pub const ZWIPER: &str = include_str!("zwiper.txt");

/// ZWIPER logo printer.
///
/// # Example
///
/// ```rust,ignore
/// Zwiper::print(); // Prints logo to stdout
/// ```
pub struct Zwiper;

impl Zwiper {
    /// Prints the ZWIPER ASCII art logo to stdout.
    pub fn print() {
        println!("{ZWIPER}");
    }
}
