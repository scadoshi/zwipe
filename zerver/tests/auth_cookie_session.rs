//! The browser's session routes through the real router: the refresh token
//! only ever travels in the `HttpOnly` cookie, never in a body.
//!
//! Requires `DATABASE_URL` (dev's value works): `set -a; source zerver/.env`.
#![expect(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: its helpers read a reply and panic the test when it is not the shape asserted"
)]

mod common;

use axum::http::{HeaderMap, Method, StatusCode, header};
use common::{TEST_ORIGIN, TestApp};
use zwipe_core::http::paths::{COOKIE_LOGIN_ROUTE, COOKIE_LOGOUT_ROUTE, COOKIE_REFRESH_ROUTE};

const COOKIE: &str = "__Host-zwipe_refresh";

/// The `Set-Cookie` line for the refresh cookie.
fn set_cookie(headers: &HeaderMap) -> String {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with(COOKIE))
        .unwrap_or_else(|| panic!("no {COOKIE} in {headers:?}"))
        .to_string()
}

/// `name=value` from a `Set-Cookie` line, as the browser sends it back.
fn pair(set_cookie: &str) -> String {
    set_cookie.split(';').next().unwrap().to_string()
}

fn origin() -> (header::HeaderName, String) {
    (header::ORIGIN, TEST_ORIGIN.to_string())
}

fn cookie(pair: &str) -> (header::HeaderName, String) {
    (header::COOKIE, pair.to_string())
}

/// Logs `name` in on the web route: the access token and the cookie pair.
async fn cookie_login(app: &TestApp, name: &str) -> (String, String, String) {
    app.register(name).await;
    let (status, headers, body) = app
        .send_raw(
            Method::POST,
            COOKIE_LOGIN_ROUTE,
            Some(serde_json::json!({ "identifier": name, "password": "TestPass123!" })),
            None,
            &[origin()],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "web login: {body}");
    let line = set_cookie(&headers);
    let token = body["access_token"]["value"].as_str().unwrap().to_string();
    (token, line.clone(), pair(&line))
}

#[sqlx::test]
async fn login_sets_a_locked_down_cookie_and_keeps_the_token_out_of_the_body(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    app.register("webby").await;
    let (status, headers, body) = app
        .send_raw(
            Method::POST,
            COOKIE_LOGIN_ROUTE,
            Some(serde_json::json!({ "identifier": "webby", "password": "TestPass123!" })),
            None,
            &[origin()],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["refresh_token"]["value"], "",
        "body carries no refresh token"
    );
    assert!(body["access_token"]["value"].as_str().is_some());

    let line = set_cookie(&headers);
    for attribute in [
        "HttpOnly",
        "Secure",
        "SameSite=Strict",
        "Path=/",
        "Max-Age=",
    ] {
        assert!(line.contains(attribute), "{attribute} missing from {line}");
    }
    assert!(
        !line.contains("Domain"),
        "a __Host- cookie has no Domain: {line}"
    );
}

#[sqlx::test]
async fn refresh_rotates_the_cookie_and_a_spent_one_is_refused(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, _, first) = cookie_login(&app, "rotor").await;

    let (status, headers, body) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            Some(serde_json::json!({})),
            None,
            &[origin(), cookie(&first)],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "refresh: {body}");
    assert_eq!(body["refresh_token"]["value"], "");
    let second = pair(&set_cookie(&headers));
    assert_ne!(first, second, "the cookie rotates");

    // The spent cookie is refused, and the reply expires it.
    let (status, headers, _) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            Some(serde_json::json!({})),
            None,
            &[origin(), cookie(&first)],
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(
        set_cookie(&headers).contains("Max-Age=0"),
        "a refused cookie is expired"
    );
}

#[sqlx::test]
async fn refresh_needs_the_cookie_and_an_allowed_origin(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, _, good) = cookie_login(&app, "guarded").await;
    let empty = Some(serde_json::json!({}));

    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            empty.clone(),
            None,
            &[origin()],
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "no cookie");

    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            empty.clone(),
            None,
            &[cookie(&good)],
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "no origin");

    let foreign = (header::ORIGIN, "https://evil.example".to_string());
    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            empty.clone(),
            None,
            &[foreign, cookie(&good)],
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "foreign origin");

    let forged = format!(
        "{COOKIE}=00000000-0000-0000-0000-000000000000.{}",
        "0".repeat(64)
    );
    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            empty,
            None,
            &[origin(), cookie(&forged)],
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "forged cookie");
}

#[sqlx::test]
async fn logout_expires_the_cookie_and_revokes_the_session(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _, pair) = cookie_login(&app, "leaver").await;

    let (status, headers, _) = app
        .send_raw(
            Method::POST,
            COOKIE_LOGOUT_ROUTE,
            None,
            Some(&token),
            &[origin(), cookie(&pair)],
        )
        .await;
    assert!(status.is_success(), "logout: {status}");
    assert!(
        set_cookie(&headers).contains("Max-Age=0"),
        "logout expires the cookie"
    );

    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            Some(serde_json::json!({})),
            None,
            &[origin(), cookie(&pair)],
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "the revoked cookie no longer refreshes"
    );
}
