//! Navigation plumbing shared across routes.

/// OS back-intent bridge (edge swipe / hardware back → router `go_back`).
pub mod back_handler;

/// The main menu behind the header's hamburger.
pub mod menu;
