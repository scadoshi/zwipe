//! Authentication and authorization domain logic.
//!
//! This module provides the core business logic for user authentication, session management,
//! and account credential changes. It implements secure authentication workflows including
//! password hashing with Argon2, JWT-based access tokens, and refresh token rotation.
//!
//! # Architecture
//!
//! - **models/**: Request/response types and domain entities for auth operations
//! - **ports/**: Repository and service trait interfaces for dependency injection
//! - **services/**: Business logic implementations orchestrating auth workflows

/// Authentication entities and value objects.
pub mod models;
pub use models::{UserWithPasswordHash, access_token, password};

/// Authentication request types (login, registration, password changes, etc.).
pub mod requests;
pub use requests::{
    authenticate_user, change_email, change_password, change_username, confirm_email_change,
    create_session, delete_user, enforce_session_maximum, refresh_session, register_user,
    request_password_reset, reset_password, revoke_sessions, verify_email,
};

/// Failed-login counters keyed by account and client address.
pub mod lockout;
/// One "you already have an account" email per address per hour.
pub mod notice_cooldown;

/// Port traits (interfaces) for authentication operations.
pub mod ports;

/// Service implementations for authentication business logic.
pub mod services;
