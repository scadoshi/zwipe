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
mod back_arrow;
mod back_online;
mod buy;
mod card_actions;
mod card_details;
mod card_details_sheet;
mod card_filter;
mod card_group;
mod card_icon_button;
mod card_list_fold;
mod card_proxy;
mod card_role_chips;
mod card_row;
mod card_skeleton;
mod card_stack;
mod changelog;
mod charts;
mod color_bars;
mod cover_flow;
mod danger_zone;
pub mod data_action;
mod deck_cards_view;
pub mod deck_cursor;
mod deck_group;
mod deck_hover_keys;
mod deck_page;
mod deck_stats;
mod details_clear;
mod details_eye;
mod drawer_sheet;
mod drawer_swap;
pub mod featured_strip;
mod flippable_card_image;
mod fold;
mod group_buy;
mod guides;
pub mod hint;
mod home_colors;
mod home_decks;
mod hover_stack;
mod identity_pips;
pub mod keys;
mod keyword_chips;
mod layer_presence;
mod load_failed;
mod media;
mod media_frame;
mod meld;
mod menu_section;
mod oracle_tag_examples;
mod oracle_text;
mod picker_search;
mod pressable;
mod printing_sheet;
mod role_tags;
pub mod screen_return;
pub mod settle;
mod share_url;
mod site_footer;
mod stacks_ghost;
mod staple_decks_chip;
pub mod swipe_stack;
mod theme;

pub use architecture::ZwipeArchitecture;
pub use back_arrow::BackArrow;
pub use back_online::use_back_online;
pub use buy::{CART, CartButton, StoreDialog, store_actions};
pub use card_actions::{CardBoardMoves, CardEditButtons};
pub use card_details::{
    CardDetails, CardDetailsCost, CardDetailsPanel, CardDetailsPanelSkeleton, card_face_count,
};
pub use card_details_sheet::CardDetailsSheet;
pub use card_filter::{CardFilterPanel, DictionarySlot, OPEN_SECTION_SCROLL_JS};
pub use card_group::{CardGroup, TailPlace, group_tail};
pub use card_icon_button::{
    CardIconButton, IMAGE_ICON, IMAGE_LABEL, ImageButton, MELD, STAR, StarButton,
};
pub use card_list_fold::CardListFold;
pub use card_proxy::{CardProxy, FIT_JS as CARD_PROXY_FIT_JS, ProxyFace, use_card_proxy_fit};
pub use card_role_chips::CardRoleChips;
pub use card_row::CardRow;
pub use card_skeleton::CardSkeleton;
pub use card_stack::{CardStack, StackCard};
pub use changelog::{
    ChangelogCollapseAll, ChangelogFilters, ChangelogLook, ChangelogReleases,
    Filter as ChangelogFilter,
};
pub use charts::{
    Bar, BarChart, BarOrientation, ChartLabel, ColorChart, DeckCharts, DrawOdds, LabelCase,
    ManaCurve, ManaFulfillment,
};
pub use color_bars::{ColorBars, ColorBarsGhost};
pub use cover_flow::CoverFlow;
pub use danger_zone::DangerZoneAction;
pub use deck_cards_view::{
    CardsViewSwitch, DRAWER_CLASSES, DeckCardsView, DeckEditActions, DeckRow, DeckRowHost,
    DeckStackDrawer, DeckStacks, OnPrinting, StackDetails, deck_sections, use_stack_details_sync,
};
pub use deck_group::DeckGroupSection;
pub use deck_hover_keys::{DeckHoverKeys, use_deck_hover_keys};
pub use deck_page::{
    DeckBand, DeckControls, DeckFacts, DeckFold, DeckHead, DeckPage, DeckSection, FeaturedCard,
    NO_CARDS, sections,
};
pub use deck_stats::{
    BudgetBody, CurrencyChips, DistributionsBody, DrawOddsStepper, InfoRow, ManaBody, StatHints,
    StatTopic, TagsBody, charted, draw_odds_hint,
};
pub use details_clear::use_details_clear;
pub use details_eye::{
    DETAILS_HIDDEN_CLASS, DetailsEye, DetailsEyeHost, DockEye, EyePress, Surface, beside_fits,
    eye_lit, starts_hidden, use_details_eye,
};
pub use drawer_sheet::DrawerSheet;
pub use flippable_card_image::{FlippableCardImage, reset_image_ease};
pub use fold::{FOLD_AT, Fold, SEE_LESS, SEE_MORE, SeeMore, fold_split};
pub use group_buy::{GroupBuy, GroupBuyDialog, use_group_buy};
pub use guides::{GuideArticle, GuideIndex, GuideMissing};
pub use home_colors::{ColorsEmpty, HomeColors};
pub use home_decks::HomeDecks;
pub use hover_stack::{HoverStack, HoverStackHost, use_hover_stack};
pub use identity_pips::IdentityPips;
pub use keyword_chips::{KeywordChips, KeywordReminders};
pub use layer_presence::{LAYER_OUT, LayerPresence, use_layer_presence};
pub use load_failed::LoadFailed;
pub use media::{
    COMPUTER_QUERY, KIT_MODAL_PX, POINTER_QUERY, WIDE_PX, use_computer, use_min_width, use_pointer,
    use_wide,
};
pub use media_frame::{MediaFrame, MediaSize};
pub use meld::{MeldFetch, MeldSource, MeldToggle, use_meld};
pub use menu_section::MenuSection;
pub use oracle_tag_examples::{ExamplesPage, ExamplesSource, OracleTagExamples};
pub use oracle_text::OracleText;
pub use orangui::{
    ALLOWED_THEMES, APP_CSS, ActionBar, ActionButton, ActionDock, AlertDialog, AppLifecycle,
    AppPhase, BOOT_BG, BOOT_HEAD, BRAND_RESET_JS, Banner, BootReveal, BottomSheet, Button,
    ButtonVariant, COMPONENTS_CSS, ChartTip, Chip, ClearWrap, CollapseSet, DIALOG_CSS,
    DarkModeButton, Decode, DialogAction, DialogHostView, DockAction, DockFilter, DockPlace,
    GalleryFooter, GalleryFrame, GridCells, HEAT_CELL, HEAT_ROWS, HeatCell, HeatGrid, HeatHit,
    HeatReadout, HintBullet, HintBullets, HintDialog, HintKey, HintLine, InfoButton,
    KEYBOARD_INSET_JS, ModeIcon, NAV_FOLD_CSS, NAV_GLIDE_JS, NavBar, NavCtaIcon, OverlayBackStack,
    PageHeader, PageMeta, PaletteIcon, Panel, PasswordInput, RETIRED_THEMES, REVEAL_JS, Replay,
    SCROLL_FADE_JS, SITE_CSS, SheetDim, SheetFrame, SheetWide, SiteMeta, Splash, SplashGate,
    StatsStrip, THEME_GROUPS, THEMES_CSS, TOAST_LONG, TOAST_NORMAL, TOAST_QUICK, TextInput,
    ThemeConfig, ThemePicker, ThemeSheet, heat_span, icons, peak_indices, status_uses,
    use_app_lifecycle, use_app_lifecycle_root, use_collapse_set, use_dialog_host, use_heat_pick,
    use_on_resume, use_os_back, use_overlay_back, use_overlay_back_action, use_overlay_back_stack,
    use_page_kill_recovery, use_persisted_theme, use_theme_follow, use_theme_wipe, with_separators,
};
#[cfg(feature = "links")]
pub use orangui::{OpenUrlError, open_url, use_in_app_links};
#[cfg(feature = "toast")]
pub use orangui::{TOAST_CSS, ToastOptions, ToastStack, Toasts, use_toast};
pub use picker_search::use_picker_search;
pub use pressable::{Pressable, is_press};
pub use printing_sheet::{
    ArtistChip, CardPricesGhost, PrintingSheet, PrintingsFetch, PrintingsSource,
};
pub use role_tags::{OtagDef, RoleTags};
pub use settle::use_settle;
pub use share_url::ShareUrl;
pub use site_footer::SiteFooter;
pub use stacks_ghost::DeckStacksGhost;
pub use staple_decks_chip::StapleDecksChip;
pub use theme::{pip_style_class, theme_from_settings};

/// The `JetBrains Mono` faces at 400, 500 and 700, from the complete font files,
/// which each client bundles unhashed at `/assets/`. Load it before every
/// other sheet. The logos' block characters need the complete files: a
/// Latin subset leaves them to a system font, which draws them heavier.
pub const FONTS_CSS: &str = include_str!("../assets/fonts.css");

/// The guide pages' rules (components' `GuideIndex`, `GuideArticle` and
/// [`MediaFrame`]), with the card grid, media frame and gallery chrome
/// zwipe.net's other pages share. A host loads it after its own sheet and
/// before the kit's `SITE_CSS`, with [`MEDIA_FRAME_JS`].
pub const GUIDES_CSS: &str = include_str!("../assets/guides.css");

/// Marks a [`MediaFrame`]'s image or video ready once it can paint, which
/// fades it in over its skeleton. A host runs it once per page.
pub const MEDIA_FRAME_JS: &str = include_str!("../assets/media_frame.js");

/// The deck page's cards on the two web sites: the wide page, the list's two columns from
/// 1400px, the rows' rules and the Stacks view. Load it after [`MAGIC_CSS`] and the screen rules.
/// The phone apps draw their stacks with their own rules and leave it out.
pub const DECK_PAGE_CSS: &str = include_str!("../assets/deck_page.css");

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
