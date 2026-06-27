//! Reusable UI components.
//!
//! Provides Leptos components used throughout the application for consistent UX.

/// Expandable accordion component.
pub mod accordion;
/// Modal alert/confirmation dialog component.
pub mod alert_dialog;
/// Authentication-related components (login form, register form).
pub mod auth;
/// Slide-up bottom sheet overlay component.
pub mod bottom_sheet;
/// Selectable chip button (shared `.chip` styling).
pub mod chip;
/// Form field components (text input, password input, etc.).
pub mod fields;
/// One-time hint dialogs (per-account, tracked via `hints_shown`).
pub mod hint_dialog;
/// Interactive components (buttons, links, etc.).
pub mod interactions;
/// Logout confirmation dialog.
pub mod logout_dialog;
/// Shared screen header (centered title + optional "?" hint trigger).
pub mod screen_header;
/// Global floating help/support button + bottom sheet.
pub mod support;
/// Usage telemetry buffer + flush loop.
pub mod telemetry;
/// Toast notification component.
pub mod toast;
/// Three-state toggle component (true/false/any).
#[allow(unpredictable_function_pointer_comparisons)]
pub mod tri_toggle;
/// Blocking "Update required" screen (min-version gate).
pub mod update_required;
