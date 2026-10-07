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
//! **CSS cascade order matters:** load themes first, then components, then the
//! site's own stylesheet (`THEMES_CSS`, then `COMPONENTS_CSS`, then site CSS) so
//! component rules resolve theme variables and site rules can override
//! component defaults at equal specificity.
//!
//! These components deliberately depend only on base `dioxus` (no platform
//! features) and `zwipe-core`, so any Dioxus target can consume them.

mod action_bar;
mod banner;
mod bottom_sheet;
mod button;
mod card_details;
mod card_role_chips;
mod card_row;
mod changelog;
mod charts;
mod chip;
mod diagram;
mod entrance;
mod flippable_card_image;
mod hint;
mod keyword_chips;
mod nav_bar;
mod nav_dropdown;
mod oracle_text;
mod overlay_stack;
mod page_header;
mod page_meta;
mod panel;
mod stats_strip;
mod theme_picker;
mod theme_sheet;
mod theme_store;
mod theme_wipe;
#[cfg(feature = "toast")]
mod toast_stack;
mod toast_timing;

pub use action_bar::ActionBar;
pub use banner::{Banner, BannerStatus};
pub use bottom_sheet::BottomSheet;
pub use button::{Button, ButtonVariant};
pub use card_details::{CardDetails, card_face_count};
pub use card_role_chips::CardRoleChips;
pub use card_row::CardRow;
pub use changelog::Changelog;
pub use charts::{ChartLabel, DeckCharts, DrawOdds, ManaCurve, ManaFulfillment};
pub use chip::Chip;
pub use diagram::{DIAGRAM_NODE_HEIGHT, DiagramArrow, DiagramDefs, DiagramNode, DiagramTone};
pub use entrance::{CountUp, Decode, Figure, Replay, with_separators};
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use hint::{HintBullet, HintBullets, HintKey, HintLine, InfoButton};
pub use keyword_chips::{KeywordChips, KeywordReminders};
pub use nav_bar::{BRAND_RESET_JS, NavBar};
pub use nav_dropdown::NavDropdown;
pub use oracle_text::OracleText;
pub use overlay_stack::{
    OverlayBackStack, use_overlay_back, use_overlay_back_action, use_overlay_back_stack,
};
pub use page_header::PageHeader;
pub use page_meta::{PageMeta, SiteMeta};
pub use panel::Panel;
pub use stats_strip::StatsStrip;
pub use theme_picker::ThemePicker;
pub use theme_sheet::ThemeSheet;
pub use theme_store::use_persisted_theme;
pub use theme_wipe::{ThemeFollow, use_theme_follow, use_theme_wipe};
#[cfg(feature = "toast")]
pub use toast_stack::ToastStack;
pub use toast_timing::{TOAST_LONG, TOAST_NORMAL, TOAST_QUICK};
// The theme domain types live in zwipe-core (user preferences persist them
// server-side); re-exported here so UI consumers have one import path.
pub use zwipe_core::domain::user::{
    models::theme::ThemeConfig,
    preferences::{ALLOWED_THEMES, COLORBLIND_THEMES, display_theme_name},
};

/// The shared component rules, for consumers outside this workspace.
pub const COMPONENTS_CSS: &str = include_str!("../assets/components.css");
/// The shared theme palettes (31 themes, dark + light), for consumers outside
/// this workspace.
pub const THEMES_CSS: &str = include_str!("../assets/themes.css");
/// The sites' scroll reveal: elements marked `data-reveal` (every [`Panel`])
/// that start below the fold fade up as they scroll in. Inline it with
/// `document::Script`; the host's stylesheet styles `.reveal-pending` and
/// `.reveal-in`.
pub const REVEAL_JS: &str = include_str!("../assets/reveal.js");
/// The sites' nav glide: the children of every element marked
/// `data-nav-glide` (the [`NavBar`] links and the [`ThemePicker`]) slide to
/// their new places when a neighbor changes width. Inline it with
/// `document::Script`.
pub const NAV_GLIDE_JS: &str = include_str!("../assets/nav-glide.js");
/// The app shell's rules (screen, page header, bottom sheet, settings rows,
/// hints), for the apps only. Load it after [`COMPONENTS_CSS`] and before the
/// app's own stylesheet. Sites leave it out: they style these class names
/// their own way.
pub const APP_CSS: &str = include_str!("../assets/app.css");
/// The toast stack's rules, for [`ToastStack`]. Load it after the app's own
/// stylesheet, as the apps load their toast sheet today.
#[cfg(feature = "toast")]
pub const TOAST_CSS: &str = include_str!("../assets/toast.css");
