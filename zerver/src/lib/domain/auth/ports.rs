//! Port traits for authentication operations.
//!
//! This module defines the interfaces (ports) for authentication in hexagonal architecture.
//! These traits decouple the domain logic from infrastructure concerns (database, HTTP, etc.).

use std::future::Future;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::username::Username;

use crate::domain::auth::models::UserWithPasswordHash;
use crate::domain::auth::models::access_token::JwtSecret;
use crate::domain::auth::requests::authenticate_user::{AuthenticateUser, AuthenticateUserError};
use crate::domain::auth::requests::cancel_email_change::CancelEmailChangeError;
use crate::domain::auth::requests::change_email::{ChangeEmail, ChangeEmailError};
use crate::domain::auth::requests::change_password::{ChangePassword, ChangePasswordError};
use crate::domain::auth::requests::change_username::{ChangeUsername, ChangeUsernameError};
use crate::domain::auth::requests::check_reset_token::CheckResetTokenError;
use crate::domain::auth::requests::confirm_email_change::{
    ConfirmEmailChange, ConfirmEmailChangeError, ConfirmedEmailChange,
};
use crate::domain::auth::requests::create_session::{
    CreateSession, CreateSessionError, IssuedRefreshToken,
};
use crate::domain::auth::requests::delete_user::{DeleteUser, DeleteUserError};
use crate::domain::auth::requests::refresh_session::{
    RefreshSession, RefreshSessionError, RotatedRefreshToken,
};
use crate::domain::auth::requests::register_user::{
    RegisterOutcome, RegisterUser, RegisterUserError,
};
use crate::domain::auth::requests::request_password_reset::{
    RequestPasswordReset, RequestPasswordResetError,
};
use crate::domain::auth::requests::resend_email_change::ResendEmailChangeError;
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
    /// Generates and stores a new refresh token for the specified user ID in
    /// a new family, tagged with the client `platform` and `client_version`
    /// (`None` if unknown/not reported).
    fn create_refresh_token(
        &self,
        request: Uuid,
        platform: Option<ClientPlatform>,
        client_version: Option<String>,
    ) -> impl Future<Output = Result<IssuedRefreshToken, CreateSessionError>> + Send;

    /// Exchanges a refresh token for a new one (token rotation).
    ///
    /// Validates the old refresh token, deletes it, and generates a new one.
    /// This implements refresh token rotation to prevent replay attacks. The
    /// new token keeps the family's platform, which comes back with it.
    fn use_refresh_token(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<RotatedRefreshToken, RefreshSessionError>> + Send;

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

    /// Deletes `user_id`'s pending email change, expired or not.
    ///
    /// Fails with [`CancelEmailChangeError::NoPendingChange`] when no
    /// unexpired change was waiting.
    fn delete_email_change_request(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CancelEmailChangeError>> + Send;

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

    /// Deletes every refresh token of `user_id` in `family_id` (logout of
    /// one session). A family the user does not hold deletes nothing.
    fn delete_refresh_token_family(
        &self,
        user_id: Uuid,
        family_id: Uuid,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;

    /// The family of the refresh token in `request`, live or rotated, when
    /// it is `request.user_id`'s; `None` when no such token is stored.
    fn fetch_refresh_family(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<Option<Uuid>, RevokeSessionsError>> + Send;

    // == email verification ==

    /// Stores a new email verification token for the given user.
    fn store_email_verification_token(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), RegisterUserError>> + Send;

    /// Whether a password reset token with this hash exists and has not
    /// expired. Reads only: the token stays as it is.
    fn password_reset_token_is_live(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<bool, anyhow::Error>> + Send;

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

    /// Replaces `user`'s pending email change with a fresh link and expiry
    /// for the same address, and mails the link when no other account has
    /// the address, answering the same either way, as
    /// [`Self::request_email_change`] does.
    ///
    /// Fails with [`ResendEmailChangeError::NoPendingChange`] when `user`
    /// has no unexpired change waiting.
    fn resend_email_change(
        &self,
        user: &User,
    ) -> impl Future<Output = Result<(), ResendEmailChangeError>> + Send;

    /// Drops `user_id`'s pending email change, so its link no longer
    /// confirms and the account keeps its address.
    ///
    /// Fails with [`CancelEmailChangeError::NoPendingChange`] when no
    /// unexpired change is waiting.
    fn cancel_email_change(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CancelEmailChangeError>> + Send;

    // == delete ==

    /// Deletes a user account after password verification.
    ///
    /// Verifies password, deletes user and all associated data.
    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;

    /// Revokes the sessions `request.scope` names: one refresh token family
    /// (a logout) or every session the user holds.
    fn revoke_sessions(
        &self,
        request: &RevokeSessions,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;

    /// The refresh token family the token in `request` belongs to, so a
    /// browser's logout can name its session by its cookie. `None` when the
    /// token is not stored for that user.
    fn fetch_refresh_family(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<Option<Uuid>, RevokeSessionsError>> + Send;

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

    /// Checks that a reset token is live without spending or extending it.
    ///
    /// Returns [`CheckResetTokenError::InvalidToken`] when it is unknown, used
    /// or expired.
    fn check_reset_token(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<(), CheckResetTokenError>> + Send;

    /// Completes a password reset using the one-time token and updates the password.
    ///
    /// Revokes all existing sessions after a successful reset. Returns the id of
    /// the user whose password was reset so callers can emit audit entries.
    fn reset_password(
        &self,
        request: &ResetPassword,
    ) -> impl Future<Output = Result<uuid::Uuid, ResetPasswordError>> + Send;
}
