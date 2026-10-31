//! Drawing pieces the screens share, each in the account theme's [`Palette`](crate::domain::theme::Palette).
//!
//! A widget no screen uses yet carries an `expect(dead_code)` outside tests on its `mod` line here, and an unused item in a widget that is otherwise in use carries one of its own; the screen that starts using it deletes that line.

/// The bottom row of key hints.
pub(crate) mod action_bar;
/// A titled alert with buttons, one focused.
pub(crate) mod alert;
/// Opening a link in the browser.
pub(crate) mod browser;
/// A card's info block, details and text stand-in.
pub(crate) mod card_info;
pub(crate) mod card_motion;
pub(crate) mod card_rows;
/// A card as lines of text.
pub(crate) mod card_text;
/// Bar charts from the deck metrics.
pub(crate) mod charts;
/// A row of chips to pick one or several.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no screen draws a chip row through it yet; the screens build their chips with `pill`"
    )
)]
pub(crate) mod chips;
/// Copying text with OSC 52.
pub(crate) mod clipboard;
/// Sections that open and close, and the accordion over them.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no screen uses it: the deck view shows its sections as panels"
    )
)]
pub(crate) mod collapsible;
/// The color map's tiles and key.
pub(crate) mod color_map;
pub(crate) mod count_up;
pub(crate) mod entrance;
/// The compact strip of three rolling figures.
pub(crate) mod figures;
/// The backdrop of faint tiles, still or raining in.
pub(crate) mod grid;
/// The screen header: `!`, the title, and `?`.
pub(crate) mod header;
pub(crate) mod help;
pub(crate) mod hero;
pub(crate) mod hint;
/// The "?" popup of bullet points.
pub(crate) mod info;
/// A list of items of different heights, one focused, paged to keep it in view.
pub(crate) mod item_list;
pub(crate) mod logo;
/// Mana colors and pips.
pub(crate) mod mana;
/// Text widths and heights in cells.
pub(crate) mod measure;
/// A sheet of buttons, one focused.
pub(crate) mod menu;
pub(crate) mod otag;
/// A framed panel centered over the screen.
pub(crate) mod panel;
pub(crate) mod pill;
/// The bottom sheet with a scrolling body.
pub(crate) mod sheet;
/// Ghost rows for content still loading.
pub(crate) mod skeleton;
/// The turning glyph of a request in flight.
pub(crate) mod spinner;
/// The opening over the first screen.
pub(crate) mod splash;
pub(crate) mod stagger;
/// Figures in a row, rolling while they load.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "no screen shows a stats strip yet")
)]
pub(crate) mod stats;
/// A text box of many lines.
pub(crate) mod text_area;
/// A one-line text box.
pub(crate) mod text_input;
pub(crate) mod toast;
/// The sweep from one theme to the next.
pub(crate) mod wipe;
