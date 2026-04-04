//! Port traits for authentication operations.
//!
//! This module defines the interfaces (ports) for authentication in hexagonal architecture.
//! These traits decouple the domain logic from infrastructure concerns (database, HTTP, etc.).

use crate::domain::{
    auth::{
        models::{
            UserWithPasswordHash,
            access_token::JwtSecret,
            refresh_token::RefreshToken,
            session::Session,
        },
        requests::{
            authenticate_user::{AuthenticateUser, AuthenticateUserError},
            change_email::{ChangeEmail, ChangeEmailError},
            change_password::{ChangePassword, ChangePasswordError},
            change_username::{ChangeUsername, ChangeUsernameError},
            create_session::{CreateSession, CreateSessionError},
            delete_expired_sessions::DeleteExpiredSessionsError,
            delete_user::{DeleteUser, DeleteUserError},
            refresh_session::{RefreshSession, RefreshSessionError},
            register_user::{RegisterUser, RegisterUserError},
            request_password_reset::{RequestPasswordReset, RequestPasswordResetError},
            reset_password::{ResetPassword, ResetPasswordError},
            revoke_sessions::{RevokeSessions, RevokeSessionsError},
            verify_email::{VerifyEmail, VerifyEmailError},
        },
    },
};
use zwipe_core::domain::user::User;
use chrono::NaiveDateTime;
use std::future::Future;
use uuid::Uuid;

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
    // ========
    //  create
    // ========

    /// Creates a new user and initial refresh token (registration).
    ///
    /// Inserts user into database and generates first refresh token for immediate login.
    fn create_user_and_refresh_token(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<(User, RefreshToken), RegisterUserError>> + Send;

    /// Creates a new refresh token for an existing user (new session).
    ///
    /// Generates and stores a new refresh token for the specified user ID.
    fn create_refresh_token(
        &self,
        request: Uuid,
    ) -> impl Future<Output = Result<RefreshToken, CreateSessionError>> + Send;

    /// Exchanges a refresh token for a new one (token rotation).
    ///
    /// Validates the old refresh token, deletes it, and generates a new one.
    /// This implements refresh token rotation to prevent replay attacks.
    fn use_refresh_token(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<RefreshToken, RefreshSessionError>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves user data with password hash for authentication.
    ///
    /// Looks up user by email, username, or ID and returns full user data
    /// including the Argon2id password hash for verification.
    fn get_user_with_password_hash(
        &self,
        request: &AuthenticateUser,
    ) -> impl Future<Output = Result<UserWithPasswordHash, AuthenticateUserError>> + Send;

    // ========
    //  lockout
    // ========

    /// Increments the failed login counter for a user.
    ///
    /// Uses a sliding 30-minute window: if the last failure was more than 30 min ago,
    /// the counter resets to 1. After 5 failures within the window, sets `lockout_until`
    /// to `NOW() + 30 minutes`. All changes are a single atomic UPDATE.
    fn increment_failed_attempts(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), AuthenticateUserError>> + Send;

    /// Resets failed login counter and clears lockout on successful authentication.
    fn reset_failed_attempts(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), AuthenticateUserError>> + Send;

    // ========
    //  update
    // ========

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

    /// Updates a user's email address after verification.
    ///
    /// Verifies password, checks uniqueness, then updates email.
    fn change_email(
        &self,
        request: &ChangeEmail,
    ) -> impl Future<Output = Result<User, ChangeEmailError>> + Send;

    // ========
    //  delete
    // ========

    /// Deletes a user account after password verification.
    ///
    /// Verifies password, then deletes user (cascades to sessions, decks, etc.).
    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;

    /// Deletes all expired refresh tokens (cleanup operation).
    ///
    /// Removes refresh tokens past their 14-day expiration.
    fn delete_expired_refresh_tokens(
        &self,
    ) -> impl Future<Output = Result<(), DeleteExpiredSessionsError>> + Send;

    /// Deletes all refresh tokens for a specific user (logout all devices).
    ///
    /// Removes all sessions for the user, logging them out everywhere.
    fn delete_users_refresh_tokens(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;

    // ========================
    //  email verification
    // ========================

    /// Stores a new email verification token for the given user.
    fn store_email_verification_token(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: NaiveDateTime,
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

    // ========================
    //  password reset
    // ========================

    /// Looks up a user ID by email. Returns `None` if the email is not registered.
    ///
    /// Never exposes `UserNotFound` in error — only DB failures are returned.
    fn get_user_id_by_email(
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
        expires_at: NaiveDateTime,
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
    // ========
    //  config
    // ========

    /// Returns the JWT secret for signing access tokens.
    fn jwt_secret(&self) -> &JwtSecret;

    // ========
    //  create
    // ========

    /// Registers a new user and creates their first session.
    ///
    /// Creates user, generates JWT access token and refresh token, returns complete session.
    fn register_user(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<Session, RegisterUserError>> + Send;

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

    // ========
    //  update
    // ========

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

    /// Changes a user's email after verifying password.
    ///
    /// Verifies password, checks uniqueness, updates email.
    fn change_email(
        &self,
        request: &ChangeEmail,
    ) -> impl Future<Output = Result<User, ChangeEmailError>> + Send;

    // ========
    //  delete
    // ========

    /// Deletes a user account after password verification.
    ///
    /// Verifies password, deletes user and all associated data.
    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;

    /// Deletes all expired sessions (cleanup operation).
    ///
    /// Typically called by background job to clean up old refresh tokens.
    fn delete_expired_sessions(
        &self,
    ) -> impl Future<Output = Result<(), DeleteExpiredSessionsError>> + Send;

    /// Revokes all sessions for a user (logout everywhere).
    ///
    /// Deletes all refresh tokens for the user, logging them out from all devices.
    fn revoke_sessions(
        &self,
        request: &RevokeSessions,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;

    // ========================
    //  email verification
    // ========================

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

    // ========================
    //  password reset
    // ========================

    /// Initiates a password reset flow for the given email.
    ///
    /// Always returns `Ok(())` — user-not-found and cooldown are silently swallowed
    /// to prevent email enumeration.
    fn request_password_reset(
        &self,
        request: &RequestPasswordReset,
    ) -> impl Future<Output = Result<(), RequestPasswordResetError>> + Send;

    /// Completes a password reset using the one-time token and updates the password.
    ///
    /// Revokes all existing sessions after a successful reset.
    fn reset_password(
        &self,
        request: &ResetPassword,
    ) -> impl Future<Output = Result<(), ResetPasswordError>> + Send;
}
