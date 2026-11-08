//! Public changelog handler (release-history feed).

use std::sync::LazyLock;

use axum::Json;
use axum::body::Bytes;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};
use zwipe_core::http::contracts::changelog::HttpChangelog;

/// `Cache-Control` on every changelog answer: a cache may keep a copy but
/// revalidates it before each use, so a deploy's changelog shows on the next
/// load, and a copy that is still current costs a bodyless 304.
const CHANGELOG_CACHE: &str = "public, no-cache";

/// The changelog as served: its JSON body and the strong `ETag` over it.
/// The changelog is compiled into the binary, so both are built once.
#[derive(Debug)]
struct Feed {
    body: Bytes,
    etag: HeaderValue,
}

impl Feed {
    /// `None` only if the changelog fails to serialize, which a plain data
    /// struct never does; the handler then serves it without an `ETag`.
    fn build() -> Option<Self> {
        let body = serde_json::to_vec(&HttpChangelog::current()).ok()?;
        let digest = hex::encode(Sha256::digest(&body));
        let etag = HeaderValue::from_str(&format!("\"{digest}\"")).ok()?;
        Some(Self {
            body: Bytes::from(body),
            etag,
        })
    }
}

static FEED: LazyLock<Option<Feed>> = LazyLock::new(Feed::build);

/// Returns the changelog: upcoming teasers plus the shipped release history.
///
/// Public and unauthenticated: the changelog is identical for every user and
/// wanted pre-login. The data is compiled into the server binary
/// (`zwipe_core::content::changelog`), so updating it is a plain server
/// deploy, no app resubmit. Clients fall back to their own compiled-in copy
/// if the fetch fails.
///
/// Sent with `no-cache` and an `ETag`; a request whose `If-None-Match` names
/// the current `ETag` gets a 304 with no body.
pub async fn get_changelog(headers: HeaderMap) -> Response {
    let cache = (
        header::CACHE_CONTROL,
        HeaderValue::from_static(CHANGELOG_CACHE),
    );
    let Some(feed) = FEED.as_ref() else {
        return ([cache], Json(HttpChangelog::current())).into_response();
    };
    let etag = (header::ETAG, feed.etag.clone());
    let current = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| names_etag(value, &feed.etag));
    if current {
        return (StatusCode::NOT_MODIFIED, [cache, etag]).into_response();
    }
    let json = (
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    ([cache, etag, json], feed.body.clone()).into_response()
}

/// Whether an `If-None-Match` value names `etag`: `*`, or a comma-separated
/// list holding it. Compared weakly, since an edge that compresses the body
/// hands the browser a weak `W/` form of the strong tag.
fn names_etag(if_none_match: &str, etag: &HeaderValue) -> bool {
    let Ok(etag) = etag.to_str() else {
        return false;
    };
    if_none_match.split(',').map(str::trim).any(|candidate| {
        candidate == "*" || candidate.strip_prefix("W/").unwrap_or(candidate) == etag
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_etag_follows_the_body() {
        let feed = Feed::build().unwrap();
        let again = Feed::build().unwrap();
        assert_eq!(feed.etag, again.etag, "one body, one tag");
        assert_eq!(feed.etag.len(), 66, "a quoted SHA-256 in hex");
    }

    #[test]
    fn if_none_match_names_the_tag_in_any_form() {
        let etag = HeaderValue::from_static("\"abc\"");
        assert!(names_etag("\"abc\"", &etag));
        assert!(names_etag("W/\"abc\"", &etag), "weakened by the edge");
        assert!(names_etag("\"old\", \"abc\"", &etag), "in a list");
        assert!(names_etag("*", &etag));
        assert!(!names_etag("\"old\"", &etag));
        assert!(!names_etag("abc", &etag), "unquoted is another tag");
    }
}
