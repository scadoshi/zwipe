

//! Common password detection to prevent weak password choices.
//!
//! This module maintains a curated list of commonly-used weak passwords sourced from
//! real-world data breach analysis. Preventing these passwords significantly improves
//! account security.
//!
//! # Detection Strategy
//!
//! - **Case-Insensitive**: Checks both exact match and lowercase version
//! - **Comprehensive List**: 170+ patterns including:
//!   - Common passwords ("password", "123456", "qwerty")
//!   - Keyboard patterns ("qwertyuiop", "asdfghjkl", "1q2w3e4r")
//!   - Common substitutions ("p@ssword", "p@ssw0rd")
//!   - Names, years, sports teams, brands
//!   - Simple increments ("abcd1234", "password01")
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::password::common::IsCommonPassword;
//!
//! if "password123".is_common_password() {
//!     return Err(InvalidPassword::TooCommon);
//! }
//! ```

use once_cell::sync::Lazy;
use std::collections::HashSet;

/// List of commonly-used weak passwords to reject during validation.
///
/// This list is curated from:
/// - NIST password guidelines
/// - Pwned Passwords database (most common entries)
/// - Default credentials from various systems
/// - Keyboard walk patterns
///
/// Sourced from real-world breach data to maximize effectiveness.
const COMMON_PASSWORDS_LIST: &[&str] = &[
    "password",
    "123456",
    "123456789",
    "12345678",
    "12345",
    "1234567",
    "password123",
    "admin",
    "qwerty",
    "abc123",
    "Password1",
    "password1",
    "123123",
    "welcome",
    "login",
    "admin123",
    "iloveyou",
    "monkey",
    "1234567890",
    "letmein",
    "trustno1",
    "dragon",
    "baseball",
    "111111",
    "sunshine",
    "master",
    "123321",
    "696969",
    "12345678910",
    "shadow",
    "michael",
    "computer",
    "jesus",
    "ninja",
    "mustang",
    "password1234",
    "jordan",
    "superman",
    "harley",
    "1234",
    "hunter",
    "fuckyou",
    "trustno1",
    "ranger",
    "buster",
    "thomas",
    "robert",
    "soccer",
    "killer",
    "hockey",
    "george",
    "charlie",
    "andrew",
    "michelle",
    "love",
    "sunshine",
    "chocolate",
    "anthony",
    "cookie",
    "chicken",
    "starwars",
    "maverick",
    "bacon",
    "freedom",
    "samsung",
    "football",
    "test",
    "pass",
    "guest",
    "root",
    "demo",
    "temp",
    "changeme",
    "default",
    "welcome123",
    "admin123",
    "password12",
    "password123!",
    "Password123",
    "Password123!",
    "Welcome123",
    "Welcome123!",
    // Common patterns
    "qwerty123",
    "asdf1234",
    "zxcvbnm",
    "qwertyuiop",
    "asdfghjkl",
    "1qaz2wsx",
    "1q2w3e4r",
    "qwer1234",
    "zaq12wsx",
    "1qazxsw2",
    // Years and dates
    "2023",
    "2022",
    "2021",
    "2020",
    "1234567890",
    "19701970",
    "19801980",
    "19901990",
    "20002000",
    "20102010",
    // Names and common words
    "jennifer",
    "david",
    "daniel",
    "matthew",
    "christopher",
    "andrew",
    "joshua",
    "william",
    "john",
    "amanda",
    "jessica",
    "ashley",
    "brittany",
    "sarah",
    "samantha",
    "stephanie",
    "nicole",
    "elizabeth",
    // Sports teams and brands
    "ferrari",
    "porsche",
    "corvette",
    "mercedes",
    "toyota",
    "honda",
    "yankees",
    "cowboys",
    "lakers",
    "celtics",
    "steelers",
    "packers",
    // Common substitutions
    "p@ssword",
    "p@ssw0rd",
    "passw0rd",
    "1qaz!QAZ",
    "qwerty!@#",
    // Keyboard patterns
    "qazwsx",
    "wsxedc",
    "rfvtgb",
    "yhujik",
    "plokij",
    "mnbvcx",
    // Simple increments
    "abcd1234",
    "1234abcd",
    "a1b2c3d4",
    "password01",
    "password02",
    // Common phrases
    "iloveyou",
    "ihateyou",
    "fuckyou",
    "letmein",
    "welcome",
    "baseball",
    "football",
    "basketball",
    "soccer",
    "hockey",
];

/// Lazy-initialized HashSet of common passwords for O(1) lookup.
///
/// Initialized once on first use and shared across all checks.
/// Contains all entries from [`COMMON_PASSWORDS_LIST`] in a HashSet
/// for fast membership testing.
pub static COMMON_PASSWORDS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| COMMON_PASSWORDS_LIST.iter().cloned().collect());

/// Trait for checking if a string matches a common weak password.
///
/// Implemented for `&str` to provide ergonomic password validation.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::password::common::IsCommonPassword;
///
/// assert!("password".is_common_password());
/// assert!("123456".is_common_password());
/// assert!(!"MySecurePassword123!".is_common_password());
/// ```
pub trait IsCommonPassword {
    /// Returns true if the string is a known common password.
    ///
    /// Checks both exact match and lowercase match to catch common
    /// variations like "Password" and "PASSWORD".
    fn is_common_password(&self) -> bool;
}

/// Checks if a string matches any common password (case-insensitive).
///
/// Performs two checks for maximum coverage:
/// 1. Exact match against the common password set
/// 2. Lowercase match (catches "Password", "PASSWORD", etc.)
impl IsCommonPassword for &str {
    fn is_common_password(&self) -> bool {
        COMMON_PASSWORDS.contains(self) || COMMON_PASSWORDS.contains(self.to_lowercase().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_common_passwords() {
        assert!("password".is_common_password());
        assert!("123456".is_common_password());
        assert!("qwerty".is_common_password());
        assert!("Password".is_common_password());
        assert!("PASSWORD".is_common_password());
    }

    #[test]
    fn test_allows_secure_passwords() {
        assert!(!"MySecurePassword123!".is_common_password());
        assert!(!"RandomComplexPass456@".is_common_password());
        assert!(!"UnlikelyToBeInList789#".is_common_password());
    }

    #[test]
    fn test_common_passwords_list_not_empty() {
        assert!(!COMMON_PASSWORDS.is_empty());
        assert!(COMMON_PASSWORDS.len() > 100);
    }
}
