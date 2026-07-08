//! Shared Dioxus UI components for the Zwipe surfaces.
//!
//! `zwiper` (the app) and `zite` (the marketing/site) both depend on this crate
//! so buttons, chips, and action bars look and behave identically across them.
//! Styling ships alongside: `assets/components.css` (the components' rules) and
//! `assets/themes.css` (the theme palettes those rules resolve against). The
//! workspace apps copy both into their own asset bundles at build time;
//! external consumers (e.g. the portfolio site, via a git dependency) can't
//! reach the crate's files by path, so the same CSS is also exported as the
//! [`COMPONENTS_CSS`] / [`THEMES_CSS`] string constants to inline via
//! `document::Style`.
//!
//! These components deliberately depend only on base `dioxus` (no platform
//! features) and `zwipe-core`, so any Dioxus target can consume them.

mod action_bar;
mod button;
mod chip;
mod keyword_chips;
mod oracle_text;
mod theme_picker;

pub use action_bar::ActionBar;
pub use button::{Button, ButtonVariant};
pub use chip::Chip;
pub use keyword_chips::KeywordChips;
pub use oracle_text::OracleText;
pub use theme_picker::ThemePicker;
// The theme domain types live in zwipe-core (user preferences persist them
// server-side); re-exported here so UI consumers have one import path.
pub use zwipe_core::domain::user::{models::theme::ThemeConfig, preferences::ALLOWED_THEMES};

/// The shared component rules, for consumers outside this workspace.
pub const COMPONENTS_CSS: &str = include_str!("../assets/components.css");
/// The shared theme palettes (14 themes, dark + light), for consumers outside
/// this workspace.
pub const THEMES_CSS: &str = include_str!("../assets/themes.css");
