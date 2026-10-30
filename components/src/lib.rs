//! The Magic pieces of the Zwipe UI, over [orangui](https://github.com/scadoshi/orangui).
//!
//! orangui is the generic kit: themes, panels, nav, charts, the entrance
//! effects and the app and site kits. This crate re-exports, by name, the
//! orangui items `zwiper` (the app) and `zite` (the site) use, so they keep one
//! import path, and adds what needs `zwipe-core`: [`CardDetails`], [`CardRow`],
//! [`CardRoleChips`], [`RoleTags`], [`OtagDef`], [`KeywordChips`],
//! [`OracleText`], [`FlippableCardImage`], [`Changelog`], the deck charts
//! ([`BarChart`], [`DeckCharts`], [`ColorChart`], [`ManaCurve`], [`ManaFulfillment`],
//! [`DrawOdds`], [`ChartLabel`]), the [`ColorMap`] and [`ZwipeArchitecture`].
//!
//! Styling ships as string constants to inline through `document::Style`.
//! orangui's sheets come through as they are; [`MAGIC_CSS`] holds this crate's
//! own rules (oracle text, keyword chips, card rows, the flippable card image,
//! the changelog, the color map, the mana colors and the mana glyph inside a
//! chip). **CSS cascade order
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
mod color_map;
mod flippable_card_image;
mod keyword_chips;
mod oracle_text;
mod role_tags;
mod theme;

pub use architecture::ZwipeArchitecture;
pub use card_details::{CardDetails, card_face_count};
pub use card_role_chips::CardRoleChips;
pub use card_row::CardRow;
pub use changelog::Changelog;
pub use charts::{
    Bar, BarChart, BarOrientation, ChartLabel, ColorChart, DeckCharts, DrawOdds, LabelCase,
    ManaCurve, ManaFulfillment,
};
pub use color_map::{ColorMap, ColorMapCounts, ColorMapGhost, ColorMapModel, seed_from_id};
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use keyword_chips::{KeywordChips, KeywordReminders};
pub use oracle_text::OracleText;
pub use orangui::{
    ALLOWED_THEMES, APP_CSS, ActionBar, ActionButton, AlertDialog, BOOT_BG, BOOT_HEAD,
    BRAND_RESET_JS, Banner, BootReveal, BottomSheet, Button, ButtonVariant, COMPONENTS_CSS,
    ChartTip, Chip, DarkModeButton, Decode, DialogAction, DialogHostView, GalleryFooter,
    GalleryFrame, GridCells, HEAT_CELL, HEAT_ROWS, HeatCell, HeatGrid, HeatHit, HeatReadout,
    HintBullet, HintBullets, HintDialog, HintKey, HintLine, InfoButton, ModeIcon, NAV_GLIDE_JS,
    NavBar, PageHeader, PageMeta, Panel, PasswordInput, RETIRED_THEMES, REVEAL_JS, Replay,
    SCROLL_FADE_JS, SITE_CSS, SiteMeta, Splash, SplashGate, StatsStrip, THEME_GROUPS, THEMES_CSS,
    TOAST_LONG, TOAST_NORMAL, TOAST_QUICK, TextInput, ThemeConfig, ThemePicker, ThemeSheet,
    heat_span, peak_indices, status_uses, use_dialog_host, use_heat_pick, use_os_back,
    use_overlay_back, use_overlay_back_action, use_overlay_back_stack, use_persisted_theme,
    use_theme_follow, use_theme_wipe, with_separators,
};
#[cfg(feature = "toast")]
pub use orangui::{TOAST_CSS, ToastStack};
pub use role_tags::{OtagDef, RoleTags};
pub use theme::{pip_style_class, theme_from_preferences};

/// The Magic rules: oracle text, keyword chips, card rows, the flippable card
/// image, the changelog and the mana glyph inside a chip. Load it after
/// orangui's [`COMPONENTS_CSS`] and before [`APP_CSS`] or [`SITE_CSS`].
pub const MAGIC_CSS: &str = include_str!("../assets/magic.css");
