//! External service integrations (API client, session management).
//!
//! Handles communication with external services from the frontend:
//! - **Client**: HTTP API client for backend communication
//! - **Session**: User session and authentication state management

/// Android private-storage path resolution, shared by `session` + `theme_store`.
#[cfg(target_os = "android")]
pub mod android_fs;
/// URL construction for external card retailer bulk-buy tools.
pub mod buy_links;
/// HTTP API client for backend communication.
pub mod client;
/// Open a URL with the OS default handler (browser, mail app, etc.).
pub mod open_url;
/// User session and authentication state management.
pub mod session;
/// Local theme cache so pre-auth screens render in the last-used theme.
pub mod theme_store;
