//! The Add screen's view of its deck and the rules around its search: what the deck already holds, its land and price running totals, the filter it starts from, and the messages a search answers with.

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::deck_price;
use zwipe_core::domain::deck::{Deck, DeckEntry, Format};
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;

use crate::deck_signals::land_count;
use crate::filter::{AddContext, ensure_lands_excluded};
use crate::timing;

/// The deck as Add sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct AddDeck {
    /// The deck's ID.
    pub id: Uuid,
    /// The deck's format, if set.
    pub format: Option<Format>,
    /// The commander's combined color identity, when the format checks it.
    pub colors: Option<Colors>,
    /// Whether the deck has a commander.
    pub has_commander: bool,
    /// Oracle IDs on every board and in the command zone, which searches never serve.
    pub in_deck: HashSet<Uuid>,
    /// The maybeboard, which the Maybeboard source swipes.
    pub maybeboard: Vec<DeckEntry>,
    /// Lands on the main board, counting copies.
    pub land_count: i32,
    /// The deck's own land target, which the toast uses; the format default only stops lands.
    pub land_target: Option<i32>,
    /// The main board's price.
    pub total_price: f64,
    /// The deck's price target, if set.
    pub budget: Option<f64>,
    /// The currency the price and budget are in.
    pub currency: PriceCurrency,
}

impl AddDeck {
    /// The deck as Add sees it, read from the full deck.
    #[must_use]
    pub fn from_deck(deck: &Deck) -> Self {
        let profile = &deck.deck_profile;
        let in_deck: HashSet<Uuid> = deck
            .entries
            .iter()
            .map(|e| &e.card)
            .chain(&deck.command_zone_cards)
            .filter_map(|c| c.scryfall_data.oracle_id)
            .collect();
        let has_commander = profile.commander_id.is_some();
        let colors = profile
            .format
            .filter(|f| f.checks_color_identity() && has_commander)
            .map(|_| commander_identity(deck));
        let currency = profile.price_target_currency.unwrap_or(PriceCurrency::Usd);
        Self {
            id: profile.id,
            format: profile.format,
            colors,
            has_commander,
            in_deck,
            maybeboard: deck
                .entries
                .iter()
                .filter(|e| e.deck_card.board.is_maybeboard())
                .cloned()
                .collect(),
            land_count: land_count(&deck.entries),
            land_target: profile.land_target,
            total_price: deck_price(&deck.entries, &deck.command_zone_cards, currency),
            budget: profile.price_target,
            currency,
        }
    }

    /// Whether the main board reached its land target, or the format's default when the deck sets none.
    #[must_use]
    pub fn lands_at_target(&self) -> bool {
        self.land_target
            .or_else(|| self.format.and_then(|f| f.default_land_target()))
            .is_some_and(|target| self.land_count >= target)
    }

    /// The deck facts Add's search filter is built from.
    #[must_use]
    pub fn context(&self) -> AddContext {
        AddContext {
            format: self.format,
            colors: self.colors.clone(),
            has_commander: self.has_commander,
            lands_at_target: self.lands_at_target(),
        }
    }

    /// Fills the deck context into a filter as the app does on load: synergy follows the commander unless the filter was remembered, lands stop at the target, and legality and commander colors fill in when the filter has none.
    pub fn seed(&self, filter: &mut CardQueryBuilder, remembered: bool) {
        if !remembered {
            filter.set_synergy(self.has_commander);
        }
        if self.lands_at_target() {
            ensure_lands_excluded(filter);
        }
        if let Some(format) = self.format
            && filter.legalities_contains_any().is_none()
        {
            filter.set_legalities_contains_any(vec![format.to_legality_key().to_owned()]);
        }
        if let Some(colors) = &self.colors
            && filter.color_identity_within().is_none()
        {
            filter.set_color_identity_within(colors.clone());
        }
    }

    /// Whether the deck holds a card with this oracle ID.
    #[must_use]
    pub fn holds(&self, card: &Card) -> bool {
        card.scryfall_data
            .oracle_id
            .is_some_and(|id| self.in_deck.contains(&id))
    }
}

/// The combined, sorted color identity of the commander, partner and background.
fn commander_identity(deck: &Deck) -> Colors {
    let profile = &deck.deck_profile;
    let zone = [
        profile.commander_id,
        profile.partner_commander_id,
        profile.background_id,
    ];
    let mut colors: Vec<Color> = deck
        .command_zone_cards
        .iter()
        .filter(|c| zone.contains(&Some(c.scryfall_data.id)))
        .flat_map(|c| c.scryfall_data.color_identity.iter().copied())
        .collect();
    colors.sort();
    colors.dedup();
    colors.into()
}

/// The filter as Add sends it: no tokens, a page of 25, always from the top.
#[must_use]
pub fn normalized(filter: &CardQueryBuilder) -> CardQueryBuilder {
    let mut b = filter.clone();
    b.set_is_token(false)
        .set_limit(timing::PAGE_SIZE)
        .set_offset(0);
    b
}

/// The printing IDs from where the server says related cards begin.
#[must_use]
pub fn related_ids(page: &HttpDeckCardSearch) -> HashSet<Uuid> {
    page.related_from
        .and_then(|from| page.cards.get(from..))
        .unwrap_or_default()
        .iter()
        .map(|c| c.card_profile.scryfall_data_id)
        .collect()
}

/// The empty-results toast, with the synergy note when synergy is on.
#[must_use]
pub fn no_results_message(synergy: bool, warming: bool) -> String {
    let suffix = match (synergy, warming) {
        (false, _) => "",
        (true, true) => "; Synergy is warming up, so all cards were searched",
        (true, false) => "; tap Synergy to turn it off",
    };
    format!("No results for this filter{suffix}")
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::search_card::card_type::CardType;
    use zwipe_core::domain::deck::Board;

    use super::*;
    use crate::test_support::{card, deck, deck_profile};

    fn commander_deck() -> Deck {
        let mut commander = card("Commander");
        commander.scryfall_data.color_identity = vec![Color::Green, Color::Blue].into();
        let mut spell = card("Spell");
        spell.scryfall_data.color_identity = vec![Color::Red].into();
        let mut profile = deck_profile("Simic");
        profile.commander_id = Some(commander.scryfall_data.id);
        profile.signature_spell_id = Some(spell.scryfall_data.id);
        profile.format = Some(Format::Commander);
        let mut maybe = card("Maybe");
        maybe.scryfall_data.type_line = Some("Land".to_owned());
        let mut d = deck(profile, vec![card("Main"), maybe]);
        d.entries[1].deck_card.board = Board::Maybeboard;
        d.command_zone_cards = vec![commander, spell];
        d
    }

    #[test]
    fn a_commander_deck_seeds_synergy_legality_and_colors() {
        let d = commander_deck();
        let add = AddDeck::from_deck(&d);
        assert!(add.has_commander);
        assert_eq!(add.in_deck.len(), 4, "boards and command zone");
        assert_eq!(add.maybeboard.len(), 1);
        assert_eq!(add.land_count, 0, "maybeboard lands do not count");
        assert_eq!(
            add.colors.as_deref(),
            Some(&[Color::Blue, Color::Green][..])
        );

        let mut filter = CardQueryBuilder::default();
        add.seed(&mut filter, false);
        assert!(filter.synergy());
        assert_eq!(
            filter.legalities_contains_any(),
            Some(&["commander".to_owned()][..])
        );
        assert!(filter.color_identity_within().is_some());
        assert_eq!(filter.card_type_excludes_any(), None);

        let mut remembered = CardQueryBuilder::default();
        add.seed(&mut remembered, true);
        assert!(
            !remembered.synergy(),
            "a remembered filter keeps its toggle"
        );
    }

    #[test]
    fn lands_stop_at_the_deck_target_or_the_format_default() {
        let mut add = AddDeck::from_deck(&commander_deck());
        add.land_count = Format::Commander.default_land_target().unwrap();
        assert!(add.lands_at_target(), "the format's default stands in");
        add.land_target = Some(40);
        assert!(!add.lands_at_target());
        add.land_count = 40;
        let mut filter = CardQueryBuilder::default();
        add.seed(&mut filter, true);
        assert_eq!(filter.card_type_excludes_any(), Some(&[CardType::Land][..]));
    }

    #[test]
    fn a_pile_has_no_context() {
        let add = AddDeck::from_deck(&deck(deck_profile("Pile"), Vec::new()));
        assert!(!add.lands_at_target());
        let mut filter = CardQueryBuilder::default();
        add.seed(&mut filter, false);
        assert!(!filter.has_search_intent());
        assert!(!add.context().serves(&filter));
    }

    #[test]
    fn related_ids_start_where_the_server_says() {
        let cards = vec![card("A"), card("B"), card("C")];
        let page = HttpDeckCardSearch {
            cards: cards.clone(),
            synergy_applied: true,
            related_from: Some(1),
        };
        let ids = related_ids(&page);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&cards[2].scryfall_data.id));
        assert!(
            related_ids(&HttpDeckCardSearch {
                related_from: None,
                ..page
            })
            .is_empty()
        );
    }

    #[test]
    fn empty_results_explain_synergy() {
        assert_eq!(
            no_results_message(false, true),
            "No results for this filter"
        );
        assert_eq!(
            no_results_message(true, false),
            "No results for this filter; tap Synergy to turn it off"
        );
        assert!(no_results_message(true, true).ends_with("so all cards were searched"));
        let b = normalized(&CardQueryBuilder::default());
        assert_eq!((b.limit(), b.offset(), b.is_token()), (25, 0, Some(false)));
    }
}
