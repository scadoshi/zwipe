//! Wording helpers for text the clients show.

/// `text` with its first character uppercased, for an error whose
/// `Display` is lowercase (the Rust convention) shown where the UI's
/// messages read in sentence case: zwipe-core's own, which the server's
/// messages use too.
pub use zwipe_core::text::sentence_case;

#[cfg(test)]
mod tests {
    use super::sentence_case;

    #[test]
    fn the_first_letter_is_uppercased() {
        assert_eq!(
            sentence_case("deck name minimum length is 1 character"),
            "Deck name minimum length is 1 character"
        );
    }

    #[test]
    fn only_the_first_character_changes() {
        assert_eq!(
            sentence_case("email is not valid: use a@b.c"),
            "Email is not valid: use a@b.c"
        );
        assert_eq!(sentence_case("Already Capitalized"), "Already Capitalized");
    }

    #[test]
    fn an_empty_string_stays_empty() {
        assert_eq!(sentence_case(""), "");
    }

    #[test]
    fn a_non_ascii_first_letter_is_uppercased() {
        assert_eq!(sentence_case("élan"), "Élan");
        assert_eq!(sentence_case("ßa"), "SSa");
    }

    #[test]
    fn a_non_letter_first_character_is_left_alone() {
        assert_eq!(sentence_case("1 character minimum"), "1 character minimum");
    }
}
