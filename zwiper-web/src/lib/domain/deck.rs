//! Deck rules the app keeps in zwiper that zwipe-ui does not hold yet, as
//! pure functions. Each belongs in zwipe-ui's `deck` module.

use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{DeckOtherTag, DeckProfile, Format, PowerLevel};
use zwipe_core::http::contracts::deck::{HttpCreateDeckProfile, HttpUpdateDeckProfile};
use zwipe_core::http::helpers::Patch;
use zwipe_ui::deck::form::{parse_price, price_text};
use zwipe_ui::hint::{Block, HintContent, HintTopic, Span};

// Belongs in zwipe-ui
/// The deck's command-zone art crops for its list row, in the order
/// commander, partner, background, signature spell, only those it has.
pub fn command_zone_art(profile: &DeckProfile) -> Vec<&str> {
    [
        profile.commander_art_url.as_deref(),
        profile.partner_commander_art_url.as_deref(),
        profile.background_art_url.as_deref(),
        profile.signature_spell_art_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

// Belongs in zwipe-ui
/// Whether two slug lists pick the same tags, ignoring order: a picker's
/// `Save` stays off while its pick matches the one it opened on.
pub fn same_set(a: &[String], b: &[String]) -> bool {
    let mut a: Vec<&str> = a.iter().map(String::as_str).collect();
    let mut b: Vec<&str> = b.iter().map(String::as_str).collect();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

/// What the deck form holds, as Create sends it and Edit compares it. The
/// command zone is each card's printing id.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormValues {
    /// The deck name as typed.
    pub name: String,
    /// The format, if picked.
    pub format: Option<Format>,
    /// The commander, or the oathbreaker.
    pub commander: Option<Uuid>,
    /// The partner.
    pub partner: Option<Uuid>,
    /// The background.
    pub background: Option<Uuid>,
    /// The signature spell.
    pub signature_spell: Option<Uuid>,
    /// Deck tag slugs, in pick order.
    pub tags: Vec<String>,
    /// The land target; `None` falls back to the format's count.
    pub land_target: Option<i32>,
    /// The price box's text; empty, zero or unparsable is no budget.
    pub price_text: String,
    /// The price target's currency.
    pub currency: PriceCurrency,
    /// The power level, if picked.
    pub power_level: Option<PowerLevel>,
    /// Other tags, in pick order.
    pub other_tags: Vec<DeckOtherTag>,
    /// Oracle tag slugs, in pick order.
    pub oracle_tags: Vec<String>,
}

impl FormValues {
    /// What the form holds for a saved deck before its command-zone cards
    /// load: every field but the command zone, which each card fills as it
    /// arrives.
    pub fn saved(profile: &DeckProfile) -> Self {
        Self {
            name: profile.name.to_string(),
            format: profile.format,
            commander: None,
            partner: None,
            background: None,
            signature_spell: None,
            tags: profile.tags.clone(),
            land_target: profile.land_target,
            price_text: profile.price_target.map(price_text).unwrap_or_default(),
            currency: profile.price_target_currency.unwrap_or(PriceCurrency::Usd),
            power_level: profile.power_level,
            other_tags: profile.other_tags.clone(),
            oracle_tags: profile.oracle_tags.clone(),
        }
    }
}

/// `Some(v)` for a non-empty list, `None` for an empty one.
fn non_empty<T>(v: Vec<T>) -> Option<Vec<T>> {
    (!v.is_empty()).then_some(v)
}

// Belongs in zwipe-ui
/// Create's request from `values`, as the app builds it: empty lists and a
/// budget of zero or less are left out, and the currency goes only with a
/// budget. The name is checked before this.
pub fn create_request(values: &FormValues) -> HttpCreateDeckProfile {
    let price = parse_price(&values.price_text);
    HttpCreateDeckProfile::builder(&values.name)
        .commander_id(values.commander)
        .partner_commander_id(values.partner)
        .background_id(values.background)
        .signature_spell_id(values.signature_spell)
        .format(values.format.map(|f| f.to_legality_key().to_owned()))
        .tags(non_empty(values.tags.clone()))
        .power_level(values.power_level.map(|p| p.to_string()))
        .other_tags(non_empty(
            values.other_tags.iter().map(ToString::to_string).collect(),
        ))
        .oracle_tags(non_empty(values.oracle_tags.clone()))
        .land_target(values.land_target)
        .price_target(price)
        .price_target_currency(price.map(|_| values.currency))
        .build()
}

/// One field's patch: `Set(new)` when it differs from the original.
fn patch<T: PartialEq, U>(original: &T, current: &T, new: impl FnOnce() -> Option<U>) -> Patch<U> {
    if original == current {
        Patch::Unchanged
    } else {
        Patch::Set(new())
    }
}

// Belongs in zwipe-ui
/// Edit's request: every field that differs from `original`, the rest left
/// off the wire. `None` when nothing changed, which hides `Save changes`.
/// Tags compare in order; the budget compares as parsed, so `50` and `50.0`
/// are the same and clearing it sends a null.
pub fn update_request(
    original: &FormValues,
    current: &FormValues,
) -> Option<HttpUpdateDeckProfile> {
    let name = (original.name != current.name).then_some(current.name.as_str());
    let commander = patch(&original.commander, &current.commander, || {
        current.commander
    });
    let partner = patch(&original.partner, &current.partner, || current.partner);
    let background = patch(&original.background, &current.background, || {
        current.background
    });
    let signature_spell = patch(&original.signature_spell, &current.signature_spell, || {
        current.signature_spell
    });
    let format = patch(&original.format, &current.format, || {
        current.format.map(|f| f.to_legality_key().to_owned())
    });
    let tags = patch(&original.tags, &current.tags, || Some(current.tags.clone()));
    let land_target = patch(&original.land_target, &current.land_target, || {
        current.land_target
    });
    let price_now = parse_price(&current.price_text);
    let price = patch(&parse_price(&original.price_text), &price_now, || price_now);
    let currency = patch(&original.currency, &current.currency, || {
        Some(current.currency)
    });
    let power_level = patch(&original.power_level, &current.power_level, || {
        current.power_level.map(|p| p.to_string())
    });
    let other_tags = patch(&original.other_tags, &current.other_tags, || {
        Some(current.other_tags.iter().map(ToString::to_string).collect())
    });
    let oracle_tags = patch(&original.oracle_tags, &current.oracle_tags, || {
        Some(current.oracle_tags.clone())
    });
    let changed = name.is_some()
        || [
            commander.is_changed(),
            partner.is_changed(),
            background.is_changed(),
            signature_spell.is_changed(),
            format.is_changed(),
            tags.is_changed(),
            land_target.is_changed(),
            price.is_changed(),
            currency.is_changed(),
            power_level.is_changed(),
            other_tags.is_changed(),
            oracle_tags.is_changed(),
        ]
        .contains(&true);
    changed.then(|| {
        HttpUpdateDeckProfile::builder()
            .name(name)
            .commander_id(commander)
            .partner_commander_id(partner)
            .background_id(background)
            .signature_spell_id(signature_spell)
            .format(format)
            .tags(tags)
            .land_target(land_target)
            .price_target(price)
            .price_target_currency(currency)
            .power_level(power_level)
            .other_tags(other_tags)
            .oracle_tags(oracle_tags)
            .build()
    })
}

// Belongs in zwipe-ui
/// The pickers' definition bar before a tap.
pub const PICKER_BAR_HINT: &str = "Hint";

// Belongs in zwipe-ui
/// The format picker's bar before a tap.
pub const FORMAT_BAR_EMPTY: &str = "Tap a format to see its details here.";

// Belongs in zwipe-ui
/// The deck tag picker's bar before a tap.
pub const DECK_TAG_BAR_EMPTY: &str = "Tap a tag to see its definition here.";

// Belongs in zwipe-ui
/// The oracle tag picker's bar before a tap.
pub const ORACLE_TAG_BAR_EMPTY: &str =
    "Tap a tag to see what it means. Picking a deck tag pre-selects some of these for you.";

// Belongs in zwipe-ui
/// The oracle tag picker's bar for a tag with no description.
pub const ORACLE_TAG_NO_DESCRIPTION: &str = "No description for this tag.";

fn bullet(text: &str) -> Block {
    Block::Bullet(vec![Span::Text(text.to_owned())])
}

// Belongs in zwipe-ui
/// The format picker's `?`.
pub fn format_picker_hint() -> HintContent {
    HintContent {
        title: "Format".to_owned(),
        blocks: vec![
            bullet("Tap the picked format again to clear it"),
            bullet("Changing the format clears the commander and signature spell"),
        ],
    }
}

// Belongs in zwipe-ui
/// The deck tag picker's `?`: the deck tags explainer, then how the picker
/// works.
pub fn deck_tag_picker_hint() -> HintContent {
    let mut blocks = HintTopic::DeckTags.blocks();
    blocks.push(bullet(
        "Tap a tag to add or remove it. The bar up top shows what it means",
    ));
    HintContent {
        title: "Deck tags".to_owned(),
        blocks,
    }
}

// Belongs in zwipe-ui
/// The oracle tag picker's `?`, the oracle tags explainer. Its dialog adds a
/// `Dictionary` action.
pub fn oracle_tag_picker_hint() -> HintContent {
    HintContent {
        title: "Oracle tags".to_owned(),
        blocks: HintTopic::OracleTags.blocks(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures;

    fn values() -> FormValues {
        FormValues {
            name: "Goblins".to_owned(),
            format: Some(Format::Commander),
            commander: Some(Uuid::nil()),
            tags: vec!["burn".to_owned(), "aggro".to_owned()],
            price_text: "50".to_owned(),
            ..FormValues::default()
        }
    }

    #[test]
    fn picker_sets_ignore_order() {
        let a = ["x".to_owned(), "y".to_owned()];
        let b = ["y".to_owned(), "x".to_owned()];
        assert!(same_set(&a, &b));
        assert!(!same_set(&a, &b[..1]));
    }

    #[test]
    fn nothing_changed_sends_nothing() {
        assert!(update_request(&values(), &values()).is_none());
    }

    #[test]
    fn the_same_budget_written_two_ways_is_unchanged() {
        let mut now = values();
        now.price_text = "50.0".to_owned();
        assert!(update_request(&values(), &now).is_none());
    }

    #[test]
    fn only_changed_fields_go_on_the_wire() {
        let mut now = values();
        now.price_text = String::new();
        now.commander = None;
        let wire = serde_json::to_value(update_request(&values(), &now).unwrap()).unwrap();
        assert_eq!(
            wire,
            serde_json::json!({ "commander_id": null, "price_target": null })
        );
    }

    #[test]
    fn tag_order_counts_as_a_change() {
        let mut now = values();
        now.tags.reverse();
        assert!(update_request(&values(), &now).is_some());
    }

    #[test]
    fn create_leaves_out_empty_lists_and_a_zero_budget() {
        let mut v = values();
        v.tags.clear();
        v.price_text = "0".to_owned();
        let wire = serde_json::to_value(create_request(&v)).unwrap();
        assert_eq!(wire["tags"], serde_json::Value::Null);
        assert_eq!(wire["price_target"], serde_json::Value::Null);
        assert_eq!(wire["price_target_currency"], serde_json::Value::Null);
        assert_eq!(wire["format"], "commander");
    }

    #[test]
    fn art_follows_the_command_zone_order() {
        let mut profile = test_fixtures::deck_profile("Pair");
        profile.background_art_url = Some("bg".to_owned());
        profile.commander_art_url = Some("cmd".to_owned());
        assert_eq!(command_zone_art(&profile), ["cmd", "bg"]);
    }
}
