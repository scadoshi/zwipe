//! The fold every URL slug shares, so a deck's slug and a card's slug of the same name can never
//! disagree: lower case, common accented Latin letters folded to ASCII, every other run of
//! characters one dash, no dash at either end. A deck's slug caps it
//! ([`deck_slug`](crate::domain::share_link::deck_slug)); a card's keeps the whole name
//! ([`card_slug`](crate::domain::card::slug::card_slug)).

/// `name` folded to a slug, uncapped: empty when it has no letter or digit to keep.
#[must_use]
pub(crate) fn slugify(name: &str) -> String {
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
    slug
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
