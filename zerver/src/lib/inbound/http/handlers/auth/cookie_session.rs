//! The browser's login, refresh and logout.
//!
//! The app's routes hand the refresh token back in the JSON body, where any
//! script on a web page could read it. These keep it out of the body and in
//! an `HttpOnly` cookie instead, which the browser stores and sends but never
//! shows to the page:
//!
//! - `__Host-` prefix: the browser takes it only over HTTPS, with `Path=/`
//!   and no `Domain`, so no other subdomain can set or read it.
//! - `HttpOnly`: no script reads it.
//! - `Secure`: it never travels over plain HTTP.
//! - `SameSite=Strict`: another site's page cannot make the browser send it.
//!
//! The cookie holds `<user id>.<refresh token>`, and every session these
//! return has a blank refresh token. Refresh and logout also refuse a
//! request whose `Origin` is not one of `ALLOWED_ORIGINS`, a second guard
//! against a forged request behind `SameSite`.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::{Extension, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::{HttpAuthenticateUser, HttpCookieRefresh, HttpSession};
use zwipe_core::version::Version;

use super::authenticate_user::login;
use super::refresh_session::refresh;
use super::revoke_sessions::revoke_sessions;
use crate::domain::auth::requests::refresh_session::RefreshSession;
use crate::inbound::http::middleware::{AuthenticatedUser, ClientIp};
use crate::inbound::http::{ApiError, AppState};

/// The cookie's name. The `__Host-` prefix is a browser rule, not a label:
/// a cookie so named is refused unless it is `Secure`, `Path=/` and has no
/// `Domain`.
pub const REFRESH_COOKIE: &str = "__Host-zwipe_refresh";

/// The origins allowed to drive the cookie routes, the same list CORS
/// allows. Added to the router as an extension.
#[derive(Debug, Clone)]
pub struct CookieOrigins(pub Arc<[HeaderValue]>);

/// Logs in from a browser: the session in the body with a blank refresh
/// token, and the refresh token in the cookie.
///
/// # Errors
///
/// As the app's login: 422 for a missing identifier or password, 401 for bad
/// credentials, 500 on a database failure.
pub async fn cookie_login(
    State(state): State<AppState>,
    ClientIp(client_ip): ClientIp,
    jar: CookieJar,
    Json(body): Json<HttpAuthenticateUser>,
) -> Result<(CookieJar, Json<HttpSession>), ApiError> {
    let session = login(&state, client_ip, &body).await?;
    Ok((jar.add(session_cookie(&session)), Json(redacted(session))))
}

/// Rotates the browser's session through its cookie: the old token is spent,
/// the new one replaces it in the cookie, and the body's is blank.
///
/// # Errors
///
/// 403 for an origin outside `ALLOWED_ORIGINS`; 401 with no cookie, a
/// malformed one, or a spent, expired or revoked token, which also expires
/// the cookie; 500 on a database failure.
pub async fn cookie_refresh(
    State(state): State<AppState>,
    Extension(origins): Extension<CookieOrigins>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(body): Json<HttpCookieRefresh>,
) -> (CookieJar, Result<Json<HttpSession>, ApiError>) {
    if let Err(e) = check_origin(&headers, &origins) {
        return (jar, Err(e));
    }
    let Some((user_id, token)) = jar.get(REFRESH_COOKIE).and_then(|c| parse(c.value())) else {
        return (jar.remove(removal()), Err(invalid()));
    };
    let Ok(mut request) = RefreshSession::new(&user_id, &token) else {
        return (jar.remove(removal()), Err(invalid()));
    };
    request.client_version = body
        .client_version
        .as_deref()
        .and_then(|raw| raw.parse::<Version>().ok())
        .map(|parsed| parsed.to_string());
    match refresh(&state, &request).await {
        Ok(session) => (
            jar.add(session_cookie(&session)),
            Ok(Json(redacted(session))),
        ),
        Err(e @ (ApiError::Unauthorized(_) | ApiError::Forbidden(_))) => {
            (jar.remove(removal()), Err(e))
        }
        Err(e) => (jar, Err(e)),
    }
}

/// Logs the browser out: revokes every session the user holds, as the app's
/// logout does, and expires the cookie.
///
/// # Errors
///
/// 401 without a valid access token, 403 for an origin outside
/// `ALLOWED_ORIGINS`, 500 on a database failure.
pub async fn cookie_logout(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Extension(origins): Extension<CookieOrigins>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    check_origin(&headers, &origins)?;
    let status = revoke_sessions(user, State(state)).await?;
    Ok((jar.remove(removal()), status))
}

/// Refuses a request with no `Origin`, or one outside the allowed list.
fn check_origin(headers: &HeaderMap, origins: &CookieOrigins) -> Result<(), ApiError> {
    match headers.get(header::ORIGIN) {
        Some(origin) if origins.0.iter().any(|allowed| allowed == origin) => Ok(()),
        _ => Err(ApiError::Forbidden("origin not allowed".to_string())),
    }
}

/// The refresh cookie for `session`, living as long as its refresh token.
fn session_cookie(session: &Session) -> Cookie<'static> {
    let value = format!("{}.{}", session.user.id, session.refresh_token.value);
    let seconds = (session
        .refresh_token
        .expires_at
        .signed_duration_since(Utc::now()))
    .num_seconds()
    .max(0);
    let mut cookie = base(value);
    cookie.set_max_age(cookie::time::Duration::seconds(seconds));
    cookie
}

/// The cookie that expires the refresh cookie. A `__Host-` cookie is only
/// replaced by one with the same attributes, so it is built from [`base`].
fn removal() -> Cookie<'static> {
    base(String::new())
}

/// The refresh cookie's fixed attributes.
fn base(value: String) -> Cookie<'static> {
    Cookie::build((REFRESH_COOKIE, value))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .build()
}

/// `<user id>.<token>` split in two. A UUID has no `.` and the token is hex,
/// so the first `.` is the seam.
fn parse(value: &str) -> Option<(String, String)> {
    let (user_id, token) = value.split_once('.')?;
    if user_id.is_empty() || token.is_empty() {
        return None;
    }
    Some((user_id.to_string(), token.to_string()))
}

/// The session as the browser gets it: everything but the refresh token,
/// which stays in the cookie.
fn redacted(mut session: Session) -> HttpSession {
    session.refresh_token.value = String::new();
    HttpSession::from(session)
}

/// One answer for every unusable cookie, as the app's refresh gives one for
/// every unusable token.
fn invalid() -> ApiError {
    ApiError::Unauthorized("invalid refresh token".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cookie_value_splits_at_the_first_dot() {
        let id = "6f1c2b9e-0000-4000-8000-000000000001";
        let parsed = parse(&format!("{id}.abc123"));
        assert_eq!(parsed, Some((id.to_string(), "abc123".to_string())));
        assert_eq!(parse("no-seam"), None);
        assert_eq!(parse(".abc"), None);
        assert_eq!(parse("id."), None);
    }

    #[test]
    fn the_cookie_is_host_only_secure_http_only_and_strict() {
        let cookie = base("v".to_string());
        assert!(REFRESH_COOKIE.starts_with("__Host-"));
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.domain(), None);
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Strict));
    }

    #[test]
    fn an_origin_outside_the_list_is_refused() {
        let origins = CookieOrigins(Arc::from(vec![HeaderValue::from_static(
            "https://app.zwipe.net",
        )]));
        let mut headers = HeaderMap::new();
        assert!(check_origin(&headers, &origins).is_err(), "no origin");
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://evil.example"),
        );
        assert!(check_origin(&headers, &origins).is_err(), "foreign origin");
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://app.zwipe.net"),
        );
        assert!(check_origin(&headers, &origins).is_ok(), "allowed origin");
    }
}
