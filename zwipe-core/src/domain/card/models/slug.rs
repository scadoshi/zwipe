//! A card's URL slug: its whole name folded the way a deck's slug is, with no length cap, so
//! `Lim-Dûl's Vault` is `lim-dul-s-vault` and `Delver of Secrets // Insectile Aberration` is
//! `delver-of-secrets-insectile-aberration`. A card with more than one face also answers to its
//! front face's slug ([`front_slug`]), `delver-of-secrets`.

use std::fmt;
use std::str::FromStr;

use crate::domain::slug;

/// What sits between the faces in a multi-faced card's name, as Scryfall writes it.
const FACE_SEPARATOR: &str = " // ";

/// A card's URL slug: one or more runs of ASCII lower-case letters and digits joined by single
/// dashes, `[a-z0-9]+(-[a-z0-9]+)*`. [`card_slug`] makes one from a name; [`FromStr`] checks one
/// read from a URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardSlug(String);

impl CardSlug {
    /// The slug as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CardSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for CardSlug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<CardSlug> for String {
    fn from(slug: CardSlug) -> Self {
        slug.0
    }
}

/// Text that is not a card slug: empty, or holding something besides lower-case ASCII letters,
/// digits and single inner dashes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid card slug {0:?}")]
pub struct InvalidCardSlug(pub String);

impl FromStr for CardSlug {
    type Err = InvalidCardSlug;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let runs_ok = s.split('-').all(|run| {
            !run.is_empty()
                && run
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        });
        if runs_ok {
            Ok(Self(s.to_owned()))
        } else {
            Err(InvalidCardSlug(s.to_owned()))
        }
    }
}

impl TryFrom<String> for CardSlug {
    type Error = InvalidCardSlug;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// The slug of the card named `name`, or `None` when the name has no letter or digit to keep
/// (`_____`).
#[must_use]
pub fn card_slug(name: &str) -> Option<CardSlug> {
    let slug = slug::slugify(name);
    (!slug.is_empty()).then_some(CardSlug(slug))
}

/// The slug of the front face of the card named `name`, for a name with more than one face
/// (`Fire // Ice` answers to `fire`); `None` for a one-faced card, or when the front folds to
/// nothing.
#[must_use]
pub fn front_slug(name: &str) -> Option<CardSlug> {
    let (front, _) = name.split_once(FACE_SEPARATOR)?;
    card_slug(front)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slug(name: &str) -> String {
        card_slug(name).map_or_else(String::new, String::from)
    }

    #[test]
    fn a_card_name_folds_like_a_deck_name() {
        for (name, want) in [
            ("Lim-Dûl's Vault", "lim-dul-s-vault"),
            ("Æther Vial", "aether-vial"),
            ("Circle of Protection: Red", "circle-of-protection-red"),
            ("Borrowing 100,000 Arrows", "borrowing-100-000-arrows"),
            ("Fire // Ice", "fire-ice"),
            (
                "Delver of Secrets // Insectile Aberration",
                "delver-of-secrets-insectile-aberration",
            ),
        ] {
            assert_eq!(slug(name), want, "{name}");
        }
    }

    #[test]
    fn a_card_slug_keeps_the_whole_name() {
        let name = "Our Market Research Shows That Players Like Really Long Card Names So We Made This Card to Have the Absolute Longest Card Name Ever Elemental";
        let got = slug(name);
        assert!(got.len() > 100, "{got}");
        assert!(got.ends_with("ever-elemental"), "{got}");
    }

    #[test]
    fn a_name_with_nothing_to_keep_has_no_slug() {
        assert_eq!(card_slug("_____"), None);
        assert_eq!(card_slug(""), None);
    }

    #[test]
    fn only_a_multi_faced_name_has_a_front_slug() {
        assert_eq!(
            front_slug("Delver of Secrets // Insectile Aberration").map(String::from),
            Some("delver-of-secrets".to_owned())
        );
        assert_eq!(
            front_slug("Bind // Liberate").map(String::from),
            Some("bind".to_owned())
        );
        assert_eq!(front_slug("Bind"), None);
        assert_eq!(front_slug("_____ // Thing"), None);
    }

    #[test]
    fn a_slug_parses_only_in_its_own_shape() {
        for ok in ["bind", "fire-ice", "borrowing-100-000-arrows", "a1-2b"] {
            assert_eq!(ok.parse::<CardSlug>().map(String::from), Ok(ok.to_owned()));
        }
        for bad in [
            "",
            "-bind",
            "bind-",
            "fire--ice",
            "Bind",
            "lim-dûl",
            "a b",
            "a/b",
            "a_b",
        ] {
            assert!(bad.parse::<CardSlug>().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn every_made_slug_parses() {
        for name in [
            "Lim-Dûl's Vault",
            "Straße Æther",
            "  --Krenko's   Goblins!!-- ",
            "忍者 Ninja",
        ] {
            let made = card_slug(name).unwrap();
            assert_eq!(
                made.as_str().parse::<CardSlug>(),
                Ok(made.clone()),
                "{name}"
            );
        }
    }
}
