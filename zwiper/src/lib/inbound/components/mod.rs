//! Reusable UI components.
//!
//! Provides Dioxus components used throughout the application for consistent UX.

/// Expandable accordion component.
pub mod accordion;
/// Authentication-related components (login form, register form).
pub mod auth;
/// App-wide cache of slow-changing filter catalogs (artists, sets, oracle tags…).
pub mod catalog_cache;
/// Generation counter that keeps a stale async reply from landing over a newer one.
pub mod generation;
/// One-time hint dialogs (per-account, tracked via `hints_shown`).
pub mod hint_dialog;
/// App-root receiver that renders on-demand "?" hint dialogs (sender/receiver).
pub mod hint_host;
/// Home's Decks panel: the deck count and the All decks chip over the latest decks as tiles.
pub mod home_decks;
/// The Featured flavor panel, on home and the auth screens.
pub mod home_flavor;
/// The mark with the public counters, on home and the auth screens.
pub mod home_hero;
/// Home's Colors panel: a bar per color with the card count across the user's decks, and the Playstyle chip.
pub mod home_play_style;
/// Persistent, on-demand "?" help button (not one-time, no session dependency).
pub mod info_button;
/// Interactive components (buttons, links, etc.).
pub mod interactions;
/// Logout confirmation dialog.
pub mod logout_dialog;
/// Navigation plumbing (OS back-intent bridge).
pub mod navigation;
/// Runs a closure on each counter bump after mount.
pub mod on_bump;
/// Shared screen header (centered title + optional "?" hint trigger).
pub mod screen_header;
/// Global floating help/support button + bottom sheet.
pub mod support;
/// Usage telemetry buffer + flush loop.
pub mod telemetry;
/// Saving the theme to the account, shared by Settings and the main menu.
pub mod theme_save;
/// Three-state toggle component (true/false/any).
#[expect(
    unpredictable_function_pointer_comparisons,
    reason = "TriToggle's generated props derive PartialEq over its fn pointer props, which always hold the same fn items"
)]
pub mod tri_toggle;
/// Raises zwipe-ui's toasts through the app's toaster.
pub mod ui_toast;
/// Blocking "Update required" screen (min-version gate).
pub mod update_required;
