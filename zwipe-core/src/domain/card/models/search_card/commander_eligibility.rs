//! Commander eligibility rules by format.
//!
//! This is the authoritative definition of commander eligibility.
//! The SQL filter in zerver's `search_scryfall_data` must mirror this logic.

use crate::domain::{card::Card, deck::Format};

/// Checks whether a card is a valid commander for the given format.
pub fn is_valid_commander(card: &Card, format: &Format) -> bool {
    let sd = &card.scryfall_data;
    let type_line = sd.type_line.as_deref().unwrap_or("");
    let oracle_text = sd.oracle_text.as_deref().unwrap_or("");

    match format {
        // Legendary creature, legendary vehicle/spacecraft with P/T,
        // or "can be your commander" oracle text
        Format::Commander | Format::Duel | Format::Predh => {
            let is_legendary = type_line.contains("Legendary");
            let is_creature = type_line.contains("Creature");
            let has_pt = sd.power.is_some() && sd.toughness.is_some();
            let can_be_commander = oracle_text
                .to_lowercase()
                .contains("can be your commander");

            (is_legendary && (is_creature || has_pt)) || can_be_commander
        }

        // Legendary creature OR legendary planeswalker
        Format::Brawl | Format::StandardBrawl | Format::HistoricBrawl => {
            let is_legendary = type_line.contains("Legendary");
            let is_creature = type_line.contains("Creature");
            let is_planeswalker = type_line.contains("Planeswalker");

            is_legendary && (is_creature || is_planeswalker)
        }

        // Uncommon creature (not legendary-required)
        Format::PauperCommander => {
            use crate::domain::card::scryfall_data::rarity::Rarity;

            let is_creature = type_line.contains("Creature");
            let is_uncommon = sd.rarity == Rarity::Uncommon;

            is_creature && is_uncommon
        }

        // Any planeswalker
        Format::Oathbreaker => type_line.contains("Planeswalker"),

        // Non-commander formats — nothing is a valid commander
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::scryfall_data::rarity::Rarity;
    use crate::test_utils::make_card;

    #[test]
    fn legendary_creature_passes_commander() {
        let mut card = make_card("Atraxa, Praetors' Voice");
        card.scryfall_data.type_line =
            Some("Legendary Creature — Phyrexian Angel Horror".to_string());
        assert!(is_valid_commander(&card, &Format::Commander));
    }

    #[test]
    fn legendary_creature_passes_brawl() {
        let mut card = make_card("Atraxa, Praetors' Voice");
        card.scryfall_data.type_line =
            Some("Legendary Creature — Phyrexian Angel Horror".to_string());
        assert!(is_valid_commander(&card, &Format::Brawl));
    }

    #[test]
    fn non_legendary_creature_fails_commander() {
        let mut card = make_card("Llanowar Elves");
        card.scryfall_data.type_line = Some("Creature — Elf Druid".to_string());
        assert!(!is_valid_commander(&card, &Format::Commander));
    }

    #[test]
    fn uncommon_creature_passes_pauper_commander() {
        let mut card = make_card("Burning-Tree Emissary");
        card.scryfall_data.type_line = Some("Creature — Human Shaman".to_string());
        card.scryfall_data.rarity = Rarity::Uncommon;
        assert!(is_valid_commander(&card, &Format::PauperCommander));
    }

    #[test]
    fn rare_creature_fails_pauper_commander() {
        let mut card = make_card("Tarmogoyf");
        card.scryfall_data.type_line = Some("Creature — Lhurgoyf".to_string());
        card.scryfall_data.rarity = Rarity::Rare;
        assert!(!is_valid_commander(&card, &Format::PauperCommander));
    }

    #[test]
    fn legendary_planeswalker_passes_brawl() {
        let mut card = make_card("Teferi, Hero of Dominaria");
        card.scryfall_data.type_line = Some("Legendary Planeswalker — Teferi".to_string());
        assert!(is_valid_commander(&card, &Format::Brawl));
    }

    #[test]
    fn legendary_planeswalker_fails_commander() {
        let mut card = make_card("Teferi, Hero of Dominaria");
        card.scryfall_data.type_line = Some("Legendary Planeswalker — Teferi".to_string());
        assert!(!is_valid_commander(&card, &Format::Commander));
    }

    #[test]
    fn can_be_your_commander_passes_commander() {
        let mut card = make_card("Grist, the Hunger Tide");
        card.scryfall_data.type_line = Some("Legendary Planeswalker — Grist".to_string());
        card.scryfall_data.oracle_text =
            Some("Grist, the Hunger Tide can be your commander.".to_string());
        assert!(is_valid_commander(&card, &Format::Commander));
    }

    #[test]
    fn legendary_vehicle_with_pt_passes_commander() {
        let mut card = make_card("Parhelion II");
        card.scryfall_data.type_line = Some("Legendary Artifact — Vehicle".to_string());
        card.scryfall_data.power = Some("5".to_string());
        card.scryfall_data.toughness = Some("5".to_string());
        assert!(is_valid_commander(&card, &Format::Commander));
    }

    #[test]
    fn regular_planeswalker_passes_oathbreaker() {
        let mut card = make_card("Nissa, Who Shakes the World");
        card.scryfall_data.type_line = Some("Legendary Planeswalker — Nissa".to_string());
        assert!(is_valid_commander(&card, &Format::Oathbreaker));
    }

    #[test]
    fn regular_planeswalker_fails_commander() {
        let mut card = make_card("Nissa, Who Shakes the World");
        card.scryfall_data.type_line = Some("Legendary Planeswalker — Nissa".to_string());
        assert!(!is_valid_commander(&card, &Format::Commander));
    }

    #[test]
    fn non_commander_format_returns_false() {
        let mut card = make_card("Atraxa, Praetors' Voice");
        card.scryfall_data.type_line =
            Some("Legendary Creature — Phyrexian Angel Horror".to_string());
        assert!(!is_valid_commander(&card, &Format::Standard));
        assert!(!is_valid_commander(&card, &Format::Modern));
    }
}
