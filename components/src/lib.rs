//! The Magic pieces of the Zwipe UI, over [orangui](https://github.com/scadoshi/orangui).
//!
//! orangui is the generic kit: themes, panels, nav, charts, the entrance
//! effects and the app and site kits. This crate re-exports, by name, the
//! orangui items `zwiper` (the app) and `zite` (the site) use, so they keep one
//! import path, and adds what needs `zwipe-core`: [`CardDetails`], [`CardRow`],
//! [`CardRoleChips`], [`RoleTags`], [`OtagDef`], [`KeywordChips`],
//! [`OracleText`], [`FlippableCardImage`], [`ChangelogFilters`] and
//! [`ChangelogReleases`], the deck charts ([`BarChart`], [`DeckCharts`], [`ColorChart`], [`ManaCurve`], [`ManaFulfillment`],
//! [`DrawOdds`], [`ChartLabel`], with [`Fold`] and [`SeeMore`] behind their
//! long rows), the [`ColorBars`], the deck page the sites share ([`DeckPage`],
//! [`DeckHead`], [`FeaturedCard`], [`DeckControls`], [`DeckBand`],
//! [`DeckFold`] and [`sections`]), [`ZwipeArchitecture`] and the [`hint`]
//! renderer that draws zwipe-ui's typed hint copy. orangui's [`icons`] and
//! dock come through by name like the rest of the kit. The pieces the two
//! clients' chrome shares sit here too: [`DangerZoneAction`], the one-row
//! Danger zone panel, [`MenuSection`], the main menu's section list, and
//! Home's [`HomeDecks`] and [`HomeColors`] panels with [`ColorsEmpty`], the
//! line the Colors bars show before any deck has a card. [`SiteFooter`] is
//! zwipe.net's footer, which the web app draws too.
//!
//! Styling ships as string constants to inline through `document::Style`.
//! orangui's sheets come through as they are; [`MAGIC_CSS`] holds this crate's
//! own rules (oracle text, keyword chips, card rows, the flippable card image,
//! the changelog, the color bars, the mana colors and the mana glyph inside a
//! chip). **CSS cascade order
//! matters:** `THEMES_CSS`, then `COMPONENTS_CSS`, then `MAGIC_CSS`, then
//! `APP_CSS` or `SITE_CSS`, then [`SCREENS_CSS`] in the app and the web, then
//! the host's own stylesheet, so component rules
//! resolve theme variables and host rules override shared defaults at equal
//! specificity.
//!
//! The hosts hold orangui's [`ThemeConfig`]; [`theme_from_settings`] builds
//! it from a user's saved settings, and a test in `theme.rs` keeps
//! zwipe-core's theme list equal to orangui's.
//!
//! These components deliberately depend only on base `dioxus` (no platform
//! features), orangui, `zwipe-core` and `zwipe-ui`, so any Dioxus target can
//! consume them. The `toast` feature enables orangui's toast stack with
//! `use_toast`, `Toasts` and `ToastOptions`, and `links` its in-app browser
//! (`open_url`, `use_in_app_links`).

mod architecture;
mod card_details;
mod card_filter;
mod card_role_chips;
mod card_row;
mod changelog;
mod charts;
mod color_bars;
mod danger_zone;
mod deck_page;
mod flippable_card_image;
mod fold;
pub mod hint;
mod home_colors;
mod home_decks;
mod hover_stack;
mod keyword_chips;
mod menu_section;
mod oracle_text;
mod role_tags;
mod site_footer;
mod theme;

pub use architecture::ZwipeArchitecture;
pub use card_details::{CardDetails, card_face_count};
pub use card_filter::{CardFilterPanel, DictionarySlot};
pub use card_role_chips::CardRoleChips;
pub use card_row::CardRow;
pub use changelog::{
    ChangelogFilters, ChangelogLook, ChangelogReleases, Filter as ChangelogFilter,
};
pub use charts::{
    Bar, BarChart, BarOrientation, ChartLabel, ColorChart, DeckCharts, DrawOdds, LabelCase,
    ManaCurve, ManaFulfillment,
};
pub use color_bars::{ColorBars, ColorBarsGhost};
pub use danger_zone::DangerZoneAction;
pub use deck_page::{
    DeckBand, DeckControls, DeckFacts, DeckFilter, DeckFold, DeckHead, DeckPage, DeckSection,
    FeaturedCard, NO_CARDS, sections,
};
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use fold::{FOLD_AT, Fold, SEE_LESS, SEE_MORE, SeeMore, fold_split};
pub use home_colors::{ColorsEmpty, HomeColors};
pub use home_decks::HomeDecks;
pub use hover_stack::{HoverStack, HoverStackHost, use_hover_stack};
pub use keyword_chips::{KeywordChips, KeywordReminders};
pub use menu_section::MenuSection;
pub use oracle_text::OracleText;
pub use orangui::{
    ALLOWED_THEMES, APP_CSS, ActionBar, ActionButton, ActionDock, AlertDialog, BOOT_BG, BOOT_HEAD,
    BRAND_RESET_JS, Banner, BootReveal, BottomSheet, Button, ButtonVariant, COMPONENTS_CSS,
    ChartTip, Chip, ClearWrap, DarkModeButton, Decode, DialogAction, DialogHostView, DockAction,
    DockFilter, DockPlace, GalleryFooter, GalleryFrame, GridCells, HEAT_CELL, HEAT_ROWS, HeatCell,
    HeatGrid, HeatHit, HeatReadout, HintBullet, HintBullets, HintDialog, HintKey, HintLine,
    InfoButton, KEYBOARD_INSET_JS, ModeIcon, NAV_GLIDE_JS, NavBar, NavCtaIcon, PageHeader,
    PageMeta, PaletteIcon, Panel, PasswordInput, RETIRED_THEMES, REVEAL_JS, Replay, SCROLL_FADE_JS,
    SITE_CSS, SheetFrame, SheetWide, SiteMeta, Splash, SplashGate, StatsStrip, THEME_GROUPS,
    THEMES_CSS, TOAST_LONG, TOAST_NORMAL, TOAST_QUICK, TextInput, ThemeConfig, ThemePicker,
    ThemeSheet, heat_span, icons, peak_indices, status_uses, use_dialog_host, use_heat_pick,
    use_os_back, use_overlay_back, use_overlay_back_action, use_overlay_back_stack,
    use_persisted_theme, use_theme_follow, use_theme_wipe, with_separators,
};
#[cfg(feature = "links")]
pub use orangui::{OpenUrlError, open_url, use_in_app_links};
#[cfg(feature = "toast")]
pub use orangui::{TOAST_CSS, ToastOptions, ToastStack, Toasts, use_toast};
pub use role_tags::{OtagDef, RoleTags};
pub use site_footer::SiteFooter;
pub use theme::{pip_style_class, theme_from_settings};

/// The Magic rules: oracle text, keyword chips, card rows, the flippable card
/// image, the changelog, the color bars and the mana glyph inside a chip. Load it after
/// orangui's [`COMPONENTS_CSS`] and before [`APP_CSS`] or [`SITE_CSS`].
pub const MAGIC_CSS: &str = include_str!("../assets/magic.css");

/// The screen rules the app and the web both draw with, in `main.css`'s
/// order. zwiper and zwiper-web load it right after [`APP_CSS`] and before
/// their own stylesheets; zite does not load it.
pub const SCREENS_CSS: &str = concat!(
    include_str!("../assets/screens/base.css"),
    include_str!("../assets/screens/components.css"),
    include_str!("../assets/screens/screens-a.css"),
    include_str!("../assets/screens/screens-b.css"),
);

/// The filter accordion's rules. zwiper loads it after its own `main.css`
/// and zwiper-web after [`SCREENS_CSS`], each before `TOAST_CSS`; zite
/// does not load it.
pub const ACCORDION_CSS: &str = include_str!("../assets/accordion.css");

/// The [`CardFilterPanel`]'s own rules: its title bar's `?`, the Oracle tags
/// trigger's focus ring, the chips' wrappers and the picker's highlighted
/// result. A host that draws the panel loads it after its other shared
/// sheets and before its own stylesheet.
pub const FILTER_CSS: &str = include_str!("../assets/filter.css");
