//! Text shown to people.
//!
//! Error types and API messages stay lowercase, so they compose (`invalid
//! username: must be at least 3 characters long`) and read cleanly in logs.
//! Clients turn them into sentence case where they reach the screen.

/// `message` with its first character uppercased: `"username already in use"`
/// reads `"Username already in use"`. The rest is left as written.
#[must_use]
pub fn sentence_case(message: &str) -> String {
    let mut chars = message.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uppercases_only_the_first_character() {
        assert_eq!(
            sentence_case("username already in use"),
            "Username already in use"
        );
        assert_eq!(sentence_case("missing @ symbol"), "Missing @ symbol");
    }

    #[test]
    fn leaves_capitalized_and_empty_text_alone() {
        assert_eq!(sentence_case("Deck Not Found"), "Deck Not Found");
        assert_eq!(sentence_case(""), "");
        assert_eq!(sentence_case("@ is missing"), "@ is missing");
    }
}
