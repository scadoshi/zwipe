//! The Add screen's view of its deck and the rules around its search: what the deck already holds, its land and price running totals, the filter it starts from, and the messages a search answers with.

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{Deck, DeckEntry, Format, deck_metrics};
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;

use crate::domain::deck_signals;
use crate::domain::filter::{AddContext, ensure_lands_excluded};

/// Cards fetched per page.
pub(crate) const PAGE_SIZE: u32 = 25;

/// The next page is fetched once the cursor is this close to the end.
pub(crate) const LOAD_MORE_THRESHOLD: usize = 5;

/// The deck as Add sees it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AddDeck {
    pub(crate) id: Uuid,
    pub(crate) format: Option<Format>,
    /// The commander's combined color identity, when the format checks it.
    pub(crate) colors: Option<Colors>,
    pub(crate) has_commander: bool,
    /// Oracle IDs on every board and in the command zone, which searches never serve.
    pub(crate) in_deck: HashSet<Uuid>,
    /// The maybeboard, which the Maybeboard source swipes.
    pub(crate) maybeboard: Vec<DeckEntry>,
    pub(crate) land_count: i32,
    /// The deck's own land target, which the toast uses; the format default only stops lands.
    pub(crate) land_target: Option<i32>,
    pub(crate) total_price: f64,
    pub(crate) budget: Option<f64>,
    pub(crate) currency: PriceCurrency,
}

impl AddDeck {
    pub(crate) fn from_deck(deck: &Deck) -> Self {
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
            land_count: deck_signals::land_count(&deck.entries),
            land_target: profile.land_target,
            total_price: deck_metrics::deck_price(
                &deck.entries,
                &deck.command_zone_cards,
                currency,
            ),
            budget: profile.price_target,
            currency,
        }
    }

    /// Whether the main board reached its land target, or the format's default when the deck sets none.
    pub(crate) fn lands_at_target(&self) -> bool {
        self.land_target
            .or_else(|| self.format.and_then(|f| f.default_land_target()))
            .is_some_and(|target| self.land_count >= target)
    }

    pub(crate) fn context(&self) -> AddContext {
        AddContext {
            format: self.format,
            colors: self.colors.clone(),
            has_commander: self.has_commander,
            lands_at_target: self.lands_at_target(),
        }
    }

    /// Fills the deck context into a filter as the app does on load: synergy follows the commander unless the filter was remembered, lands stop at the target, and legality and commander colors fill in when the filter has none.
    pub(crate) fn seed(&self, filter: &mut CardQueryBuilder, remembered: bool) {
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
    pub(crate) fn holds(&self, card: &Card) -> bool {
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
pub(crate) fn normalized(filter: &CardQueryBuilder) -> CardQueryBuilder {
    let mut b = filter.clone();
    b.set_is_token(false).set_limit(PAGE_SIZE).set_offset(0);
    b
}

/// The printing IDs from where the server says related cards begin.
pub(crate) fn related_ids(page: &HttpDeckCardSearch) -> HashSet<Uuid> {
    page.related_from
        .and_then(|from| page.cards.get(from..))
        .unwrap_or_default()
        .iter()
        .map(|c| c.card_profile.scryfall_data_id)
        .collect()
}

/// The empty-results toast, with the synergy note when synergy is on.
pub(crate) fn no_results_message(synergy: bool, warming: bool) -> String {
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
