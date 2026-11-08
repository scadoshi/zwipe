//! The screen that opened a deck, so the header's back arrow on the deck
//! returns to it, as the app's does.
//!
//! Home and Decks record the deck they open with their own route. The back
//! arrow on that deck goes to the recorded route while it is the same deck,
//! and to Decks otherwise: a deck reached another way (Create deck, a clone,
//! a link) has no opener on record. The record outlives the screens inside
//! the deck (Edit, the card screens), so their back arrow up to the deck,
//! then the deck's own, still finds it.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::inbound::router::Route;

/// The last deck opened from a screen that records it, provided once at the
/// app root.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeckOpener(Signal<Option<Opened>>);

/// One deck with the route that opened it.
#[derive(Debug, Clone, PartialEq)]
struct Opened {
    deck_id: Uuid,
    from: Route,
}

impl DeckOpener {
    /// Provides an empty record to the app below this component.
    pub fn use_provider() {
        use_context_provider(|| Self(Signal::new(None)));
    }

    /// Records that `from` opened `deck_id`.
    pub fn record(self, deck_id: Uuid, from: Route) {
        let mut opened = self.0;
        opened.set(Some(Opened { deck_id, from }));
    }

    /// Where the back arrow on `deck_id` goes: the screen that opened it, or
    /// Decks when another deck or none is on record.
    #[must_use]
    pub fn back_from(self, deck_id: Uuid) -> Route {
        back_route(self.0.read().as_ref(), deck_id)
    }
}

/// [`DeckOpener::back_from`] over a plain record.
fn back_route(opened: Option<&Opened>, deck_id: Uuid) -> Route {
    opened
        .filter(|o| o.deck_id == deck_id)
        .map_or(Route::DeckList {}, |o| o.from.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_returns_to_the_screen_that_opened_the_deck() {
        let deck_id = Uuid::new_v4();
        let opened = Opened {
            deck_id,
            from: Route::Home {},
        };
        assert_eq!(back_route(Some(&opened), deck_id), Route::Home {});
    }

    #[test]
    fn back_falls_back_to_decks() {
        let deck_id = Uuid::new_v4();
        let other = Opened {
            deck_id: Uuid::new_v4(),
            from: Route::Home {},
        };
        assert_eq!(back_route(Some(&other), deck_id), Route::DeckList {});
        assert_eq!(back_route(None, deck_id), Route::DeckList {});
    }
}
