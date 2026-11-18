//! The cards the deck's owner has flagged, as a deck page provides them to the tiles under it:
//! the owner's page and the deck's shared link. A page that doesn't provide them marks no card.

use std::collections::HashSet;

use dioxus::prelude::*;
use uuid::Uuid;

/// The flagged cards' printing ids, provided by a deck page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flagged(pub ReadSignal<HashSet<Uuid>>);

/// Whether the card `id` shows its flag: flagged on a page that provides [`Flagged`].
#[must_use]
pub fn is_flagged(id: Uuid) -> bool {
    try_consume_context::<Flagged>().is_some_and(|Flagged(cards)| cards.read().contains(&id))
}
