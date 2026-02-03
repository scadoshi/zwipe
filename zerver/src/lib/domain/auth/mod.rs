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
//!
//! # Core Concepts
//!
//! ## Authentication Flow
//!
//! 1. User registers or authenticates with email/username and password
//! 2. Password is validated against security policy and hashed with Argon2
//! 3. Session is created with a JWT access token (24h expiry) and refresh token (14d expiry)
//! 4. Access tokens authenticate API requests
//! 5. Refresh tokens obtain new access tokens when expired
//!
//! ## Security Features
//!
//! - **Password Hashing**: Argon2id with random salts
//! - **Password Policy**: Enforces complexity requirements (length, character diversity, no common passwords)
//! - **JWT Tokens**: Short-lived access tokens with user claims
//! - **Refresh Token Rotation**: Single-use refresh tokens with SHA-256 hashing
//! - **Session Limits**: Maximum 5 concurrent sessions per user
//! - **Re-authentication**: Sensitive operations require password verification
//!
//! ## Account Management
//!
//! All credential changes (username, email, password) require re-authentication with the
//! current password to prevent unauthorized account takeover.
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::{
//!     register_user::RegisterUser,
//!     authenticate_user::AuthenticateUser,
//! };
//!
//! // Register new user
//! let register = RegisterUser::from_str("username:john email:john@example.com password:SecurePass123!")?;
//! let session = auth_service.register_user(register).await?;
//!
//! // Authenticate existing user
//! let auth = AuthenticateUser::from_str("identifier:john password:SecurePass123!")?;
//! let session = auth_service.authenticate_user(auth).await?;
//!
//! // Session contains access token for API requests
//! println!("Access token: {}", session.access_token);
//! ```

/// Authentication models and value objects.
pub mod models;

/// Port traits (interfaces) for authentication operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for authentication business logic.
#[cfg(feature = "zerver")]
pub mod services;
