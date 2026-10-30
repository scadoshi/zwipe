//! The swipe stack: the query that fills it, the cards in it, and what each swipe or undo asks the server to do.
//!
//! The server drops a swiped card from the deck's pool, so every page is fetched from offset 0 and the stack only has to skip cards it already holds.

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::error::InvalidCardCriteria;
use zwipe_core::domain::card::search_card::card_filter::{CardQuery, CardSortKey};
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::deck::Deck;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;

/// Cards fetched per page, as the app does.
const PAGE_SIZE: u32 = 25;

/// The next page is fetched once this few cards are left to swipe.
const REFILL_AT: usize = 5;

/// The query that serves cards for `deck`, matching the app's defaults.
///
/// A deck with a commander gets the commander's synergy ordering and color identity. A deck with a format gets that format's legal cards. Lands stop coming once the deck reaches its land target. A deck with neither commander nor format is served by EDHREC popularity.
///
/// # Errors
///
/// Returns [`InvalidCardCriteria`] when the criteria contradict each other, which these defaults never do.
pub(crate) fn query_for(deck: &Deck) -> Result<CardQuery, InvalidCardCriteria> {
    let profile = &deck.deck_profile;
    let mut builder = CardQueryBuilder::default();
    builder.set_synergy(profile.commander_id.is_some());

    if let Some(format) = profile.format {
        builder.set_legalities_contains_any(vec![format.to_legality_key().to_owned()]);
        if format.checks_color_identity() && profile.commander_id.is_some() {
            builder.set_color_identity_within(commander_identity(deck));
        }
    }

    let land_target = profile
        .land_target
        .or_else(|| profile.format.and_then(|f| f.default_land_target()));
    if land_target.is_some_and(|target| land_count(deck) >= target) {
        builder.set_card_type_excludes_any(vec![CardType::Land]);
    }

    if profile.commander_id.is_none() && profile.format.is_none() {
        builder.set_sort(CardSortKey::EdhrecRank);
    }
    builder.set_is_token(false);
    builder.set_limit(PAGE_SIZE);
    builder.set_offset(0);
    builder.build()
}

/// The combined color identity of the commander, partner and background. The signature spell does not count.
fn commander_identity(deck: &Deck) -> Colors {
    let signature = deck.deck_profile.signature_spell_id;
    let mut colors: Vec<Color> = deck
        .command_zone_cards
        .iter()
        .filter(|card| Some(card.scryfall_data.id) != signature)
        .flat_map(|card| card.scryfall_data.color_identity.iter().copied())
        .collect();
    colors.sort();
    colors.dedup();
    colors.into()
}

/// Lands on the main board, counting copies.
fn land_count(deck: &Deck) -> i32 {
    deck.entries
        .iter()
        .filter(|e| e.deck_card.board.is_active() && e.card.scryfall_data.is_land())
        .fold(0, |total, e| total.saturating_add(*e.deck_card.quantity))
}

/// The cards already in the deck or its command zone, by oracle ID, which the stack never serves.
pub(crate) fn cards_in(deck: &Deck) -> HashSet<Uuid> {
    deck.entries
        .iter()
        .map(|e| &e.card)
        .chain(&deck.command_zone_cards)
        .filter_map(|card| card.scryfall_data.oracle_id)
        .collect()
}

/// A swipe direction, as the app names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Swipe {
    /// Right: add one copy to the deck.
    Add,
    /// Up: add one copy to the maybeboard.
    Maybe,
    /// Left: pass, so the server stops serving it for this deck.
    Pass,
}

/// What a swipe or an undo asks the server to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    Add {
        scryfall_data_id: Uuid,
        oracle_id: Option<Uuid>,
        maybeboard: bool,
    },
    Skip {
        oracle_id: Uuid,
    },
    Remove {
        scryfall_data_id: Uuid,
    },
    Unskip {
        oracle_id: Uuid,
    },
}

/// The cards being swiped, the position in them, and the swipes that can be undone.
#[derive(Debug, Default)]
pub(crate) struct Stack {
    cards: Vec<Card>,
    /// The card on screen; equal to the length once every card is swiped.
    index: usize,
    /// One entry per swipe, newest last.
    history: Vec<Swipe>,
    /// Oracle IDs the deck holds, plus every card added here.
    in_deck: HashSet<Uuid>,
    /// Printing IDs already in the stack.
    held: HashSet<Uuid>,
    /// Set when a page brought nothing new.
    exhausted: bool,
    /// The printing ID of the first card served after the commander's synergy pool ran out.
    related_start: Option<Uuid>,
}

impl Stack {
    pub(crate) fn new(in_deck: HashSet<Uuid>) -> Self {
        Self {
            in_deck,
            ..Self::default()
        }
    }

    /// The card to swipe next.
    pub(crate) fn current(&self) -> Option<&Card> {
        self.cards.get(self.index)
    }

    pub(crate) fn remaining(&self) -> usize {
        self.cards.len().saturating_sub(self.index)
    }

    pub(crate) fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    /// Whether the next page should be fetched now.
    pub(crate) fn needs_more(&self) -> bool {
        !self.exhausted && self.remaining() <= REFILL_AT
    }

    /// Whether the card on screen is the first one past the commander's synergy picks.
    pub(crate) fn at_related_start(&self) -> bool {
        self.related_start.is_some()
            && self.current().map(|c| c.scryfall_data.id) == self.related_start
    }

    /// Appends a page, skipping cards already held or already in the deck. A page with nothing new marks the stack exhausted.
    pub(crate) fn push_page(&mut self, page: HttpDeckCardSearch) {
        if self.related_start.is_none() {
            self.related_start = page
                .related_from
                .and_then(|i| page.cards.get(i))
                .map(|c| c.scryfall_data.id);
        }
        let before = self.cards.len();
        for card in page.cards {
            let in_deck = card
                .scryfall_data
                .oracle_id
                .is_some_and(|id| self.in_deck.contains(&id));
            if !in_deck && self.held.insert(card.scryfall_data.id) {
                self.cards.push(card);
            }
        }
        self.exhausted = self.cards.len() == before;
    }

    /// Swipes the card on screen, and says what to tell the server. `None` when there is no card, or when passing on a card without an oracle ID.
    pub(crate) fn swipe(&mut self, swipe: Swipe) -> Option<Effect> {
        let data = &self.cards.get(self.index)?.scryfall_data;
        let (scryfall_data_id, oracle_id) = (data.id, data.oracle_id);
        self.index = self.index.saturating_add(1);
        self.history.push(swipe);
        match swipe {
            Swipe::Add | Swipe::Maybe => {
                self.in_deck.extend(oracle_id);
                Some(Effect::Add {
                    scryfall_data_id,
                    oracle_id,
                    maybeboard: swipe == Swipe::Maybe,
                })
            }
            Swipe::Pass => oracle_id.map(|oracle_id| Effect::Skip { oracle_id }),
        }
    }

    /// Steps back to the last swiped card, and says which swipe it undid and how to reverse it on the server. `None` when nothing has been swiped.
    pub(crate) fn undo(&mut self) -> Option<(Swipe, Option<Effect>)> {
        let swipe = self.history.pop()?;
        self.index = self.index.saturating_sub(1);
        let data = &self.cards.get(self.index)?.scryfall_data;
        let effect = match swipe {
            Swipe::Add | Swipe::Maybe => {
                if let Some(oracle_id) = data.oracle_id {
                    self.in_deck.remove(&oracle_id);
                }
                Some(Effect::Remove {
                    scryfall_data_id: data.id,
                })
            }
            Swipe::Pass => data.oracle_id.map(|oracle_id| Effect::Unskip { oracle_id }),
        };
        Some((swipe, effect))
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::scryfall_data::colors::Color;
    use zwipe_core::domain::deck::Format;

    use super::*;
    use crate::test_support::{card, deck, deck_profile};

    fn wire(deck: &Deck) -> serde_json::Value {
        serde_json::to_value(query_for(deck).unwrap()).unwrap()
    }

    #[test]
    fn a_deck_without_commander_or_format_is_served_by_popularity() {
        let query = wire(&deck(deck_profile("Pile"), Vec::new()));
        assert_eq!(query["synergy"], false);
        assert_eq!(query["order_by"], "EdhrecRank");
        assert_eq!(query["limit"], 25);
        assert_eq!(query["offset"], 0);
        assert_eq!(query["is_token"], false);
    }

    #[test]
    fn a_commander_deck_uses_synergy_legality_and_the_commanders_colors() {
        let mut commander = card("Commander");
        commander.scryfall_data.color_identity = vec![Color::Green, Color::Blue].into();
        let mut signature = card("Signature");
        signature.scryfall_data.color_identity = vec![Color::Red].into();

        let mut profile = deck_profile("Simic");
        profile.commander_id = Some(commander.scryfall_data.id);
        profile.signature_spell_id = Some(signature.scryfall_data.id);
        profile.format = Some(Format::Commander);
        let mut deck = deck(profile, Vec::new());
        deck.command_zone_cards = vec![commander.clone(), signature.clone()];

        let query = wire(&deck);
        assert_eq!(query["synergy"], true);
        assert!(query.get("order_by").is_none());
        assert_eq!(
            query["legalities_contains_any"],
            serde_json::json!(["commander"])
        );
        let colors = serde_json::to_value(Colors::from(vec![Color::Blue, Color::Green])).unwrap();
        assert_eq!(query["color_identity_within"], colors);

        let held = cards_in(&deck);
        assert!(held.contains(&commander.scryfall_data.oracle_id.unwrap()));
        assert!(held.contains(&signature.scryfall_data.oracle_id.unwrap()));
    }

    #[test]
    fn lands_stop_once_the_deck_reaches_its_land_target() {
        let land = |name: &str| {
            let mut c = card(name);
            c.scryfall_data.type_line = Some("Basic Land — Forest".to_owned());
            c
        };
        let mut profile = deck_profile("Lands");
        profile.land_target = Some(2);
        let short = deck(profile.clone(), vec![land("Forest A")]);
        assert!(wire(&short).get("card_type_excludes_any").is_none());

        let full = deck(profile, vec![land("Forest A"), land("Forest B")]);
        assert_eq!(
            wire(&full)["card_type_excludes_any"],
            serde_json::json!(["Land"])
        );
    }

    fn page(cards: Vec<Card>, related_from: Option<usize>) -> HttpDeckCardSearch {
        HttpDeckCardSearch {
            cards,
            synergy_applied: true,
            related_from,
        }
    }

    fn names(stack: &Stack) -> Vec<String> {
        stack
            .cards
            .iter()
            .map(|c| c.scryfall_data.name.clone())
            .collect()
    }

    #[test]
    fn a_page_skips_held_cards_and_cards_in_the_deck() {
        let owned = card("Owned");
        let fresh = card("Fresh");
        let mut stack = Stack::new(HashSet::from([owned.scryfall_data.oracle_id.unwrap()]));
        stack.push_page(page(vec![owned, fresh.clone()], None));
        stack.push_page(page(vec![fresh, card("Next")], None));
        assert_eq!(names(&stack), ["Fresh", "Next"]);
        assert!(!stack.is_exhausted());
    }

    #[test]
    fn a_page_with_nothing_new_ends_the_stack() {
        let only = card("Only");
        let mut stack = Stack::new(HashSet::new());
        stack.push_page(page(vec![only.clone()], None));
        stack.push_page(page(vec![only], None));
        assert!(stack.is_exhausted());
        assert!(!stack.needs_more());
    }

    #[test]
    fn swipes_say_what_to_send_and_undo_reverses_them_in_order() {
        let (a, b, c) = (card("A"), card("B"), card("C"));
        let (a_id, b_oracle, c_id) = (
            a.scryfall_data.id,
            b.scryfall_data.oracle_id.unwrap(),
            c.scryfall_data.id,
        );
        let mut stack = Stack::new(HashSet::new());
        stack.push_page(page(vec![a.clone(), b, c], None));

        assert_eq!(
            stack.swipe(Swipe::Add),
            Some(Effect::Add {
                scryfall_data_id: a_id,
                oracle_id: a.scryfall_data.oracle_id,
                maybeboard: false,
            })
        );
        assert_eq!(
            stack.swipe(Swipe::Pass),
            Some(Effect::Skip {
                oracle_id: b_oracle
            })
        );
        assert!(matches!(
            stack.swipe(Swipe::Maybe),
            Some(Effect::Add {
                maybeboard: true,
                ..
            })
        ));
        assert!(stack.current().is_none());

        assert_eq!(
            stack.undo(),
            Some((
                Swipe::Maybe,
                Some(Effect::Remove {
                    scryfall_data_id: c_id
                })
            ))
        );
        assert_eq!(
            stack.undo(),
            Some((
                Swipe::Pass,
                Some(Effect::Unskip {
                    oracle_id: b_oracle
                })
            ))
        );
        assert_eq!(
            stack.current().map(|c| c.scryfall_data.name.as_str()),
            Some("B")
        );
    }

    #[test]
    fn an_added_card_is_not_served_again_until_undone() {
        let a = card("A");
        let mut stack = Stack::new(HashSet::new());
        stack.push_page(page(vec![a.clone()], None));
        stack.swipe(Swipe::Add);
        let oracle = a.scryfall_data.oracle_id.unwrap();
        assert!(stack.in_deck.contains(&oracle));
        stack.undo();
        assert!(!stack.in_deck.contains(&oracle));
    }

    #[test]
    fn undo_with_nothing_swiped_does_nothing() {
        let mut stack = Stack::new(HashSet::new());
        stack.push_page(page(vec![card("A")], None));
        assert_eq!(stack.undo(), None);
        assert_eq!(stack.remaining(), 1);
    }

    #[test]
    fn the_stack_asks_for_more_when_few_cards_remain() {
        let mut stack = Stack::new(HashSet::new());
        stack.push_page(page((0..8).map(|i| card(&i.to_string())).collect(), None));
        assert!(!stack.needs_more());
        for _ in 0..3 {
            stack.swipe(Swipe::Pass);
        }
        assert!(stack.needs_more());
    }

    #[test]
    fn the_first_related_card_is_marked() {
        let (a, b) = (card("Synergy"), card("Related"));
        let mut stack = Stack::new(HashSet::new());
        stack.push_page(page(vec![a, b], Some(1)));
        assert!(!stack.at_related_start());
        stack.swipe(Swipe::Pass);
        assert!(stack.at_related_start());
    }
}
