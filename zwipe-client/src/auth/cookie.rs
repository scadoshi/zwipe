//! The browser's session calls. The server keeps the refresh token in an
//! `HttpOnly` cookie the page cannot read, so every [`Session`](zwipe_core::domain::auth::models::session::Session) these return
//! has a blank refresh token; the browser sends the cookie on its own.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::{HttpAuthenticateUser, HttpCookieRefresh};
use zwipe_core::http::endpoints::auth::{
    CookieLogin, CookieLogout, CookieLogoutAlone, CookieRefresh,
};

use super::login::stamped;
use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Logs in from a browser. The refresh token is set as a cookie and is
    /// blank in the returned session.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn login_cookie(
        &self,
        request: HttpAuthenticateUser,
    ) -> Result<Session, ClientError> {
        self.call(CookieLogin(stamped(request)), None).await
    }

    /// Rotates the browser's session through its cookie.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status (401 with no or a spent cookie), or the reply doesn't decode.
    pub async fn refresh_cookie(&self) -> Result<Session, ClientError> {
        let request = HttpCookieRefresh {
            client_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };
        self.call(CookieRefresh(request), None).await
    }

    /// Logs the browser out: ends the cookie's session and expires the cookie.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn logout_cookie(&self, session: &Session) -> Result<(), ClientError> {
        self.call(CookieLogout, Some(session)).await
    }

    /// Logs the browser out by its cookie alone, for when no access token can be had: ends the
    /// session the cookie belongs to and expires the cookie.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete or the server answers with an
    /// error status.
    pub async fn logout_cookie_alone(&self) -> Result<(), ClientError> {
        self.call(CookieLogoutAlone, None).await
    }
}
