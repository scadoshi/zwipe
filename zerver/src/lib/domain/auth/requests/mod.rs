//! Authentication request types.
//!
//! This module contains all request/response types for authentication operations.
//! Entities and value objects live in the sibling [`super::models`] module.
//!
//! # Module Organization
//!
//! - [`authenticate_user`]: User login requests and responses
//! - [`change_email`]: Email change requests with password verification
//! - [`change_password`]: Password change requests with current password verification
//! - [`change_username`]: Username change requests with password verification
//! - [`create_session`]: Create new session for a user
//! - [`delete_expired_sessions`]: Cleanup expired sessions
//! - [`delete_user`]: Account deletion requests with password verification
//! - [`enforce_session_maximum`]: Enforce max session limit per user
//! - [`refresh_session`]: Exchange refresh token for new access token
//! - [`register_user`]: New user registration requests
//! - [`request_password_reset`]: Password reset initiation
//! - [`reset_password`]: Password reset completion
//! - [`revoke_sessions`]: Delete all user sessions (logout)
//! - [`verify_email`]: Email verification

pub mod authenticate_user;
pub mod change_email;
pub mod change_password;
pub mod change_username;
pub mod create_session;
pub mod delete_expired_sessions;
pub mod delete_user;
pub mod enforce_session_maximum;
pub mod refresh_session;
pub mod register_user;
pub mod request_password_reset;
pub mod reset_password;
pub mod revoke_sessions;
pub mod verify_email;
