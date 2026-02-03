//! Port traits for authentication operations.
//!
//! This module defines the interfaces (ports) for authentication in hexagonal architecture.
//! These traits decouple the domain logic from infrastructure concerns (database, HTTP, etc.).
//!
//! # Hexagonal Architecture
//!
//! - **AuthRepository**: Database port (data persistence operations)
//! - **AuthService**: Service port (orchestrates business logic + repository calls)
//!
//! # Implementation
//!
//! - Repositories are implemented in `outbound/sqlx/auth` (PostgreSQL)
//! - Services are implemented in `domain/auth/services` (business logic)

use crate::domain::{
    auth::models::{
        access_token::JwtSecret,
        authenticate_user::{AuthenticateUser, AuthenticateUserError},
        change_email::{ChangeEmail, ChangeEmailError},
        change_password::{ChangePassword, ChangePasswordError},
        change_username::{ChangeUsername, ChangeUsernameError},
        delete_user::{DeleteUser, DeleteUserError},
        refresh_token::RefreshToken,
        register_user::{RegisterUser, RegisterUserError},
        session::{
            create_session::{CreateSession, CreateSessionError},
            delete_expired_sessions::DeleteExpiredSessionsError,
            refresh_session::{RefreshSession, RefreshSessionError},
            revoke_sessions::{RevokeSessions, RevokeSessionsError},
            Session,
        },
        UserWithPasswordHash,
    },
    user::models::User,
};
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
    //  update
    // ========

    /// Updates a user's password after verification.
    ///
    /// Verifies current password, then updates to new password hash.
    fn change_password(
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

    /// Changes a user's password after verifying current password.
    ///
    /// Verifies current password, updates to new password hash.
    fn change_password(
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
}
