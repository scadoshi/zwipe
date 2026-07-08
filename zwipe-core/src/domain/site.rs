//! Site-wide public constants: base URLs and contact points, shared by every
//! surface (app, site, server) so they can never drift apart.
//!
//! The URL constants are debug/release-gated: debug builds resolve to the
//! local dev servers so flows like share links and verification emails are
//! testable end-to-end without touching prod; release builds resolve to the
//! public domains. `zerver` layers an env override on top for its config
//! (see `zerver/src/lib/config.rs`); the clients bake these in directly.
//! (`zite/build.rs` keeps one mirrored literal for sitemap generation —
//! build scripts can't import the lib.)

/// Public web base URL (zite), no trailing slash.
#[cfg(debug_assertions)]
pub const WEB_BASE: &str = "http://localhost:8080";
/// Public web base URL (zite), no trailing slash.
#[cfg(not(debug_assertions))]
pub const WEB_BASE: &str = "https://zwipe.net";

/// API base URL (zerver), no trailing slash.
#[cfg(debug_assertions)]
pub const API_BASE: &str = "http://localhost:3000";
/// API base URL (zerver), no trailing slash.
#[cfg(not(debug_assertions))]
pub const API_BASE: &str = "https://api.zwipe.net";

/// User-facing support email address.
pub const SUPPORT_EMAIL: &str = "support@zwipe.net";

/// Discord community invite link.
pub const DISCORD_URL: &str = "https://discord.gg/s2UReqUUeg";
