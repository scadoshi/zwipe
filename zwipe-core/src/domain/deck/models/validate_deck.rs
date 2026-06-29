//! Deck validation logic.
//!
//! Generates informational warnings about deck-building rule violations
//! based on the deck's format. Does not prevent invalid states.

use crate::domain::{
    card::{
        scryfall_data::legalities::LegalityKind,
        search_card::commander_eligibility::{
            are_valid_partners, has_choose_a_background, is_background_card,
            is_signature_spell_in_color_identity, is_valid_commander,
            is_valid_signature_spell_type, partner_kind,
        },
        Card,
    },
    deck::{deck::DeckEntry, DeckProfile, DeckWarning, Format, WarningAction},
};

/// Cards in the command zone (stored on the profile, not in deck_cards).
pub struct DeckCommandZone<'a> {
    /// Primary commander (or oathbreaker planeswalker).
    pub commander: Option<&'a Card>,
    /// Partner commander (Partner / Friends Forever / Doctor's Companion).
    pub partner_commander: Option<&'a Card>,
    /// Background enchantment (Choose a Background).
    pub background: Option<&'a Card>,
    /// Signature spell (Oathbreaker instant/sorcery).
    pub signature_spell: Option<&'a Card>,
}

/// Validates a deck against its format rules and returns warnings.
///
/// If the deck has no format set, returns an empty list.
/// The `command_zone` provides card data for commander/partner/background/spell
/// when they are not part of the deck entries.
pub fn validate_deck(
    deck_profile: &DeckProfile,
    entries: &[DeckEntry],
    command_zone: &DeckCommandZone,
) -> Vec<DeckWarning> {
    let Some(format) = &deck_profile.format else {
        return vec![];
    };

    // Only validate active deck cards, not maybeboard or sideboard
    let active_entries: Vec<DeckEntry> = entries
        .iter()
        .filter(|e| e.deck_card.board.is_active())
        .cloned()
        .collect();

    let mut warnings = Vec::new();

    check_card_count(format, deck_profile, &mut warnings);
    check_land_target(format, deck_profile, &active_entries, &mut warnings);
    check_commander_required(format, deck_profile, &mut warnings);
    check_legality(format, &active_entries, &mut warnings);
    check_copy_limits(format, &active_entries, &mut warnings);
    check_color_identity(format, &active_entries, command_zone, &mut warnings);
    check_commander_eligibility(format, deck_profile, command_zone, &mut warnings);
    check_partner_validity(format, deck_profile, command_zone, &mut warnings);
    check_background_validity(format, deck_profile, command_zone, &mut warnings);
    check_signature_spell_validity(format, deck_profile, command_zone, &mut warnings);
    check_sideboard_limits(format, entries, &mut warnings);

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
    if profile.partner_commander_id.is_some() {
        count += 1;
    }
    if profile.background_id.is_some() {
        count += 1;
    }
    if profile.signature_spell_id.is_some() {
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

/// Warns when the mainboard has fewer lands than the deck's land target — the
/// user's explicit override if set, otherwise the format heuristic. Mirrors the
/// other format-derived checks, which warn off the format's rules regardless.
fn check_land_target(
    format: &Format,
    profile: &DeckProfile,
    active_entries: &[DeckEntry],
    warnings: &mut Vec<DeckWarning>,
) {
    let Some(target) = profile.land_target.or_else(|| format.default_land_target()) else {
        return;
    };
    let land_count: i32 = active_entries
        .iter()
        .filter(|e| e.card.scryfall_data.is_land())
        .map(|e| *e.deck_card.quantity)
        .sum();
    if land_count < target {
        let land_word = if land_count == 1 { "land" } else { "lands" };
        warnings.push(DeckWarning::new(format!(
            "deck has {land_count} {land_word}, below the land target of {target}"
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
            warnings.push(DeckWarning::with_action(
                format!(
                    "{} exceeds copy limit ({}/{})",
                    entry.card.scryfall_data.name.to_lowercase(), qty, max
                ),
                entry.card.scryfall_data.id,
                WarningAction::FixQuantity(max as i32),
            ));
        }
    }
}

fn check_color_identity(
    format: &Format,
    entries: &[DeckEntry],
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    if !format.checks_color_identity() {
        return;
    }

    // Resolve commander oracle_id from command zone (always populated by get_deck)
    let Some(commander_oid) = command_zone.commander.and_then(|c| c.scryfall_data.oracle_id) else {
        return;
    };

    // Collect the commander's color identity (check entries first, fall back to command zone)
    let commander_identity = entries
        .iter()
        .find(|e| e.card.scryfall_data.oracle_id == Some(commander_oid))
        .map(|e| &e.card.scryfall_data.color_identity)
        .or_else(|| command_zone.commander.map(|c| &c.scryfall_data.color_identity));

    let Some(base_colors) = commander_identity else {
        return;
    };

    // Build unified color identity: commander + partner + background
    let mut allowed_colors: Vec<_> = base_colors.iter().collect();

    if let Some(partner) = command_zone.partner_commander {
        for color in partner.scryfall_data.color_identity.iter() {
            if !allowed_colors.contains(&color) {
                allowed_colors.push(color);
            }
        }
    }

    if let Some(bg) = command_zone.background {
        for color in bg.scryfall_data.color_identity.iter() {
            if !allowed_colors.contains(&color) {
                allowed_colors.push(color);
            }
        }
    }

    // Collect command zone oracle_ids to skip in color identity checks
    let partner_oid = command_zone.partner_commander.and_then(|c| c.scryfall_data.oracle_id);
    let bg_oid = command_zone.background.and_then(|c| c.scryfall_data.oracle_id);

    for entry in entries {
        if entry.card.scryfall_data.oracle_id == Some(commander_oid) {
            continue;
        }
        if partner_oid.is_some() && entry.card.scryfall_data.oracle_id == partner_oid {
            continue;
        }
        if bg_oid.is_some() && entry.card.scryfall_data.oracle_id == bg_oid {
            continue;
        }

        for color in entry.card.scryfall_data.color_identity.iter() {
            if !allowed_colors.contains(&color) {
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
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    if !format.has_commander() {
        return;
    }

    if profile.commander_id.is_none() {
        return; // Already warned by check_commander_required
    };

    // Use command zone directly (always populated by get_deck)
    let Some(card) = command_zone.commander else {
        return; // Can't validate without the card data
    };

    if !is_valid_commander(card, format) {
        warnings.push(DeckWarning::with_action(
            format!(
                "{} is not a valid commander for {}",
                card.scryfall_data.name.to_lowercase(),
                format.display_name().to_lowercase()
            ),
            card.scryfall_data.id,
            WarningAction::ClearCommander,
        ));
    }
}

fn check_partner_validity(
    format: &Format,
    profile: &DeckProfile,
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    let Some(partner_id) = profile.partner_commander_id else {
        return;
    };

    if !format.supports_partner() {
        warnings.push(DeckWarning::new(format!(
            "{} does not support partner commanders",
            format.display_name().to_lowercase()
        )));
        return;
    }

    if let (Some(commander), Some(partner)) =
        (command_zone.commander, command_zone.partner_commander)
    {
        // Both must have a partner ability
        if partner_kind(commander).is_none() {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} does not have a partner ability",
                    commander.scryfall_data.name.to_lowercase()
                ),
                commander.scryfall_data.id,
            ));
            return;
        }

        if !are_valid_partners(commander, partner) {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} and {} cannot be partners",
                    commander.scryfall_data.name.to_lowercase(),
                    partner.scryfall_data.name.to_lowercase()
                ),
                partner_id,
            ));
        }
    }
}

fn check_background_validity(
    format: &Format,
    profile: &DeckProfile,
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    let Some(bg_id) = profile.background_id else {
        return;
    };

    if !format.supports_background() {
        warnings.push(DeckWarning::new(format!(
            "{} does not support backgrounds",
            format.display_name().to_lowercase()
        )));
        return;
    }

    // Commander must have "Choose a Background"
    if let Some(commander) = command_zone.commander
        && !has_choose_a_background(commander)
    {
        warnings.push(DeckWarning::with_card(
            format!(
                "{} does not have 'choose a background'",
                commander.scryfall_data.name.to_lowercase()
            ),
            commander.scryfall_data.id,
        ));
    }

    // Background card must actually be a Background
    if let Some(bg) = command_zone.background
        && !is_background_card(bg)
    {
        warnings.push(DeckWarning::with_card(
            format!(
                "{} is not a valid background enchantment",
                bg.scryfall_data.name.to_lowercase()
            ),
            bg_id,
        ));
    }

    // Mutual exclusivity: can't have both partner and background
    if profile.partner_commander_id.is_some() {
        warnings.push(DeckWarning::new(
            "a commander cannot have both a partner and a background".to_string(),
        ));
    }
}

fn check_sideboard_limits(
    format: &Format,
    entries: &[DeckEntry],
    warnings: &mut Vec<DeckWarning>,
) {
    let sideboard_count: u32 = entries
        .iter()
        .filter(|e| e.deck_card.board.is_sideboard())
        .map(|e| *e.deck_card.quantity as u32)
        .sum();

    if sideboard_count == 0 {
        return;
    }

    if !format.has_sideboard() {
        warnings.push(DeckWarning::new(format!(
            "{} does not use sideboards",
            format.display_name().to_lowercase()
        )));
        return;
    }

    if let Some(max) = format.sideboard_max()
        && sideboard_count > max
    {
        warnings.push(DeckWarning::new(format!(
            "sideboard has {} cards, {} allows at most {}",
            sideboard_count,
            format.display_name().to_lowercase(),
            max
        )));
    }
}

fn check_signature_spell_validity(
    format: &Format,
    profile: &DeckProfile,
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    if profile.signature_spell_id.is_none() {
        // Warn if format requires one but none selected
        if format.has_signature_spell() {
            warnings.push(DeckWarning::new(
                "oathbreaker format requires a signature spell".to_string(),
            ));
        }
        return;
    }

    if !format.has_signature_spell() {
        warnings.push(DeckWarning::new(format!(
            "{} does not use signature spells",
            format.display_name().to_lowercase()
        )));
        return;
    }

    if let Some(spell) = command_zone.signature_spell {
        // Must be instant or sorcery
        if !is_valid_signature_spell_type(spell) {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} must be an instant or sorcery to be a signature spell",
                    spell.scryfall_data.name.to_lowercase()
                ),
                spell.scryfall_data.id,
            ));
        }

        // Must be within oathbreaker's color identity
        if let Some(oathbreaker) = command_zone.commander
            && !is_signature_spell_in_color_identity(spell, oathbreaker)
        {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} is outside the oathbreaker's color identity",
                    spell.scryfall_data.name.to_lowercase()
                ),
                spell.scryfall_data.id,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::deck::DeckName;

    fn empty_command_zone() -> DeckCommandZone<'static> {
        DeckCommandZone {
            commander: None,
            partner_commander: None,
            background: None,
            signature_spell: None,
        }
    }

    #[test]
    fn no_warnings_without_format() {
        let profile = test_profile(None);
        let warnings = validate_deck(&profile, &[], &empty_command_zone());
        assert!(warnings.is_empty());
    }

    fn test_profile(format: Option<Format>) -> DeckProfile {
        DeckProfile {
            id: uuid::Uuid::new_v4(),
            name: DeckName::new("test").unwrap(),
            commander_id: None,
            partner_commander_id: None,
            background_id: None,
            signature_spell_id: None,
            format,
            tags: Vec::new(),
            land_target: None,
            user_id: uuid::Uuid::new_v4(),
            card_count: 0,
            commander_name: None,
            partner_commander_name: None,
            background_name: None,
            signature_spell_name: None,
        }
    }

    mod land_target {
        use super::*;
        use crate::domain::deck::{Board, DeckCard, Quantity};
        use crate::test_utils::make_card;

        fn land_entry(name: &str, qty: i32) -> DeckEntry {
            let mut card = make_card(name);
            card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
            let deck_card = DeckCard {
                deck_id: uuid::Uuid::new_v4(),
                scryfall_data_id: card.scryfall_data.id,
                oracle_id: card.scryfall_data.oracle_id.unwrap_or_default(),
                quantity: Quantity::new(qty).unwrap(),
                board: Board::Deck,
            };
            DeckEntry { card, deck_card }
        }

        #[test]
        fn warns_when_below_explicit_target() {
            let mut profile = test_profile(Some(Format::Commander));
            profile.land_target = Some(5);
            let warnings =
                validate_deck(&profile, &[land_entry("Forest", 3)], &empty_command_zone());
            assert!(warnings.iter().any(|w| w.contains("land target")));
        }

        #[test]
        fn no_warning_at_or_above_target() {
            let mut profile = test_profile(Some(Format::Commander));
            profile.land_target = Some(3);
            let warnings =
                validate_deck(&profile, &[land_entry("Forest", 3)], &empty_command_zone());
            assert!(!warnings.iter().any(|w| w.contains("land target")));
        }

        #[test]
        fn warns_against_format_heuristic_when_no_explicit_target() {
            // No explicit target, but a Commander deck with no lands is below the
            // format heuristic (37), so the warning fires like the other rules.
            let profile = test_profile(Some(Format::Commander));
            let warnings = validate_deck(&profile, &[], &empty_command_zone());
            assert!(warnings.iter().any(|w| w.contains("land target")));
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

            let cz = DeckCommandZone {
                commander: Some(&card),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[], &cz);
            assert!(
                !warnings
                    .iter()
                    .any(|w| w.to_string().contains("not a valid commander")),
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

            let cz = DeckCommandZone {
                commander: Some(&card),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[], &cz);
            assert!(
                warnings
                    .iter()
                    .any(|w| w.to_string().contains("not a valid commander")),
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

            let cz = DeckCommandZone {
                commander: Some(&card),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[], &cz);
            assert!(
                !warnings
                    .iter()
                    .any(|w| w.to_string().contains("not a valid commander")),
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

            let cz = DeckCommandZone {
                commander: Some(&card),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[], &cz);
            assert!(
                warnings
                    .iter()
                    .any(|w| w.to_string().contains("not a valid commander")),
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

            let cz = DeckCommandZone {
                commander: Some(&card),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[], &cz);
            assert!(
                !warnings
                    .iter()
                    .any(|w| w.to_string().contains("not a valid commander")),
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

            let cz = DeckCommandZone {
                commander: Some(&card),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[], &cz);
            assert!(
                warnings
                    .iter()
                    .any(|w| w.to_string().contains("not a valid commander")),
                "expected commander eligibility warning for rare creature"
            );
        }

        /// Commander stored with printing A, command_zone has same card
        /// with a different printing (different scryfall_data.id, same oracle_id).
        /// Eligibility check should still find the commander via oracle_id.
        #[test]
        fn different_printing_still_validates_commander() {
            let mut card = make_card("Atraxa, Praetors' Voice");
            card.scryfall_data.type_line =
                Some("Legendary Creature — Phyrexian Angel Horror".to_string());
            // Profile stores a different scryfall_data_id (different printing)
            let different_printing_id = uuid::Uuid::new_v4();
            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(different_printing_id);

            // Command zone has the card (same oracle_id, different scryfall_data.id)
            let cz = DeckCommandZone {
                commander: Some(&card),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[], &cz);
            assert!(
                !warnings
                    .iter()
                    .any(|w| w.to_string().contains("not a valid commander")),
                "different printing should still be recognized as valid commander"
            );
        }
    }

    mod color_identity {
        use super::*;
        use crate::domain::card::scryfall_data::colors::{Color, Colors};
        use crate::test_utils::{make_card, make_entry};

        #[test]
        fn card_outside_commander_color_identity_warns() {
            let mut commander = make_card("Omnath, Locus of Creation");
            commander.scryfall_data.type_line =
                Some("Legendary Creature — Elemental".to_string());
            commander.scryfall_data.color_identity =
                Colors::from([Color::Red, Color::Green, Color::White, Color::Blue]);

            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(commander.scryfall_data.id);

            // Card with black — outside RGWU identity
            let mut entry = make_entry("Doom Blade", 1);
            entry.card.scryfall_data.color_identity = Colors::from([Color::Black]);

            let cz = DeckCommandZone {
                commander: Some(&commander),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[entry], &cz);
            assert!(
                warnings
                    .iter()
                    .any(|w| w.to_string().contains("outside commander's color identity")),
                "expected color identity warning for black card in RGWU deck"
            );
        }

        #[test]
        fn card_within_commander_color_identity_no_warning() {
            let mut commander = make_card("Omnath, Locus of Creation");
            commander.scryfall_data.type_line =
                Some("Legendary Creature — Elemental".to_string());
            commander.scryfall_data.color_identity =
                Colors::from([Color::Red, Color::Green, Color::White, Color::Blue]);

            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(commander.scryfall_data.id);

            let mut entry = make_entry("Lightning Bolt", 1);
            entry.card.scryfall_data.color_identity = Colors::from([Color::Red]);

            let cz = DeckCommandZone {
                commander: Some(&commander),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[entry], &cz);
            assert!(
                !warnings
                    .iter()
                    .any(|w| w.to_string().contains("outside commander's color identity")),
                "expected no color identity warning for red card in RGWU deck"
            );
        }

        /// Commander is in entries with a different printing than the profile stores.
        /// Color identity check should still skip the commander entry (matched by oracle_id).
        #[test]
        fn commander_in_entries_different_printing_not_warned() {
            let mut commander = make_card("Atraxa, Praetors' Voice");
            commander.scryfall_data.type_line =
                Some("Legendary Creature — Phyrexian Angel Horror".to_string());
            commander.scryfall_data.color_identity =
                Colors::from([Color::White, Color::Blue, Color::Black, Color::Green]);
            let oracle_id = commander.scryfall_data.oracle_id;

            // Profile stores a different printing
            let different_printing_id = uuid::Uuid::new_v4();
            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(different_printing_id);

            // Entry has the commander as a deck card (same oracle_id, different scryfall_data.id)
            let mut entry = make_entry("Atraxa, Praetors' Voice", 1);
            entry.card.scryfall_data.oracle_id = oracle_id;
            entry.card.scryfall_data.color_identity =
                Colors::from([Color::White, Color::Blue, Color::Black, Color::Green]);

            let cz = DeckCommandZone {
                commander: Some(&commander),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[entry], &cz);
            assert!(
                !warnings
                    .iter()
                    .any(|w| w.to_string().contains("outside commander's color identity")),
                "commander entry should be skipped by oracle_id, not warned about"
            );
        }

        /// Partner commander's color identity should extend allowed colors.
        /// Cards within the combined identity should not warn.
        #[test]
        fn partner_extends_color_identity() {
            let mut commander = make_card("Tymna the Weaver");
            commander.scryfall_data.type_line =
                Some("Legendary Creature — Human Cleric".to_string());
            commander.scryfall_data.color_identity =
                Colors::from([Color::White, Color::Black]);

            let mut partner = make_card("Thrasios, Triton Hero");
            partner.scryfall_data.type_line =
                Some("Legendary Creature — Merfolk Wizard".to_string());
            partner.scryfall_data.color_identity =
                Colors::from([Color::Green, Color::Blue]);

            let mut profile = test_profile(Some(Format::Commander));
            profile.commander_id = Some(commander.scryfall_data.id);
            profile.partner_commander_id = Some(partner.scryfall_data.id);

            // Blue card — within partner's identity but not commander's
            let mut entry = make_entry("Counterspell", 1);
            entry.card.scryfall_data.color_identity = Colors::from([Color::Blue]);

            let cz = DeckCommandZone {
                commander: Some(&commander),
                partner_commander: Some(&partner),
                ..empty_command_zone()
            };
            let warnings = validate_deck(&profile, &[entry], &cz);
            assert!(
                !warnings
                    .iter()
                    .any(|w| w.to_string().contains("outside commander's color identity")),
                "blue card should be allowed via partner's color identity"
            );
        }
    }
}
