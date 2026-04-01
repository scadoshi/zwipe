//! Deck validation logic.
//!
//! Generates informational warnings about deck-building rule violations
//! based on the deck's format. Does not prevent invalid states.

use crate::domain::{
    card::models::{scryfall_data::legalities::LegalityKind, Card},
    deck::models::deck::{
        deck_profile::DeckProfile, deck_warning::DeckWarning, format::Format, DeckEntry,
    },
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

    if let Some(min) = format.min_cards() {
        if count < min {
            warnings.push(DeckWarning::new(format!(
                "deck has {} {}, {} requires at least {}",
                count,
                plural(count),
                format.display_name().to_lowercase(),
                min
            )));
        }
    }

    if let Some(max) = format.max_cards() {
        if count > max {
            warnings.push(DeckWarning::new(format!(
                "deck has {} {}, {} allows at most {}",
                count,
                plural(count),
                format.display_name().to_lowercase(),
                max
            )));
        }
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
                warnings.push(DeckWarning::new(format!(
                    "{} is not legal in {}",
                    entry.card.scryfall_data.name.to_lowercase(),
                    format.display_name().to_lowercase()
                )));
            }
            Some(LegalityKind::Banned) => {
                warnings.push(DeckWarning::new(format!(
                    "{} is banned in {}",
                    entry.card.scryfall_data.name.to_lowercase(),
                    format.display_name().to_lowercase()
                )));
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
            warnings.push(DeckWarning::new(format!(
                "{} exceeds copy limit ({}/{})",
                entry.card.scryfall_data.name.to_lowercase(), qty, max
            )));
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
    if !format.checks_color_identity() || profile.commander_id.is_none() {
        return;
    }

    // Find commander's color identity — check entries first, fall back to provided card
    let commander_id = profile.commander_id.unwrap();
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
                warnings.push(DeckWarning::new(format!(
                    "{} is outside commander's color identity",
                    entry.card.scryfall_data.name.to_lowercase()
                )));
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_warnings_without_format() {
        let profile = test_profile(None);
        let warnings = validate_deck(&profile, &[], None);
        assert!(warnings.is_empty());
    }

    fn test_profile(format: Option<Format>) -> DeckProfile {
        DeckProfile {
            id: uuid::Uuid::new_v4(),
            name: crate::domain::deck::models::deck::deck_name::DeckName::new("test").unwrap(),
            commander_id: None,
            format,
            user_id: uuid::Uuid::new_v4(),
            card_count: 0,
            commander_name: None,
        }
    }
}
