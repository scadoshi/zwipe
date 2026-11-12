//! A shared deck's public link: `app.zwipe.net/<owner>/deck/<slug>/<token>`, GitHub style.
//!
//! Only the share token identifies the deck. The owner's username and the slug of the deck's
//! name are cosmetic: the web app loads the deck by its token and swaps a stale owner or slug for
//! the current ones. zwipe.net answers the same path with a redirect here, for links a phone
//! built before the move still makes. The four-segment `/<owner>/deck/<slug>/<token>` shape is the only shared-deck
//! form, so it never collides with the site's own one- and two-segment pages.

use uuid::Uuid;

use crate::domain::site::APP_BASE;

/// The longest a slug runs, in characters, before it is cut at a dash where it can be.
pub const SLUG_MAX: usize = 60;

/// The slug a deck with no letters or digits in its name gets.
pub const SLUG_FALLBACK: &str = "deck";

/// The URL slug for a deck called `name`: lower case, common accented Latin letters folded to
/// ASCII, every other run of characters one dash, no dash at either end, at most [`SLUG_MAX`]
/// characters, and [`SLUG_FALLBACK`] when nothing is left.
#[must_use]
pub fn deck_slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    // A dash waits until the next kept character, so runs collapse and the
    // ends stay clean.
    let mut dash = false;
    for c in name.chars().flat_map(char::to_lowercase) {
        let folded = fold(c);
        if folded.is_empty() {
            dash = !slug.is_empty();
            continue;
        }
        if dash {
            slug.push('-');
            dash = false;
        }
        slug.push_str(folded);
    }
    let cut = cap(&slug);
    if cut.is_empty() {
        SLUG_FALLBACK.to_owned()
    } else {
        cut.to_owned()
    }
}

/// `slug` cut to [`SLUG_MAX`] characters, at its last dash inside the cap when there is one,
/// with no dash left at the end. A slug is ASCII, so a byte offset is a character offset.
fn cap(slug: &str) -> &str {
    if slug.len() <= SLUG_MAX {
        return slug;
    }
    let head = slug.get(..SLUG_MAX).unwrap_or(slug);
    let at_word = head
        .rfind('-')
        .map_or(head, |dash| head.get(..dash).unwrap_or(head));
    at_word.trim_end_matches('-')
}

/// What `c`, already lower case, adds to a slug: itself for an ASCII letter or digit, the ASCII
/// letters of a common accented Latin letter, and nothing for anything else (a separator).
fn fold(c: char) -> &'static str {
    const ASCII: &str = "abcdefghijklmnopqrstuvwxyz0123456789";
    if c.is_ascii_alphanumeric() {
        let at = ASCII.find(c).unwrap_or(0);
        return ASCII.get(at..at.saturating_add(1)).unwrap_or("");
    }
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'ç' | 'ć' | 'č' => "c",
        'ď' | 'đ' | 'ð' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => "e",
        'ğ' => "g",
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => "i",
        'ł' | 'ľ' => "l",
        'ñ' | 'ń' | 'ň' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => "o",
        'ř' => "r",
        'ś' | 'š' | 'ş' => "s",
        'ť' | 'ţ' => "t",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' => "u",
        'ý' | 'ÿ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        'þ' => "th",
        _ => "",
    }
}

/// `segment` percent-encoded for one URL path segment: ASCII letters, digits and `-._~` stay,
/// every other byte of its UTF-8 becomes `%XX`.
#[must_use]
pub fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push('%');
            for nibble in [byte >> 4, byte & 0x0f] {
                let digit = char::from_digit(u32::from(nibble), 16).unwrap_or('0');
                out.push(digit.to_ascii_uppercase());
            }
        }
    }
    out
}

/// The path of the deck `owner` shares as `deck_name` under `token`:
/// `/<owner>/deck/<slug>/<token>`.
#[must_use]
pub fn shared_deck_path(owner: &str, deck_name: &str, token: Uuid) -> String {
    format!(
        "/{}/deck/{}/{token}",
        encode_segment(owner),
        deck_slug(deck_name)
    )
}

/// The full public link: [`APP_BASE`], the web app, then [`shared_deck_path`].
#[must_use]
pub fn shared_deck_url(owner: &str, deck_name: &str, token: Uuid) -> String {
    format!("{APP_BASE}{}", shared_deck_path(owner, deck_name, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_becomes_lower_kebab_case() {
        assert_eq!(deck_slug("Elves Go Wide"), "elves-go-wide");
        assert_eq!(deck_slug("Atraxa, Grand Unifier"), "atraxa-grand-unifier");
        assert_eq!(deck_slug("  --Krenko's   Goblins!!-- "), "krenko-s-goblins");
        assert_eq!(deck_slug("Test"), "test");
        assert_eq!(deck_slug("5c Superfriends v2"), "5c-superfriends-v2");
    }

    #[test]
    fn accented_letters_fold_to_ascii() {
        assert_eq!(deck_slug("Éowyn's Señor Café"), "eowyn-s-senor-cafe");
        assert_eq!(deck_slug("Straße Æther Œuvre"), "strasse-aether-oeuvre");
        assert_eq!(deck_slug("Jötun Grunt Ñ"), "jotun-grunt-n");
    }

    #[test]
    fn letters_with_no_ascii_form_become_dashes() {
        assert_eq!(deck_slug("Ninja 忍者 Deck"), "ninja-deck");
        assert_eq!(deck_slug("🔥 Burn 🔥"), "burn");
    }

    #[test]
    fn a_name_with_nothing_to_keep_falls_back() {
        assert_eq!(deck_slug(""), SLUG_FALLBACK);
        assert_eq!(deck_slug("!!!"), SLUG_FALLBACK);
        assert_eq!(deck_slug("忍者"), SLUG_FALLBACK);
    }

    #[test]
    fn a_long_name_stops_at_the_cap_on_a_word_boundary() {
        let slug = deck_slug(&"the quick brown fox jumps over ".repeat(5));
        assert!(slug.len() <= SLUG_MAX, "{slug}");
        assert!(!slug.ends_with('-'), "{slug}");
        assert!(slug.starts_with("the-quick-brown-fox"), "{slug}");
        // A single word longer than the cap is cut at the cap.
        let word = deck_slug(&"a".repeat(100));
        assert_eq!(word.len(), SLUG_MAX);
    }

    #[test]
    fn a_username_segment_is_percent_encoded() {
        assert_eq!(encode_segment("scotty"), "scotty");
        assert_eq!(encode_segment("a-b_c.d~e"), "a-b_c.d~e");
        assert_eq!(encode_segment("a/b?c#d%e"), "a%2Fb%3Fc%23d%25e");
        assert_eq!(encode_segment("josé"), "jos%C3%A9");
    }

    #[test]
    fn the_path_has_the_owner_deck_slug_token_shape() {
        let token = Uuid::nil();
        assert_eq!(
            shared_deck_path("scotty", "Elves Go Wide", token),
            "/scotty/deck/elves-go-wide/00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            shared_deck_path("a/b", "", token),
            "/a%2Fb/deck/deck/00000000-0000-0000-0000-000000000000"
        );
        assert!(shared_deck_url("scotty", "Test", token).starts_with(APP_BASE));
    }
}
