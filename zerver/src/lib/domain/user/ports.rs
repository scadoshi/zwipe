//! Port traits for user profile operations.
//!
//! This module defines the interfaces (ports) for user data access.
//! User profiles are read-only in this module - mutations (register, change password,
//! etc.) are handled by the auth module.
//!
//! # Separation of Concerns
//!
//! - **User Module**: Read operations (get user profile)
//! - **Auth Module**: Write operations (register, update profile, delete)
//!
//! This separation keeps authentication logic isolated from user data access.

use std::future::Future;
use uuid::Uuid;

use crate::domain::user::models::{
    get_user::{GetUser, GetUserError},
    User,
};

/// Database port for user profile operations.
///
/// Provides read-only access to user data. Write operations are in `auth` module.
pub trait UserRepository: Clone + Send + Sync + 'static {
    /// Retrieves a user profile by ID.
    ///
    /// Returns user data without password hash (use AuthRepository for that).
    fn get_user(&self, user_id: Uuid) -> impl Future<Output = Result<User, GetUserError>> + Send;
}

/// Service port for user profile business logic.
///
/// Currently minimal - just passes through to repository.
/// Future expansion: user statistics, profile caching, etc.
pub trait UserService: Clone + Send + Sync + 'static {
    // =====
    //  get
    // =====

    /// Retrieves a user profile by ID.
    ///
    /// Returns public user data (username, email, ID).
    fn get_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;
}
