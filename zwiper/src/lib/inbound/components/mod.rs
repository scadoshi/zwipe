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
/// One-time hint dialogs (per-account, tracked via `hints_shown`).
pub mod hint_dialog;
/// Logout confirmation dialog.
pub mod logout_dialog;
/// Interactive components (buttons, links, etc.).
pub mod interactions;
/// Usage telemetry buffer + flush loop.
pub mod telemetry;
/// Toast notification component.
pub mod toast;
/// Blocking "Update required" screen (min-version gate).
pub mod update_required;
/// Three-state toggle component (true/false/any).
#[allow(unpredictable_function_pointer_comparisons)]
pub mod tri_toggle;
/// Global floating help/support button + bottom sheet.
pub mod support;
/// Shared screen header (centered title + optional "?" hint trigger).
pub mod screen_header;
/// Selectable chip button (shared `.chip` styling).
pub mod chip;
