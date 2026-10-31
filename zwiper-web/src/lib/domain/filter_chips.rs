//! The chips each chip row of the card filter offers, which of them are
//! selected, and what a tap writes. zwipe-ui names the rows
//! ([`ChipSet`]); the app keeps their chips inline in each filter section.

// Belongs in zwipe-ui: the app keeps these inline in each filter section.

use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::scryfall_data::rarity::Rarity;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Format;
use zwipe_ui::filter::fields::{self, Exclude, MatchGroup, RangeGroup, SideGroup, StrList, Toggle};
use zwipe_ui::filter::sections::{self, ChipSet, Modes};

/// The mana symbols the Produces rows offer: the five colors and colorless.
const PRODUCED: [(&str, &str); 6] = [
    ("W", "White"),
    ("U", "Blue"),
    ("B", "Black"),
    ("R", "Red"),
    ("G", "Green"),
    ("C", "Colorless"),
];

/// One chip: what it shows and the value a tap writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChipOption {
    /// The chip's words; empty for a mana symbol chip.
    pub label: String,
    /// The value the chip stands for.
    pub value: String,
    /// The mana symbol a symbol chip shows (`w`, `u`, `c` and so on).
    pub symbol: Option<String>,
    /// The hover title, a symbol's long name.
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

/// The chips `set` offers. Card roles come from the catalog, so they are
/// empty until it loads.
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

/// The config chip's value as the builder holds it.
fn tri(value: &str) -> Option<bool> {
    match value {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Whether the chip for `value` shows selected.
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
        ChipSet::ProducedExcludes => has(MatchGroup::Produced.exclude().values(b)),
        ChipSet::Rarity => has(SideGroup::Rarity.values(b)),
        ChipSet::RolesInclude => has(MatchGroup::Roles.values(b)),
        ChipSet::RolesExclude => has(MatchGroup::Roles.exclude().values(b)),
        ChipSet::CommanderIn => b
            .is_commander_in_format()
            .is_some_and(|f| f.to_legality_key() == value),
        ChipSet::LegalIn => has(StrList::Legal.get(b)),
        ChipSet::Currency => sections::currency(b).label() == value,
        ChipSet::SortBy => b.sort().is_some_and(|k| k.to_string() == value),
        // The order chip is a plain toggle, never drawn selected.
        ChipSet::SortOrder => false,
        ChipSet::Config(toggle) => toggle.get(b) == tri(value),
    }
}

/// A tap on the chip for `value`: toggles it in or out, or for a single
/// choice picks it, and a second tap on the picked one unsets it.
pub fn tap(set: ChipSet, b: &mut CardQueryBuilder, modes: &Modes, value: &str) {
    match set {
        ChipSet::BasicIncludes => {
            let mode = modes.match_of(b, MatchGroup::BasicTypes);
            let values = fields::toggled(MatchGroup::BasicTypes.values(b), value);
            MatchGroup::BasicTypes.write(b, mode, values);
        }
        ChipSet::BasicExcludes => {
            let values = fields::toggled(Exclude::BasicTypes.values(b), value);
            Exclude::BasicTypes.write(b, values);
        }
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
        ChipSet::LegalIn => {
            let values = fields::toggled(StrList::Legal.get(b), value);
            StrList::Legal.set(b, values);
        }
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
        ChipSet::Config(toggle) => toggle_config(b, toggle, value),
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
    let exclude = group.exclude();
    let values = fields::toggled(exclude.values(b), value);
    exclude.write(b, values);
}

/// A Yes, No or Any chip: a tap on the selected Yes or No goes back to Any.
fn toggle_config(b: &mut CardQueryBuilder, toggle: Toggle, value: &str) {
    toggle.tap(b, tri(value));
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::search_card::card_type::CardType;

    use super::*;

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
    fn single_choice_chips_unset_on_a_second_tap() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        tap(ChipSet::SortBy, &mut b, &modes, "Name");
        assert_eq!(b.sort(), Some(CardSortKey::Name));
        tap(ChipSet::SortBy, &mut b, &modes, "Name");
        assert_eq!(b.sort(), None);
        tap(ChipSet::CommanderIn, &mut b, &modes, "commander");
        assert_eq!(b.is_commander_in_format(), Some(&Format::Commander));
        tap(ChipSet::CommanderIn, &mut b, &modes, "commander");
        assert_eq!(b.is_commander_in_format(), None);
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
    }

    #[test]
    fn colors_and_produced_mana_are_symbols() {
        let b = CardQueryBuilder::default();
        let colors = options(ChipSet::Colors, &[], &b);
        assert_eq!(colors.len(), 5);
        assert_eq!(colors[0].symbol.as_deref(), Some("w"));
        let produced = options(ChipSet::ProducedIncludes, &[], &b);
        assert_eq!(produced.last().map(|c| c.value.as_str()), Some("C"));
        let mut b = b;
        tap(ChipSet::Colors, &mut b, &Modes::default(), "G");
        assert!(b.color_identity_equals().is_some(), "Exact by default");
    }
}
