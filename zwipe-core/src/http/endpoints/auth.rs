//! Login, registration, session refresh and account recovery.

use std::borrow::Cow;

use crate::domain::auth::models::session::Session;
use crate::http::contracts::auth::{
    HttpAuthenticateUser, HttpCheckResetToken, HttpConfirmEmailChange, HttpCookieRefresh,
    HttpRefreshSession, HttpRegisterUser, HttpRegistrationAccepted, HttpRequestPasswordReset,
    HttpResetPassword, HttpVerifyEmail,
};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::{
    CHECK_RESET_TOKEN_ROUTE, CONFIRM_EMAIL_CHANGE_ROUTE, COOKIE_LOGIN_ROUTE, COOKIE_LOGOUT_ROUTE,
    COOKIE_REFRESH_ROUTE, FORGOT_PASSWORD_ROUTE, LOGIN_ROUTE, LOGOUT_ROUTE, REFRESH_SESSION_ROUTE,
    REGISTER_ROUTE, RESEND_VERIFICATION_ROUTE, RESET_PASSWORD_ROUTE, VERIFY_EMAIL_ROUTE,
};

/// Exchange credentials for a session.
#[derive(Debug)]
pub struct Login(pub HttpAuthenticateUser);
impl Endpoint for Login {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpAuthenticateUser;
    type Response = Session;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(LOGIN_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Create an account without a session; the caller logs in afterwards.
///
/// Answers 202 and the same body whether or not the email already has an
/// account.
#[derive(Debug)]
pub struct Register(pub HttpRegisterUser);
impl Endpoint for Register {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpRegisterUser;
    type Response = HttpRegistrationAccepted;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(REGISTER_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Rotate a refresh token into a fresh session.
#[derive(Debug)]
pub struct Refresh(pub HttpRefreshSession);
impl Endpoint for Refresh {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpRefreshSession;
    type Response = Session;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(REFRESH_SESSION_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Revoke the current session.
#[derive(Debug)]
pub struct Logout;
impl Endpoint for Logout {
    const METHOD: Method = Method::Post;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(LOGOUT_ROUTE)
    }
}

/// The browser's login. The reply's refresh token is blank; the real one is
/// set as an `HttpOnly` cookie the page cannot read.
#[derive(Debug)]
pub struct CookieLogin(pub HttpAuthenticateUser);
impl Endpoint for CookieLogin {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    const CREDENTIALS: bool = true;
    type Request = HttpAuthenticateUser;
    type Response = Session;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COOKIE_LOGIN_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// The browser's refresh, carried by the `HttpOnly` cookie. The reply's
/// refresh token is blank, as on [`CookieLogin`].
#[derive(Debug)]
pub struct CookieRefresh(pub HttpCookieRefresh);
impl Endpoint for CookieRefresh {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    const CREDENTIALS: bool = true;
    type Request = HttpCookieRefresh;
    type Response = Session;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COOKIE_REFRESH_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// The browser's logout: ends the cookie's session and expires the cookie.
#[derive(Debug)]
pub struct CookieLogout;
impl Endpoint for CookieLogout {
    const METHOD: Method = Method::Post;
    const CREDENTIALS: bool = true;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COOKIE_LOGOUT_ROUTE)
    }
}

/// Send another verification email to the signed-in user.
#[derive(Debug)]
pub struct ResendVerification;
impl Endpoint for ResendVerification {
    const METHOD: Method = Method::Post;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(RESEND_VERIFICATION_ROUTE)
    }
}

/// Start a password reset by email.
#[derive(Debug)]
pub struct RequestPasswordReset(pub HttpRequestPasswordReset);
impl Endpoint for RequestPasswordReset {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpRequestPasswordReset;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(FORGOT_PASSWORD_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Confirm an email address with the token from the verification link.
///
/// Unauthenticated: the link is opened in a browser that has no session.
#[derive(Debug)]
pub struct VerifyEmail(pub HttpVerifyEmail);
impl Endpoint for VerifyEmail {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpVerifyEmail;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(VERIFY_EMAIL_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Switch to a pending email address with the token from the link sent to it.
///
/// Unauthenticated, for the same reason as [`VerifyEmail`].
#[derive(Debug)]
pub struct ConfirmEmailChange(pub HttpConfirmEmailChange);
impl Endpoint for ConfirmEmailChange {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpConfirmEmailChange;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CONFIRM_EMAIL_CHANGE_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Finish a password reset with the token from the reset link.
///
/// Unauthenticated, for the same reason as [`VerifyEmail`].
#[derive(Debug)]
pub struct ResetPassword(pub HttpResetPassword);
impl Endpoint for ResetPassword {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpResetPassword;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(RESET_PASSWORD_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Check that a reset link's token is still live, without spending it, so the
/// page can say a dead link is dead before a password is chosen.
///
/// Unauthenticated, for the same reason as [`VerifyEmail`]. A dead token
/// answers 400.
#[derive(Debug)]
pub struct CheckResetToken(pub HttpCheckResetToken);
impl Endpoint for CheckResetToken {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpCheckResetToken;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CHECK_RESET_TOKEN_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}
