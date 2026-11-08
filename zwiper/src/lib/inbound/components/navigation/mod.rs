//! Navigation plumbing shared across routes.

/// OS back-intent bridge (edge swipe / hardware back → router `go_back`).
pub mod back_handler;

/// The screen that opened a deck, for the deck view's Back.
pub mod deck_opener;

/// Asks before a screen with unsaved changes is left.
pub mod leave_guard;

/// The main menu behind the header's hamburger.
pub mod menu;

/// The main menu's theme buttons: the palette opening the Themes sheet, and
/// the sun or moon flipping Dark/Light.
pub mod menu_theme;
