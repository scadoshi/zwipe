//! The length cap on the text a [`CardCriteria`](super::CardCriteria) carries
//! off the wire.
//!
//! The server matches each text value as a substring of every candidate row,
//! so its cost grows with the value's length. The cap sits above the longest
//! name, type line, oracle text and flavor text in the card data, so a capped
//! value matches the same cards the full value did (LIKE wildcards aside).

use serde::{Deserialize, Deserializer};

/// Most characters a text criterion keeps, per string and per list item.
pub const MAX_TEXT_CRITERION_CHARS: usize = 2048;

/// `value` cut to its first [`MAX_TEXT_CRITERION_CHARS`] characters.
fn capped(mut value: String) -> String {
    if let Some((end, _)) = value.char_indices().nth(MAX_TEXT_CRITERION_CHARS) {
        value.truncate(end);
    }
    value
}

/// Deserializes an optional string, capped.
pub(super) fn text<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.map(capped))
}

/// Deserializes an optional string list, each item capped.
pub(super) fn text_list<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<Vec<String>>::deserialize(deserializer)?
        .map(|items| items.into_iter().map(capped).collect()))
}
