//! Login, registration, session refresh and account recovery.

use crate::{
    domain::auth::models::session::Session,
    http::{
        contracts::auth::{
            HttpAuthenticateUser, HttpRefreshSession, HttpRegisterUser, HttpRequestPasswordReset,
            HttpResetPassword, HttpVerifyEmail,
        },
        endpoint::{Endpoint, Method},
        paths::{
            FORGOT_PASSWORD_ROUTE, LOGIN_ROUTE, LOGOUT_ROUTE, REFRESH_SESSION_ROUTE,
            REGISTER_ROUTE, RESEND_VERIFICATION_ROUTE, RESET_PASSWORD_ROUTE, VERIFY_EMAIL_ROUTE,
        },
    },
};
use std::borrow::Cow;

/// Exchange credentials for a session.
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

/// Create an account. Answers 201.
pub struct Register(pub HttpRegisterUser);
impl Endpoint for Register {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpRegisterUser;
    type Response = Session;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(REGISTER_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Rotate a refresh token into a fresh session.
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
pub struct Logout;
impl Endpoint for Logout {
    const METHOD: Method = Method::Post;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(LOGOUT_ROUTE)
    }
}

/// Send another verification email to the signed-in user.
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

/// Finish a password reset with the token from the reset link.
///
/// Unauthenticated, for the same reason as [`VerifyEmail`].
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
