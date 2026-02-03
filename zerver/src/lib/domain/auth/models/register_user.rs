//! User registration operation.
//!
//! This module handles new user account creation with comprehensive validation
//! and security measures. Registration involves:
//!
//! 1. Validating username, email, and password against policies
//! 2. Checking for duplicate usernames or emails
//! 3. Hashing the password with Argon2id
//! 4. Creating the user account in the database
//! 5. Automatically creating a session for immediate login
//!
//! # Security Features
//!
//! - **Password Policy Enforcement**: Validates complexity requirements
//! - **Password Hashing**: Argon2id with random salts
//! - **Username Moderation**: Rejects profanity and inappropriate content
//! - **Email Validation**: Ensures valid email format
//! - **Duplicate Prevention**: Prevents username/email collisions
//!
//! # Registration Flow
//!
//! ```text
//! Client Request
//!     ↓
//! Validate Credentials (username, email, password)
//!     ↓
//! Hash Password (Argon2id)
//!     ↓
//! Check for Duplicates
//!     ↓
//! Create User in Database
//!     ↓
//! Create Session (access + refresh tokens)
//!     ↓
//! Return Session to Client
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::register_user::RegisterUser;
//!
//! // Create registration request
//! let request = RegisterUser::new(
//!     "newuser",
//!     "newuser@example.com",
//!     "SecurePass123!"
//! )?;
//!
//! // Register user and get session
//! let session = auth_service.register_user(request).await?;
//!
//! // User is now registered and logged in
//! println!("Welcome, {}!", session.user.username);
//! ```

#[cfg(feature = "zerver")]
use crate::domain::auth::models::password::HashedPassword;
#[cfg(feature = "zerver")]
use crate::domain::auth::models::session::create_session::CreateSessionError;
use crate::domain::user::models::username::Username;
use crate::domain::{
    auth::models::password::{InvalidPassword, Password},
    user::models::username::InvalidUsername,
};
use email_address::EmailAddress;
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "zerver")]
/// Errors that can occur during user registration.
///
/// Registration involves multiple steps (validation, duplicate checking, user creation,
/// session creation), each of which can fail. These errors distinguish between
/// validation failures (user's fault) and system failures (server's fault).
#[derive(Debug, Error)]
pub enum RegisterUserError {
    /// A user with the same username or email already exists.
    ///
    /// Usernames and emails must be unique across all users. The client should
    /// prompt the user to choose a different username or email.
    #[error("user with name or email already exists")]
    Duplicate,

    /// Database operation failed during user creation.
    #[error(transparent)]
    Database(anyhow::Error),

    /// Failed to generate JWT access token for the new user's session.
    #[error("failed to generate access token: {0}")]
    FailedAccessToken(anyhow::Error),

    /// User was created but the database returned invalid/corrupted data.
    ///
    /// This is a critical error indicating database schema issues or data corruption.
    #[error("user created but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),

    /// Failed to create initial session for the newly registered user.
    #[error(transparent)]
    CreateSessionError(#[from] CreateSessionError),
}

/// Errors that can occur while constructing a [`RawRegisterUser`] (before hashing).
#[derive(Debug, Error)]
pub enum InvalidRawRegisterUser {
    /// Username doesn't meet requirements.
    #[error(transparent)]
    Username(InvalidUsername),
    /// Email format is invalid.
    #[error(transparent)]
    Email(email_address::Error),
    /// Password doesn't meet security requirements.
    #[error(transparent)]
    Password(InvalidPassword),
}

impl From<InvalidUsername> for InvalidRawRegisterUser {
    fn from(value: InvalidUsername) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for InvalidRawRegisterUser {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

impl From<InvalidPassword> for InvalidRawRegisterUser {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
}

/// Errors that can occur while constructing a [`RegisterUser`] (with password hashing).
#[derive(Debug, Error)]
pub enum InvalidRegisterUser {
    /// Username doesn't meet requirements.
    #[error(transparent)]
    Username(InvalidUsername),
    /// Email format is invalid.
    #[error(transparent)]
    Email(email_address::Error),
    /// Password doesn't meet security requirements.
    #[error(transparent)]
    Password(InvalidPassword),
    /// Password hashing failed (Argon2id error).
    #[error("failed to hash password: {0}")]
    FailedPasswordHash(anyhow::Error),
}

impl From<InvalidUsername> for InvalidRegisterUser {
    fn from(value: InvalidUsername) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for InvalidRegisterUser {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

impl From<InvalidPassword> for InvalidRegisterUser {
    fn from(value: InvalidPassword) -> Self {
        Self::Password(value)
    }
}

/// Raw registration data (validated but password not yet hashed).
///
/// This is an intermediate type used when you need validated registration
/// data but don't want to hash the password yet. Used internally by
/// [`RegisterUser::new`] before password hashing.
///
/// Most code should use [`RegisterUser`] instead, which includes the hashed password.
#[derive(Debug)]
pub struct RawRegisterUser {
    /// Validated username (3-20 chars, no profanity).
    pub username: Username,
    /// Validated email address.
    pub email: EmailAddress,
    /// Validated password (meets all security requirements).
    pub password: Password,
}

impl RawRegisterUser {
    /// Creates a new raw registration with validation.
    ///
    /// # Parameters
    ///
    /// - `username`: Username string (will be validated)
    /// - `email`: Email address string (will be validated)
    /// - `password`: Password string (will be validated but NOT hashed)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRawRegisterUser`] if any field fails validation.
    pub fn new(
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<Self, InvalidRawRegisterUser> {
        let username = Username::new(username)?;
        let email = EmailAddress::from_str(email)?;
        let password = Password::new(password)?;
        Ok(Self {
            username,
            email,
            password,
        })
    }
}

#[cfg(feature = "zerver")]
/// Request to register a new user account.
///
/// This type validates all inputs (username, email, password) and hashes the password
/// before the service layer processes the registration. It guarantees that any instance
/// contains valid, policy-compliant data ready for database insertion.
///
/// # Validation
///
/// Creation via [`new()`](Self::new) enforces:
///
/// - **Username**: 3-20 chars, no whitespace, no profanity
/// - **Email**: Valid email address format
/// - **Password**: 8-128 chars, complexity requirements (see [`Password`])
/// - **Password Hashing**: Argon2id hashing must succeed
///
/// # Security
///
/// The password is hashed immediately during construction and the plaintext is
/// discarded. Only the hash is stored in this struct and sent to the database.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::register_user::RegisterUser;
///
/// // Create registration request (validates and hashes)
/// let request = RegisterUser::new(
///     "alice",
///     "alice@example.com",
///     "SecurePass123!"
/// )?;
///
/// // Password is now hashed, safe to store
/// assert!(!request.password_hash.to_string().contains("SecurePass"));
///
/// // Register the user
/// let session = auth_service.register_user(request).await?;
/// ```
#[derive(Debug)]
pub struct RegisterUser {
    /// Validated username for the new account.
    pub username: Username,

    /// Validated email address for the new account.
    pub email: EmailAddress,

    /// Argon2id hash of the user's password.
    ///
    /// Never contains the plaintext password.
    pub password_hash: HashedPassword,
}

#[cfg(feature = "zerver")]
impl RegisterUser {
    /// Creates a new registration request with validated and hashed credentials.
    ///
    /// This constructor validates all inputs and hashes the password. If any validation
    /// fails, an error is returned with details about which field failed.
    ///
    /// # Arguments
    ///
    /// * `username` - Desired username (3-20 chars, no profanity)
    /// * `email` - Email address (must be valid format)
    /// * `password` - Plaintext password (must meet complexity requirements)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRegisterUser`] if:
    /// - Username is invalid (too short/long, contains profanity)
    /// - Email is malformed
    /// - Password doesn't meet complexity requirements
    /// - Password hashing fails (extremely rare)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = RegisterUser::new(
    ///     "johndoe",
    ///     "john@example.com",
    ///     "MySecure123!"
    /// )?;
    /// ```
    pub fn new(username: &str, email: &str, password: &str) -> Result<Self, InvalidRegisterUser> {
        use std::str::FromStr;

        use crate::domain::auth::models::password::Password;

        let username = Username::new(username)?;
        let email = EmailAddress::from_str(email)?;
        let password = Password::new(password)?;
        let password_hash = HashedPassword::generate(password)
            .map_err(|e| InvalidRegisterUser::FailedPasswordHash(e.into()))?;

        Ok(RegisterUser {
            username,
            email,
            password_hash,
        })
    }
}
