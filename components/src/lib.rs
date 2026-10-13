//! The Magic pieces of the Zwipe UI, over [orangui](https://github.com/scadoshi/orangui).
//!
//! orangui is the generic kit: themes, panels, nav, charts, the entrance
//! effects and the app and site kits. This crate re-exports all of it
//! (`pub use orangui::*`) so `zwiper` (the app) and `zite` (the site) keep one
//! import path, and adds what needs `zwipe-core`: [`CardDetails`], [`CardRow`],
//! [`CardRoleChips`], [`KeywordChips`], [`OracleText`], [`FlippableCardImage`],
//! [`Changelog`], the deck charts ([`DeckCharts`], [`ManaCurve`],
//! [`ManaFulfillment`], [`DrawOdds`], [`ChartLabel`]) and [`ZwipeArchitecture`].
//!
//! Styling ships as string constants to inline through `document::Style`.
//! orangui's sheets come through as they are; [`MAGIC_CSS`] holds this crate's
//! own rules (oracle text, keyword chips, card rows, the flippable card image,
//! the changelog and the mana glyph inside a chip). **CSS cascade order
//! matters:** `THEMES_CSS`, then `COMPONENTS_CSS`, then `MAGIC_CSS`, then
//! `APP_CSS` or `SITE_CSS`, then the host's own stylesheet, so component rules
//! resolve theme variables and host rules override shared defaults at equal
//! specificity.
//!
//! The hosts hold orangui's [`ThemeConfig`]; [`theme_from_preferences`] builds
//! it from a user's saved preferences, and a test in `theme.rs` keeps
//! zwipe-core's theme list equal to orangui's.
//!
//! These components deliberately depend only on base `dioxus` (no platform
//! features), orangui and `zwipe-core`, so any Dioxus target can consume them.
//! The `toast` feature enables orangui's.

mod architecture;
mod card_details;
mod card_role_chips;
mod card_row;
mod changelog;
mod charts;
mod flippable_card_image;
mod keyword_chips;
mod oracle_text;
mod theme;

pub use orangui::*;

pub use architecture::ZwipeArchitecture;
pub use card_details::{CardDetails, card_face_count};
pub use card_role_chips::CardRoleChips;
pub use card_row::CardRow;
pub use changelog::Changelog;
pub use charts::{ChartLabel, DeckCharts, DrawOdds, ManaCurve, ManaFulfillment};
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use keyword_chips::{KeywordChips, KeywordReminders};
pub use oracle_text::OracleText;
pub use theme::theme_from_preferences;

/// The Magic rules: oracle text, keyword chips, card rows, the flippable card
/// image, the changelog and the mana glyph inside a chip. Load it after
/// orangui's [`COMPONENTS_CSS`] and before [`APP_CSS`] or [`SITE_CSS`].
pub const MAGIC_CSS: &str = include_str!("../assets/magic.css");
