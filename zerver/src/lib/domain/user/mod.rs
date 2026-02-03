//! User domain logic and models.
//!
//! This module provides read-only access to user account information. User creation
//! and modification are handled in the [`auth`](crate::domain::auth) module, as they
//! require authentication and authorization.
//!
//! # Responsibilities
//!
//! - **User Retrieval**: Fetch user profile information by ID
//! - **Username Validation**: Enforce username constraints (length, content, profanity)
//! - **Public User Data**: Expose safe, non-sensitive user information
//!
//! # User vs Authentication
//!
//! The separation between `user` and `auth` modules reflects different concerns:
//!
//! - **user**: Public profile data (read operations)
//! - **auth**: Credentials and sessions (write operations, password handling)
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::user::models::get_user::GetUser;
//!
//! // Fetch user profile
//! let request = GetUser::from(user_id);
//! let user = user_service.get_user(request).await?;
//!
//! println!("User: {} ({})", user.username, user.email);
//! ```

/// User models and value objects (User, Username, operations).
pub mod models;

/// Port traits (interfaces) for user operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for user business logic.
#[cfg(feature = "zerver")]
pub mod services;
