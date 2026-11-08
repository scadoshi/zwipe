//! The deck form's rules, ported from the app's `deck_fields.rs`, `format_select.rs`, `create.rs` and `edit.rs`: the form's values with Create's request and Edit's sparse update, the knock-on clears and which slots show (`values`), the land stepper, the price box, the format details, the oracle tag seeding, and the four command-zone searches.

mod values;

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::error::InvalidCardCriteria;
use zwipe_core::domain::card::search_card::commander_eligibility::{
    self, PartnerKind, partner_kind,
};
use zwipe_core::domain::deck::deck_profile::MAX_LAND_TARGET;
use zwipe_core::domain::deck::{Format, MAX_DECK_ORACLE_TAGS};

pub use self::values::{Cascade, FormValues, clears_mates, shows};
pub use crate::deck::command_zone::Slot;
use crate::timing;

/// Which way a stepper button moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// One less.
    Down,
    /// One more.
    Up,
}

/// The land target after pressing a stepper button. From "Not set" either button lands on the format's usual count, or 0 with no format; then it stays within 0 and [`MAX_LAND_TARGET`].
#[must_use]
pub fn step_land_target(current: Option<i32>, format: Option<Format>, step: Step) -> i32 {
    match (current, step) {
        (None, _) => format.and_then(|f| f.default_land_target()).unwrap_or(0),
        (Some(v), Step::Down) => v.saturating_sub(1).max(0),
        (Some(v), Step::Up) => v.saturating_add(1).min(MAX_LAND_TARGET),
    }
}

/// The land target after the format changes from `old` to `new`. A target
/// still at `old`'s usual count moves to `new`'s, since the user never set
/// it; a target the user set, an unset one, and any target when the format
/// is cleared stay as they are.
#[must_use]
pub fn follow_land_target(
    current: Option<i32>,
    old: Option<Format>,
    new: Option<Format>,
) -> Option<i32> {
    let (Some(target), Some(new)) = (current, new) else {
        return current;
    };
    if old.and_then(|f| f.default_land_target()) == Some(target) {
        new.default_land_target().or(current)
    } else {
        current
    }
}

/// Whether `c` may be typed into the price box: digits and dots only, as the app's filter keeps.
#[must_use]
pub fn price_char(c: char) -> bool {
    c.is_ascii_digit() || c == '.'
}

/// The budget the price box holds: a number above zero, else none. Two dots fail to parse and count as none, as in the app.
#[must_use]
pub fn parse_price(text: &str) -> Option<f64> {
    text.parse::<f64>().ok().filter(|v| *v > 0.0)
}

/// A stored price target as the box shows it: `50.0` as `50`.
#[must_use]
pub fn price_text(price: f64) -> String {
    format!("{price}")
}

/// The format picker's "Cards" line: the size, then singleton or four copies.
#[must_use]
pub fn size_text(format: Format) -> String {
    let copies = if format.copy_max() == 1 {
        "singleton"
    } else {
        "up to 4 copies"
    };
    let count = match (format.min_cards(), format.max_cards()) {
        (Some(a), Some(b)) if a == b => format!("{a} cards"),
        (Some(a), Some(b)) => format!("{a} to {b} cards"),
        (Some(a), None) => format!("{a}+ cards"),
        _ => "Any size".to_owned(),
    };
    format!("{count}, {copies}")
}

/// What the format's command zone holds.
#[must_use]
pub fn command_zone_text(format: Format) -> &'static str {
    if format.has_signature_spell() {
        "Planeswalker commander + signature spell"
    } else if format == Format::PauperCommander {
        "1 uncommon creature commander"
    } else if format.has_commander() {
        if format.supports_partner() {
            "1 legendary commander (partners & backgrounds OK)"
        } else {
            "1 legendary commander"
        }
    } else {
        "None"
    }
}

/// The format picker's details for `format`, one line each.
#[must_use]
pub fn format_details(format: Format) -> Vec<String> {
    let mut lines = vec![
        format!("Pool: {}", format.card_pool()),
        format!("Cards: {}", size_text(format)),
        format!("Life: {}", format.starting_life()),
    ];
    if format.has_commander() || format.has_signature_spell() {
        lines.push(format!("Command zone: {}", command_zone_text(format)));
    }
    if let Some(damage) = format.commander_damage() {
        lines.push(format!("Commander damage: {damage}"));
    }
    lines
}

/// The oracle tags after the deck tags change: the old seed's tags go, manual picks stay, and the new seed's tags are added while there is room. Returns the tags and whether any were added.
#[must_use]
pub fn reseed_oracle_tags(
    oracle_tags: &[String],
    old_seed: &[String],
    new_seed: &[String],
) -> (Vec<String>, bool) {
    let mut tags: Vec<String> = oracle_tags
        .iter()
        .filter(|s| !old_seed.contains(s))
        .cloned()
        .collect();
    let mut added = false;
    for slug in new_seed {
        if !tags.contains(slug) && tags.len() < MAX_DECK_ORACLE_TAGS {
            tags.push(slug.clone());
            added = true;
        }
    }
    (tags, added)
}

/// The type line every Time Lord Doctor carries, which a Doctor's companion partners with.
const TIME_LORD_DOCTOR: &str = "Time Lord Doctor";

/// Narrows a partner search toward the cards `commander` can pair with, by its partner ability; with no commander, toward every card with one. Only fields the user's filter leaves unset are narrowed, and the narrowing only trims the pages: [`legal_partners`] decides.
pub fn narrow_to_partners(builder: &mut CardQueryBuilder, commander: Option<&Card>) {
    let Some(commander) = commander else {
        builder.set_is_partner(true);
        return;
    };
    let keywords_free = builder.keywords_contains_all().is_none();
    match partner_kind(commander) {
        Some(PartnerKind::Named(name)) => {
            if builder.name_contains().is_none() {
                builder.set_name_contains(name);
            }
        }
        Some(PartnerKind::Generic) => {
            if keywords_free {
                builder.set_keywords_contains_all(["Partner"]);
            }
            if builder.keywords_excludes().is_none() {
                builder.set_keywords_excludes(["Partner with"]);
            }
        }
        Some(PartnerKind::FriendsForever) => {
            if keywords_free {
                builder.set_keywords_contains_all(["Friends forever"]);
            }
        }
        Some(PartnerKind::DoctorsCompanion) => {
            if builder.type_line_contains().is_none() {
                builder.set_type_line_contains(TIME_LORD_DOCTOR);
            }
        }
        None if is_time_lord_doctor(commander) => {
            if keywords_free {
                builder.set_keywords_contains_all(["Doctor's companion"]);
            }
        }
        None => {
            builder.set_is_partner(true);
        }
    }
}

/// `cards` without the ones `commander` cannot pair with, by `validate_deck`'s own rule ([`commander_eligibility::are_valid_partners`]), and without another printing of the commander itself.
#[must_use]
pub fn legal_partners(commander: &Card, cards: Vec<Card>) -> Vec<Card> {
    let own = commander.scryfall_data.oracle_id;
    cards
        .into_iter()
        .filter(|card| {
            (own.is_none() || card.scryfall_data.oracle_id != own)
                && commander_eligibility::are_valid_partners(commander, card)
        })
        .collect()
}

/// Whether `card` is a Time Lord Doctor, the card a Doctor's companion pairs with.
fn is_time_lord_doctor(card: &Card) -> bool {
    card.scryfall_data
        .type_line
        .as_deref()
        .is_some_and(|line| line.contains(TIME_LORD_DOCTOR))
}

/// The typeahead search for `slot`: the name typed, narrowed by the slot's rule while its Filter is on.
///
/// # Errors
///
/// Fails when nothing narrows the search, which the server would refuse.
pub fn search_query(
    slot: Slot,
    name: &str,
    filter_on: bool,
    format: Option<Format>,
    commander: Option<&Card>,
) -> Result<CardQuery, InvalidCardCriteria> {
    let mut builder = match slot {
        Slot::Commander | Slot::SignatureSpell => CardQueryBuilder::with_name_contains(name),
        Slot::Partner | Slot::Background => {
            let mut builder = CardQueryBuilder::new();
            builder.set_name_contains(name);
            builder
        }
    };
    if filter_on {
        match slot {
            Slot::Commander => {
                if let Some(format) = format {
                    builder.set_is_commander_in_format(format);
                }
            }
            Slot::Partner => narrow_to_partners(&mut builder, commander),
            Slot::Background => {
                builder.set_is_background(true);
            }
            Slot::SignatureSpell => {
                builder.set_is_signature_spell(true);
                if let Some(commander) = commander {
                    builder
                        .set_color_identity_within(commander.scryfall_data.color_identity.clone());
                }
            }
        }
    }
    builder.set_limit(slot.limit());
    builder.build()
}

/// The swipe picker's page for `slot` from `offset`: the user's `filter` with the slot's rule on top, no tokens, 25 a page, in the server's popularity order unless the filter sets a sort.
///
/// # Errors
///
/// Fails when nothing narrows the search, which cannot happen with the slot rule set.
pub fn swipe_query(
    filter: &CardQueryBuilder,
    slot: Slot,
    format: Option<Format>,
    commander: Option<&Card>,
    offset: u32,
) -> Result<CardQuery, InvalidCardCriteria> {
    let mut builder = filter.clone();
    match slot {
        Slot::Commander => {
            if let Some(format) = format {
                builder.set_is_commander_in_format(format);
            }
        }
        Slot::Partner => narrow_to_partners(&mut builder, commander),
        Slot::Background => {
            builder.set_is_background(true);
        }
        Slot::SignatureSpell => {
            builder.set_is_signature_spell(true);
            if let Some(commander) = commander {
                builder.set_color_identity_within(commander.scryfall_data.color_identity.clone());
            }
        }
    }
    builder.set_is_token(false);
    builder.set_limit(timing::PAGE_SIZE);
    builder.set_offset(offset);
    builder.build()
}

/// The card among `hits` that is the named partner `mate`: an exact name match ignoring case, else a front-face match.
#[must_use]
pub fn named_partner<'hits>(hits: &'hits [Card], mate: &str) -> Option<&'hits Card> {
    hits.iter()
        .find(|c| c.scryfall_data.name.eq_ignore_ascii_case(mate))
        .or_else(|| {
            hits.iter().find(|c| {
                c.scryfall_data
                    .name
                    .split(" // ")
                    .next()
                    .is_some_and(|front| front.trim().eq_ignore_ascii_case(mate))
            })
        })
}

/// The toast after an import: its kind and message, from how many cards came in and how many did not.
#[must_use]
pub fn import_outcome(imported: usize, unresolved: usize) -> (ImportTone, String) {
    let plural = |n: usize| if n == 1 { "card" } else { "cards" };
    match (imported, unresolved) {
        (0, 0) => (ImportTone::Info, "No cards found".to_owned()),
        (0, n) => (ImportTone::Error, format!("{n} {} failed", plural(n))),
        (n, 0) => (ImportTone::Success, format!("Imported {n} {}", plural(n))),
        (n, m) => (ImportTone::Info, format!("Imported {n}, {m} failed")),
    }
}

/// The kind of toast an import raises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportTone {
    /// Neutral: some lines did not resolve.
    Info,
    /// Nothing imported.
    Error,
    /// Every line imported.
    Success,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::card;

    /// A legendary creature named `name` with these keywords, rules text and type line.
    fn commander(name: &str, keywords: &[&str], text: &str, type_line: &str) -> Card {
        let mut card = card(name);
        let data = &mut card.scryfall_data;
        data.keywords = Some(keywords.iter().map(|k| (*k).to_owned()).collect());
        data.oracle_text = Some(text.to_owned());
        data.type_line = Some(type_line.to_owned());
        card
    }

    fn tymna() -> Card {
        commander(
            "Tymna the Weaver",
            &["Partner"],
            "Lifelink\nPartner",
            "Legendary Creature \u{2014} Human Cleric",
        )
    }

    fn names(cards: &[Card]) -> Vec<&str> {
        cards
            .iter()
            .map(|c| c.scryfall_data.name.as_str())
            .collect()
    }

    #[test]
    fn tymna_is_offered_only_plain_partners() {
        let tymna = tymna();
        let mut tymna_again = card("Tymna the Weaver");
        tymna_again.scryfall_data.oracle_id = tymna.scryfall_data.oracle_id;
        tymna_again.scryfall_data.keywords = Some(vec!["Partner".to_owned()]);
        let pile = vec![
            commander(
                "Thrasios, Triton Hero",
                &["Partner"],
                "Partner",
                "Legendary Creature \u{2014} Merfolk Wizard",
            ),
            commander(
                "Pir, Imaginative Rascal",
                &["Partner with", "Partner"],
                "Partner with Toothy, Imaginary Friend (When this creature enters, target player may put Toothy into their hand.)",
                "Legendary Creature \u{2014} Human",
            ),
            commander(
                "Ian Chesterton",
                &["Doctor's companion"],
                "Doctor's companion",
                "Legendary Creature \u{2014} Human Advisor",
            ),
            commander(
                "Will the Wise",
                &["Friends forever"],
                "Friends forever",
                "Legendary Creature \u{2014} Human Wizard",
            ),
            commander(
                "The Prismatic Piper",
                &["Partner"],
                "If The Prismatic Piper is your commander, choose a color before the game begins.\nPartner",
                "Legendary Creature \u{2014} Shapeshifter",
            ),
            tymna_again,
        ];
        assert_eq!(
            names(&legal_partners(&tymna, pile)),
            ["Thrasios, Triton Hero", "The Prismatic Piper"]
        );
    }

    #[test]
    fn each_partner_ability_narrows_the_search_its_own_way() {
        let mut builder = CardQueryBuilder::new();
        narrow_to_partners(&mut builder, Some(&tymna()));
        assert_eq!(
            builder.keywords_excludes(),
            Some(&["Partner with".to_owned()][..])
        );
        let pir = commander(
            "Pir, Imaginative Rascal",
            &["Partner with", "Partner"],
            "Partner with Toothy, Imaginary Friend (When this creature enters, target player may put Toothy into their hand.)",
            "Legendary Creature \u{2014} Human",
        );
        let mut builder = CardQueryBuilder::new();
        narrow_to_partners(&mut builder, Some(&pir));
        assert_eq!(builder.name_contains(), Some("Toothy, Imaginary Friend"));
        let doctor = commander(
            "The Fourteenth Doctor",
            &[],
            "",
            "Legendary Creature \u{2014} Time Lord Doctor",
        );
        let mut builder = CardQueryBuilder::new();
        narrow_to_partners(&mut builder, Some(&doctor));
        assert_eq!(
            builder.keywords_contains_all(),
            Some(&["Doctor's companion".to_owned()][..])
        );
        let mut builder = CardQueryBuilder::new();
        narrow_to_partners(&mut builder, None);
        assert_eq!(builder.is_partner(), Some(true));
    }

    #[test]
    fn a_doctor_is_offered_its_companions() {
        let doctor = commander(
            "The Fourteenth Doctor",
            &[],
            "",
            "Legendary Creature \u{2014} Time Lord Doctor",
        );
        let pile = vec![
            commander(
                "Ian Chesterton",
                &["Doctor's companion"],
                "Doctor's companion",
                "Legendary Creature \u{2014} Human Advisor",
            ),
            commander(
                "Thrasios, Triton Hero",
                &["Partner"],
                "Partner",
                "Legendary Creature \u{2014} Merfolk Wizard",
            ),
        ];
        assert_eq!(names(&legal_partners(&doctor, pile)), ["Ian Chesterton"]);
    }

    #[test]
    fn a_default_land_target_follows_the_format_and_a_set_one_stays() {
        let commander = Some(Format::Commander);
        let oathbreaker = Some(Format::Oathbreaker);
        assert_eq!(
            super::follow_land_target(Some(37), commander, oathbreaker),
            Some(17),
            "Commander's 37 becomes Oathbreaker's 17"
        );
        assert_eq!(
            super::follow_land_target(Some(17), oathbreaker, commander),
            Some(37),
            "and back"
        );
        assert_eq!(
            super::follow_land_target(Some(36), commander, oathbreaker),
            Some(36),
            "a target the user set stays"
        );
        assert_eq!(
            super::follow_land_target(None, commander, oathbreaker),
            None,
            "unset stays unset"
        );
        assert_eq!(
            super::follow_land_target(Some(37), commander, None),
            Some(37),
            "clearing the format keeps it"
        );
        assert_eq!(
            super::follow_land_target(Some(37), None, oathbreaker),
            Some(37),
            "with no format before, nothing was a default"
        );
    }

    #[test]
    fn the_land_stepper_starts_at_the_formats_count_and_stays_in_range() {
        assert_eq!(
            step_land_target(None, Some(Format::Commander), Step::Up),
            37
        );
        assert_eq!(step_land_target(None, Some(Format::Modern), Step::Down), 17);
        assert_eq!(step_land_target(None, None, Step::Up), 0);
        assert_eq!(step_land_target(Some(0), None, Step::Down), 0);
        assert_eq!(step_land_target(Some(100), None, Step::Up), 100);
        assert_eq!(step_land_target(Some(36), None, Step::Up), 37);
    }

    #[test]
    fn the_price_box_takes_digits_and_dots_and_needs_more_than_zero() {
        let typed: String = "$12.5a0".chars().filter(|&c| price_char(c)).collect();
        assert_eq!(typed, "12.50");
        assert_eq!(parse_price("12.50"), Some(12.5));
        assert_eq!(parse_price("0"), None);
        assert_eq!(parse_price("1.2.3"), None);
        assert_eq!(parse_price(""), None);
        assert_eq!(price_text(50.0), "50");
        assert_eq!(price_text(12.5), "12.5");
    }

    #[test]
    fn format_details_match_the_app() {
        assert_eq!(size_text(Format::Commander), "100 cards, singleton");
        assert_eq!(size_text(Format::Modern), "60+ cards, up to 4 copies");
        let commander = format_details(Format::Commander);
        assert!(commander.contains(&"Life: 40".to_owned()));
        assert!(commander.contains(
            &"Command zone: 1 legendary commander (partners & backgrounds OK)".to_owned()
        ));
        assert!(commander.contains(&"Commander damage: 21".to_owned()));
        assert_eq!(
            command_zone_text(Format::Oathbreaker),
            "Planeswalker commander + signature spell"
        );
        assert_eq!(
            command_zone_text(Format::PauperCommander),
            "1 uncommon creature commander"
        );
        assert!(
            !format_details(Format::Modern)
                .iter()
                .any(|l| l.starts_with("Command zone"))
        );
    }

    #[test]
    fn reseeding_swaps_the_old_seed_for_the_new_and_keeps_manual_picks() {
        let tags = |v: &[&str]| v.iter().map(|&s| s.to_owned()).collect::<Vec<_>>();
        let (got, added) = reseed_oracle_tags(
            &tags(&["ramp", "manual"]),
            &tags(&["ramp"]),
            &tags(&["sac-outlet", "manual"]),
        );
        assert_eq!(got, ["manual", "sac-outlet"]);
        assert!(added);
        let (got, added) = reseed_oracle_tags(&tags(&["a"]), &[], &[]);
        assert_eq!(got, ["a"]);
        assert!(!added);
        let full: Vec<String> = (0..MAX_DECK_ORACLE_TAGS).map(|i| i.to_string()).collect();
        let (got, added) = reseed_oracle_tags(&full, &[], &tags(&["more"]));
        assert_eq!(got.len(), MAX_DECK_ORACLE_TAGS);
        assert!(!added, "no room for the seed");
    }

    #[test]
    fn each_slot_has_the_apps_search_rules() {
        assert_eq!(
            Slot::ALL.map(Slot::min_chars),
            [3, 1, 1, 3],
            "commander, partner, background, signature spell"
        );
        assert_eq!(Slot::ALL.map(Slot::limit), [5, 10, 10, 5]);
        assert!(search_query(Slot::Commander, "kre", true, Some(Format::Commander), None).is_ok());
        assert!(search_query(Slot::Partner, "a", false, None, None).is_ok());
        assert!(swipe_query(&CardQueryBuilder::new(), Slot::Background, None, None, 25).is_ok());
        assert_eq!(
            Slot::Commander.noun(Some(Format::Oathbreaker)),
            "oathbreaker"
        );
        assert_eq!(Slot::SignatureSpell.noun(None), "signature spell");
    }

    #[test]
    fn the_named_partner_matches_the_name_or_its_front_face() {
        let hits = [card("Other"), card("Pir, Imaginative Rascal // Back")];
        assert_eq!(
            named_partner(&hits, "pir, imaginative rascal").map(|c| c.scryfall_data.name.as_str()),
            Some("Pir, Imaginative Rascal // Back")
        );
        assert!(named_partner(&hits, "Toothy").is_none());
    }

    #[test]
    fn import_toasts_count_cards_singular_and_plural() {
        assert_eq!(
            import_outcome(0, 0),
            (ImportTone::Info, "No cards found".to_owned())
        );
        assert_eq!(
            import_outcome(0, 1),
            (ImportTone::Error, "1 card failed".to_owned())
        );
        assert_eq!(
            import_outcome(3, 0),
            (ImportTone::Success, "Imported 3 cards".to_owned())
        );
        assert_eq!(
            import_outcome(1, 2),
            (ImportTone::Info, "Imported 1, 2 failed".to_owned())
        );
    }
}
