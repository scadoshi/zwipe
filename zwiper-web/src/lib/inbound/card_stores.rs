//! The stores the card screens share for the visit, zwipe-ui's, held in
//! root signals: each screen's filter per deck, Add's parked stacks and its
//! live search stack, the per-deck undo log and the adds still in flight.
//!
//! They live in the root because work outlives the screen that started it:
//! an add's answer lands after Add is gone and still records its undo entry.
//! Nothing here is saved; a reload forgets it, as an app restart does.

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::Board;
use zwipe_ui::stack::{AddAction, CardStack};
use zwipe_ui::stores::filters::FilterStore;
use zwipe_ui::stores::in_flight::InFlightAdds;
use zwipe_ui::stores::in_flight_removes::InFlightRemoves;
use zwipe_ui::stores::parked::AddStackCache;
use zwipe_ui::stores::undo::UndoStore;

use crate::inbound::root_signal::use_root_signal;

/// A removed card to put back: the card, its board, its copies and its star time.
pub type Restore = (Card, Board, i32, Option<DateTime<Utc>>);

/// The shared card stores. `Copy`; read with `stores.undo.read()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardStores {
    /// Each card screen's filter per deck.
    pub filters: Signal<FilterStore>,
    /// Add's search stacks parked per deck.
    pub stacks: Signal<AddStackCache>,
    /// The per-deck undo log.
    pub undo: Signal<UndoStore>,
    /// Adds whose create call has not answered.
    pub in_flight: Signal<InFlightAdds>,
    /// Moves from the maybeboard to the deck whose PATCH has not answered, counted as
    /// `in_flight` counts adds: an undo pressed meanwhile waits on the move.
    pub promotes: Signal<InFlightAdds>,
    /// Remove's deletes that have not answered, and the restores undos parked on
    /// them, which go out once the delete lands even after Remove is gone.
    pub removes: Signal<InFlightRemoves<Restore>>,
    /// Add's search stack, kept across visits so coming back to the same
    /// deck resumes mid-stack. Only ever the stack of `add_stack_deck`: a
    /// screen takes it through [`CardStores::claim_add_stack`].
    pub add_stack: Signal<CardStack<AddAction>>,
    /// The deck `add_stack` belongs to.
    pub add_stack_deck: Signal<Option<Uuid>>,
    /// The filter that produced `add_stack`; `None` while a search is out.
    pub last_search: Signal<Option<CardQueryBuilder>>,
}

/// Provides [`CardStores`] to everything under the root, in signals the
/// root scope owns, since the root tasks the card screens start write them.
/// Call once, in the root.
pub fn use_card_stores() {
    let stores = CardStores {
        filters: use_root_signal(FilterStore::default),
        stacks: use_root_signal(AddStackCache::default),
        undo: use_root_signal(UndoStore::default),
        in_flight: use_root_signal(InFlightAdds::default),
        promotes: use_root_signal(InFlightAdds::default),
        removes: use_root_signal(InFlightRemoves::default),
        add_stack: use_root_signal(CardStack::default),
        add_stack_deck: use_root_signal(|| None),
        last_search: use_root_signal(|| None),
    };
    use_context_provider(|| stores);
}

impl CardStores {
    /// Hands `deck`'s Add screen the search stack ([`claim_add_stack`]). Add calls it as it
    /// opens, before it restores a parked stack or searches.
    pub fn claim_add_stack(self, deck: Uuid) {
        let Self {
            mut add_stack,
            mut last_search,
            mut add_stack_deck,
            ..
        } = self;
        claim_add_stack(
            &mut add_stack.write(),
            &mut last_search.write(),
            &mut add_stack_deck.write(),
            deck,
        );
    }
}

/// Hands `deck`'s Add screen the search stack: the one it left when it is this deck's, else an
/// empty one, so one deck's swiped cards and undo history never show in another's. `owner` is
/// the deck the stack belongs to.
pub fn claim_add_stack(
    stack: &mut CardStack<AddAction>,
    last_search: &mut Option<CardQueryBuilder>,
    owner: &mut Option<Uuid>,
    deck: Uuid,
) {
    if *owner == Some(deck) {
        return;
    }
    stack.reset();
    *last_search = None;
    *owner = Some(deck);
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::Card;
    use zwipe_ui::stores::parked::ParkedStack;

    use super::*;

    /// A real card under its own id.
    fn card(n: u128) -> Card {
        let mut card: Card = serde_json::from_str(include_str!(
            "../../../../zwipe-ui/src/test_support/card.json"
        ))
        .unwrap();
        card.scryfall_data.id = Uuid::from_u128(n);
        card
    }

    /// What Add does on opening `deck`: claims the stack, takes the deck's parked stack when
    /// there is one, else searches, keeping the swiped cards behind the cursor.
    fn open(
        stack: &mut CardStack<AddAction>,
        last_search: &mut Option<CardQueryBuilder>,
        owner: &mut Option<Uuid>,
        parked: &mut AddStackCache,
        deck: Uuid,
        found: Vec<Card>,
    ) {
        claim_add_stack(stack, last_search, owner, deck);
        if let Some(p) = parked.take(deck) {
            stack.restore(p.cards, p.index, p.history);
            return;
        }
        stack.clear_ahead();
        stack.replace_ahead(found);
        *last_search = Some(CardQueryBuilder::new());
    }

    /// What Add does on leaving `deck`: parks the stack under it.
    fn leave(
        stack: &CardStack<AddAction>,
        last_search: Option<&CardQueryBuilder>,
        parked: &mut AddStackCache,
        deck: Uuid,
    ) {
        if let Some(filter) = last_search.cloned()
            && stack.has_served()
        {
            let (cards, index, history) = stack.park_state();
            parked.park(
                deck,
                ParkedStack {
                    cards,
                    index,
                    history,
                    filter,
                    exhausted: false,
                },
            );
        }
    }

    #[test]
    fn deck_b_s_add_never_offers_an_undo_from_deck_a() {
        let (red, blue) = (Uuid::from_u128(0xa), Uuid::from_u128(0xb));
        let mut stack = CardStack::default();
        let mut last_search = None;
        let mut owner = None;
        let mut parked = AddStackCache::default();

        // Deck A: add a card, skip a card, leave.
        open(
            &mut stack,
            &mut last_search,
            &mut owner,
            &mut parked,
            red,
            (1..=4).map(card).collect(),
        );
        stack.record(AddAction::Add);
        let _ = stack.advance();
        stack.record(AddAction::Skip);
        let _ = stack.advance();
        leave(&stack, last_search.as_ref(), &mut parked, red);

        // Deck B: open Add.
        open(
            &mut stack,
            &mut last_search,
            &mut owner,
            &mut parked,
            blue,
            (11..=14).map(card).collect(),
        );
        assert!(!stack.has_history(), "Undo is not lit in deck B");
        assert_eq!(
            stack.current().map(|c| c.scryfall_data.id),
            Some(Uuid::from_u128(11)),
            "deck B's own first card"
        );
        assert!(stack.pop_action().is_none(), "nothing of deck A's to undo");
        leave(&stack, last_search.as_ref(), &mut parked, blue);

        // Back to deck A: its add and skip are still there to undo.
        open(
            &mut stack,
            &mut last_search,
            &mut owner,
            &mut parked,
            red,
            Vec::new(),
        );
        assert_eq!(stack.park_state().2, [AddAction::Add, AddAction::Skip]);
        assert_eq!(
            stack.current().map(|c| c.scryfall_data.id),
            Some(Uuid::from_u128(3))
        );
    }
}
