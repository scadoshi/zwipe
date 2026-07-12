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
//! site's own stylesheet — `THEMES_CSS` → `COMPONENTS_CSS` → site CSS — so
//! component rules resolve theme variables and site rules can override
//! component defaults at equal specificity.
//!
//! These components deliberately depend only on base `dioxus` (no platform
//! features) and `zwipe-core`, so any Dioxus target can consume them.

mod action_bar;
mod banner;
mod button;
mod card_details;
mod card_role_chips;
mod card_row;
mod changelog;
mod chip;
mod flippable_card_image;
mod keyword_chips;
mod nav_bar;
mod nav_dropdown;
mod oracle_text;
mod page_meta;
mod panel;
mod theme_picker;

pub use action_bar::ActionBar;
pub use banner::{Banner, BannerStatus};
pub use button::{Button, ButtonVariant};
pub use card_details::CardDetails;
pub use card_role_chips::CardRoleChips;
pub use card_row::CardRow;
pub use changelog::Changelog;
pub use chip::Chip;
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use keyword_chips::KeywordChips;
pub use nav_bar::{BRAND_RESET_JS, NavBar};
pub use nav_dropdown::NavDropdown;
pub use oracle_text::OracleText;
pub use page_meta::{PageMeta, SiteMeta};
pub use panel::Panel;
pub use theme_picker::ThemePicker;
// The theme domain types live in zwipe-core (user preferences persist them
// server-side); re-exported here so UI consumers have one import path.
pub use zwipe_core::domain::user::{models::theme::ThemeConfig, preferences::ALLOWED_THEMES};

/// The shared component rules, for consumers outside this workspace.
pub const COMPONENTS_CSS: &str = include_str!("../assets/components.css");
/// The shared theme palettes (14 themes, dark + light), for consumers outside
/// this workspace.
pub const THEMES_CSS: &str = include_str!("../assets/themes.css");
