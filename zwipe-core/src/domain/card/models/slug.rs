//! A card's URL slug: its whole name folded the way a deck's slug is, with no length cap, so
//! `Lim-Dûl's Vault` is `lim-dul-s-vault` and `Delver of Secrets // Insectile Aberration` is
//! `delver-of-secrets-insectile-aberration`. A card with more than one face also answers to its
//! front face's slug ([`front_slug`]), `delver-of-secrets`.

use std::fmt;
use std::str::FromStr;

use uuid::Uuid;

use crate::domain::site::APP_BASE;
use crate::domain::slug;

/// The path segment a printing link carries for a card whose name folds to nothing. The segment
/// is cosmetic there: the link loads by the printing's id.
pub const PRINTING_SLUG_FALLBACK: &str = "card";

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

/// The web app's path to the card `slug` names: `/card/<slug>`.
#[must_use]
pub fn card_path(slug: &CardSlug) -> String {
    format!("/card/{slug}")
}

/// The web app's path to one printing, `id`, of the card named `name`: `/card/<slug>/<id>`, with
/// [`PRINTING_SLUG_FALLBACK`] as the slug when the name folds to nothing.
#[must_use]
pub fn printing_path(name: &str, id: Uuid) -> String {
    let slug = card_slug(name);
    let segment = slug
        .as_ref()
        .map_or(PRINTING_SLUG_FALLBACK, CardSlug::as_str);
    format!("/card/{segment}/{id}")
}

/// The full link to [`card_path`]: [`APP_BASE`], the web app, then the path.
#[must_use]
pub fn card_url(slug: &CardSlug) -> String {
    format!("{APP_BASE}{}", card_path(slug))
}

/// The full link to [`printing_path`]: [`APP_BASE`], the web app, then the path.
#[must_use]
pub fn printing_url(name: &str, id: Uuid) -> String {
    format!("{APP_BASE}{}", printing_path(name, id))
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
    fn card_links_have_the_card_and_printing_shapes() {
        let fire: CardSlug = "fire-ice".parse().unwrap();
        assert_eq!(card_path(&fire), "/card/fire-ice");
        assert_eq!(
            printing_path("Fire // Ice", Uuid::nil()),
            "/card/fire-ice/00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            printing_path("_____", Uuid::nil()),
            "/card/card/00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(card_url(&fire), format!("{APP_BASE}/card/fire-ice"));
        assert!(printing_url("Fire // Ice", Uuid::nil()).starts_with(APP_BASE));
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
