use argon2::{
    password_hash::{self, rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};

#[derive(Debug, Clone, PartialEq)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn new(password: &str) -> Result<Self, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================
    // Password Hashing Tests
    // ================================

    #[test]
    fn test_hash_password_success_creates_valid_hashes() {
        let result = HashedPassword::new("test_password");
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());

        // Argon2 hashes should start with $argon2 and have multiple $ delimited sections
        assert!(hash.0.starts_with("$argon2"));
        assert!(hash.0.matches('$').count() >= 4); // Format: $argon2$variant$params$salt$hash
    }

    #[test]
    fn test_hash_password_produces_unique_hashes_with_same_input() {
        let password = "identical_password";
        let hash1 = HashedPassword::new(password).unwrap();
        let hash2 = HashedPassword::new(password).unwrap();

        // Should be different due to unique salt generation
        assert_ne!(hash1.0, hash2.0);
    }

    #[test]
    fn test_hash_password_produces_different_hashes_for_different_inputs() {
        let hash1 = HashedPassword::new("password1").unwrap();
        let hash2 = HashedPassword::new("password2").unwrap();

        assert_ne!(hash1.0, hash2.0);
    }

    #[test]
    fn test_hash_password_handles_empty_input() {
        let result = HashedPassword::new("");
        assert!(result.is_ok()); // Empty passwords should be allowed (validation happens elsewhere)

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    #[test]
    fn test_hash_password_handles_long_input() {
        let long_password = "a".repeat(1000); // Very long password
        let result = HashedPassword::new(&long_password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    #[test]
    fn test_hash_password_handles_special_characters() {
        let special_password = "p@ssw0rd!#$%^&*(){}[]|\\:;\"'<>,.?/~`";
        let result = HashedPassword::new(special_password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    #[test]
    fn test_hash_password_handles_unicode_characters() {
        let unicode_password = "пароль🔒日本語";
        let result = HashedPassword::new(unicode_password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.0.is_empty());
    }

    // ================================
    // Password Verification Tests
    // ================================

    #[test]
    fn test_verify_password_success_with_correct_password() {
        let password = "correct_password";
        let hash = HashedPassword::new(password).unwrap();

        let result = hash.verify(password);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_verify_password_fails_with_wrong_password() {
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = HashedPassword::new(correct_password).unwrap();

        let result = hash.verify(wrong_password);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_verify_password_case_sensitive() {
        let password = "CaseSensitive";
        let hash = HashedPassword::new(password).unwrap();

        // Exact match should work
        assert_eq!(hash.verify("CaseSensitive").unwrap(), true);

        // Different case should fail
        assert_eq!(hash.verify("casesensitive").unwrap(), false);
        assert_eq!(hash.verify("CASESENSITIVE").unwrap(), false);
    }

    #[test]
    fn test_verify_password_handles_empty_password_with_valid_hash() {
        let empty_password = "";
        let hash = HashedPassword::new(empty_password).unwrap();

        // Should successfully verify empty password
        assert_eq!(hash.verify("").unwrap(), true);

        // Non-empty password should fail against empty password hash
        assert_eq!(hash.verify("non_empty").unwrap(), false);
    }

    #[test]
    fn test_verify_password_with_special_characters() {
        let special_password = "p@ssw0rd!#$%";
        let hash = HashedPassword::new(special_password).unwrap();

        assert_eq!(hash.verify(special_password).unwrap(), true);
        assert_eq!(hash.verify("p@ssw0rd!#$").unwrap(), false); // Missing %
    }

    #[test]
    fn test_verify_password_with_unicode_characters() {
        let unicode_password = "пароль🔒";
        let hash = HashedPassword::new(unicode_password).unwrap();

        assert_eq!(hash.verify(unicode_password).unwrap(), true);
        assert_eq!(hash.verify("пароль").unwrap(), false); // Missing emoji
    }

    // ================================
    // Security & Edge Case Tests
    // ================================

    #[test]
    fn test_hash_verification_round_trip_with_various_inputs() {
        let test_passwords = vec![
            "simple".to_string(),
            "with spaces".to_string(),
            "123456789".to_string(),
            "MixedCase123!".to_string(),
            "".to_string(),
            "a".repeat(100),      // Long password
            "🚀🔒💻".to_string(), // Emoji only
        ];

        for password in test_passwords {
            let hash = HashedPassword::new(&password).unwrap();
            assert_eq!(hash.verify(&password).unwrap(), true);

            // Verify that a slightly different password fails
            let wrong_password = format!("{}x", password);
            assert_eq!(hash.verify(wrong_password.as_str()).unwrap(), false);
        }
    }

    #[test]
    fn test_salt_uniqueness_across_multiple_hashes() {
        let password = "same_password";
        let mut hashes = Vec::new();

        // Generate multiple hashes of the same password
        for _ in 0..10 {
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
            assert_eq!(hash.verify(password).unwrap(), true);
        }
    }

    // ================================
    // Integration Tests
    // ================================

    #[test]
    fn test_complete_password_authentication_flow() {
        // Simulate complete user registration and login flow
        let user_password = "user_chosen_password_123!";

        // Registration: hash the password
        let stored_hash = HashedPassword::new(user_password).unwrap();

        // Later login attempt: verify the password
        let login_result = stored_hash.verify(user_password).unwrap();
        assert!(login_result);

        // Invalid login attempt: wrong password
        let wrong_login_result = stored_hash.verify("wrong_password").unwrap();
        assert!(!wrong_login_result);

        // Verify hash format is suitable for database storage
        assert!(stored_hash.0.len() > 50); // Reasonable length for storage
        assert!(stored_hash.0.is_ascii()); // Safe for database storage
    }
}
