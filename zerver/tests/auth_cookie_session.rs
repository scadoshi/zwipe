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
use zwipe_core::http::paths::{
    COOKIE_LOGIN_ROUTE, COOKIE_LOGOUT_ROUTE, COOKIE_REFRESH_ROUTE, REFRESH_SESSION_ROUTE,
};

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
async fn refresh_rotates_the_cookie_and_a_spent_one_is_refused_past_the_window(pool: sqlx::PgPool) {
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

    // A second tab racing on the spent cookie, inside the reuse window,
    // gets the same rotated cookie rather than a second one.
    let (status, headers, body) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            Some(serde_json::json!({})),
            None,
            &[origin(), cookie(&first)],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "the racing tab: {body}");
    assert_eq!(
        pair(&set_cookie(&headers)),
        second,
        "one cookie for both tabs"
    );

    // Past the window the spent cookie is refused, and the reply expires it.
    sqlx::query(
        "UPDATE refresh_tokens SET replaced_at = replaced_at - INTERVAL '1 minute' WHERE replaced_at IS NOT NULL",
    )
    .execute(&app.pool)
    .await
    .unwrap();
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

#[sqlx::test]
async fn logout_ends_the_browsers_session_and_keeps_the_app_signed_in(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, _, pair) = cookie_login(&app, "twoplaces").await;
    let (status, phone) = app
        .post(
            "/api/auth/login",
            serde_json::json!({ "identifier": "twoplaces", "password": "TestPass123!" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "app login: {phone}");

    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_LOGOUT_ROUTE,
            None,
            Some(&token),
            &[origin(), cookie(&pair)],
        )
        .await;
    assert!(status.is_success(), "logout: {status}");

    let refresh = serde_json::json!({
        "user_id": phone["user"]["id"],
        "refresh_token": phone["refresh_token"]["value"],
    });
    let (status, body) = app.post(REFRESH_SESSION_ROUTE, refresh, None).await;
    assert_eq!(status, StatusCode::OK, "the app's session survives: {body}");
}

#[sqlx::test]
async fn logout_with_the_cookie_alone_ends_its_session(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, _, pair) = cookie_login(&app, "nobearer").await;

    // No access token: it expired, or the page could not refresh it first.
    let (status, headers, _) = app
        .send_raw(
            Method::POST,
            COOKIE_LOGOUT_ROUTE,
            None,
            None,
            &[origin(), cookie(&pair)],
        )
        .await;
    assert!(status.is_success(), "logout by the cookie: {status}");
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
        "the cookie no longer refreshes for the next person on this browser"
    );
}

#[sqlx::test]
async fn a_forged_cookie_logs_nobody_out(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, _, pair) = cookie_login(&app, "victim").await;
    let user_id = pair
        .split_once('=')
        .and_then(|(_, value)| value.split_once('.'))
        .map(|(id, _)| id.to_string())
        .unwrap();

    // The victim's id with a token that is not theirs.
    let forged = format!("{COOKIE}={user_id}.deadbeef");
    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_LOGOUT_ROUTE,
            None,
            None,
            &[origin(), cookie(&forged)],
        )
        .await;
    assert!(status.is_success(), "nothing to end: {status}");

    let (status, _, body) = app
        .send_raw(
            Method::POST,
            COOKIE_REFRESH_ROUTE,
            Some(serde_json::json!({})),
            None,
            &[origin(), cookie(&pair)],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "the victim stays signed in: {body}");
}

#[sqlx::test]
async fn logout_by_the_cookie_still_needs_an_allowed_origin(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, _, pair) = cookie_login(&app, "elsewhere").await;
    let (status, _, _) = app
        .send_raw(
            Method::POST,
            COOKIE_LOGOUT_ROUTE,
            None,
            None,
            &[
                (header::ORIGIN, "https://evil.example".to_string()),
                cookie(&pair),
            ],
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
