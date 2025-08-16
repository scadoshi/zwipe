mod common_passwords;
use common_passwords::IsCommonPassword;

use std::collections::HashSet;

use thiserror::Error;

use argon2::{
    password_hash::{self, rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};

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

#[derive(Debug, Clone, Error)]
#[error("Password must not contain more than {} repeated characters", 0)]
pub struct TooManyRepeats(u8);

impl From<TooManyRepeats> for PasswordError {
    fn from(value: TooManyRepeats) -> Self {
        PasswordError::TooManyRepeats(value)
    }
}

#[derive(Debug, Clone, Error)]
#[error("Password must contain at least {} unique characters", 0)]
pub struct TooFewUniqueChars(u8);

impl From<TooFewUniqueChars> for PasswordError {
    fn from(value: TooFewUniqueChars) -> Self {
        PasswordError::TooFewUniqueChars(value)
    }
}

const SYMBOLS: &str = r#"~!@#$%^&*()_+=[]{}\/?|:;<>,."#;

#[derive(Debug, Clone)]
pub struct Password(String);

impl Password {
    pub fn new(raw: &str) -> Result<Self, PasswordError> {
        raw.meets_all_requirements()?;
        Ok(Password(raw.to_string()))
    }
}

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

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn new(password: Password) -> Result<Self, password_hash::Error> {
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

    // ================================
    // Password Hashing Tests
    // ================================

    #[test]
    fn test_hash_password_success_creates_valid_hashes() {
        let password = Password::new("TestPassword123!").unwrap();
        let result = HashedPassword::new(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());

        // Argon2 hashes should start with $argon2 and have multiple $ delimited sections
        assert!(hash.0.starts_with("$argon2"));
        assert!(hash.0.matches('$').count() >= 4); // Format: $argon2$variant$params$salt$hash
    }

    #[test]
    fn test_hash_password_produces_unique_hashes_with_same_input() {
        let password = Password::new("IdenticalPassword123!").unwrap();
        let hash1 = HashedPassword::new(password.clone()).unwrap();
        let hash2 = HashedPassword::new(password).unwrap();

        // Should be different due to unique salt generation
        assert_ne!(hash1.0, hash2.0);
    }

    #[test]
    fn test_hash_password_produces_different_hashes_for_different_inputs() {
        let password1 = Password::new("Password123!").unwrap();
        let password2 = Password::new("DifferentPass456@").unwrap();
        let hash1 = HashedPassword::new(password1).unwrap();
        let hash2 = HashedPassword::new(password2).unwrap();

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
        let result = HashedPassword::new(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    #[test]
    fn test_hash_password_handles_special_characters() {
        let special_password = "P@ssw0rd!#$%^&*(){}[]|\\:;\"'<>,.?/~`";
        let password = Password::new(special_password).unwrap();
        let result = HashedPassword::new(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    #[test]
    fn test_hash_password_handles_unicode_characters() {
        let unicode_password = "Пароль1🔒!";
        let password = Password::new(unicode_password).unwrap();
        let result = HashedPassword::new(password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    // ================================
    // Password Verification Tests
    // ================================

    #[test]
    fn test_verify_password_success_with_correct_password() {
        let password_str = "CorrectPassword123!";
        let password = Password::new(password_str).unwrap();
        let hash = HashedPassword::new(password).unwrap();

        let result = hash.verify(password_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_verify_password_fails_with_wrong_password() {
        let correct_password = Password::new("CorrectPassword123!").unwrap();
        let hash = HashedPassword::new(correct_password).unwrap();

        let result = hash.verify("WrongPassword456@");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_verify_password_case_sensitive() {
        let password = Password::new("CaseSensitive123!").unwrap();
        let hash = HashedPassword::new(password).unwrap();

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
        let hash = HashedPassword::new(special_password).unwrap();

        assert_eq!(hash.verify(special_password_str).unwrap(), true);
        assert_eq!(hash.verify("P@ssw0rd!#$").unwrap(), false); // Missing %
    }

    #[test]
    fn test_verify_password_with_unicode_characters() {
        let unicode_password_str = "Пароль1🔒!";
        let unicode_password = Password::new(unicode_password_str).unwrap();
        let hash = HashedPassword::new(unicode_password).unwrap();

        assert_eq!(hash.verify(unicode_password_str).unwrap(), true);
        assert_eq!(hash.verify("Пароль1🔒").unwrap(), false); // Missing !
    }

    // ================================
    // Security & Edge Case Tests
    // ================================

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
            let hash = HashedPassword::new(password).unwrap();
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
            hashes.push(HashedPassword::new(password).unwrap());
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

    // ================================
    // Integration Tests
    // ================================

    #[test]
    fn test_complete_password_authentication_flow() {
        // Simulate complete user registration and login flow
        let user_password_str = "UserChosenPassword123!";

        // Registration: validate and hash the password
        let password = Password::new(user_password_str).unwrap();
        let stored_hash = HashedPassword::new(password).unwrap();

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
}
