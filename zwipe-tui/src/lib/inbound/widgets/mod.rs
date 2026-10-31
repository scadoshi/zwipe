//! Drawing pieces the screens share, each in the account theme's [`Palette`](crate::domain::theme::Palette).
//!
//! A widget the screens use only part of, or none of yet, carries an `expect(dead_code)` outside tests; the screen that uses the rest deletes that line.

/// The bottom row of key hints.
pub(crate) mod action_bar;
/// A titled alert with buttons, one focused.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the screens use part of its API so far")
)]
pub(crate) mod alert;
/// Opening a link in the browser.
pub(crate) mod browser;
/// Bar charts from the deck metrics.
pub(crate) mod charts;
/// A row of chips to pick one or several.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the screens use part of its API so far")
)]
pub(crate) mod chips;
/// Copying text with OSC 52.
#[expect(
    dead_code,
    reason = "no screen uses it yet; its tests write to a buffer, not the terminal"
)]
pub(crate) mod clipboard;
/// Sections that open and close, and the accordion over them.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod collapsible;
/// The screen header: `!`, the title, and `?`.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod header;
/// The `!` Help & feedback dialog.
pub(crate) mod help;
/// `?` topics and one-time hint dialogs.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod hint;
/// The "?" popup of bullet points.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod info;
/// Mana colors and pips.
pub(crate) mod mana;
/// Text widths and heights in cells.
pub(crate) mod measure;
/// A framed panel centered over the screen.
pub(crate) mod panel;
/// The bottom sheet with a scrolling body.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod sheet;
/// Ghost rows for content still loading.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod skeleton;
/// Figures in a row, rolling while they load.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen uses it yet"))]
pub(crate) mod stats;
/// A one-line text box.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the screens use part of its API so far")
)]
pub(crate) mod text_input;
/// Short messages in the bottom-right corner.
#[cfg_attr(not(test), expect(dead_code, reason = "no screen raises a toast yet"))]
pub(crate) mod toast;
