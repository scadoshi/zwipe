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

/// [`sentence_case`] for an owned message, in place when its first letter
/// is ASCII.
#[must_use]
pub fn into_sentence_case(mut message: String) -> String {
    match message.chars().next() {
        Some(first) if first.is_ascii() => {
            if let Some(head) = message.get_mut(..1) {
                head.make_ascii_uppercase();
            }
            message
        }
        Some(_) => sentence_case(&message),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_owned_form_matches_the_borrowed_one() {
        for message in ["", "username taken", "Already", "élan vital", "1 card"] {
            assert_eq!(
                into_sentence_case(message.to_owned()),
                sentence_case(message),
                "{message:?}"
            );
        }
    }

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
