//! Frontend UI layer (Leptos components and routing).
//!
//! Contains the Leptos WASM frontend application structure:
//! - **Components**: Reusable UI components
//! - **Router**: Client-side routing configuration
//! - **Screens**: Top-level page components

/// Reusable UI components (buttons, forms, cards, etc.).
pub mod components;
/// Client-side routing configuration.
pub mod router;
/// Top-level page components (auth, deck builder, profile).
pub mod screens;
