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

#[sqlx::test]
async fn the_changelog_answers_every_origin_alike_and_never_with_credentials(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    // The edge cache can hand one stored copy to any requester, so the copy must suit them all:
    // the web app, zwipe.net, an origin off the list, and the native app, which sends none. The
    // edge revalidates each load, so a 304 must say the same as a 200.
    for origin in [
        Some(common::TEST_ORIGIN),
        Some("https://zwipe.net"),
        Some("https://example.com"),
        None,
    ] {
        let origin_header: Vec<(header::HeaderName, String)> =
            origin.map_or_else(Vec::new, |o| vec![(header::ORIGIN, o.to_owned())]);
        let (_, full, _) = app
            .send_raw(Method::GET, CHANGELOG_ROUTE, None, None, &origin_header)
            .await;
        let mut conditional = origin_header.clone();
        conditional.push((
            header::IF_NONE_MATCH,
            full.get(header::ETAG).unwrap().to_str().unwrap().to_owned(),
        ));
        let (status, not_modified, _) = app
            .send_raw(Method::GET, CHANGELOG_ROUTE, None, None, &conditional)
            .await;
        assert_eq!(status, StatusCode::NOT_MODIFIED);
        for headers in [full, not_modified] {
            assert_eq!(
                headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
                "*",
                "origin {origin:?}"
            );
            assert!(
                !headers.contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
                "no credentials for origin {origin:?}"
            );
        }
    }
}

#[sqlx::test]
async fn other_routes_still_answer_only_a_listed_origin_with_credentials(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, headers, _) = app
        .send_raw(
            Method::GET,
            zwipe_core::http::paths::MIN_CLIENT_VERSION_ROUTE,
            None,
            None,
            &[(header::ORIGIN, common::TEST_ORIGIN.to_owned())],
        )
        .await;
    assert_eq!(
        headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
        common::TEST_ORIGIN
    );
    assert_eq!(
        headers
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .unwrap(),
        "true"
    );
    let (_, headers, _) = app
        .send_raw(
            Method::GET,
            zwipe_core::http::paths::MIN_CLIENT_VERSION_ROUTE,
            None,
            None,
            &[(header::ORIGIN, "https://example.com".to_owned())],
        )
        .await;
    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
}
