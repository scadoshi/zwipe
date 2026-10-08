//! Password validation.
//!
//! This module implements the password security policy with validation
//! rules to prevent weak credentials.
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

use std::collections::HashSet;
use thiserror::Error;

// == errors ==

/// Password validation errors indicating which policy requirement failed.
///
/// Each variant corresponds to a specific password policy rule. The error messages
/// are user-friendly and can be displayed directly in API responses or UI.
#[derive(Debug, Clone, Error)]
pub enum InvalidPassword {
    /// Password is shorter than 8 characters.
    #[error("must be at least {MIN_LENGTH} characters long")]
    TooShort,

    /// Password exceeds 128 characters.
    #[error("must not exceed {MAX_LENGTH} characters")]
    TooLong,

    /// Password lacks uppercase letters (A-Z).
    #[error("must have at least one uppercase letter")]
    MissingUpperCase,

    /// Password lacks lowercase letters (a-z).
    #[error("must have at least one lowercase letter")]
    MissingLowerCase,

    /// Password lacks digits (0-9).
    #[error("must have at least one number")]
    MissingNumber,

    /// Password lacks required symbols.
    ///
    /// The error message includes the full list of acceptable symbols.
    #[error("must have at least one symbol from {0}")]
    MissingSymbol(String),

    /// Password contains whitespace (spaces, tabs, newlines).
    #[error("must not contain whitespace characters")]
    ContainsWhitespace,

    /// Password has too many consecutive repeated characters.
    ///
    /// Maximum allowed is 3 consecutive repeats (e.g., "aaa" is allowed, "aaaa" is not).
    #[error(transparent)]
    TooManyRepeats(#[from] TooManyRepeats),

    /// Password has insufficient character diversity.
    ///
    /// Minimum required is 6 unique characters.
    #[error(transparent)]
    TooFewUniqueChars(#[from] TooFewUniqueChars),
}

/// Error indicating too many consecutive repeated characters in the password.
///
/// The maximum allowed is 3 consecutive repeats. For example:
/// - "aaa" is valid (3 repeats)
/// - "aaaa" is invalid (4 repeats)
///
/// This prevents trivial passwords like "aaaaaaa!" or "111111!!".
#[derive(Debug, Clone, Error)]
#[error("must not contain more than {0} repeated characters")]
pub struct TooManyRepeats(u8);

/// Error indicating insufficient unique characters in the password.
///
/// The minimum required is 6 unique characters to ensure adequate entropy.
/// For example:
/// - "aAbBcC1!" has 8 unique chars (valid)
/// - "aaa111!!" has 3 unique chars (invalid)
///
/// This prevents trivial passwords constructed from repeated patterns.
#[derive(Debug, Clone, Error)]
#[error("must contain at least {0} unique characters")]
pub struct TooFewUniqueChars(u8);

/// Required symbols for password policy.
///
/// At least one of these characters must be present in every password.
pub const SYMBOLS: &str = r#"~!@#$%^&*()_+=[]{}\/?|:;<>,."#;

/// Fewest characters a password may have.
pub const MIN_LENGTH: usize = 8;

/// Most characters a password may have.
pub const MAX_LENGTH: usize = 128;

/// Fewest distinct characters a password may have.
pub const MIN_UNIQUE_CHARS: u8 = 6;

/// Most times one character may repeat in a row.
pub const MAX_REPEATS: u8 = 3;

/// The policy as lines a client can show before the user types, one per rule
/// [`validate`] checks, in the order it checks them. Built from the same
/// constants, so the list cannot drift from what is enforced.
pub fn requirements() -> Vec<String> {
    vec![
        format!("{MIN_LENGTH} to {MAX_LENGTH} characters"),
        "At least one uppercase letter".to_string(),
        "At least one lowercase letter".to_string(),
        "At least one number".to_string(),
        format!("At least one symbol from {SYMBOLS}"),
        "No spaces".to_string(),
        format!("At least {MIN_UNIQUE_CHARS} different characters"),
        format!("No character more than {MAX_REPEATS} times in a row"),
    ]
}

// == validation ==

/// Validates a password against the full password policy.
///
/// Returns `Ok(())` if all requirements are met, or an `Err(InvalidPassword)`
/// indicating the first requirement that failed.
///
/// # Example
///
/// ```
/// use zwipe_core::domain::auth::password::{validate, InvalidPassword};
///
/// assert!(validate("SecurePass123!").is_ok());
/// assert!(matches!(validate("weak"), Err(InvalidPassword::TooShort)));
/// ```
pub fn validate(password: &str) -> Result<(), InvalidPassword> {
    if password.len() < MIN_LENGTH {
        return Err(InvalidPassword::TooShort);
    }
    if password.len() > MAX_LENGTH {
        return Err(InvalidPassword::TooLong);
    }
    if !password.chars().any(|x| x.is_uppercase()) {
        return Err(InvalidPassword::MissingUpperCase);
    }
    if !password.chars().any(|x| x.is_lowercase()) {
        return Err(InvalidPassword::MissingLowerCase);
    }
    if !password.chars().any(|x| x.is_numeric()) {
        return Err(InvalidPassword::MissingNumber);
    }
    if !password.chars().any(|x| SYMBOLS.contains(x)) {
        return Err(InvalidPassword::MissingSymbol(SYMBOLS.to_string()));
    }
    if password.chars().any(|x| x.is_whitespace()) {
        return Err(InvalidPassword::ContainsWhitespace);
    }

    let unique_chars: HashSet<char> = password.chars().collect();
    if unique_chars.len() < usize::from(MIN_UNIQUE_CHARS) {
        return Err(TooFewUniqueChars(MIN_UNIQUE_CHARS).into());
    }

    let mut repeat_count: u8 = 1;
    let mut last_char_opt: Option<char> = None;
    for ch in password.chars() {
        if last_char_opt.is_none() {
            last_char_opt = Some(ch);
            continue;
        }
        if let Some(last_char) = last_char_opt {
            if ch == last_char {
                repeat_count += 1;
                if repeat_count > MAX_REPEATS {
                    return Err(TooManyRepeats(MAX_REPEATS).into());
                }
            } else {
                repeat_count = 1;
            }
            last_char_opt = Some(ch);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requirements_name_every_rule_validate_checks() {
        let lines = requirements();
        // One line per check in validate: length, upper, lower, number,
        // symbol, whitespace, unique, repeats.
        assert_eq!(lines.len(), 8);
        let all = lines.join("\n");
        assert!(all.contains(&format!("{MIN_LENGTH} to {MAX_LENGTH}")));
        assert!(all.contains(SYMBOLS));
        assert!(all.contains(&format!("{MIN_UNIQUE_CHARS} different")));
        assert!(all.contains(&format!("{MAX_REPEATS} times")));
    }

    #[test]
    fn requirements_match_the_enforced_bounds() {
        let base = "Abcdef1!";
        assert_eq!(base.len(), MIN_LENGTH);
        assert!(validate(base).is_ok());
        let short = "Abcde1!";
        assert_eq!(short.len(), MIN_LENGTH - 1);
        assert!(matches!(validate(short), Err(InvalidPassword::TooShort)));
        let pad: String = "wxyz"
            .chars()
            .cycle()
            .take(MAX_LENGTH - MIN_LENGTH)
            .collect();
        let max = format!("{base}{pad}");
        assert!(validate(&max).is_ok());
        assert!(matches!(
            validate(&format!("{max}y")),
            Err(InvalidPassword::TooLong)
        ));
        let repeats = format!("Ab1!{}", "z".repeat(usize::from(MAX_REPEATS)));
        assert!(validate(&format!("{repeats}xy")).is_ok());
        assert!(matches!(
            validate(&format!("{repeats}zxy")),
            Err(InvalidPassword::TooManyRepeats(_))
        ));
    }

    #[test]
    fn test_valid_password() {
        assert!(validate("SecurePass123!").is_ok());
    }

    #[test]
    fn test_too_short() {
        assert!(matches!(
            validate("Short1!"),
            Err(InvalidPassword::TooShort)
        ));
    }

    #[test]
    fn test_too_long() {
        let long = "A".repeat(129) + "a1!";
        assert!(matches!(validate(&long), Err(InvalidPassword::TooLong)));
    }

    #[test]
    fn test_missing_uppercase() {
        assert!(matches!(
            validate("nouppercase1!"),
            Err(InvalidPassword::MissingUpperCase)
        ));
    }

    #[test]
    fn test_missing_lowercase() {
        assert!(matches!(
            validate("NOLOWERCASE1!"),
            Err(InvalidPassword::MissingLowerCase)
        ));
    }

    #[test]
    fn test_missing_number() {
        assert!(matches!(
            validate("NoNumberHere!"),
            Err(InvalidPassword::MissingNumber)
        ));
    }

    #[test]
    fn test_missing_symbol() {
        assert!(matches!(
            validate("NoSymbol123"),
            Err(InvalidPassword::MissingSymbol(_))
        ));
    }

    #[test]
    fn test_contains_whitespace() {
        assert!(matches!(
            validate("Has Space1!"),
            Err(InvalidPassword::ContainsWhitespace)
        ));
    }

    #[test]
    fn test_too_few_unique_chars() {
        assert!(matches!(
            validate("aaaBBB1!"),
            Err(InvalidPassword::TooFewUniqueChars(_))
        ));
    }

    #[test]
    fn test_too_many_repeats() {
        assert!(matches!(
            validate("aaaaBcdef1!"),
            Err(InvalidPassword::TooManyRepeats(_))
        ));
    }
}
