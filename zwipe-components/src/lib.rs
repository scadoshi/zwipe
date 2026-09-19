//! Shared Dioxus UI components for zwiper and zite, styled by
//! `assets/components.css` against the palettes in `assets/themes.css`.
//! External consumers can't reach those files by path, so both are also
//! exported as [`COMPONENTS_CSS`] and [`THEMES_CSS`] to inline via
//! `document::Style`. Load themes, then components, then the site's own CSS so
//! the cascade resolves in that order.
//!
//! Depends only on base `dioxus` and `zwipe-core`, so any Dioxus target can
//! consume it.

mod action_bar;
mod banner;
mod button;
mod card_details;
mod card_role_chips;
mod card_row;
mod changelog;
mod charts;
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
pub use card_details::{CardDetails, card_face_count};
pub use card_role_chips::CardRoleChips;
pub use card_row::CardRow;
pub use changelog::Changelog;
pub use charts::{ChartLabel, DeckCharts, DrawOdds, ManaCurve, ManaFulfillment};
pub use chip::Chip;
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use keyword_chips::{KeywordChips, KeywordReminders};
pub use nav_bar::{BRAND_RESET_JS, NavBar};
pub use nav_dropdown::NavDropdown;
pub use oracle_text::OracleText;
pub use page_meta::{PageMeta, SiteMeta};
pub use panel::Panel;
pub use theme_picker::ThemePicker;
// Re-exported so UI consumers have one import path.
pub use zwipe_core::domain::user::{models::theme::ThemeConfig, preferences::ALLOWED_THEMES};

/// The component rules, for consumers outside this workspace.
pub const COMPONENTS_CSS: &str = include_str!("../assets/components.css");
/// The theme palettes, for consumers outside this workspace.
pub const THEMES_CSS: &str = include_str!("../assets/themes.css");
