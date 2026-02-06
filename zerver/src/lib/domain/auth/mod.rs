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

/// Authentication models and value objects.
pub mod models;

/// Port traits (interfaces) for authentication operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for authentication business logic.
#[cfg(feature = "zerver")]
pub mod services;
