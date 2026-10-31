//! The terminal side: the back stack of screens, the main loop, and the widgets the screens draw with.

/// The app state: the back stack and what every screen shares.
pub(crate) mod app;
/// The main loop: keys, background requests and their results.
pub(crate) mod event_loop;
/// The back stack of screens.
pub(crate) mod nav;
/// Each screen in its own folder, and the enums that register them.
pub(crate) mod screens;
/// Which way a focus moves along its order.
pub(crate) mod toward;
/// Drawing pieces shared by the screens.
pub(crate) mod widgets;
