//! Drawing pieces the screens share, each in the account theme's [`Palette`](crate::domain::theme::Palette).
//!
//! A widget the screens use only part of, or none of yet, carries an `expect(dead_code)` outside tests; the screen that uses the rest deletes that line.

/// The bottom row of key hints.
pub(crate) mod action_bar;
/// A titled alert with buttons, one focused.
pub(crate) mod alert;
/// Opening a link in the browser.
pub(crate) mod browser;
/// A card's info block, details and text stand-in.
pub(crate) mod card_info;
/// Bar charts from the deck metrics.
pub(crate) mod charts;
/// A row of chips to pick one or several.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the screens use part of its API so far")
)]
pub(crate) mod chips;
/// Copying text with OSC 52.
pub(crate) mod clipboard;
/// Sections that open and close, and the accordion over them.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod collapsible;
/// Rolling counters.
pub(crate) mod count_up;
/// A screen's entrance.
pub(crate) mod entrance;
/// The backdrop of faint tiles, still or raining in.
pub(crate) mod grid;
/// The screen header: `!`, the title, and `?`.
pub(crate) mod header;
/// The `!` Help & feedback dialog.
pub(crate) mod help;
/// The logo and everybody's counters on the sign-in screens.
pub(crate) mod hero;
/// `?` topics and one-time hint dialogs.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod hint;
/// The "?" popup of bullet points.
pub(crate) mod info;
/// The ZWIPE logo decoding in.
pub(crate) mod logo;
/// Mana colors and pips.
pub(crate) mod mana;
/// Text widths and heights in cells.
pub(crate) mod measure;
/// A sheet of buttons, one focused.
pub(crate) mod menu;
/// A framed panel centered over the screen.
pub(crate) mod panel;
/// One chip drawn on its own.
pub(crate) mod pill;
/// The bottom sheet with a scrolling body.
pub(crate) mod sheet;
/// Ghost rows for content still loading.
pub(crate) mod skeleton;
/// The turning glyph of a request in flight.
pub(crate) mod spinner;
/// The opening over the first screen.
pub(crate) mod splash;
/// Figures in a row, rolling while they load.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod stats;
/// A text box of many lines.
pub(crate) mod text_area;
/// A one-line text box.
pub(crate) mod text_input;
/// Short messages in the top-left corner.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen raises a toast yet"))]
pub(crate) mod toast;
/// The sweep from one theme to the next.
pub(crate) mod wipe;
