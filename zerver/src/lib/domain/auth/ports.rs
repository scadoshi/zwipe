//! Port traits for authentication operations.
//!
//! This module defines the interfaces (ports) for authentication in hexagonal architecture.
//! These traits decouple the domain logic from infrastructure concerns (database, HTTP, etc.).

use std::future::Future;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::domain::auth::models::refresh_token::RefreshToken;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::username::Username;

use crate::domain::BoxFuture;
use crate::domain::auth::models::UserWithPasswordHash;
use crate::domain::auth::models::access_token::JwtSecret;
use crate::domain::auth::requests::authenticate_user::{AuthenticateUser, AuthenticateUserError};
use crate::domain::auth::requests::change_email::{ChangeEmail, ChangeEmailError};
use crate::domain::auth::requests::change_password::{ChangePassword, ChangePasswordError};
use crate::domain::auth::requests::change_username::{ChangeUsername, ChangeUsernameError};
use crate::domain::auth::requests::confirm_email_change::{
    ConfirmEmailChange, ConfirmEmailChangeError, ConfirmedEmailChange,
};
use crate::domain::auth::requests::create_session::{CreateSession, CreateSessionError};
use crate::domain::auth::requests::delete_user::{DeleteUser, DeleteUserError};
use crate::domain::auth::requests::refresh_session::{RefreshSession, RefreshSessionError};
use crate::domain::auth::requests::register_user::{
    RegisterOutcome, RegisterUser, RegisterUserError,
};
use crate::domain::auth::requests::request_password_reset::{
    RequestPasswordReset, RequestPasswordResetError,
};
use crate::domain::auth::requests::reset_password::{ResetPassword, ResetPasswordError};
use crate::domain::auth::requests::revoke_sessions::{RevokeSessions, RevokeSessionsError};
use crate::domain::auth::requests::verify_email::{VerifyEmail, VerifyEmailError};

/// Database port for authentication operations.
///
/// Defines all database operations needed for user authentication, session management,
/// and account modifications. Implemented by PostgreSQL adapter in `outbound/sqlx/auth`.
///
/// # Hexagonal Architecture
///
/// This trait is a "port" - it defines WHAT operations are needed without specifying HOW
/// they're implemented. The actual database logic is in the "adapter" layer.
pub trait AuthRepository: Clone + Send + Sync + 'static {
    // == create ==

    /// Creates a new user, unverified and without a session.
    fn create_user(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<User, RegisterUserError>> + Send;

    /// Whether any account has this username.
    fn username_exists(
        &self,
        username: &Username,
    ) -> impl Future<Output = Result<bool, RegisterUserError>> + Send;

    /// Creates a new refresh token for an existing user (new session).
    ///
    /// Generates and stores a new refresh token for the specified user ID,
    /// tagged with the client `platform` and `client_version` (`None` if
    /// unknown/not reported).
    fn create_refresh_token(
        &self,
        request: Uuid,
        platform: Option<ClientPlatform>,
        client_version: Option<String>,
    ) -> impl Future<Output = Result<RefreshToken, CreateSessionError>> + Send;

    /// Exchanges a refresh token for a new one (token rotation).
    ///
    /// Validates the old refresh token, deletes it, and generates a new one.
    /// This implements refresh token rotation to prevent replay attacks.
    fn use_refresh_token(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<RefreshToken, RefreshSessionError>> + Send;

    // == get ==

    /// Retrieves user data with password hash for authentication.
    ///
    /// Looks up user by email, username, or ID and returns full user data
    /// including the Argon2id password hash for verification.
    fn fetch_user_with_password_hash(
        &self,
        request: &AuthenticateUser,
    ) -> impl Future<Output = Result<UserWithPasswordHash, AuthenticateUserError>> + Send;

    // == update ==

    /// Updates a user's password after verification and revokes all active sessions.
    ///
    /// Verifies current password, then updates to new password hash and revokes all sessions.
    fn change_password_and_revoke_sessions(
        &self,
        request: &ChangePassword,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;

    /// Updates a user's username after verification.
    ///
    /// Verifies password, checks uniqueness, then updates username.
    fn change_username(
        &self,
        request: &ChangeUsername,
    ) -> impl Future<Output = Result<User, ChangeUsernameError>> + Send;

    /// Stores the user's pending email change, replacing any earlier one.
    fn store_email_change_request(
        &self,
        user_id: Uuid,
        new_email: &Email,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), ChangeEmailError>> + Send;

    /// Consumes an unexpired pending change by token hash and switches the
    /// user to its address, marked verified, in one transaction.
    ///
    /// Returns [`ConfirmEmailChangeError::AddressTaken`] when another account
    /// holds the address by now; the request is consumed either way.
    fn use_email_change_request(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<ConfirmedEmailChange, ConfirmEmailChangeError>> + Send;

    // == delete ==

    /// Deletes a user account after password verification.
    ///
    /// Verifies password, then deletes user (cascades to sessions, decks, etc.).
    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;

    /// Deletes all refresh tokens for a specific user (logout all devices).
    ///
    /// Removes all sessions for the user, logging them out everywhere.
    fn delete_users_refresh_tokens(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;

    // == email verification ==

    /// Stores a new email verification token for the given user.
    fn store_email_verification_token(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), RegisterUserError>> + Send;

    /// Validates expiry, deletes the token, and returns the owning `user_id`.
    ///
    /// Returns [`VerifyEmailError::InvalidToken`] if the token is not found or expired.
    fn use_email_verification_token(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Uuid, VerifyEmailError>> + Send;

    /// Sets `email_verified_at = NOW()` for the given user.
    fn mark_email_verified(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), VerifyEmailError>> + Send;

    /// Deletes all pending verification tokens for a user (called before issuing a new one).
    fn delete_email_verification_tokens(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    // == password reset ==

    /// Looks up a user ID by email. Returns `None` if the email is not registered.
    ///
    /// Never exposes `UserNotFound` in error; only DB failures are returned.
    fn fetch_user_id_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<Uuid>, anyhow::Error>> + Send;

    /// Returns `true` if a password reset token was issued for this user in the last 5 minutes.
    fn is_password_reset_on_cooldown(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<bool, anyhow::Error>> + Send;

    /// Deletes all pending password reset tokens for a user.
    fn delete_password_reset_tokens(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Stores a new password reset token for the given user.
    fn store_password_reset_token(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Validates expiry, deletes the token, and returns the owning `user_id`.
    ///
    /// Returns [`ResetPasswordError::InvalidToken`] if the token is not found or expired.
    fn use_password_reset_token(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Uuid, ResetPasswordError>> + Send;

    /// Atomically updates the password hash and revokes all sessions for the user.
    ///
    /// Forces re-login on all devices after a password reset.
    fn reset_password_and_revoke_sessions(
        &self,
        user_id: Uuid,
        new_hash: crate::domain::auth::models::password::HashedPassword,
    ) -> impl Future<Output = Result<(), ResetPasswordError>> + Send;
}

/// Service port for authentication business logic.
///
/// Orchestrates authentication operations by combining repository calls with
/// domain logic (password verification, JWT generation, session management).
///
/// # Responsibilities
///
/// - Password verification (Argon2id)
/// - JWT access token generation
/// - Session creation and refresh
/// - Account management (password/username/email changes)
/// - Session cleanup
///
/// # Implementation
///
/// Implemented in `domain/auth/services` with business logic and calls to `AuthRepository`.
pub trait AuthService: Clone + Send + Sync + 'static {
    // == config ==

    /// Returns the JWT secret for signing access tokens.
    fn jwt_secret(&self) -> &JwtSecret;

    // == create ==

    /// Registers a new user without a session.
    ///
    /// A taken username is an error; a taken email is
    /// [`RegisterOutcome::EmailTaken`], so the handler can answer it like a
    /// new one. Emails go out in the background either way.
    fn register_user(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<RegisterOutcome, RegisterUserError>> + Send;

    /// Creates a new session for an authenticated user.
    ///
    /// Enforces session maximum, generates new refresh token and access token.
    fn create_session(
        &self,
        request: &CreateSession,
    ) -> impl Future<Output = Result<Session, CreateSessionError>> + Send;

    /// Refreshes a session using a refresh token (token rotation).
    ///
    /// Validates refresh token, generates new access token and new refresh token.
    fn refresh_session(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<Session, RefreshSessionError>> + Send;

    /// Authenticates a user by email/username and password.
    ///
    /// Verifies credentials, enforces session maximum, creates new session.
    fn authenticate_user(
        &self,
        request: &AuthenticateUser,
    ) -> impl Future<Output = Result<Session, AuthenticateUserError>> + Send;

    // == update ==

    /// Changes a user's password after verifying current password and revokes all active sessions.
    ///
    /// Verifies current password, updates to new password hash and revokes all sessions.
    fn change_password_and_revoke_sessions(
        &self,
        request: &ChangePassword,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;

    /// Changes a user's username after verifying password.
    ///
    /// Verifies password, checks uniqueness, updates username.
    fn change_username(
        &self,
        request: &ChangeUsername,
    ) -> impl Future<Output = Result<User, ChangeUsernameError>> + Send;

    /// Holds a new email as pending after verifying the password, and mails a
    /// confirmation link to it when no other account has it.
    ///
    /// Answers the same way whether or not the address is taken: the user as
    /// they are, with `pending_email` set. The email goes out in the
    /// background so the response time doesn't say either.
    fn request_email_change(
        &self,
        request: &ChangeEmail,
    ) -> impl Future<Output = Result<User, ChangeEmailError>> + Send;

    /// Switches a user to their pending email from the confirmation link and
    /// tells the old address.
    fn confirm_email_change(
        &self,
        request: &ConfirmEmailChange,
    ) -> impl Future<Output = Result<User, ConfirmEmailChangeError>> + Send;

    // == delete ==

    /// Deletes a user account after password verification.
    ///
    /// Verifies password, deletes user and all associated data.
    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;

    /// Revokes all sessions for a user (logout everywhere).
    ///
    /// Deletes all refresh tokens for the user, logging them out from all devices.
    fn revoke_sessions(
        &self,
        request: &RevokeSessions,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;

    // == email verification ==

    /// Generates and stores a verification token, then sends the verification email.
    ///
    /// Called on registration and on `resend-verification`. Fire-and-forget on registration
    /// (errors are logged but do not fail the registration).
    fn send_verification_email(
        &self,
        user_id: Uuid,
        to_email: &str,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Marks the email address as verified using the one-time token.
    fn verify_email(
        &self,
        request: &VerifyEmail,
    ) -> impl Future<Output = Result<(), VerifyEmailError>> + Send;

    // == password reset ==

    /// Initiates a password reset flow for the given email.
    ///
    /// Always returns `Ok(())`; user-not-found and cooldown are silently swallowed
    /// to prevent email enumeration.
    fn request_password_reset(
        &self,
        request: &RequestPasswordReset,
    ) -> impl Future<Output = Result<(), RequestPasswordResetError>> + Send;

    /// Completes a password reset using the one-time token and updates the password.
    ///
    /// Revokes all existing sessions after a successful reset. Returns the id of
    /// the user whose password was reset so callers can emit audit entries.
    fn reset_password(
        &self,
        request: &ResetPassword,
    ) -> impl Future<Output = Result<uuid::Uuid, ResetPasswordError>> + Send;
}

/// Object-safe wrapper used by `AppState` so the concrete service type stays
/// out of the generic parameter list. Auto-implemented for any `AuthService`.
pub trait ErasedAuthService: Send + Sync + 'static {
    /// See [`AuthService::jwt_secret`].
    fn jwt_secret(&self) -> &JwtSecret;

    /// See [`AuthService::register_user`].
    fn register_user<'a>(
        &'a self,
        request: &'a RegisterUser,
    ) -> BoxFuture<'a, Result<RegisterOutcome, RegisterUserError>>;

    /// See [`AuthService::create_session`].
    fn create_session<'a>(
        &'a self,
        request: &'a CreateSession,
    ) -> BoxFuture<'a, Result<Session, CreateSessionError>>;

    /// See [`AuthService::refresh_session`].
    fn refresh_session<'a>(
        &'a self,
        request: &'a RefreshSession,
    ) -> BoxFuture<'a, Result<Session, RefreshSessionError>>;

    /// See [`AuthService::authenticate_user`].
    fn authenticate_user<'a>(
        &'a self,
        request: &'a AuthenticateUser,
    ) -> BoxFuture<'a, Result<Session, AuthenticateUserError>>;

    /// See [`AuthService::change_password_and_revoke_sessions`].
    fn change_password_and_revoke_sessions<'a>(
        &'a self,
        request: &'a ChangePassword,
    ) -> BoxFuture<'a, Result<(), ChangePasswordError>>;

    /// See [`AuthService::change_username`].
    fn change_username<'a>(
        &'a self,
        request: &'a ChangeUsername,
    ) -> BoxFuture<'a, Result<User, ChangeUsernameError>>;

    /// See [`AuthService::request_email_change`].
    fn request_email_change<'a>(
        &'a self,
        request: &'a ChangeEmail,
    ) -> BoxFuture<'a, Result<User, ChangeEmailError>>;

    /// See [`AuthService::confirm_email_change`].
    fn confirm_email_change<'a>(
        &'a self,
        request: &'a ConfirmEmailChange,
    ) -> BoxFuture<'a, Result<User, ConfirmEmailChangeError>>;

    /// See [`AuthService::delete_user`].
    fn delete_user<'a>(
        &'a self,
        request: &'a DeleteUser,
    ) -> BoxFuture<'a, Result<(), DeleteUserError>>;

    /// See [`AuthService::revoke_sessions`].
    fn revoke_sessions<'a>(
        &'a self,
        request: &'a RevokeSessions,
    ) -> BoxFuture<'a, Result<(), RevokeSessionsError>>;

    /// See [`AuthService::send_verification_email`].
    fn send_verification_email<'a>(
        &'a self,
        user_id: Uuid,
        to_email: &'a str,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>>;

    /// See [`AuthService::verify_email`].
    fn verify_email<'a>(
        &'a self,
        request: &'a VerifyEmail,
    ) -> BoxFuture<'a, Result<(), VerifyEmailError>>;

    /// See [`AuthService::request_password_reset`].
    fn request_password_reset<'a>(
        &'a self,
        request: &'a RequestPasswordReset,
    ) -> BoxFuture<'a, Result<(), RequestPasswordResetError>>;

    /// See [`AuthService::reset_password`].
    fn reset_password<'a>(
        &'a self,
        request: &'a ResetPassword,
    ) -> BoxFuture<'a, Result<uuid::Uuid, ResetPasswordError>>;
}

impl<T> ErasedAuthService for T
where
    T: AuthService,
{
    fn jwt_secret(&self) -> &JwtSecret {
        AuthService::jwt_secret(self)
    }

    fn register_user<'a>(
        &'a self,
        request: &'a RegisterUser,
    ) -> BoxFuture<'a, Result<RegisterOutcome, RegisterUserError>> {
        Box::pin(AuthService::register_user(self, request))
    }

    fn create_session<'a>(
        &'a self,
        request: &'a CreateSession,
    ) -> BoxFuture<'a, Result<Session, CreateSessionError>> {
        Box::pin(AuthService::create_session(self, request))
    }

    fn refresh_session<'a>(
        &'a self,
        request: &'a RefreshSession,
    ) -> BoxFuture<'a, Result<Session, RefreshSessionError>> {
        Box::pin(AuthService::refresh_session(self, request))
    }

    fn authenticate_user<'a>(
        &'a self,
        request: &'a AuthenticateUser,
    ) -> BoxFuture<'a, Result<Session, AuthenticateUserError>> {
        Box::pin(AuthService::authenticate_user(self, request))
    }

    fn change_password_and_revoke_sessions<'a>(
        &'a self,
        request: &'a ChangePassword,
    ) -> BoxFuture<'a, Result<(), ChangePasswordError>> {
        Box::pin(AuthService::change_password_and_revoke_sessions(
            self, request,
        ))
    }

    fn change_username<'a>(
        &'a self,
        request: &'a ChangeUsername,
    ) -> BoxFuture<'a, Result<User, ChangeUsernameError>> {
        Box::pin(AuthService::change_username(self, request))
    }

    fn request_email_change<'a>(
        &'a self,
        request: &'a ChangeEmail,
    ) -> BoxFuture<'a, Result<User, ChangeEmailError>> {
        Box::pin(AuthService::request_email_change(self, request))
    }

    fn confirm_email_change<'a>(
        &'a self,
        request: &'a ConfirmEmailChange,
    ) -> BoxFuture<'a, Result<User, ConfirmEmailChangeError>> {
        Box::pin(AuthService::confirm_email_change(self, request))
    }

    fn delete_user<'a>(
        &'a self,
        request: &'a DeleteUser,
    ) -> BoxFuture<'a, Result<(), DeleteUserError>> {
        Box::pin(AuthService::delete_user(self, request))
    }

    fn revoke_sessions<'a>(
        &'a self,
        request: &'a RevokeSessions,
    ) -> BoxFuture<'a, Result<(), RevokeSessionsError>> {
        Box::pin(AuthService::revoke_sessions(self, request))
    }

    fn send_verification_email<'a>(
        &'a self,
        user_id: Uuid,
        to_email: &'a str,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(AuthService::send_verification_email(
            self, user_id, to_email,
        ))
    }

    fn verify_email<'a>(
        &'a self,
        request: &'a VerifyEmail,
    ) -> BoxFuture<'a, Result<(), VerifyEmailError>> {
        Box::pin(AuthService::verify_email(self, request))
    }

    fn request_password_reset<'a>(
        &'a self,
        request: &'a RequestPasswordReset,
    ) -> BoxFuture<'a, Result<(), RequestPasswordResetError>> {
        Box::pin(AuthService::request_password_reset(self, request))
    }

    fn reset_password<'a>(
        &'a self,
        request: &'a ResetPassword,
    ) -> BoxFuture<'a, Result<uuid::Uuid, ResetPasswordError>> {
        Box::pin(AuthService::reset_password(self, request))
    }
}
