//! Drawing pieces the screens share, each in the account theme's [`Palette`](crate::domain::theme::Palette).

/// The bottom row of key hints.
pub(crate) mod action_bar;
/// Bar charts from the deck metrics.
pub(crate) mod charts;
/// Text widths and heights in cells.
pub(crate) mod measure;
/// A framed panel centered over the screen.
pub(crate) mod panel;
