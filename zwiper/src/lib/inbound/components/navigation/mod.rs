//! Navigation plumbing shared across routes.

/// OS back-intent bridge (edge swipe / hardware back → router `go_back`).
pub mod back_handler;

/// The main menu behind the header's hamburger.
pub mod menu;

/// The main menu's theme row: the theme's name opening the Themes sheet, and
/// the Dark/Light switch.
pub mod menu_theme;
