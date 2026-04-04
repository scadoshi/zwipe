//! Deck validation logic.
//!
//! Generates informational warnings about deck-building rule violations
//! based on the deck's format. Does not prevent invalid states.

use crate::domain::{
    card::{scryfall_data::legalities::LegalityKind, Card},
    deck::{deck::DeckEntry, DeckProfile, DeckWarning, Format},
};

/// Validates a deck against its format rules and returns warnings.
///
/// If the deck has no format set, returns an empty list.
/// The optional `commander_card` is needed for color identity checks
/// when the commander is not part of the deck entries.
pub fn validate_deck(
    deck_profile: &DeckProfile,
    entries: &[DeckEntry],
    commander_card: Option<&Card>,
) -> Vec<DeckWarning> {
    let Some(format) = &deck_profile.format else {
        return vec![];
    };

    let mut warnings = Vec::new();

    check_card_count(format, deck_profile, &mut warnings);
    check_commander_required(format, deck_profile, &mut warnings);
    check_legality(format, entries, &mut warnings);
    check_copy_limits(format, entries, &mut warnings);
    check_color_identity(format, deck_profile, entries, commander_card, &mut warnings);
    check_commander_eligibility(format, deck_profile, entries, commander_card, &mut warnings);

    warnings
}

fn plural(n: u32) -> &'static str {
    if n == 1 { "card" } else { "cards" }
}

fn check_card_count(format: &Format, profile: &DeckProfile, warnings: &mut Vec<DeckWarning>) {
    let mut count = profile.card_count as u32;

    // The commander is stored separately from deck entries, so include it in the total.
    if format.has_commander() && profile.commander_id.is_some() {
        count += 1;
    }

    if let Some(min) = format.min_cards()
        && count < min
    {
        warnings.push(DeckWarning::new(format!(
            "deck has {} {}, {} requires at least {}",
            count,
            plural(count),
            format.display_name().to_lowercase(),
            min
        )));
    }

    if let Some(max) = format.max_cards()
        && count > max
    {
        warnings.push(DeckWarning::new(format!(
            "deck has {} {}, {} allows at most {}",
            count,
            plural(count),
            format.display_name().to_lowercase(),
            max
        )));
    }
}

fn check_commander_required(
    format: &Format,
    profile: &DeckProfile,
    warnings: &mut Vec<DeckWarning>,
) {
    if format.has_commander() && profile.commander_id.is_none() {
        warnings.push(DeckWarning::new(format!(
            "the format, {}, requires a commander",
            format.display_name().to_lowercase()
        )));
    }
}

fn check_legality(format: &Format, entries: &[DeckEntry], warnings: &mut Vec<DeckWarning>) {
    for entry in entries {
        let legality = entry.card.scryfall_data.legalities.get(format);

        match legality {
            Some(LegalityKind::NotLegal) => {
                warnings.push(DeckWarning::with_card(
                    format!(
                        "{} is not legal in {}",
                        entry.card.scryfall_data.name.to_lowercase(),
                        format.display_name().to_lowercase()
                    ),
                    entry.card.scryfall_data.id,
                ));
            }
            Some(LegalityKind::Banned) => {
                warnings.push(DeckWarning::with_card(
                    format!(
                        "{} is banned in {}",
                        entry.card.scryfall_data.name.to_lowercase(),
                        format.display_name().to_lowercase()
                    ),
                    entry.card.scryfall_data.id,
                ));
            }
            _ => {}
        }
    }
}

fn check_copy_limits(format: &Format, entries: &[DeckEntry], warnings: &mut Vec<DeckWarning>) {
    let base_max = format.copy_max();

    for entry in entries {
        if entry.card.scryfall_data.is_basic_land() {
            continue;
        }

        let qty = *entry.deck_card.quantity as u32;

        // Vintage restricted cards are limited to 1 copy
        let max = if *format == Format::Vintage {
            match entry.card.scryfall_data.legalities.get(format) {
                Some(LegalityKind::Restricted) => 1,
                _ => base_max,
            }
        } else {
            base_max
        };

        if qty > max {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} exceeds copy limit ({}/{})",
                    entry.card.scryfall_data.name.to_lowercase(), qty, max
                ),
                entry.card.scryfall_data.id,
            ));
        }
    }
}

fn check_color_identity(
    format: &Format,
    profile: &DeckProfile,
    entries: &[DeckEntry],
    commander_card: Option<&Card>,
    warnings: &mut Vec<DeckWarning>,
) {
    if !format.checks_color_identity() {
        return;
    }

    // Find commander's color identity — check entries first, fall back to provided card
    let Some(commander_id) = profile.commander_id else {
        return;
    };
    let commander_identity = entries
        .iter()
        .find(|e| e.card.scryfall_data.id == commander_id)
        .map(|e| &e.card.scryfall_data.color_identity)
        .or_else(|| commander_card.map(|c| &c.scryfall_data.color_identity));

    let Some(commander_colors) = commander_identity else {
        return;
    };

    for entry in entries {
        if entry.card.scryfall_data.id == commander_id {
            continue;
        }

        for color in entry.card.scryfall_data.color_identity.iter() {
            if !commander_colors.contains(color) {
                warnings.push(DeckWarning::with_card(
                    format!(
                        "{} is outside commander's color identity",
                        entry.card.scryfall_data.name.to_lowercase()
                    ),
                    entry.card.scryfall_data.id,
                ));
                break;
            }
        }
    }
}

fn check_commander_eligibility(
    format: &Format,
    profile: &DeckProfile,
    entries: &[DeckEntry],
    commander_card: Option<&Card>,
    warnings: &mut Vec<DeckWarning>,
) {
    if !format.has_commander() {
        return;
    }

    let Some(commander_id) = profile.commander_id else {
        return; // Already warned by check_commander_required
    };

    // Find the commander card from entries or the provided card
    let commander = entries
        .iter()
        .find(|e| e.card.scryfall_data.id == commander_id)
        .map(|e| &e.card)
        .or(commander_card);

    let Some(card) = commander else {
        return; // Can't validate without the card data
    };

    use crate::domain::card::search_card::commander_eligibility::is_valid_commander;

    if !is_valid_commander(card, format) {
        warnings.push(DeckWarning::with_card(
            format!(
                "{} is not a valid commander for {}",
                card.scryfall_data.name.to_lowercase(),
                format.display_name().to_lowercase()
            ),
            card.scryfall_data.id,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::deck::DeckName;

    #[test]
    fn no_warnings_without_format() {
        let profile = test_profile(None);
        let warnings = validate_deck(&profile, &[], None);
        assert!(warnings.is_empty());
    }

    fn test_profile(format: Option<Format>) -> DeckProfile {
        DeckProfile {
            id: uuid::Uuid::new_v4(),
            name: DeckName::new("test").unwrap(),
            commander_id: None,
            format,
            user_id: uuid::Uuid::new_v4(),
            card_count: 0,
            commander_name: None,
        }
    }

    mod commander_eligibility {
        use super::*;
        use crate::domain::card::scryfall_data::rarity::Rarity;
        use crate::test_utils::make_card;

        #[test]
        fn valid_legendary_creature_no_warning() {
            let mut card = make_card("Atraxa, Praetors' Voice");
            card.scryfall_data.type_line =
                Some("Legendary Creature — Phyrexian Angel Horror".to_string());
            let commander_id = card.scryfall_data.id;

            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(commander_id);

            let warnings = validate_deck(&profile, &[], Some(&card));
            assert!(
                !warnings.iter().any(|w| w.to_string().contains("not a valid commander")),
                "expected no commander eligibility warning"
            );
        }

        #[test]
        fn non_legendary_creature_as_commander_warns() {
            let mut card = make_card("Llanowar Elves");
            card.scryfall_data.type_line = Some("Creature — Elf Druid".to_string());
            let commander_id = card.scryfall_data.id;

            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(commander_id);

            let warnings = validate_deck(&profile, &[], Some(&card));
            assert!(
                warnings.iter().any(|w| w.to_string().contains("not a valid commander")),
                "expected commander eligibility warning"
            );
        }

        #[test]
        fn planeswalker_as_brawl_commander_no_warning() {
            let mut card = make_card("Teferi, Hero of Dominaria");
            card.scryfall_data.type_line =
                Some("Legendary Planeswalker — Teferi".to_string());
            let commander_id = card.scryfall_data.id;

            let mut profile = test_profile(Some(Format::Brawl));
            profile.commander_id = Some(commander_id);

            let warnings = validate_deck(&profile, &[], Some(&card));
            assert!(
                !warnings.iter().any(|w| w.to_string().contains("not a valid commander")),
                "expected no commander eligibility warning for brawl planeswalker"
            );
        }

        #[test]
        fn planeswalker_as_commander_format_warns() {
            let mut card = make_card("Teferi, Hero of Dominaria");
            card.scryfall_data.type_line =
                Some("Legendary Planeswalker — Teferi".to_string());
            let commander_id = card.scryfall_data.id;

            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(commander_id);

            let warnings = validate_deck(&profile, &[], Some(&card));
            assert!(
                warnings.iter().any(|w| w.to_string().contains("not a valid commander")),
                "expected commander eligibility warning"
            );
        }

        #[test]
        fn uncommon_creature_as_pauper_commander_no_warning() {
            let mut card = make_card("Burning-Tree Emissary");
            card.scryfall_data.type_line = Some("Creature — Human Shaman".to_string());
            card.scryfall_data.rarity = Rarity::Uncommon;
            let commander_id = card.scryfall_data.id;

            let mut profile = test_profile(Some(Format::PauperCommander));
            profile.commander_id = Some(commander_id);

            let warnings = validate_deck(&profile, &[], Some(&card));
            assert!(
                !warnings.iter().any(|w| w.to_string().contains("not a valid commander")),
                "expected no commander eligibility warning for uncommon creature"
            );
        }

        #[test]
        fn rare_creature_as_pauper_commander_warns() {
            let mut card = make_card("Tarmogoyf");
            card.scryfall_data.type_line = Some("Creature — Lhurgoyf".to_string());
            card.scryfall_data.rarity = Rarity::Rare;
            let commander_id = card.scryfall_data.id;

            let mut profile = test_profile(Some(Format::PauperCommander));
            profile.commander_id = Some(commander_id);

            let warnings = validate_deck(&profile, &[], Some(&card));
            assert!(
                warnings.iter().any(|w| w.to_string().contains("not a valid commander")),
                "expected commander eligibility warning for rare creature"
            );
        }
    }
}
