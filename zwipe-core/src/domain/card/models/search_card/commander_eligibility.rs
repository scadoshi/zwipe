//! Commander eligibility rules by format.
//!
//! This is the authoritative definition of commander eligibility.
//! The SQL filter in zerver's `search_scryfall_data` must mirror this logic.

use crate::domain::{card::Card, deck::Format};

// =================================
// Commander Eligibility
// =================================

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

// =================================
// Partner Eligibility
// =================================

/// The kind of partner ability a card has, if any.
#[derive(Debug, Clone, PartialEq)]
pub enum PartnerKind {
    /// Generic "Partner" keyword — compatible with any other Generic partner.
    Generic,
    /// "Partner with [Name]" — compatible only with the named card.
    Named(String),
    /// "Friends forever" — compatible with any other FriendsForever card.
    FriendsForever,
    /// "Doctor's companion" — compatible with Time Lord Doctor cards.
    DoctorsCompanion,
}

/// Returns the partner kind for a card, if it has one.
pub fn partner_kind(card: &Card) -> Option<PartnerKind> {
    let sd = &card.scryfall_data;
    let oracle_text = sd.oracle_text.as_deref().unwrap_or("");
    let keywords = sd.keywords.as_deref().unwrap_or(&[]);

    // Check named partner first (more specific than generic Partner keyword)
    if let Some(name) = extract_named_partner(oracle_text) {
        return Some(PartnerKind::Named(name));
    }

    if keywords.iter().any(|k| k == "Friends forever") {
        return Some(PartnerKind::FriendsForever);
    }

    if keywords.iter().any(|k| k == "Doctor's companion") {
        return Some(PartnerKind::DoctorsCompanion);
    }

    // Generic Partner — has "Partner" keyword but NOT "Partner with"
    if keywords.iter().any(|k| k == "Partner") {
        return Some(PartnerKind::Generic);
    }

    None
}

/// Checks whether two cards can be partners.
pub fn are_valid_partners(card_a: &Card, card_b: &Card) -> bool {
    let kind_a = partner_kind(card_a);
    let kind_b = partner_kind(card_b);

    match (kind_a, kind_b) {
        (Some(PartnerKind::Generic), Some(PartnerKind::Generic)) => true,
        (Some(PartnerKind::Named(name)), _) => {
            card_b.scryfall_data.name.eq_ignore_ascii_case(&name)
        }
        (_, Some(PartnerKind::Named(name))) => {
            card_a.scryfall_data.name.eq_ignore_ascii_case(&name)
        }
        (Some(PartnerKind::FriendsForever), Some(PartnerKind::FriendsForever)) => true,
        (Some(PartnerKind::DoctorsCompanion), _) => card_b
            .scryfall_data
            .type_line
            .as_deref()
            .unwrap_or("")
            .contains("Time Lord Doctor"),
        (_, Some(PartnerKind::DoctorsCompanion)) => card_a
            .scryfall_data
            .type_line
            .as_deref()
            .unwrap_or("")
            .contains("Time Lord Doctor"),
        _ => false,
    }
}

/// Extracts the partner name from oracle text like "Partner with Brallin, Skyshark Rider".
fn extract_named_partner(oracle_text: &str) -> Option<String> {
    let lower = oracle_text.to_lowercase();
    let pos = lower.find("partner with ")?;
    let after = &oracle_text[pos + "partner with ".len()..];
    // Partner name ends at newline, period, or parenthesis
    let end = after.find(['\n', '(']).unwrap_or(after.len());
    let name = after[..end].trim().trim_end_matches('.');
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

// =================================
// Background Eligibility
// =================================

/// Whether a card has "Choose a Background" (making it background-eligible as commander).
pub fn has_choose_a_background(card: &Card) -> bool {
    card.scryfall_data
        .oracle_text
        .as_deref()
        .unwrap_or("")
        .to_lowercase()
        .contains("choose a background")
}

/// Whether a card is a Background enchantment.
pub fn is_background_card(card: &Card) -> bool {
    let type_line = card.scryfall_data.type_line.as_deref().unwrap_or("");
    type_line.contains("Legendary")
        && type_line.contains("Enchantment")
        && type_line.contains("Background")
}

// =================================
// Signature Spell Eligibility
// =================================

/// Whether a card is a valid signature spell type (instant or sorcery).
pub fn is_valid_signature_spell_type(card: &Card) -> bool {
    let type_line = card.scryfall_data.type_line.as_deref().unwrap_or("");
    type_line.contains("Instant") || type_line.contains("Sorcery")
}

/// Whether a signature spell is within the oathbreaker's color identity.
pub fn is_signature_spell_in_color_identity(spell: &Card, oathbreaker: &Card) -> bool {
    let ob_colors = &oathbreaker.scryfall_data.color_identity;
    for color in spell.scryfall_data.color_identity.iter() {
        if !ob_colors.contains(color) {
            return false;
        }
    }
    true
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

    // =================================
    // Partner Tests
    // =================================

    fn make_partner(name: &str, keyword: &str) -> Card {
        let mut card = make_card(name);
        card.scryfall_data.type_line =
            Some("Legendary Creature — Human Warrior".to_string());
        card.scryfall_data.keywords = Some(vec![keyword.to_string()]);
        card
    }

    #[test]
    fn generic_partner_detected() {
        let card = make_partner("Kodama of the East Tree", "Partner");
        assert_eq!(partner_kind(&card), Some(PartnerKind::Generic));
    }

    #[test]
    fn generic_partner_pair_valid() {
        let a = make_partner("Kodama of the East Tree", "Partner");
        let b = make_partner("Sakashima of a Thousand Faces", "Partner");
        assert!(are_valid_partners(&a, &b));
    }

    #[test]
    fn named_partner_detected() {
        let mut card = make_card("Brallin, Skyshark Rider");
        card.scryfall_data.type_line =
            Some("Legendary Creature — Human Shaman".to_string());
        card.scryfall_data.oracle_text =
            Some("Partner with Shabraz, the Skyshark".to_string());
        card.scryfall_data.keywords = Some(vec!["Partner with".to_string()]);
        assert_eq!(
            partner_kind(&card),
            Some(PartnerKind::Named("Shabraz, the Skyshark".to_string()))
        );
    }

    #[test]
    fn named_partner_pair_valid() {
        let mut a = make_card("Brallin, Skyshark Rider");
        a.scryfall_data.type_line =
            Some("Legendary Creature — Human Shaman".to_string());
        a.scryfall_data.oracle_text =
            Some("Partner with Shabraz, the Skyshark".to_string());

        let mut b = make_card("Shabraz, the Skyshark");
        b.scryfall_data.type_line =
            Some("Legendary Creature — Shark Bird".to_string());
        b.scryfall_data.oracle_text =
            Some("Partner with Brallin, Skyshark Rider".to_string());

        assert!(are_valid_partners(&a, &b));
    }

    #[test]
    fn named_partner_wrong_card_invalid() {
        let mut a = make_card("Brallin, Skyshark Rider");
        a.scryfall_data.type_line =
            Some("Legendary Creature — Human Shaman".to_string());
        a.scryfall_data.oracle_text =
            Some("Partner with Shabraz, the Skyshark".to_string());

        let b = make_partner("Kodama of the East Tree", "Partner");
        assert!(!are_valid_partners(&a, &b));
    }

    #[test]
    fn friends_forever_pair_valid() {
        let a = make_partner("Cecily, Haunted Mage", "Friends forever");
        let b = make_partner("Othelm, Sigardian Outcast", "Friends forever");
        assert!(are_valid_partners(&a, &b));
    }

    #[test]
    fn friends_forever_and_generic_partner_invalid() {
        let a = make_partner("Cecily, Haunted Mage", "Friends forever");
        let b = make_partner("Kodama of the East Tree", "Partner");
        assert!(!are_valid_partners(&a, &b));
    }

    #[test]
    fn doctors_companion_with_time_lord_doctor_valid() {
        let companion = make_partner("Jo Grant", "Doctor's companion");
        let mut doctor = make_card("The Fourth Doctor");
        doctor.scryfall_data.type_line =
            Some("Legendary Creature — Time Lord Doctor".to_string());
        doctor.scryfall_data.keywords = Some(vec![]);
        assert!(are_valid_partners(&companion, &doctor));
    }

    #[test]
    fn doctors_companion_with_non_doctor_invalid() {
        let companion = make_partner("Jo Grant", "Doctor's companion");
        let other = make_partner("Kodama of the East Tree", "Partner");
        assert!(!are_valid_partners(&companion, &other));
    }

    #[test]
    fn no_partner_ability_returns_none() {
        let card = make_card("Lightning Bolt");
        assert_eq!(partner_kind(&card), None);
    }

    // =================================
    // Background Tests
    // =================================

    #[test]
    fn choose_a_background_detected() {
        let mut card = make_card("Gut, True Soul Zealot");
        card.scryfall_data.oracle_text =
            Some("Choose a Background\nOther creatures you control have menace.".to_string());
        assert!(has_choose_a_background(&card));
    }

    #[test]
    fn no_choose_a_background() {
        let card = make_card("Kodama of the East Tree");
        assert!(!has_choose_a_background(&card));
    }

    #[test]
    fn background_card_detected() {
        let mut card = make_card("Criminal Past");
        card.scryfall_data.type_line =
            Some("Legendary Enchantment — Background".to_string());
        assert!(is_background_card(&card));
    }

    #[test]
    fn non_background_enchantment_rejected() {
        let mut card = make_card("Doubling Season");
        card.scryfall_data.type_line = Some("Enchantment".to_string());
        assert!(!is_background_card(&card));
    }

    // =================================
    // Signature Spell Tests
    // =================================

    #[test]
    fn instant_is_valid_signature_spell() {
        let mut card = make_card("Lightning Bolt");
        card.scryfall_data.type_line = Some("Instant".to_string());
        assert!(is_valid_signature_spell_type(&card));
    }

    #[test]
    fn sorcery_is_valid_signature_spell() {
        let mut card = make_card("Ponder");
        card.scryfall_data.type_line = Some("Sorcery".to_string());
        assert!(is_valid_signature_spell_type(&card));
    }

    #[test]
    fn creature_is_not_valid_signature_spell() {
        let mut card = make_card("Llanowar Elves");
        card.scryfall_data.type_line = Some("Creature — Elf Druid".to_string());
        assert!(!is_valid_signature_spell_type(&card));
    }

    #[test]
    fn spell_in_oathbreaker_color_identity() {
        use crate::domain::card::scryfall_data::colors::Color;

        let mut spell = make_card("Lightning Bolt");
        spell.scryfall_data.color_identity = vec![Color::Red].into_iter().collect();

        let mut oathbreaker = make_card("Chandra, Torch of Defiance");
        oathbreaker.scryfall_data.color_identity = vec![Color::Red].into_iter().collect();

        assert!(is_signature_spell_in_color_identity(&spell, &oathbreaker));
    }

    #[test]
    fn spell_outside_oathbreaker_color_identity() {
        use crate::domain::card::scryfall_data::colors::Color;

        let mut spell = make_card("Counterspell");
        spell.scryfall_data.color_identity = vec![Color::Blue].into_iter().collect();

        let mut oathbreaker = make_card("Chandra, Torch of Defiance");
        oathbreaker.scryfall_data.color_identity = vec![Color::Red].into_iter().collect();

        assert!(!is_signature_spell_in_color_identity(&spell, &oathbreaker));
    }
}
