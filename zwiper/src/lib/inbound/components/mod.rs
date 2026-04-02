//! Reusable UI components.
//!
//! Provides Leptos components used throughout the application for consistent UX.

/// Expandable accordion component.
pub mod accordion;
/// Slide-up bottom sheet overlay component.
pub mod bottom_sheet;
/// Modal alert/confirmation dialog component.
pub mod alert_dialog;
/// Authentication-related components (login form, register form).
pub mod auth;
/// Form field components (text input, password input, etc.).
pub mod fields;
/// Interactive components (buttons, links, etc.).
pub mod interactions;
/// Success message display components.
pub mod success_messages;
/// Toast notification component.
pub mod toast;
/// Three-state toggle component (true/false/any).
#[allow(unpredictable_function_pointer_comparisons)]
pub mod tri_toggle;
