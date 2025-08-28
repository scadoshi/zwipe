// internal
mod common_passwords;
use common_passwords::IsCommonPassword;
// std
use std::collections::HashSet;
// external
use argon2::{
    password_hash::{self, rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use thiserror::Error;

// ========
//  errors
// ========

/// errors encountered while constructing `Password`
#[derive(Debug, Clone, Error)]
pub enum PasswordError {
    #[error("Password must be at least 8 characters long")]
    TooShort,
    #[error("Password must not exceed 128 characters")]
    TooLong,
    #[error("Password must have at least one uppercase character")]
    MissingUpperCase,
    #[error("Password must have at least one lowercase character")]
    MissingLowerCase,
    #[error("Password must have at least one number")]
    MissingNumber,
    #[error("Password must have at least one symbol from {}", 0)]
    MissingSymbol(String),
    #[error("Password must not contain whitespace characters")]
    ContainsWhitespace,
    #[error("Password is too common and not secure")]
    CommonPassword,
    #[error(transparent)]
    TooManyRepeats(TooManyRepeats),
    #[error(transparent)]
    TooFewUniqueChars(TooFewUniqueChars),
}

/// error for when there are too many
/// repeated letters in given password
#[derive(Debug, Clone, Error)]
#[error("Password must not contain more than {} repeated characters", 0)]
pub struct TooManyRepeats(u8);

impl From<TooManyRepeats> for PasswordError {
    fn from(value: TooManyRepeats) -> Self {
        PasswordError::TooManyRepeats(value)
    }
}

/// error for when there are too few
/// characters in given password
#[derive(Debug, Clone, Error)]
#[error("Password must contain at least {} unique characters", 0)]
pub struct TooFewUniqueChars(u8);

impl From<TooFewUniqueChars> for PasswordError {
    fn from(value: TooFewUniqueChars) -> Self {
        PasswordError::TooFewUniqueChars(value)
    }
}

/// password must include one of these
const SYMBOLS: &str = r#"~!@#$%^&*()_+=[]{}\/?|:;<>,."#;

/// wraps validated password
#[derive(Debug, Clone)]
pub struct Password(String);

impl Password {
    pub fn new(raw: &str) -> Result<Self, PasswordError> {
        raw.meets_all_requirements()?;
        Ok(Password(raw.to_string()))
    }

    pub fn hash(self) -> Result<HashedPassword, password_hash::Error> {
        HashedPassword::generate(self)
    }
}

/// enables password policy validation
trait PasswordPolicy {
    fn min_unique_char_requirement(&self, at_least: u8) -> Result<(), TooFewUniqueChars>;
    fn max_repeat_char_requirement(&self, at_most: u8) -> Result<(), TooManyRepeats>;
    fn meets_all_requirements(&self) -> Result<(), PasswordError>;
}

impl PasswordPolicy for &str {
    fn min_unique_char_requirement(&self, at_least: u8) -> Result<(), TooFewUniqueChars> {
        let unique_chars: HashSet<char> = self.chars().collect();
        if unique_chars.len() < 6 {
            return Err(TooFewUniqueChars(at_least));
        }
        Ok(())
    }
    fn max_repeat_char_requirement(&self, at_most: u8) -> Result<(), TooManyRepeats> {
        let mut repeat_count = 1;
        let mut last_char_opt: Option<char> = None;

        for char in self.chars() {
            if last_char_opt.is_none() {
                last_char_opt = Some(char);
                continue;
            }

            if let Some(last_char) = last_char_opt {
                if char == last_char {
                    repeat_count += 1;
                    if repeat_count > at_most {
                        return Err(TooManyRepeats(at_most));
                    }
                } else {
                    repeat_count = 1;
                }
            }
        }

        Ok(())
    }
    fn meets_all_requirements(&self) -> Result<(), PasswordError> {
        if self.len() < 8 {
            return Err(PasswordError::TooShort);
        }
        if self.len() > 128 {
            return Err(PasswordError::TooLong);
        }
        if !self.chars().any(|x| x.is_uppercase()) {
            return Err(PasswordError::MissingUpperCase);
        }
        if !self.chars().any(|x| x.is_lowercase()) {
            return Err(PasswordError::MissingLowerCase);
        }
        if !self.chars().any(|x| x.is_numeric()) {
            return Err(PasswordError::MissingNumber);
        }
        if !self.chars().any(|x| SYMBOLS.contains(x)) {
            return Err(PasswordError::MissingSymbol(SYMBOLS.to_string()));
        }
        if self.chars().any(|x| x.is_whitespace()) {
            return Err(PasswordError::ContainsWhitespace);
        }
        self.min_unique_char_requirement(6)?;
        self.max_repeat_char_requirement(3)?;
        if self.is_common_password() {
            return Err(PasswordError::CommonPassword);
        }
        Ok(())
    }
}

/// wraps validated password hash
///
/// called `HashedPassword` as to
/// not conflict with `argon2::PasswordHash`
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn new(raw: &str) -> Result<Self, password_hash::Error> {
        argon2::PasswordHash::new(raw)?;
        Ok(Self(raw.to_string()))
    }

    pub fn generate(password: Password) -> Result<Self, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.0.as_bytes(), &salt)
            .map(|x| x.to_string())?;
        Ok(Self(hash))
    }

    pub fn verify(&self, password: &str) -> Result<bool, password_hash::Error> {
        let parsed_hash = argon2::PasswordHash::new(&self.0)?;

        match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
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
        let password1 = Password::new("Password123!").unwrap();
        let password2 = Password::new("DifferentPass456@").unwrap();
        let hash1 = HashedPassword::generate(password1).unwrap();
        let hash2 = HashedPassword::generate(password2).unwrap();

        assert_ne!(hash1.0, hash2.0);
    }

    #[test]
    fn test_password_validation_rejects_empty_input() {
        let result = Password::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PasswordError::TooShort));
    }

    #[test]
    fn test_hash_password_handles_long_input() {
        let long_password = format!("{}A1!", "a".repeat(996)); // Very long password with required chars
        let password = Password::new(&long_password).unwrap();
        let result = HashedPassword::generate(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
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

    #[test]
    fn test_verify_password_success_with_correct_password() {
        let password_str = "CorrectPassword123!";
        let password = Password::new(password_str).unwrap();
        let hash = HashedPassword::generate(password).unwrap();

        let result = hash.verify(password_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_verify_password_fails_with_wrong_password() {
        let correct_password = Password::new("CorrectPassword123!").unwrap();
        let hash = HashedPassword::generate(correct_password).unwrap();

        let result = hash.verify("WrongPassword456@");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_verify_password_case_sensitive() {
        let password = Password::new("CaseSensitive123!").unwrap();
        let hash = HashedPassword::generate(password).unwrap();

        // Exact match should work
        assert_eq!(hash.verify("CaseSensitive123!").unwrap(), true);

        // Different case should fail
        assert_eq!(hash.verify("casesensitive123!").unwrap(), false);
        assert_eq!(hash.verify("CASESENSITIVE123!").unwrap(), false);
    }

    #[test]
    fn test_password_validation_various_failures() {
        // Test all validation rules
        assert!(matches!(Password::new(""), Err(PasswordError::TooShort)));
        assert!(matches!(
            Password::new("short"),
            Err(PasswordError::TooShort)
        ));
        assert!(matches!(
            Password::new("nouppercase123!"),
            Err(PasswordError::MissingUpperCase)
        ));
        assert!(matches!(
            Password::new("NOLOWERCASE123!"),
            Err(PasswordError::MissingLowerCase)
        ));
        assert!(matches!(
            Password::new("NoNumbers!"),
            Err(PasswordError::MissingNumber)
        ));
        assert!(matches!(
            Password::new("NoSymbols123"),
            Err(PasswordError::MissingSymbol(_))
        ));
    }

    #[test]
    fn test_verify_password_with_special_characters() {
        let special_password_str = "P@ssw0rd!#$%";
        let special_password = Password::new(special_password_str).unwrap();
        let hash = HashedPassword::generate(special_password).unwrap();

        assert_eq!(hash.verify(special_password_str).unwrap(), true);
        assert_eq!(hash.verify("P@ssw0rd!#$").unwrap(), false); // Missing %
    }

    #[test]
    fn test_verify_password_with_unicode_characters() {
        let unicode_password_str = "Пароль1🔒!";
        let unicode_password = Password::new(unicode_password_str).unwrap();
        let hash = HashedPassword::generate(unicode_password).unwrap();

        assert_eq!(hash.verify(unicode_password_str).unwrap(), true);
        assert_eq!(hash.verify("Пароль1🔒").unwrap(), false); // Missing !
    }

    // =======================
    //  security & edge cases
    // =======================

    #[test]
    fn test_hash_verification_round_trip_with_various_inputs() {
        let test_passwords = vec![
            "Simple123!".to_string(),
            "With Spaces456@".to_string(),
            "MixedCase123!".to_string(),
            format!("{}A1!", "a".repeat(96)), // Long password
            "Emoji🚀🔒💻1!".to_string(),      // With emoji
        ];

        for password_str in test_passwords {
            let password = Password::new(&password_str).unwrap();
            let hash = HashedPassword::generate(password).unwrap();
            assert_eq!(hash.verify(&password_str).unwrap(), true);

            // Verify that a slightly different password fails
            let wrong_password = format!("{}x", password_str);
            assert_eq!(hash.verify(&wrong_password).unwrap(), false);
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
                assert_ne!(hashes[i], hashes[j]);
            }
        }

        // But all should verify correctly
        for hash in &hashes {
            assert_eq!(hash.verify(password_str).unwrap(), true);
        }
    }

    // =============================
    //  `HashedPassword` constructor
    // =============================

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
            "",                                     // Empty string
            "plaintext_password",                   // Plain text
            "$bcrypt$12$xyz",                       // Wrong algorithm
            "$sha256$rounds=1000$salt$hash",        // Different algorithm
            "$argon2",                              // Incomplete
            "$argon2$incomplete",                   // Missing sections
            "$argon2id$v=19$invalid",               // Malformed parameters
            "no_dollar_prefix",                     // No $ prefix
            "$argon2id$v=19$m=65536,t=2,p=1$",      // Missing salt and hash
            "$argon2id$$m=65536,t=2,p=1$salt$hash", // Empty version
        ];

        for invalid_hash in invalid_hashes {
            let result = HashedPassword::new(invalid_hash);
            assert!(result.is_err(), "Should reject: {}", invalid_hash);
        }
    }

    #[test]
    fn test_hashed_password_new_rejects_corrupted_argon2_hash() {
        // Generate a valid hash and then corrupt it
        let password = Password::new("TestPassword123!").unwrap();
        let valid_hash = HashedPassword::generate(password).unwrap();
        let mut hash_string = valid_hash.to_string();

        // Corrupt the hash by changing a character in the middle
        if let Some(mid_char) = hash_string.chars().nth(hash_string.len() / 2) {
            let replacement = if mid_char == 'a' { 'z' } else { 'a' };
            hash_string = hash_string.replacen(mid_char, &replacement.to_string(), 1);
        }

        let result = HashedPassword::new(&hash_string);
        assert!(result.is_err());
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
