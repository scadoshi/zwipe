//! The terminal side: the screens' state, the main loop, and drawing.

/// Which screen is up and its state.
pub(crate) mod app;
/// The main loop: keys, background requests and their results.
pub(crate) mod event_loop;
/// Drawing each screen.
pub(crate) mod ui;
