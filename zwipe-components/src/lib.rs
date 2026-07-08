//! Shared Dioxus UI components for the Zwipe surfaces.
//!
//! `zwiper` (the app) and `zite` (the marketing/site) both depend on this crate
//! so buttons, chips, and action bars look and behave identically across them.
//! Styling ships alongside as `assets/components.css`, which each app copies
//! into its own asset bundle at build time (mirroring `shared/themes.css`);
//! the rules reference theme CSS variables, so components resolve to whichever
//! theme the host app has active.
//!
//! These components deliberately depend only on base `dioxus` (no platform
//! features) and `zwipe-core`, so any Dioxus target can consume them.

mod action_bar;
mod button;
mod chip;
mod keyword_chips;
mod oracle_text;

pub use action_bar::ActionBar;
pub use button::{Button, ButtonVariant};
pub use chip::Chip;
pub use keyword_chips::KeywordChips;
pub use oracle_text::OracleText;
