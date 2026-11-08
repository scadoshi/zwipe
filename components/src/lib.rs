//! The Magic pieces of the Zwipe UI, over [orangui](https://github.com/scadoshi/orangui).
//!
//! orangui is the generic kit: themes, panels, nav, charts, the entrance
//! effects and the app and site kits. This crate re-exports, by name, the
//! orangui items `zwiper` (the app) and `zite` (the site) use, so they keep one
//! import path, and adds what needs `zwipe-core`: [`CardDetails`], [`CardRow`] and the [`CardListFold`] that holds a list of them,
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
mod back_online;
mod card_details;
mod card_filter;
mod card_list_fold;
mod card_role_chips;
mod card_row;
mod changelog;
mod charts;
mod color_bars;
mod cover_flow;
mod danger_zone;
mod deck_page;
mod drawer_sheet;
mod flippable_card_image;
mod fold;
pub mod hint;
mod home_colors;
mod home_decks;
mod hover_stack;
mod identity_pips;
mod keyword_chips;
mod load_failed;
mod menu_section;
mod oracle_text;
mod role_tags;
mod share_url;
mod site_footer;
mod theme;

pub use architecture::ZwipeArchitecture;
pub use back_online::use_back_online;
pub use card_details::{
    CardDetails, CardDetailsCost, CardDetailsPanel, CardDetailsPanelSkeleton, card_face_count,
};
pub use card_filter::{CardFilterPanel, DictionarySlot};
pub use card_list_fold::CardListFold;
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
pub use cover_flow::CoverFlow;
pub use danger_zone::DangerZoneAction;
pub use deck_page::{
    DeckBand, DeckControls, DeckFacts, DeckFold, DeckHead, DeckPage, DeckSection, FeaturedCard,
    NO_CARDS, sections,
};
pub use drawer_sheet::DrawerSheet;
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use fold::{FOLD_AT, Fold, SEE_LESS, SEE_MORE, SeeMore, fold_split};
pub use home_colors::{ColorsEmpty, HomeColors};
pub use home_decks::HomeDecks;
pub use hover_stack::{HoverStack, HoverStackHost, use_hover_stack};
pub use identity_pips::IdentityPips;
pub use keyword_chips::{KeywordChips, KeywordReminders};
pub use load_failed::LoadFailed;
pub use menu_section::MenuSection;
pub use oracle_text::OracleText;
pub use orangui::{
    ALLOWED_THEMES, APP_CSS, ActionBar, ActionButton, ActionDock, AlertDialog, AppLifecycle,
    AppPhase, BOOT_BG, BOOT_HEAD, BRAND_RESET_JS, Banner, BootReveal, BottomSheet, Button,
    ButtonVariant, COMPONENTS_CSS, ChartTip, Chip, ClearWrap, DarkModeButton, Decode, DialogAction,
    DialogHostView, DockAction, DockFilter, DockPlace, GalleryFooter, GalleryFrame, GridCells,
    HEAT_CELL, HEAT_ROWS, HeatCell, HeatGrid, HeatHit, HeatReadout, HintBullet, HintBullets,
    HintDialog, HintKey, HintLine, InfoButton, KEYBOARD_INSET_JS, ModeIcon, NAV_GLIDE_JS, NavBar,
    NavCtaIcon, PageHeader, PageMeta, PaletteIcon, Panel, PasswordInput, RETIRED_THEMES, REVEAL_JS,
    Replay, SCROLL_FADE_JS, SITE_CSS, SheetFrame, SheetWide, SiteMeta, Splash, SplashGate,
    StatsStrip, THEME_GROUPS, THEMES_CSS, TOAST_LONG, TOAST_NORMAL, TOAST_QUICK, TextInput,
    ThemeConfig, ThemePicker, ThemeSheet, heat_span, icons, peak_indices, status_uses,
    use_app_lifecycle, use_app_lifecycle_root, use_dialog_host, use_heat_pick, use_on_resume,
    use_os_back, use_overlay_back, use_overlay_back_action, use_overlay_back_stack,
    use_page_kill_recovery, use_persisted_theme, use_theme_follow, use_theme_wipe, with_separators,
};
#[cfg(feature = "links")]
pub use orangui::{OpenUrlError, open_url, use_in_app_links};
#[cfg(feature = "toast")]
pub use orangui::{TOAST_CSS, ToastOptions, ToastStack, Toasts, use_toast};
pub use role_tags::{OtagDef, RoleTags};
pub use share_url::ShareUrl;
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

/// The card filter's rules, for the app's filter sheet and for
/// [`DrawerSheet`] and [`CardFilterPanel`]: the label rows, compact boxes,
/// stepper, floating picker results, small chips and clear buttons, the
/// layout helpers its rows use, the section accordion, and the sheet's own
/// `?`, focus ring, chip wrappers and highlighted result. Every host that
/// draws the filter loads it after [`SCREENS_CSS`] (zite, which has none,
/// after `SITE_CSS`) and before its own stylesheet.
pub const FILTER_CSS: &str = include_str!("../assets/filter.css");

/// The on-screen keyboard's helper for the apps and the web's phone layout:
/// it keeps the focused field in view above the keyboard and marks the root
/// `keyboard-up` while the keyboard shows, which hides the floating docks.
/// Inline it once with `document::Script`, beside orangui's
/// `KEYBOARD_INSET_JS`.
pub const KEYBOARD_JS: &str = include_str!("../assets/keyboard.js");
