//! A shared deck's preview image, the PNG its link unfurls with.

use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use zwipe_core::http::paths::shared_deck_preview_version;

use super::get_shared_deck::{share_token, shared_body};
use crate::inbound::http::{ApiError, AppState};

/// How long an image for the deck's current version is cached: it never
/// changes, since an edit gives the deck a new version and a new URL.
const CURRENT_CACHE: &str = "public, max-age=31536000, immutable";

/// How long an image asked for under an old or missing version is cached:
/// briefly, so it follows the deck rather than pinning one render.
const STALE_CACHE: &str = "public, max-age=300";

/// How long the fallback redirect is cached when drawing failed, so the next
/// request soon tries again.
const FALLBACK_CACHE: &str = "public, max-age=60";

/// The site-wide preview image, under the public web base URL.
const FALLBACK_IMAGE: &str = "/assets/og-default.png";

/// The preview URL's query: `v`, the version the page asked for, the layout
/// and the unix seconds of the deck's `updated_at` as
/// [`shared_deck_preview_version`] writes them. Any value parses; one that
/// isn't the current version just gets the short cache.
#[derive(Debug, Default, Deserialize)]
pub struct PreviewQuery {
    /// The version the URL names.
    #[serde(default)]
    pub v: Option<String>,
}

/// The deck's preview image: 1200x630 PNG in its owner's theme, cached a
/// year when `v` names the current layout and the deck's current version,
/// and five minutes otherwise. When drawing fails, a redirect to the site-wide image.
///
/// # Errors
///
/// Returns 404 for a malformed, revoked or unknown token, and 500 on a
/// database failure.
pub async fn get_shared_deck_preview(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Query(query): Query<PreviewQuery>,
) -> Result<Response, ApiError> {
    let token = share_token(&token)?;
    let deck = Box::pin(shared_body(&state, token)).await?;
    let current = deck
        .updated_at
        .map(|t| shared_deck_preview_version(t.timestamp()));
    let fresh = current.is_some() && current == query.v;
    let Some(png) = state.share_preview.png(token, &deck).await else {
        let fallback = format!("{}{FALLBACK_IMAGE}", state.web_base_url);
        return Ok((
            StatusCode::FOUND,
            [
                (header::LOCATION, fallback),
                (header::CACHE_CONTROL, FALLBACK_CACHE.to_owned()),
            ],
        )
            .into_response());
    };
    let cache = if fresh { CURRENT_CACHE } else { STALE_CACHE };
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, cache),
        ],
        png,
    )
        .into_response())
}
