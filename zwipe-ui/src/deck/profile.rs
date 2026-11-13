//! A deck's Profile on a phone, read the same from the owner's deck and a shared deck: the info
//! rows ([`DeckInfo`]) and the warnings a shared deck shows read only ([`shared_warnings`]).

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::validate_deck::{DeckCommandZone, validate_deck};
use zwipe_core::domain::deck::{Board, DeckName, DeckProfile, DeckWarning, Format, PowerLevel};
use zwipe_core::http::contracts::deck::HttpSharedDeck;

/// The Profile card's rows: the deck's name, card count, format, command zone and power level,
/// and for a shared deck who shares it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeckInfo {
    /// The deck's name.
    pub name: String,
    /// How many cards the deck holds, its command zone included.
    pub cards: usize,
    /// The deck's format, if set.
    pub format: Option<Format>,
    /// The commander's name, if set.
    pub commander: Option<String>,
    /// The partner commander's name, if set.
    pub partner: Option<String>,
    /// The background's name, if set.
    pub background: Option<String>,
    /// The signature spell's name, if set.
    pub signature_spell: Option<String>,
    /// The deck's power level, if set.
    pub power_level: Option<PowerLevel>,
    /// Who shares the deck; `None` on the owner's own deck.
    pub owner: Option<String>,
}

impl DeckInfo {
    /// The owner's deck from its `profile`, holding `cards` cards, with `commander` (the fetched
    /// commander card's name) in place of the profile's when it is in.
    #[must_use]
    pub fn of_profile(profile: &DeckProfile, cards: usize, commander: Option<String>) -> Self {
        Self {
            name: profile.name.to_string(),
            cards,
            format: profile.format,
            commander: commander.or_else(|| profile.commander_name.clone()),
            partner: profile.partner_commander_name.clone(),
            background: profile.background_name.clone(),
            signature_spell: profile.signature_spell_name.clone(),
            power_level: profile.power_level,
            owner: None,
        }
    }

    /// A shared deck holding `cards` cards.
    #[must_use]
    pub fn of_shared(deck: &HttpSharedDeck, cards: usize) -> Self {
        let name = |card: &Option<Card>| card.as_ref().map(|c| c.scryfall_data.name.clone());
        Self {
            name: deck.name.clone(),
            cards,
            format: deck.format,
            commander: name(&deck.commander),
            partner: name(&deck.partner_commander),
            background: name(&deck.background),
            signature_spell: name(&deck.signature_spell),
            power_level: deck.power_level,
            owner: deck.owner.clone(),
        }
    }
}

/// The warnings a shared deck raises, as the owner's deck raises them: its format's rules over
/// its entries and command zone. A name the rules can't hold raises none.
#[must_use]
pub fn shared_warnings(deck: &HttpSharedDeck) -> Vec<DeckWarning> {
    let Ok(name) = DeckName::new(deck.name.clone()) else {
        return Vec::new();
    };
    let card_count: i64 = deck
        .entries
        .iter()
        .filter(|e| e.deck_card.board == Board::Deck)
        .map(|e| i64::from(*e.deck_card.quantity))
        .fold(0, i64::saturating_add);
    let id = |card: &Option<Card>| card.as_ref().map(|c| c.scryfall_data.id);
    let label = |card: &Option<Card>| card.as_ref().map(|c| c.scryfall_data.name.clone());
    let profile = DeckProfile {
        id: Uuid::nil(),
        name,
        commander_id: id(&deck.commander),
        partner_commander_id: id(&deck.partner_commander),
        background_id: id(&deck.background),
        signature_spell_id: id(&deck.signature_spell),
        format: deck.format,
        tags: deck.tags.clone(),
        power_level: deck.power_level,
        other_tags: deck.other_tags.clone(),
        oracle_tags: deck.oracle_tags.clone(),
        land_target: deck.land_target,
        price_target: deck.price_target,
        price_target_currency: deck.price_target_currency,
        share_token: None,
        user_id: Uuid::nil(),
        card_count,
        commander_name: label(&deck.commander),
        partner_commander_name: label(&deck.partner_commander),
        background_name: label(&deck.background),
        signature_spell_name: label(&deck.signature_spell),
        color_identity: Vec::new(),
        commander_art_url: None,
        partner_commander_art_url: None,
        background_art_url: None,
        signature_spell_art_url: None,
        updated_at: None,
        total_price_usd: None,
        total_price_eur: None,
        total_price_tix: None,
    };
    let zone = DeckCommandZone {
        commander: deck.commander.as_ref(),
        partner_commander: deck.partner_commander.as_ref(),
        background: deck.background.as_ref(),
        signature_spell: deck.signature_spell.as_ref(),
    };
    validate_deck(&profile, &deck.entries, &zone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    /// A shared Commander deck holding `cards` on its main board, with `commander` in its zone.
    fn shared(cards: Vec<Card>, commander: Option<&Card>) -> HttpSharedDeck {
        let held = test_support::deck(test_support::deck_profile("Goblins"), cards);
        serde_json::from_value(serde_json::json!({
            "name": "Goblins",
            "format": "commander",
            "commander": commander,
            "partner_commander": null,
            "background": null,
            "signature_spell": null,
            "entries": held.entries,
            "owner": "scotty",
        }))
        .unwrap()
    }

    #[test]
    fn a_shared_deck_reads_its_rows_from_its_cards_and_names_its_owner() {
        let krenko = test_support::card("Krenko, Mob Boss");
        let deck = shared(Vec::new(), Some(&krenko));
        let info = DeckInfo::of_shared(&deck, 1);
        assert_eq!(info.name, "Goblins");
        assert_eq!(info.cards, 1);
        assert_eq!(info.format, Some(Format::Commander));
        assert_eq!(info.commander.as_deref(), Some("Krenko, Mob Boss"));
        assert_eq!(info.owner.as_deref(), Some("scotty"));
    }

    #[test]
    fn a_shared_deck_raises_the_warnings_its_owner_sees() {
        let deck = shared(vec![test_support::card("Goblin Guide")], None);
        let warnings = shared_warnings(&deck);
        assert!(
            warnings.iter().any(|w| w.to_string().contains("100")),
            "a one-card Commander deck is under its count: {warnings:?}"
        );
    }
}
