//! Content moderation for user-generated text.
//!
//! This module provides profanity and inappropriate content filtering for usernames,
//! deck names, and other user-generated text fields.
//!
//! # Filtering Strategy
//!
//! Uses a two-tier approach to balance effectiveness with avoiding false positives:
//!
//! ## Substring Matching
//!
//! High-severity terms (slurs, abuse, explicit sexual content) are matched as substrings.
//! These words are banned anywhere in the input because they're unambiguous and severe.
//!
//! ## Exact Matching
//!
//! Common vulgarities are only matched as complete words to avoid false positives.
//! For example, "ass" is banned, but "grass" and "classic" are allowed.
//!
//! # Categories
//!
//! Banned content includes:
//! - **Slurs**: Racial, ethnic, sexual orientation, ableist slurs
//! - **Abuse Terms**: Child abuse, sexual assault, violence
//! - **Sexual Content**: Explicit sexual terms and body parts
//! - **Vulgar Insults**: Common swear words and insults
//!
//! # Usage
//!
//! The [`ContainsBadWord`] trait is implemented for `&str`, making it easy to
//! check any string:
//!
//! ```rust,ignore
//! use zwipe::domain::moderation::ContainsBadWord;
//!
//! if username.contains_bad_word() {
//!     return Err(ValidationError::Inappropriate);
//! }
//! ```
//!
//! # Applied To
//!
//! - Username validation (3-20 chars)
//! - Deck name validation (1-64 chars)
//! - Any other user-facing text fields
//!
//! # Performance
//!
//! Uses lazy-initialized HashSets for O(1) lookup performance.
//! Checking a username takes microseconds even with large ban lists.

use std::collections::HashSet;

use once_cell::sync::Lazy;

const SUBSTRING_BANNED: &[&str] = &[
    // slurs
    "nigger",
    "nigga",
    "faggot",
    "kike",
    "chink",
    "wetback",
    "beaner",
    // abuse
    "rapist",
    "pedophile",
    "molest",
    "molester",
    "molestation",
    "incest",
    "childmolester",
    "childabuse",
    "sexoffender",
    "sexualassault",
    "sexualabuse",
    // sexual
    "pussy",
    "cock",
    "clit",
    "dildo",
    "jizz",
    "fuck",
    "jackoff",
    "jerkoff",
    "milf",
    "masturbate",
    "masturbater",
    "masturbation",
    "masturbating",
    "cumdumpster",
    "cumslut",
    "fuckface",
    "motherfucker",
    "fucktard",
];

const EXACT_MATCH_BANNED: &[&str] = &[
    // common vulgarities (can appear in compound words like "grass" or "classic")
    "shit", "ass", "dick", "bitch", "asshole", "cunt", "slut", "whore", "cum",
    // body parts
    "tit", "tits", "titty", "titties", "tiddies", "boob", "boobs",
    // slurs (less severe than substring-banned)
    "fag", "retard", "dyke", "tard", "homo", "lesbo", // compound insults
    "shithead", "dickhead", "dumbass", "jackass",
];

/// Lazy-initialized HashSet of terms banned via substring matching.
///
/// These severe terms (slurs, abuse, explicit content) are matched anywhere
/// in the input string after lowercasing.
pub static SUBSTRING_BANNED_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| SUBSTRING_BANNED.iter().cloned().collect());

/// Lazy-initialized HashSet of terms banned via exact matching.
///
/// These common vulgarities are only matched as complete words to avoid
/// false positives (e.g., "ass" is banned, but "grass" is not).
pub static EXACT_MATCH_BANNED_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| EXACT_MATCH_BANNED.iter().cloned().collect());

/// Trait for checking if a string contains inappropriate content.
///
/// Implemented for `&str` to provide easy validation of user-generated text.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::moderation::ContainsBadWord;
///
/// // Check username
/// if "inappropriate".contains_bad_word() {
///     return Err(InvalidUsername::BadWord);
/// }
///
/// // Check deck name
/// if deck_name.contains_bad_word() {
///     return Err(InvalidDeckName::BadWord);
/// }
/// ```
pub trait ContainsBadWord {
    /// Returns true if the string contains banned words or phrases.
    ///
    /// Performs two checks:
    /// 1. Exact match: Lowercased string matches any exact-match banned word
    /// 2. Substring match: Lowercased string contains any substring-banned term
    ///
    /// The input is trimmed and lowercased before checking.
    fn contains_bad_word(&self) -> bool;
}

impl ContainsBadWord for &str {
    fn contains_bad_word(&self) -> bool {
        let s = self.trim().to_lowercase();
        EXACT_MATCH_BANNED_SET.contains(s.as_str())
            || SUBSTRING_BANNED_SET.iter().any(|w| s.contains(w))
    }
}
