//! Deck rules: what validation warns about, for any deck it is handed.

use std::collections::BTreeSet;

use proptest::prelude::*;
use uuid::Uuid;

use super::strategies::colors;
use crate::domain::card::Card;
use crate::domain::card::scryfall_data::colors::{Color, Colors};
use crate::domain::card::scryfall_data::legalities::LegalityKind;
use crate::domain::card::search_card::commander_eligibility::is_signature_spell_in_color_identity;
use crate::domain::deck::deck::DeckEntry;
use crate::domain::deck::validate_deck::{DeckCommandZone, validate_deck};
use crate::domain::deck::{DeckProfile, DeckWarning, Format, Quantity, WarningAction};
use crate::test_utils::{make_card, make_entry, make_profile};

/// What the color identity warning says, after the card's name.
const OUTSIDE_IDENTITY: &str = "is outside your commander's color identity";

/// An empty deck in `format`.
fn profile(format: Format) -> DeckProfile {
    make_profile("properties", Some(format))
}

/// A card named `name` whose color identity is `identity`.
fn card_of(name: &str, identity: &[Color]) -> Card {
    let mut card = make_card(name);
    card.scryfall_data.color_identity = Colors::from(identity.iter().copied());
    card
}

/// A deck card of `name` in `identity`, one copy on the main board.
fn entry_of(name: &str, identity: &[Color]) -> DeckEntry {
    let mut entry = make_entry(name, 1);
    entry.card.scryfall_data.color_identity = Colors::from(identity.iter().copied());
    entry
}

/// Any format the app offers.
fn format() -> impl Strategy<Value = Format> {
    prop::sample::select(Format::all().to_vec())
}

/// A main board card for the copy check: its copies, whether it is a basic land, and whether
/// Vintage restricts it.
fn copy_case() -> impl Strategy<Value = (i32, bool, bool)> {
    (
        prop_oneof![1..=5i32, 1..=Quantity::MAX],
        any::<bool>(),
        any::<bool>(),
    )
}

proptest! {
    /// Guards the commander color rule: a main board card is flagged exactly when one of its
    /// colors is outside the commander's, partner's and background's together, and a
    /// command zone card listed in the deck never is.
    #[test]
    fn a_card_is_flagged_exactly_when_its_identity_leaves_the_command_zone_s(
        commander in colors(),
        partner in prop::option::of(colors()),
        background in prop::option::of(colors()),
        cards in prop::collection::vec(colors(), 0..8),
        zone_listed in any::<bool>(),
    ) {
        let commander = card_of("Commander", &commander);
        let partner = partner.map(|p| card_of("Partner", &p));
        let background = background.map(|b| card_of("Background", &b));
        let allowed: BTreeSet<Color> = [Some(&commander), partner.as_ref(), background.as_ref()]
            .into_iter()
            .flatten()
            .flat_map(|c| c.scryfall_data.color_identity.iter().copied())
            .collect();

        let mut entries: Vec<DeckEntry> = cards
            .iter()
            .enumerate()
            .map(|(i, identity)| entry_of(&format!("Card {i}"), identity))
            .collect();
        if zone_listed {
            for zone_card in [Some(&commander), partner.as_ref(), background.as_ref()].into_iter().flatten() {
                let mut entry = make_entry(&zone_card.scryfall_data.name, 1);
                entry.card = zone_card.clone();
                entry.deck_card.scryfall_data_id = zone_card.scryfall_data.id;
                entries.push(entry);
            }
        }
        let zone = DeckCommandZone {
            commander: Some(&commander),
            partner_commander: partner.as_ref(),
            background: background.as_ref(),
            signature_spell: None,
        };

        let flagged: BTreeSet<Uuid> = validate_deck(&profile(Format::Commander), &entries, &zone)
            .iter()
            .filter(|w| w.contains(OUTSIDE_IDENTITY))
            .filter_map(DeckWarning::scryfall_data_id)
            .collect();
        let outside: BTreeSet<Uuid> = entries
            .iter()
            .filter(|e| e.card.scryfall_data.color_identity.iter().any(|c| !allowed.contains(c)))
            .map(|e| e.card.scryfall_data.id)
            .collect();
        prop_assert_eq!(flagged, outside);
    }

    /// Guards Oathbreaker's signature spell: it fits exactly when every color of its identity
    /// is in the oathbreaker's.
    #[test]
    fn a_signature_spell_fits_exactly_when_its_identity_is_within_the_oathbreaker_s(
        spell in colors(),
        oathbreaker in colors(),
    ) {
        let fits = spell.iter().all(|c| oathbreaker.contains(c));
        prop_assert_eq!(
            is_signature_spell_in_color_identity(&card_of("Spell", &spell), &card_of("Oathbreaker", &oathbreaker)),
            fits
        );
    }

    /// Guards the copy limit: in every format a main board card past its limit (the format's
    /// copies, 1 for a card Vintage restricts) gets a fix to that limit, and nothing else does;
    /// a basic land never does.
    #[test]
    fn a_card_past_its_copy_limit_gets_a_fix_to_the_limit_and_basics_never_do(
        format in format(),
        cases in prop::collection::vec(copy_case(), 0..8),
    ) {
        let entries: Vec<DeckEntry> = cases
            .iter()
            .enumerate()
            .map(|(i, (copies, basic, restricted))| {
                let mut entry = make_entry(&format!("Card {i}"), *copies);
                let data = &mut entry.card.scryfall_data;
                data.type_line = Some(if *basic { "Basic Land — Island" } else { "Instant" }.to_owned());
                if *restricted {
                    data.legalities.vintage = Some(LegalityKind::Restricted);
                }
                entry
            })
            .collect();
        let zone = DeckCommandZone { commander: None, partner_commander: None, background: None, signature_spell: None };

        let fixes: BTreeSet<(Uuid, i32)> = validate_deck(&profile(format), &entries, &zone)
            .iter()
            .filter_map(|w| match w.action() {
                Some(WarningAction::FixQuantity(max)) => Some((w.scryfall_data_id()?, *max)),
                _ => None,
            })
            .collect();
        let past: BTreeSet<(Uuid, i32)> = entries
            .iter()
            .zip(&cases)
            .filter_map(|(entry, (copies, basic, restricted))| {
                let limit = if format == Format::Vintage && *restricted { 1 } else { i32::from(format.copy_max()) };
                (!basic && *copies > limit).then_some((entry.card.scryfall_data.id, limit))
            })
            .collect();
        prop_assert_eq!(fixes, past);
    }

    /// Guards the deck size rule: in every format the count (cards plus each filled command
    /// zone slot, the commander only where the format has one) under the minimum or over the
    /// maximum is named, and a count inside both is not.
    #[test]
    fn a_count_outside_the_format_s_size_is_named_and_one_inside_is_not(
        format in format(),
        card_count in prop_oneof![0..=120i64, 0..=300i64],
        slots in (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()),
    ) {
        let (commander, partner, background, signature) = slots;
        let mut profile = profile(format);
        profile.card_count = card_count;
        profile.commander_id = commander.then(Uuid::new_v4);
        profile.partner_commander_id = partner.then(Uuid::new_v4);
        profile.background_id = background.then(Uuid::new_v4);
        profile.signature_spell_id = signature.then(Uuid::new_v4);
        let zone = DeckCommandZone { commander: None, partner_commander: None, background: None, signature_spell: None };

        let filled = [format.has_commander() && commander, partner, background, signature]
            .into_iter()
            .filter(|f| *f)
            .count();
        let count = card_count.checked_add(i64::try_from(filled).unwrap()).unwrap();
        let noun = if count == 1 { "card" } else { "cards" };
        let warnings = validate_deck(&profile, &[], &zone);
        let named = |text: String| warnings.iter().any(|w| **w == *text);

        if let Some(min) = format.min_cards() {
            let text = format!("Deck has {count} {noun}, {} requires at least {min}", format.display_name());
            prop_assert_eq!(named(text), count < i64::from(min));
        }
        if let Some(max) = format.max_cards() {
            let text = format!("Deck has {count} {noun}, {} allows at most {max}", format.display_name());
            prop_assert_eq!(named(text), count > i64::from(max));
        }
    }
}
