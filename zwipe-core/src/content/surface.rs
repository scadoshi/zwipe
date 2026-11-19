//! Where a piece of shared copy shows: the app, or the web app.

/// The client a piece of shared copy is written for. Copy that reads the same everywhere carries
/// no surface; copy that differs while one client has a screen the other lacks names the client
/// it fits. zwipe.net's guides read as [`Surface::App`], the web app's own guides as
/// [`Surface::Web`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Surface {
    /// The iOS and Android app, and zwipe.net's guides.
    #[default]
    App,
    /// The web app.
    Web,
}
