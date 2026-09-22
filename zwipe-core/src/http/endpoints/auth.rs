//! Login, registration, session refresh and account recovery.

use crate::{
    domain::auth::models::session::Session,
    http::{
        endpoint::{Endpoint, Method},
        paths::{
            FORGOT_PASSWORD_ROUTE, LOGIN_ROUTE, LOGOUT_ROUTE, REFRESH_SESSION_ROUTE,
            REGISTER_ROUTE, RESEND_VERIFICATION_ROUTE, RESET_PASSWORD_ROUTE, VERIFY_EMAIL_ROUTE,
        },
    },
};
use serde_json::Value;

/// Exchange credentials for a session.
pub struct Login(pub Value);
impl Endpoint for Login {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = Session;
    fn path(&self) -> String {
        LOGIN_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Create an account. Answers 201.
pub struct Register(pub Value);
impl Endpoint for Register {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = Session;
    fn path(&self) -> String {
        REGISTER_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Rotate a refresh token into a fresh session.
pub struct Refresh(pub Value);
impl Endpoint for Refresh {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = Session;
    fn path(&self) -> String {
        REFRESH_SESSION_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Revoke the current session.
pub struct Logout;
impl Endpoint for Logout {
    const METHOD: Method = Method::Post;
    type Response = ();
    fn path(&self) -> String {
        LOGOUT_ROUTE.to_string()
    }
}

/// Send another verification email to the signed-in user.
pub struct ResendVerification;
impl Endpoint for ResendVerification {
    const METHOD: Method = Method::Post;
    type Response = ();
    fn path(&self) -> String {
        RESEND_VERIFICATION_ROUTE.to_string()
    }
}

/// Start a password reset by email.
pub struct RequestPasswordReset(pub Value);
impl Endpoint for RequestPasswordReset {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = ();
    fn path(&self) -> String {
        FORGOT_PASSWORD_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Confirm an email address with the token from the verification link.
///
/// Unauthenticated: the link is opened in a browser that has no session.
pub struct VerifyEmail(pub Value);
impl Endpoint for VerifyEmail {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = ();
    fn path(&self) -> String {
        VERIFY_EMAIL_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Finish a password reset with the token from the reset link.
///
/// Unauthenticated, for the same reason as [`VerifyEmail`].
pub struct ResetPassword(pub Value);
impl Endpoint for ResetPassword {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = ();
    fn path(&self) -> String {
        RESET_PASSWORD_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}
