//! Public changelog handler (release-history feed).

#[cfg(feature = "zerver")]
use axum::{Json, http::StatusCode};

#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::changelog::HttpChangelog;

/// Returns the changelog: upcoming teasers plus the shipped release history.
///
/// Public and unauthenticated — the changelog is identical for every user and
/// wanted pre-login. The data is compiled into the server binary
/// (`zwipe_core::domain::changelog`), so updating it is a plain server deploy,
/// no app resubmit. Clients fetch this at startup and fall back to their own
/// compiled-in copy if the fetch fails. Edge-cached by Cloudflare (no origin
/// `Cache-Control`; the TTL is configured on Cloudflare, matching the other
/// public endpoints).
#[cfg(feature = "zerver")]
pub async fn get_changelog() -> (StatusCode, Json<HttpChangelog>) {
    (StatusCode::OK, Json(HttpChangelog::current()))
}
