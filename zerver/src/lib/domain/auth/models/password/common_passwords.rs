use once_cell::sync::Lazy; // this is my change
use std::collections::HashSet;

/// common passwords from various breach databases
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

/// uses above `&str` constant to get unique passwords
pub static COMMON_PASSWORDS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| COMMON_PASSWORDS_LIST.iter().cloned().collect());

/// for checking common passwords
pub trait IsCommonPassword {
    fn is_common_password(&self) -> bool;
}

/// checks if included in common password list case insensitively
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
