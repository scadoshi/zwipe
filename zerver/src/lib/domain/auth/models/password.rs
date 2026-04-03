//! Password validation, hashing, and verification.
//!
//! This module implements a comprehensive password security policy with validation,
//! Argon2id hashing, and secure verification. It enforces industry-standard password
//! requirements to prevent weak credentials.
//!
//! # Password Policy
//!
//! Passwords must meet ALL of the following requirements:
//!
//! - **Length**: 8-128 characters
//! - **Uppercase**: At least one uppercase letter (A-Z)
//! - **Lowercase**: At least one lowercase letter (a-z)
//! - **Digit**: At least one number (0-9)
//! - **Symbol**: At least one symbol from: `~!@#$%^&*()_+=[]{}\/?|:;<>,.`
//! - **No Whitespace**: No spaces, tabs, or newlines
//! - **Unique Characters**: Minimum 6 unique characters
//! - **Repeat Limit**: Maximum 3 consecutive repeated characters
//!
//! # Security Features
//!
//! - **Argon2id**: Memory-hard hashing algorithm resistant to GPU attacks
//! - **Random Salts**: Unique salt per password using OS random number generator
//! - **Constant-Time Verification**: Prevents timing attacks
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::password::{Password, HashedPassword};
//!
//! // Validate and hash a new password
//! let password = Password::new("SecurePass123!")?;
//! let hashed = password.hash()?;
//!
//! // Store hashed password in database
//! user_repo.update_password(&user_id, hashed).await?;
//!
//! // Later: verify during authentication
//! let provided = Password::new(user_input)?;
//! let stored_hash = user_repo.get_password_hash(&user_id).await?;
//! stored_hash.verify(&provided)?; // Returns Ok(()) if matches
//! ```

#[cfg(feature = "zerver")]
use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{self, SaltString, rand_core::OsRng},
};
use std::fmt::Display;

// Re-export validation types from zwipe-core so downstream consumers
// (handlers, services) can continue importing from this module.
pub use zwipe_core::domain::auth::password::{
    InvalidPassword, SYMBOLS, TooFewUniqueChars, TooManyRepeats,
};

/// A validated password that meets all security policy requirements.
///
/// This is a value object that guarantees any instance contains a password
/// that has been validated against the comprehensive password policy. The raw
/// password string is wrapped and can only be accessed via [`read()`](Self::read).
///
/// # Validation
///
/// Creation via [`new()`](Self::new) enforces:
/// - Length: 8-128 characters
/// - At least one uppercase, lowercase, digit, and symbol
/// - No whitespace
/// - Maximum 3 consecutive repeated characters
/// - Minimum 6 unique characters
/// - Not a common/weak password
///
/// # Security
///
/// The password is stored in plaintext within this type for hashing purposes.
/// It should be:
/// - Hashed immediately after validation via [`hash()`](Self::hash)
/// - Never logged or exposed in error messages
/// - Cleared from memory as soon as possible after hashing
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::password::{Password, InvalidPassword};
///
/// // Valid password
/// let password = Password::new("SecurePass123!")?;
/// let hash = password.hash()?; // Consume and hash immediately
///
/// // Invalid passwords return specific errors
/// assert!(matches!(
///     Password::new("weak"),
///     Err(InvalidPassword::TooShort)
/// ));
/// ```
#[derive(Debug, Clone)]
pub struct Password(String);

impl Password {
    /// Creates a new validated password.
    ///
    /// Validates the input against all password policy requirements.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidPassword`] if any requirement is not met, with specific
    /// variant indicating which rule failed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let password = Password::new("MySecurePass123!")?;
    /// ```
    pub fn new(raw: impl AsRef<str>) -> Result<Self, InvalidPassword> {
        let raw = raw.as_ref();
        zwipe_core::domain::auth::password::validate(raw)?;
        Ok(Password(raw.to_string()))
    }

    /// Hashes the password using Argon2id with a random salt.
    ///
    /// Consumes the `Password` and returns a [`HashedPassword`] suitable for
    /// database storage. The hash includes the algorithm parameters and salt,
    /// so no separate salt storage is needed.
    ///
    /// # Errors
    ///
    /// Returns [`password_hash::Error`] if hashing fails (extremely rare).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let password = Password::new("SecurePass123!")?;
    /// let hashed = password.hash()?; // Consumes password
    /// // Store `hashed` in database
    /// ```
    #[cfg(feature = "zerver")]
    pub fn hash(self) -> Result<HashedPassword, password_hash::Error> {
        HashedPassword::generate(self)
    }

    /// Returns a reference to the underlying password string.
    ///
    /// # Security Warning
    ///
    /// Use sparingly and only for operations like hashing. Never log or
    /// expose this value in API responses or error messages.
    pub fn read(&self) -> &str {
        &self.0
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "zerver")]
/// An Argon2id password hash suitable for secure storage.
///
/// This value object wraps a password hash string in PHC (Password Hashing Competition)
/// format. The hash includes all necessary information for verification:
/// - Algorithm identifier (`$argon2id$`)
/// - Algorithm parameters (memory cost, time cost, parallelism)
/// - Salt (base64-encoded)
/// - Hash digest (base64-encoded)
///
/// Named `HashedPassword` to avoid conflict with `argon2::PasswordHash`.
///
/// # Format
///
/// ```text
/// $argon2id$v=19$m=19456,t=2,p=1$randomsalt$hashdigest
/// ```
///
/// # Security
///
/// - **Argon2id**: Winner of Password Hashing Competition, resistant to GPU attacks
/// - **Random Salt**: Generated using OS cryptographically secure RNG
/// - **Constant-Time Verification**: Prevents timing attacks
/// - **Self-Contained**: No separate salt storage needed
///
/// # Storage
///
/// Store the hash string directly in your database. It's safe to store alongside
/// other user data as it cannot be reversed to obtain the original password.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::password::{Password, HashedPassword};
///
/// // During registration
/// let password = Password::new(user_input)?;
/// let hashed = password.hash()?;
/// user_repo.create_user(username, email, hashed).await?;
///
/// // During authentication
/// let stored_hash: HashedPassword = user_repo.get_password_hash(&user_id).await?;
/// let provided = Password::new(user_input)?;
///
/// if stored_hash.verify(provided.read())? {
///     // Password matches - proceed with login
/// } else {
///     // Password doesn't match - reject
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct HashedPassword(String);

#[cfg(feature = "zerver")]
impl HashedPassword {
    /// Creates a `HashedPassword` from an existing hash string.
    ///
    /// Validates that the input is a properly formatted Argon2 hash.
    /// Use this when reconstructing from database storage.
    ///
    /// # Errors
    ///
    /// Returns [`password_hash::Error`] if the hash string is malformed or
    /// uses an unsupported algorithm.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // From database
    /// let hash_string: String = row.get("password_hash");
    /// let hashed = HashedPassword::new(&hash_string)?;
    /// ```
    pub fn new(raw: &str) -> Result<Self, password_hash::Error> {
        argon2::PasswordHash::new(raw)?;
        Ok(Self(raw.to_string()))
    }

    /// Generates a new password hash from a validated password.
    ///
    /// Creates a cryptographically secure hash using:
    /// - Argon2id algorithm (hybrid of Argon2i and Argon2d)
    /// - Random salt from OS RNG
    /// - Default parameters (balanced security/performance)
    ///
    /// This method is also available via [`Password::hash()`].
    ///
    /// # Errors
    ///
    /// Returns [`password_hash::Error`] if hashing fails (extremely rare,
    /// typically only on system resource exhaustion).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let password = Password::new("SecurePass123!")?;
    /// let hashed = HashedPassword::generate(password)?;
    /// ```
    pub fn generate(password: Password) -> Result<Self, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.read().as_bytes(), &salt)
            .map(|x| x.to_string())?;
        Ok(Self(hash))
    }

    /// Verifies a plaintext password against this hash.
    ///
    /// Uses constant-time comparison to prevent timing attacks. The verification
    /// extracts the salt and parameters from the hash and re-hashes the provided
    /// password, then compares the digests.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the password matches
    /// - `Ok(false)` if the password doesn't match
    /// - `Err(_)` if hash parsing or verification fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let stored_hash: HashedPassword = user_repo.get_password_hash(&user_id).await?;
    /// let user_input = "SecurePass123!";
    ///
    /// match stored_hash.verify(user_input)? {
    ///     true => println!("Authentication successful"),
    ///     false => println!("Invalid password"),
    /// }
    /// ```
    pub fn verify(&self, password: &str) -> Result<bool, password_hash::Error> {
        let parsed_hash = argon2::PasswordHash::new(&self.0)?;

        match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl std::fmt::Display for HashedPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================
    //  password hashing
    // ==================

    #[test]
    fn test_hash_password_success_creates_valid_hashes() {
        let password = Password::new("TestPassword123!").unwrap();
        let result = HashedPassword::generate(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());

        // Argon2 hashes should start with $argon2 and have multiple $ delimited sections
        assert!(hash.0.starts_with("$argon2"));
        assert!(hash.0.matches('$').count() >= 4); // format: $argon2$variant$params$salt$hash
    }

    // =================================
    //  `Password`.hash() ergonomic api
    // =================================

    #[test]
    fn test_password_hash_method_creates_valid_hashes() {
        let password = Password::new("ErgonomicPassword123!").unwrap();
        let result = password.hash();
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.to_string().is_empty());
        assert!(hash.to_string().starts_with("$argon2"));
    }

    #[test]
    fn test_password_hash_method_produces_unique_hashes() {
        let password1 = Password::new("SamePassword123!").unwrap();
        let password2 = Password::new("SamePassword123!").unwrap();

        let hash1 = password1.hash().unwrap();
        let hash2 = password2.hash().unwrap();

        // Should be different due to unique salt generation
        assert_ne!(hash1.to_string(), hash2.to_string());
    }

    #[test]
    fn test_password_hash_method_equivalent_to_generate() {
        let password_str = "EquivalenceTest456!";

        // Test both APIs produce valid, verifiable hashes
        let password1 = Password::new(password_str).unwrap();
        let password2 = Password::new(password_str).unwrap();

        let hash1 = password1.hash().unwrap();
        let hash2 = HashedPassword::generate(password2).unwrap();

        // Both should verify the original password
        assert!(hash1.verify(password_str).unwrap());
        assert!(hash2.verify(password_str).unwrap());

        // Both should have same format characteristics
        assert!(hash1.to_string().starts_with("$argon2"));
        assert!(hash2.to_string().starts_with("$argon2"));
    }

    #[test]
    fn test_password_hash_method_consumes_password() {
        let password = Password::new("ConsumedPassword789!").unwrap();
        let _hash = password.hash().unwrap();

        // This should not compile if uncommented (password moved):
        // let _another_hash = password.hash();

        // Test passes if it compiles, showing password is consumed
    }

    #[test]
    fn test_password_hash_method_with_various_inputs() {
        let test_passwords = vec!["SimpleHash123!", "Complex🔐Password456@", "Unicode密码789#"];
        for password_str in test_passwords {
            let password = Password::new(password_str).unwrap();
            let hash = password.hash().unwrap();

            // Verify the hash works
            assert!(hash.verify(password_str).unwrap());
            assert!(!hash.verify("WrongPassword123!").unwrap());
        }
    }

    #[test]
    fn test_password_hash_method_error_propagation() {
        // This test verifies error types are properly propagated
        // In practice, HashedPassword::generate rarely fails, but we test the signature
        let password = Password::new("ErrorTestPassword123!").unwrap();
        let result = password.hash();

        // Should succeed normally
        assert!(result.is_ok());

        // Error type should be password_hash::Error if it fails
        match result {
            Ok(_) => {} // Expected case
            Err(e) => {
                // If this somehow fails, error should be the right type
                let _: password_hash::Error = e;
            }
        }
    }

    #[test]
    fn test_hash_password_produces_unique_hashes_with_same_input() {
        let password = Password::new("IdenticalPassword123!").unwrap();
        let hash1 = HashedPassword::generate(password.clone()).unwrap();
        let hash2 = HashedPassword::generate(password).unwrap();

        // Should be different due to unique salt generation
        assert_ne!(hash1.0, hash2.0);
    }

    #[test]
    fn test_hash_password_produces_different_hashes_for_different_inputs() {
        let password1 = Password::new("FirstUnique123!").unwrap();
        let password2 = Password::new("DifferentPass456@").unwrap();
        let hash1 = HashedPassword::generate(password1).unwrap();
        let hash2 = HashedPassword::generate(password2).unwrap();

        assert_ne!(hash1.0, hash2.0);
    }

    #[test]
    fn test_password_validation_rejects_empty_input() {
        let result = Password::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), InvalidPassword::TooShort));
    }

    #[test]
    fn test_hash_password_handles_max_length_input() {
        // 128 chars is the maximum — use "abc" pattern to avoid TooManyRepeats
        let suffix = "A1!";
        let padding: String = "abcdef".chars().cycle().take(128 - suffix.len()).collect();
        let long_password = format!("{}{}", padding, suffix);
        assert_eq!(long_password.len(), 128);
        let password = Password::new(&long_password).unwrap();
        let hash = HashedPassword::generate(password).unwrap();
        assert!(!hash.0.is_empty());
    }

    #[test]
    fn test_hash_password_rejects_over_max_length() {
        let suffix = "A1!";
        let padding: String = "abcdef".chars().cycle().take(129 - suffix.len()).collect();
        let too_long = format!("{}{}", padding, suffix);
        assert_eq!(too_long.len(), 129);
        assert!(matches!(
            Password::new(&too_long),
            Err(InvalidPassword::TooLong)
        ));
    }

    #[test]
    fn test_hash_password_handles_special_characters() {
        let special_password = "P@ssw0rd!#$%^&*(){}[]|\\:;\"'<>,.?/~`";
        let password = Password::new(special_password).unwrap();
        let result = HashedPassword::generate(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    #[test]
    fn test_hash_password_handles_unicode_characters() {
        let unicode_password = "Пароль1🔒!";
        let password = Password::new(unicode_password).unwrap();
        let result = HashedPassword::generate(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    // =======================
    //  password verification
    // =======================

    // ======================================
    //  remaining validation rule coverage
    // ======================================

    #[test]
    fn test_password_validation_rejects_whitespace() {
        assert!(matches!(
            Password::new("Has Space123!"),
            Err(InvalidPassword::ContainsWhitespace)
        ));
        assert!(matches!(
            Password::new("HasTab\t123!A"),
            Err(InvalidPassword::ContainsWhitespace)
        ));
        assert!(matches!(
            Password::new("HasNewline\n1A!"),
            Err(InvalidPassword::ContainsWhitespace)
        ));
    }

    #[test]
    fn test_password_validation_rejects_too_many_repeats() {
        // 4 consecutive same chars anywhere in the password — exceeds limit of 3
        assert!(matches!(
            Password::new("Aaaaa1!bc"),
            Err(InvalidPassword::TooManyRepeats(_))
        ));
        assert!(matches!(
            Password::new("Abcde1111!"),
            Err(InvalidPassword::TooManyRepeats(_))
        ));

        // Exactly 3 consecutive — at the boundary, should pass
        assert!(Password::new("Aaa1!bcde").is_ok());
    }

    #[test]
    fn test_password_validation_rejects_too_few_unique_chars() {
        // 4 unique chars — below minimum of 6
        assert!(matches!(
            Password::new("AbAb1!Ab"),
            Err(InvalidPassword::TooFewUniqueChars(_))
        ));
        // 5 unique chars — still below minimum
        assert!(matches!(
            Password::new("AbcAbc1!"),
            Err(InvalidPassword::TooFewUniqueChars(_))
        ));

        // 8 unique chars — should pass
        assert!(Password::new("Abcde1!f").is_ok());
    }

    #[test]
    fn test_password_validation_minimum_length_boundary() {
        // 7 chars — one below minimum
        assert!(matches!(
            Password::new("Abcde1!"),
            Err(InvalidPassword::TooShort)
        ));

        // Exactly 8 chars — minimum valid length
        assert!(Password::new("Abcde1!x").is_ok());
    }

    #[test]
    fn test_verify_password_success_with_correct_password() {
        let password_str = "CorrectPassword123!";
        let password = Password::new(password_str).unwrap();
        let hash = HashedPassword::generate(password).unwrap();

        let result = hash.verify(password_str);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_fails_with_wrong_password() {
        let correct_password = Password::new("CorrectPassword123!").unwrap();
        let hash = HashedPassword::generate(correct_password).unwrap();

        let result = hash.verify("WrongPassword456@");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_case_sensitive() {
        let password = Password::new("CaseSensitive123!").unwrap();
        let hash = HashedPassword::generate(password).unwrap();

        // Exact match should work
        assert!(hash.verify("CaseSensitive123!").unwrap());

        // Different case should fail
        assert!(!hash.verify("casesensitive123!").unwrap());
        assert!(!hash.verify("CASESENSITIVE123!").unwrap());
    }

    #[test]
    fn test_password_validation_various_failures() {
        // Test all validation rules - inputs must only fail on the ONE rule being tested
        assert!(matches!(
            Password::new("short"),
            Err(InvalidPassword::TooShort)
        ));
        assert!(matches!(
            Password::new("nouppercase123!"),
            Err(InvalidPassword::MissingUpperCase)
        ));
        assert!(matches!(
            Password::new("NOLOWERCASE123!"),
            Err(InvalidPassword::MissingLowerCase)
        ));
        assert!(matches!(
            Password::new("NoNumbers!abcdef"),
            Err(InvalidPassword::MissingNumber)
        ));
        assert!(matches!(
            Password::new("NoSymbols123ab"),
            Err(InvalidPassword::MissingSymbol(_))
        ));
    }

    #[test]
    fn test_verify_password_with_special_characters() {
        let special_password_str = "P@ssw0rd!#$%";
        let special_password = Password::new(special_password_str).unwrap();
        let hash = HashedPassword::generate(special_password).unwrap();

        assert!(hash.verify(special_password_str).unwrap());
        assert!(!hash.verify("P@ssw0rd!#$").unwrap()); // Missing %
    }

    #[test]
    fn test_verify_password_with_unicode_characters() {
        let unicode_password_str = "Пароль1🔒!";
        let unicode_password = Password::new(unicode_password_str).unwrap();
        let hash = HashedPassword::generate(unicode_password).unwrap();

        assert!(hash.verify(unicode_password_str).unwrap());
        assert!(!hash.verify("Пароль1🔒").unwrap()); // Missing !
    }

    // =======================
    //  security & edge cases
    // =======================

    #[test]
    fn test_hash_verification_round_trip_with_various_inputs() {
        let test_passwords = vec![
            "Trekking749!xyz".to_string(),
            "NoSpacesHere456@".to_string(),
            "MixedCaseXyz123!".to_string(),
            {
                // Long password under 128, cycling chars to avoid TooManyRepeats
                let suffix = "A1!";
                let padding: String = "abcdef".chars().cycle().take(96 - suffix.len()).collect();
                format!("{}{}", padding, suffix)
            },
            "Emoji🚀🔒💻1!Ab".to_string(), // With emoji
        ];

        for password_str in test_passwords {
            let password = Password::new(&password_str).unwrap();
            let hash = HashedPassword::generate(password).unwrap();
            assert!(hash.verify(&password_str).unwrap());

            // Verify that a slightly different password fails
            let wrong_password = format!("{}x", password_str);
            assert!(!hash.verify(&wrong_password).unwrap());
        }
    }

    #[test]
    fn test_salt_uniqueness_across_multiple_hashes() {
        let password_str = "SamePassword123!";
        let mut hashes = Vec::new();

        // Generate multiple hashes of the same password
        for _ in 0..10 {
            let password = Password::new(password_str).unwrap();
            hashes.push(HashedPassword::generate(password).unwrap());
        }

        // All hashes should be unique (due to unique salts)
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes.get(i), hashes.get(j));
            }
        }

        // But all should verify correctly
        for hash in &hashes {
            assert!(hash.verify(password_str).unwrap());
        }
    }

    // ==============================
    //  `HashedPassword` constructor
    // ==============================

    #[test]
    fn test_hashed_password_new_accepts_valid_argon2_hash() {
        // Generate a valid hash first
        let password = Password::new("TestPassword123!").unwrap();
        let valid_hash = HashedPassword::generate(password).unwrap();
        let hash_string = valid_hash.to_string();

        // Should accept the valid hash string
        let result = HashedPassword::new(&hash_string);
        assert!(result.is_ok());

        let reconstructed = result.unwrap();
        assert_eq!(reconstructed.to_string(), hash_string);
    }

    #[test]
    fn test_hashed_password_new_rejects_invalid_formats() {
        let invalid_hashes = vec![
            "",                   // Empty string
            "plaintext_password", // Plain text
            "no_dollar_prefix",   // No $ prefix
            "$bcrypt$12$xyz",     // Wrong algorithm
        ];

        for invalid_hash in invalid_hashes {
            let result = HashedPassword::new(invalid_hash);
            assert!(result.is_err(), "Should reject: {}", invalid_hash);
        }
    }

    #[test]
    fn test_hashed_password_new_rejects_garbage_input() {
        // Strings that are clearly not PHC format hashes
        assert!(HashedPassword::new("not-a-hash-at-all").is_err());
        assert!(HashedPassword::new("").is_err());
        assert!(HashedPassword::new("$$$$$").is_err());
    }

    #[test]
    fn test_hashed_password_new_handles_whitespace() {
        let password = Password::new("TestPassword123!").unwrap();
        let valid_hash = HashedPassword::generate(password).unwrap();
        let hash_string = valid_hash.to_string();

        // Test with leading/trailing whitespace
        let with_whitespace = format!("  {}  ", hash_string);
        let result = HashedPassword::new(&with_whitespace);
        assert!(result.is_err(), "Should reject hash with whitespace");
    }

    #[test]
    fn test_hashed_password_new_round_trip_with_multiple_hashes() {
        let test_passwords = vec![
            "Simple123!",
            "Complex🔒Password456@",
            "AnotherValidPass789#",
        ];

        for password_str in test_passwords {
            // Generate hash
            let password = Password::new(password_str).unwrap();
            let generated_hash = HashedPassword::generate(password).unwrap();
            let hash_string = generated_hash.to_string();

            // Reconstruct from string
            let reconstructed_hash = HashedPassword::new(&hash_string).unwrap();

            // Should be identical and both should verify the original password
            assert_eq!(generated_hash.to_string(), reconstructed_hash.to_string());
            assert!(generated_hash.verify(password_str).unwrap());
            assert!(reconstructed_hash.verify(password_str).unwrap());
        }
    }

    // =============
    //  integration
    // =============

    #[test]
    fn test_complete_password_authentication_flow() {
        // Simulate complete user registration and login flow
        let user_password_str = "UserChosenPassword123!";

        // Registration: validate and hash the password
        let password = Password::new(user_password_str).unwrap();
        let stored_hash = HashedPassword::generate(password).unwrap();

        // Later login attempt: verify the password
        let login_result = stored_hash.verify(user_password_str).unwrap();
        assert!(login_result);

        // Invalid login attempt: wrong password
        let wrong_login_result = stored_hash.verify("WrongPassword456@").unwrap();
        assert!(!wrong_login_result);

        // Verify hash format is suitable for database storage
        assert!(stored_hash.0.len() > 50); // Reasonable length for storage
        assert!(stored_hash.0.is_ascii()); // Safe for database storage
    }

    #[test]
    fn test_database_hash_retrieval_simulation() {
        // Simulate storing hash in database and retrieving it
        let user_password_str = "DatabasePassword123!";

        // Registration: create and "store" hash
        let password = Password::new(user_password_str).unwrap();
        let stored_hash = HashedPassword::generate(password).unwrap();
        let hash_for_database = stored_hash.to_string();

        // Simulate database storage/retrieval cycle
        // (In real code, this would go to/from database)
        let retrieved_hash_string = hash_for_database; // Simulate DB retrieval

        // Domain validation: ensure retrieved hash is valid
        let retrieved_hash = HashedPassword::new(&retrieved_hash_string).unwrap();

        // Authentication: verify password against retrieved hash
        assert!(retrieved_hash.verify(user_password_str).unwrap());
        assert!(!retrieved_hash.verify("WrongPassword456@").unwrap());
    }
}
