//! The network and the disk: the signed-in account, its saved session, and the remembered theme.

/// The API client, the session, and the session file.
pub(crate) mod account;
/// Everybody's counts as last fetched.
pub(crate) mod counts_file;
/// The crash report a panic leaves for the next launch to send.
pub(crate) mod crash_store;
/// The app's config folder and owner-only writes to it.
pub(crate) mod files;
/// Home's counters and scope between launches.
pub(crate) mod home_file;
/// Card images from Scryfall's image CDN.
pub(crate) mod images;
/// The last account's theme, for drawing before sign-in.
pub(crate) mod theme_file;
/// Handled errors waiting for the next usage batch.
pub(crate) mod usage_buffer;
