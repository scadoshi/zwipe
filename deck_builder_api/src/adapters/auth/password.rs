use argon2::{
    password_hash::{rand_core::OsRng, Error, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

pub fn hash_password(password: &str) -> Result<String, Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|x| x.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(Error::Password) => Ok(false),
        Err(e) => Err(e),
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
        let result = hash_password("test_password");
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.is_empty());

        // Argon2 hashes should start with $argon2 and have multiple $ delimited sections
        assert!(hash.starts_with("$argon2"));
        assert!(hash.matches('$').count() >= 4); // Format: $argon2$variant$params$salt$hash
    }

    #[test]
    fn test_hash_password_produces_unique_hashes_with_same_input() {
        let password = "identical_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Should be different due to unique salt generation
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_produces_different_hashes_for_different_inputs() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password2").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_handles_empty_input() {
        let result = hash_password("");
        assert!(result.is_ok()); // Empty passwords should be allowed (validation happens elsewhere)

        let hash = result.unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_hash_password_handles_long_input() {
        let long_password = "a".repeat(1000); // Very long password
        let result = hash_password(&long_password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_hash_password_handles_special_characters() {
        let special_password = "p@ssw0rd!#$%^&*(){}[]|\\:;\"'<>,.?/~`";
        let result = hash_password(special_password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_hash_password_handles_unicode_characters() {
        let unicode_password = "пароль🔒日本語";
        let result = hash_password(unicode_password);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.is_empty());
    }

    // ================================
    // Password Verification Tests
    // ================================

    #[test]
    fn test_verify_password_success_with_correct_password() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_verify_password_fails_with_wrong_password() {
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(correct_password).unwrap();

        let result = verify_password(wrong_password, &hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_verify_password_case_sensitive() {
        let password = "CaseSensitive";
        let hash = hash_password(password).unwrap();

        // Exact match should work
        assert_eq!(verify_password("CaseSensitive", &hash).unwrap(), true);

        // Different case should fail
        assert_eq!(verify_password("casesensitive", &hash).unwrap(), false);
        assert_eq!(verify_password("CASESENSITIVE", &hash).unwrap(), false);
    }

    #[test]
    fn test_verify_password_rejects_malformed_hash() {
        let password = "test_password";

        // Invalid hash formats should return errors
        assert!(verify_password(password, "not_a_hash").is_err());
        assert!(verify_password(password, "").is_err());
        assert!(verify_password(password, "$invalid$hash$format").is_err());
    }

    #[test]
    fn test_verify_password_handles_empty_password_with_valid_hash() {
        let empty_password = "";
        let hash = hash_password(empty_password).unwrap();

        // Should successfully verify empty password
        assert_eq!(verify_password("", &hash).unwrap(), true);

        // Non-empty password should fail against empty password hash
        assert_eq!(verify_password("non_empty", &hash).unwrap(), false);
    }

    #[test]
    fn test_verify_password_with_special_characters() {
        let special_password = "p@ssw0rd!#$%";
        let hash = hash_password(special_password).unwrap();

        assert_eq!(verify_password(special_password, &hash).unwrap(), true);
        assert_eq!(verify_password("p@ssw0rd!#$", &hash).unwrap(), false); // Missing %
    }

    #[test]
    fn test_verify_password_with_unicode_characters() {
        let unicode_password = "пароль🔒";
        let hash = hash_password(unicode_password).unwrap();

        assert_eq!(verify_password(unicode_password, &hash).unwrap(), true);
        assert_eq!(verify_password("пароль", &hash).unwrap(), false); // Missing emoji
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
            let hash = hash_password(&password).unwrap();
            assert_eq!(verify_password(&password, &hash).unwrap(), true);

            // Verify that a slightly different password fails
            let wrong_password = format!("{}x", password);
            assert_eq!(
                verify_password(wrong_password.as_str(), &hash).unwrap(),
                false
            );
        }
    }

    #[test]
    fn test_salt_uniqueness_across_multiple_hashes() {
        let password = "same_password";
        let mut hashes = Vec::new();

        // Generate multiple hashes of the same password
        for _ in 0..10 {
            hashes.push(hash_password(password).unwrap());
        }

        // All hashes should be unique (due to unique salts)
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j]);
            }
        }

        // But all should verify correctly
        for hash in &hashes {
            assert_eq!(verify_password(password, hash).unwrap(), true);
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
        let stored_hash = hash_password(user_password).unwrap();

        // Later login attempt: verify the password
        let login_result = verify_password(user_password, &stored_hash).unwrap();
        assert!(login_result);

        // Invalid login attempt: wrong password
        let wrong_login_result = verify_password("wrong_password", &stored_hash).unwrap();
        assert!(!wrong_login_result);

        // Verify hash format is suitable for database storage
        assert!(stored_hash.len() > 50); // Reasonable length for storage
        assert!(stored_hash.is_ascii()); // Safe for database storage
    }
}
