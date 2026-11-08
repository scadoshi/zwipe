//! The chips each chip row of the card filter offers, which of them show selected, and what a tap writes. [`ChipSet`] names the rows; this module fills them.
//!
//! A chip stands for a string value, the one its tap writes: a card type's name, a mana symbol, a rarity's long name, a role's slug, a format's legality key, a currency's label, a sort key's name, or `yes`, `no` and `any` for a config tri-toggle.

use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::scryfall_data::rarity::Rarity;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Format;

use crate::filter::fields::{self, Exclude, MatchGroup, RangeGroup, SideGroup};
use crate::filter::sections::{self, ChipSet, Modes};

/// The mana symbols the Produces rows offer, with their long names: the five colors, then colorless.
pub const PRODUCED: [(&str, &str); 6] = [
    ("W", "White"),
    ("U", "Blue"),
    ("B", "Black"),
    ("R", "Red"),
    ("G", "Green"),
    ("C", "Colorless"),
];

/// One chip: what it shows and the value a tap writes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChipOption {
    /// The chip's words; empty for a mana symbol chip.
    pub label: String,
    /// The value the chip stands for, as [`is_selected`] and [`tap`] take it.
    pub value: String,
    /// The mana symbol a symbol chip shows, lowercase (`w`, `u`, `c` and so on).
    pub symbol: Option<String>,
    /// The hover title: a symbol's long name.
    pub title: Option<String>,
}

impl ChipOption {
    fn text(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            symbol: None,
            title: None,
        }
    }

    fn symbol(code: &str, title: &str) -> Self {
        Self {
            label: String::new(),
            value: code.to_owned(),
            symbol: Some(code.to_lowercase()),
            title: Some(title.to_owned()),
        }
    }
}

/// The chips `set` offers, in the app's order. Card roles come from the catalog, so the role rows are empty until it loads. The sort order row has one chip that names the current direction.
#[must_use]
pub fn options(set: ChipSet, roles: &[CardRoleView], b: &CardQueryBuilder) -> Vec<ChipOption> {
    match set {
        ChipSet::BasicIncludes | ChipSet::BasicExcludes => fields::card_types()
            .into_iter()
            .map(|t| ChipOption::text(t.to_string(), t.to_string()))
            .collect(),
        ChipSet::Colors => Color::all()
            .into_iter()
            .map(|c| ChipOption::symbol(&c.to_short_name(), &c.to_long_name()))
            .collect(),
        ChipSet::ProducedIncludes | ChipSet::ProducedExcludes => PRODUCED
            .into_iter()
            .map(|(code, title)| ChipOption::symbol(code, title))
            .collect(),
        ChipSet::Rarity => Rarity::all()
            .into_iter()
            .map(|r| ChipOption::text(r.to_long_name(), r.to_long_name()))
            .collect(),
        ChipSet::RolesInclude | ChipSet::RolesExclude => roles
            .iter()
            .map(|r| ChipOption::text(r.display_name.clone(), r.slug.clone()))
            .collect(),
        ChipSet::CommanderIn => Format::commander_formats()
            .iter()
            .map(|f| ChipOption::text(f.display_name(), f.to_legality_key()))
            .collect(),
        ChipSet::LegalIn => Format::all()
            .iter()
            .map(|f| ChipOption::text(f.display_name(), f.to_legality_key()))
            .collect(),
        ChipSet::Currency => PriceCurrency::all()
            .iter()
            .map(|c| ChipOption::text(c.label(), c.label()))
            .collect(),
        ChipSet::SortBy => CardSortKey::all()
            .into_iter()
            .map(|k| ChipOption::text(k.to_string(), k.to_string()))
            .collect(),
        ChipSet::SortOrder => {
            let direction = b.direction().to_string();
            vec![ChipOption::text(direction.clone(), direction)]
        }
        ChipSet::Config(_) => [("Yes", "yes"), ("No", "no"), ("Any", "any")]
            .into_iter()
            .map(|(label, value)| ChipOption::text(label, value))
            .collect(),
    }
}

/// A config chip's value as the builder holds it: `None` for `any` and anything unknown.
fn tri(value: &str) -> Option<bool> {
    match value {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Whether the chip for `value` shows selected. The sort order chip never does: it is a plain toggle.
#[must_use]
pub fn is_selected(set: ChipSet, b: &CardQueryBuilder, value: &str) -> bool {
    let has = |values: Vec<String>| values.iter().any(|v| v == value);
    match set {
        ChipSet::BasicIncludes => has(MatchGroup::BasicTypes.values(b)),
        ChipSet::BasicExcludes => has(Exclude::BasicTypes.values(b)),
        ChipSet::Colors => fields::identity_colors(b)
            .iter()
            .any(|c| c.to_short_name() == value),
        ChipSet::ProducedIncludes => has(MatchGroup::Produced.values(b)),
        ChipSet::ProducedExcludes => has(MatchGroup::Produced.excluded(b)),
        ChipSet::Rarity => has(SideGroup::Rarity.values(b)),
        ChipSet::RolesInclude => has(MatchGroup::Roles.values(b)),
        ChipSet::RolesExclude => has(MatchGroup::Roles.excluded(b)),
        ChipSet::CommanderIn => b
            .is_commander_in_format()
            .is_some_and(|f| f.to_legality_key() == value),
        ChipSet::LegalIn => has(MatchGroup::Legal.values(b)),
        ChipSet::Currency => sections::currency(b).label() == value,
        ChipSet::SortBy => b.sort().is_some_and(|k| k.to_string() == value),
        ChipSet::SortOrder => false,
        ChipSet::Config(toggle) => toggle.get(b) == tri(value),
    }
}

/// A tap on the chip for `value`. A multi-choice row toggles the value in or out, writing includes in the mode `modes` shows. A single-choice row picks it, and a second tap on the picked one unsets it, except Currency, which always holds one. The sort order chip flips the direction. A value the row does not offer changes nothing.
///
/// Only the include rows, Colors and Rarity read `modes`; the other rows ignore it.
pub fn tap(set: ChipSet, b: &mut CardQueryBuilder, modes: &Modes, value: &str) {
    match set {
        ChipSet::BasicIncludes => toggle_includes(b, modes, MatchGroup::BasicTypes, value),
        ChipSet::BasicExcludes => toggle_excludes(b, MatchGroup::BasicTypes, value),
        ChipSet::Colors => {
            let mode = modes.range_of(b, RangeGroup::ColorIdentity);
            if let Some(color) = Color::all()
                .into_iter()
                .find(|c| c.to_short_name() == value)
            {
                fields::toggle_identity(b, mode, color);
            }
        }
        ChipSet::ProducedIncludes => toggle_includes(b, modes, MatchGroup::Produced, value),
        ChipSet::ProducedExcludes => toggle_excludes(b, MatchGroup::Produced, value),
        ChipSet::Rarity => {
            let side = modes.side_of(b, SideGroup::Rarity);
            let values = fields::toggled(SideGroup::Rarity.values(b), value);
            SideGroup::Rarity.write(b, side, values);
        }
        ChipSet::RolesInclude => toggle_includes(b, modes, MatchGroup::Roles, value),
        ChipSet::RolesExclude => toggle_excludes(b, MatchGroup::Roles, value),
        ChipSet::CommanderIn => {
            if is_selected(set, b, value) {
                b.unset_is_commander_in_format();
            } else if let Some(format) = Format::commander_formats()
                .iter()
                .find(|f| f.to_legality_key() == value)
            {
                b.set_is_commander_in_format(*format);
            }
        }
        ChipSet::LegalIn => toggle_includes(b, modes, MatchGroup::Legal, value),
        ChipSet::Currency => {
            if let Some(currency) = PriceCurrency::all().iter().find(|c| c.label() == value) {
                b.set_price_currency(*currency);
            }
        }
        ChipSet::SortBy => {
            if is_selected(set, b, value) {
                b.unset_sort();
            } else if let Some(key) = CardSortKey::all()
                .into_iter()
                .find(|k| k.to_string() == value)
            {
                b.set_sort(key);
            }
        }
        ChipSet::SortOrder => {
            let flipped = b.direction().flipped();
            b.set_direction(flipped);
        }
        ChipSet::Config(toggle) => toggle.tap(b, tri(value)),
    }
}

/// Toggles `value` in `group`'s includes, in the mode its chip shows.
fn toggle_includes(b: &mut CardQueryBuilder, modes: &Modes, group: MatchGroup, value: &str) {
    let mode = modes.match_of(b, group);
    let values = fields::toggled(group.values(b), value);
    group.write(b, mode, values);
}

/// Toggles `value` in `group`'s excludes.
fn toggle_excludes(b: &mut CardQueryBuilder, group: MatchGroup, value: &str) {
    let values = fields::toggled(group.excluded(b), value);
    group.write_excludes(b, values);
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::search_card::card_type::CardType;

    use super::*;
    use crate::filter::fields::{Match, Toggle};
    use crate::filter::sections::ModeChip;

    fn values(set: ChipSet, roles: &[CardRoleView], b: &CardQueryBuilder) -> Vec<String> {
        options(set, roles, b)
            .into_iter()
            .map(|o| o.value)
            .collect()
    }

    fn role(slug: &str, name: &str) -> CardRoleView {
        CardRoleView {
            slug: slug.to_owned(),
            display_name: name.to_owned(),
            short_name: name.to_owned(),
        }
    }

    #[test]
    fn a_type_chip_toggles_in_the_mode_shown() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        tap(ChipSet::BasicIncludes, &mut b, &modes, "Creature");
        assert_eq!(b.card_type_contains_any(), Some(&[CardType::Creature][..]));
        assert!(is_selected(ChipSet::BasicIncludes, &b, "Creature"));
        tap(ChipSet::BasicIncludes, &mut b, &modes, "Creature");
        assert_eq!(b.card_type_contains_any(), None);
    }

    #[test]
    fn a_chosen_all_mode_takes_the_includes_while_they_are_empty() {
        let mut b = CardQueryBuilder::default();
        let mut modes = Modes::default();
        modes.flip(&mut b, ModeChip::Match(MatchGroup::Produced));
        assert_eq!(modes.match_of(&b, MatchGroup::Produced), Match::All);
        tap(ChipSet::ProducedIncludes, &mut b, &modes, "G");
        assert_eq!(b.produced_mana_contains_all(), Some(&["G".to_owned()][..]));
        assert_eq!(b.produced_mana_contains_any(), None);
        assert!(is_selected(ChipSet::ProducedIncludes, &b, "G"));
    }

    #[test]
    fn exclude_rows_toggle_their_own_list() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        tap(ChipSet::BasicExcludes, &mut b, &modes, "Land");
        assert_eq!(b.card_type_excludes_any(), Some(&[CardType::Land][..]));
        assert!(is_selected(ChipSet::BasicExcludes, &b, "Land"));
        assert!(!is_selected(ChipSet::BasicIncludes, &b, "Land"));

        tap(ChipSet::ProducedExcludes, &mut b, &modes, "C");
        assert_eq!(b.produced_mana_excludes(), Some(&["C".to_owned()][..]));
        assert!(!is_selected(ChipSet::ProducedIncludes, &b, "C"));

        tap(ChipSet::RolesExclude, &mut b, &modes, "ramp");
        assert_eq!(b.card_roles_excludes(), Some(&["ramp".to_owned()][..]));
        assert!(is_selected(ChipSet::RolesExclude, &b, "ramp"));
        tap(ChipSet::RolesExclude, &mut b, &modes, "ramp");
        assert_eq!(b.card_roles_excludes(), None, "the last one out unsets");
    }

    #[test]
    fn single_choice_chips_unset_on_a_second_tap() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        tap(ChipSet::SortBy, &mut b, &modes, "Name");
        assert_eq!(b.sort(), Some(CardSortKey::Name));
        tap(ChipSet::SortBy, &mut b, &modes, "Name");
        assert_eq!(b.sort(), None);
        tap(ChipSet::CommanderIn, &mut b, &modes, "commander");
        assert_eq!(b.is_commander_in_format(), Some(&Format::Commander));
        tap(ChipSet::CommanderIn, &mut b, &modes, "brawl");
        assert_eq!(b.is_commander_in_format(), Some(&Format::Brawl));
        tap(ChipSet::CommanderIn, &mut b, &modes, "brawl");
        assert_eq!(b.is_commander_in_format(), None);
    }

    #[test]
    fn legal_in_toggles_legality_keys() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        tap(ChipSet::LegalIn, &mut b, &modes, "modern");
        tap(ChipSet::LegalIn, &mut b, &modes, "legacy");
        assert_eq!(
            b.legalities_contains_any(),
            Some(&["modern".to_owned(), "legacy".to_owned()][..])
        );
        assert!(is_selected(ChipSet::LegalIn, &b, "legacy"));
        tap(ChipSet::LegalIn, &mut b, &modes, "modern");
        tap(ChipSet::LegalIn, &mut b, &modes, "legacy");
        assert_eq!(b.legalities_contains_any(), None);
    }

    #[test]
    fn legal_in_all_moves_the_formats_to_the_all_field() {
        let mut b = CardQueryBuilder::default();
        let mut modes = Modes::default();
        tap(ChipSet::LegalIn, &mut b, &modes, "modern");
        modes.flip(&mut b, ModeChip::Match(MatchGroup::Legal));
        assert_eq!(modes.match_of(&b, MatchGroup::Legal), Match::All);
        tap(ChipSet::LegalIn, &mut b, &modes, "pioneer");
        assert_eq!(b.legalities_contains_any(), None);
        assert_eq!(
            b.legalities_contains_all(),
            Some(&["modern".to_owned(), "pioneer".to_owned()][..])
        );
        assert!(is_selected(ChipSet::LegalIn, &b, "pioneer"));
        modes.flip(&mut b, ModeChip::Match(MatchGroup::Legal));
        assert_eq!(b.legalities_contains_all(), None);
        assert_eq!(b.legalities_contains_any().map(<[String]>::len), Some(2));
    }

    #[test]
    fn currency_always_holds_one_and_starts_at_usd() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        let usd = PriceCurrency::Usd.label();
        let eur = PriceCurrency::Eur.label();
        assert!(is_selected(ChipSet::Currency, &b, usd));
        tap(ChipSet::Currency, &mut b, &modes, eur);
        assert_eq!(b.price_currency(), Some(PriceCurrency::Eur));
        tap(ChipSet::Currency, &mut b, &modes, eur);
        assert_eq!(b.price_currency(), Some(PriceCurrency::Eur), "no unset");
        assert!(!is_selected(ChipSet::Currency, &b, usd));
    }

    #[test]
    fn the_sort_order_chip_names_the_direction_and_flips_it() {
        let mut b = CardQueryBuilder::default();
        let start = b.direction();
        let chip = values(ChipSet::SortOrder, &[], &b);
        assert_eq!(chip, vec![start.to_string()]);
        tap(
            ChipSet::SortOrder,
            &mut b,
            &Modes::default(),
            &start.to_string(),
        );
        assert_eq!(b.direction(), start.flipped());
        assert!(!is_selected(
            ChipSet::SortOrder,
            &b,
            &b.direction().to_string()
        ));
    }

    #[test]
    fn rarity_toggles_on_the_side_shown() {
        let mut b = CardQueryBuilder::default();
        let mut modes = Modes::default();
        tap(ChipSet::Rarity, &mut b, &modes, "Rare");
        assert_eq!(
            b.rarity_equals_any().map(|r| r.to_vec()),
            Some(vec![Rarity::Rare])
        );
        modes.flip(&mut b, ModeChip::Side(SideGroup::Rarity));
        assert_eq!(b.rarity_equals_any(), None);
        tap(ChipSet::Rarity, &mut b, &modes, "Common");
        assert_eq!(
            b.rarity_excludes_any().map(|r| r.to_vec()),
            Some(vec![Rarity::Common, Rarity::Rare])
        );
        assert!(is_selected(ChipSet::Rarity, &b, "Common"));
    }

    #[test]
    fn config_chips_follow_the_app_tri_toggle() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        let set = ChipSet::Config(Toggle::Playable);
        assert!(is_selected(set, &b, "yes"), "playable defaults to Yes");
        tap(set, &mut b, &modes, "yes");
        assert!(is_selected(set, &b, "any"));
        tap(set, &mut b, &modes, "no");
        assert_eq!(b.is_playable(), Some(false));
        tap(set, &mut b, &modes, "any");
        assert_eq!(b.is_playable(), None);
        tap(set, &mut b, &modes, "any");
        assert_eq!(b.is_playable(), None, "Any stays Any");
    }

    #[test]
    fn colors_and_produced_mana_are_symbols() {
        let b = CardQueryBuilder::default();
        let colors = options(ChipSet::Colors, &[], &b);
        assert_eq!(colors.len(), 5);
        assert_eq!(colors[0].symbol.as_deref(), Some("w"));
        assert_eq!(colors[0].title.as_deref(), Some("White"));
        assert!(colors.iter().all(|c| c.label.is_empty()));
        let produced = options(ChipSet::ProducedIncludes, &[], &b);
        assert_eq!(produced.last().map(|c| c.value.as_str()), Some("C"));
        assert_eq!(produced, options(ChipSet::ProducedExcludes, &[], &b));
        let mut b = b;
        tap(ChipSet::Colors, &mut b, &Modes::default(), "G");
        assert!(b.color_identity_equals().is_some(), "Exact by default");
        assert!(is_selected(ChipSet::Colors, &b, "G"));
        tap(ChipSet::Colors, &mut b, &Modes::default(), "G");
        assert_eq!(fields::identity_colors(&b), Vec::<Color>::new());
    }

    #[test]
    fn each_list_offers_the_app_chips_in_order() {
        let b = CardQueryBuilder::default();
        let types: Vec<String> = fields::card_types()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(values(ChipSet::BasicIncludes, &[], &b), types);
        assert_eq!(values(ChipSet::BasicExcludes, &[], &b), types);
        assert_eq!(
            values(ChipSet::Rarity, &[], &b),
            Rarity::all()
                .iter()
                .map(Rarity::to_long_name)
                .collect::<Vec<_>>()
        );
        let commander: Vec<&str> = Format::commander_formats()
            .iter()
            .map(Format::to_legality_key)
            .collect();
        assert_eq!(values(ChipSet::CommanderIn, &[], &b), commander);
        assert_eq!(values(ChipSet::LegalIn, &[], &b).len(), Format::all().len());
        assert_eq!(
            options(ChipSet::LegalIn, &[], &b)
                .first()
                .map(|o| o.label.as_str()),
            Format::all().first().map(Format::display_name)
        );
        assert_eq!(values(ChipSet::Currency, &[], &b), ["USD", "EUR", "TIX"]);
        assert_eq!(
            values(ChipSet::SortBy, &[], &b),
            CardSortKey::all()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        let config = options(ChipSet::Config(Toggle::Promo), &[], &b);
        let labels: Vec<&str> = config.iter().map(|o| o.label.as_str()).collect();
        assert_eq!(labels, ["Yes", "No", "Any"]);
    }

    #[test]
    fn roles_come_from_the_catalog() {
        let b = CardQueryBuilder::default();
        assert_eq!(options(ChipSet::RolesInclude, &[], &b), Vec::new());
        let roles = [role("ramp", "Ramp"), role("draw", "Card draw")];
        let chips = options(ChipSet::RolesExclude, &roles, &b);
        assert_eq!(chips.len(), 2);
        assert_eq!(chips[1].label, "Card draw");
        assert_eq!(chips[1].value, "draw");
        let mut b = b;
        tap(ChipSet::RolesInclude, &mut b, &Modes::default(), "draw");
        assert_eq!(b.card_roles_contains_any(), Some(&["draw".to_owned()][..]));
    }

    #[test]
    fn a_tap_on_any_offered_chip_selects_it() {
        let roles = [role("ramp", "Ramp")];
        let mut sets = vec![
            ChipSet::BasicIncludes,
            ChipSet::BasicExcludes,
            ChipSet::Colors,
            ChipSet::ProducedIncludes,
            ChipSet::ProducedExcludes,
            ChipSet::Rarity,
            ChipSet::RolesInclude,
            ChipSet::RolesExclude,
            ChipSet::CommanderIn,
            ChipSet::LegalIn,
            ChipSet::Currency,
            ChipSet::SortBy,
        ];
        sets.extend(Toggle::ALL.map(ChipSet::Config));
        for set in sets {
            for chip in options(set, &roles, &CardQueryBuilder::default()) {
                let mut b = CardQueryBuilder::default();
                b.unset_is_playable()
                    .unset_digital()
                    .unset_oversized()
                    .unset_promo()
                    .unset_content_warning();
                tap(set, &mut b, &Modes::default(), &chip.value);
                assert!(is_selected(set, &b, &chip.value), "{set:?} {chip:?}");
            }
        }
    }

    #[test]
    fn an_unknown_value_changes_nothing() {
        let modes = Modes::default();
        for set in [
            ChipSet::Colors,
            ChipSet::CommanderIn,
            ChipSet::Currency,
            ChipSet::SortBy,
        ] {
            let mut b = CardQueryBuilder::default();
            tap(set, &mut b, &modes, "nonsense");
            assert_eq!(b, CardQueryBuilder::default(), "{set:?}");
        }
    }
}
