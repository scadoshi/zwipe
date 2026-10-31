//! The terminal side: the back stack of screens, the main loop, and the widgets the screens draw with.

/// The app state: the back stack and what every screen shares.
pub(crate) mod app;
pub(crate) mod event_loop;
/// The back stack of screens.
pub(crate) mod nav;
pub(crate) mod screens;
pub(crate) mod toward;
/// Drawing pieces shared by the screens.
pub(crate) mod widgets;
