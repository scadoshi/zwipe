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
/// Expandable card rows with their classification, printings and image keys.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Activity, Playstyle and the Commander maybeboard list their cards with it"
    )
)]
pub(crate) mod card_rows;
/// A card as lines of text.
pub(crate) mod card_text;
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
/// The color map's tiles and key.
pub(crate) mod color_map;
/// A screen's entrance.
pub(crate) mod entrance;
/// The compact strip of three rolling figures.
pub(crate) mod figures;
/// One-time hint dialogs.
pub(crate) mod hint;
/// The "?" popup of bullet points.
pub(crate) mod info;
/// A list of items of different heights, one focused, paged to keep it in view.
pub(crate) mod item_list;
/// The ZWIPE logo decoding in.
pub(crate) mod logo;
/// Mana colors and pips.
pub(crate) mod mana;
/// Text widths and heights in cells.
pub(crate) mod measure;
/// An oracle tag's definition, and a row of tag chips with one open.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "card rows and the Activity and Playstyle charts reveal tags with it"
    )
)]
pub(crate) mod otag;
/// A framed panel centered over the screen.
pub(crate) mod panel;
/// The bottom sheet with a scrolling body.
pub(crate) mod sheet;
/// Ghost rows for content still loading.
pub(crate) mod skeleton;
/// Delayed and rising entrances inside a screen, and the hero's scope-in.
pub(crate) mod stagger;
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
