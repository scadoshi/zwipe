//! Public changelog endpoint. Serves the compiled-in release history so clients
//! can fetch fresh entries without an app resubmit. No auth, no DB.
//!
//! Requires `DATABASE_URL` (to build the app): `set -a; source zerver/.env; set +a`.

use zwipe_core::http::paths::CHANGELOG_ROUTE;
mod common;

use axum::http::{Method, StatusCode, header};
use common::TestApp;
use zwipe_core::content::changelog::{DISPLAY_DATE_FORMAT, RELEASES, UPCOMING};

#[sqlx::test]
async fn changelog_serves_the_compiled_in_history(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get(CHANGELOG_ROUTE, None).await;
    assert_eq!(status, StatusCode::OK, "GET /api/changelog: {body}");

    let releases = body["releases"].as_array().unwrap();
    let upcoming = body["upcoming"].as_array().unwrap();
    assert_eq!(
        releases.len(),
        RELEASES.len(),
        "release count matches source"
    );
    assert_eq!(
        upcoming.len(),
        UPCOMING.len(),
        "upcoming count matches source"
    );

    // Newest first, and each entry carries its notes.
    let latest = &releases[0];
    assert_eq!(
        latest["version"].as_str().unwrap(),
        RELEASES[0].version.to_string()
    );
    // `date` keeps the display string old apps render; `released_on` is the
    // same day as an ISO date, added alongside it.
    let day = RELEASES[0].date.unwrap();
    assert_eq!(
        latest["date"].as_str().unwrap(),
        day.format(DISPLAY_DATE_FORMAT).to_string()
    );
    assert_eq!(
        latest["released_on"].as_str().unwrap(),
        day.format("%Y-%m-%d").to_string()
    );
    assert_eq!(
        latest["entries"].as_array().unwrap().len(),
        RELEASES[0].entries.len(),
        "latest release keeps all its bullets"
    );

    // Upcoming has not shipped: an empty `date` and no `released_on`.
    for release in upcoming {
        assert_eq!(release["date"].as_str().unwrap(), "");
        assert!(release.get("released_on").is_none(), "{release}");
    }
}

/// The changelog changes on a deploy, so every load revalidates: `no-cache`
/// with an `ETag`, and a 304 with no body when the copy is still current.
#[sqlx::test]
async fn changelog_revalidates_on_every_load(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    let (status, headers, _) = app
        .send_raw(Method::GET, CHANGELOG_ROUTE, None, None, &[])
        .await;
    assert_eq!(status, StatusCode::OK);
    let cache = headers
        .get(header::CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(cache, "public, no-cache", "no max-age to outlive a deploy");
    let etag = headers
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        etag.starts_with('"') && etag.ends_with('"'),
        "strong ETag: {etag}"
    );

    let (status, headers, body) = app
        .send_raw(
            Method::GET,
            CHANGELOG_ROUTE,
            None,
            None,
            &[(header::IF_NONE_MATCH, etag.clone())],
        )
        .await;
    assert_eq!(status, StatusCode::NOT_MODIFIED, "current copy: {body}");
    assert_eq!(body, serde_json::Value::Null, "a 304 has no body");
    assert_eq!(headers.get(header::ETAG).unwrap().to_str().unwrap(), etag);
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .unwrap()
            .to_str()
            .unwrap(),
        "public, no-cache"
    );

    let (status, _, _) = app
        .send_raw(
            Method::GET,
            CHANGELOG_ROUTE,
            None,
            None,
            &[(header::IF_NONE_MATCH, "\"stale\"".to_owned())],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "a stale copy gets the new body");
}
