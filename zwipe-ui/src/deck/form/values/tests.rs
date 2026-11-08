use serde_json::{Value, json};

use super::*;
use crate::test_support::{card, deck_profile};

/// The app's form state before the move: one signal per field, read here as plain values. The two functions below are the app's code from `create.rs` and `edit.rs` with each signal read swapped for a field, so the shared requests can be compared to what the app sent.
#[derive(Clone, Default)]
struct AppForm {
    deck_name: String,
    selected_format: Option<Format>,
    selected_tags: Vec<String>,
    commander: Option<Card>,
    partner_commander: Option<Card>,
    background: Option<Card>,
    signature_spell: Option<Card>,
    land_target: Option<i32>,
    price_target: String,
    price_target_currency: PriceCurrency,
    power_level: Option<PowerLevel>,
    other_tags: Vec<DeckOtherTag>,
    oracle_tags: Vec<String>,
}

impl AppForm {
    fn values(&self) -> FormValues {
        FormValues {
            name: self.deck_name.clone(),
            format: self.selected_format,
            commander: self.commander.clone(),
            partner: self.partner_commander.clone(),
            background: self.background.clone(),
            signature_spell: self.signature_spell.clone(),
            tags: self.selected_tags.clone(),
            oracle_tags: self.oracle_tags.clone(),
            other_tags: self.other_tags.clone(),
            power_level: self.power_level,
            land_target: self.land_target,
            price_text: self.price_target.clone(),
            currency: self.price_target_currency,
        }
    }
}

/// `create.rs` `attempt_submit`, as the app built the request.
fn app_create(f: &AppForm) -> HttpCreateDeckProfile {
    let commander_id = f.commander.clone().map(|c| c.scryfall_data.id);
    let format_str = f.selected_format.map(|f| f.to_legality_key().to_string());
    let tags: Vec<String> = f.selected_tags.clone();
    let other: Vec<String> = f.other_tags.iter().map(ToString::to_string).collect();
    let price_target_val: Option<f64> = f.price_target.parse().ok().filter(|v: &f64| *v > 0.0);
    HttpCreateDeckProfile::builder(&f.deck_name)
        .commander_id(commander_id)
        .partner_commander_id(f.partner_commander.clone().map(|c| c.scryfall_data.id))
        .background_id(f.background.clone().map(|c| c.scryfall_data.id))
        .signature_spell_id(f.signature_spell.clone().map(|c| c.scryfall_data.id))
        .format(format_str)
        .tags(if tags.is_empty() { None } else { Some(tags) })
        .power_level(f.power_level.map(|p| p.to_string()))
        .other_tags(if other.is_empty() { None } else { Some(other) })
        .oracle_tags({
            let ot = f.oracle_tags.clone();
            if ot.is_empty() { None } else { Some(ot) }
        })
        .land_target(f.land_target)
        .price_target(price_target_val)
        .price_target_currency(price_target_val.map(|_| f.price_target_currency))
        .build()
}

/// `edit.rs`'s change memos, `has_made_changes` and `do_submit`, as the app built the request. `None` when the app showed no Save.
#[expect(
    clippy::too_many_lines,
    reason = "a verbatim copy of the app's thirteen change memos"
)]
fn app_update(cur: &AppForm, orig: &AppForm) -> Option<HttpUpdateDeckProfile> {
    let deck_name_update = if cur.deck_name == orig.deck_name {
        None
    } else {
        Some(cur.deck_name.clone())
    };
    let commander_id_update = if cur.commander == orig.commander {
        Patch::Unchanged
    } else {
        Patch::Set(cur.commander.clone().map(|c| c.scryfall_data.id))
    };
    let partner_id_update = if cur.partner_commander == orig.partner_commander {
        Patch::Unchanged
    } else {
        Patch::Set(cur.partner_commander.clone().map(|c| c.scryfall_data.id))
    };
    let background_id_update = if cur.background == orig.background {
        Patch::Unchanged
    } else {
        Patch::Set(cur.background.clone().map(|c| c.scryfall_data.id))
    };
    let signature_spell_id_update = if cur.signature_spell == orig.signature_spell {
        Patch::Unchanged
    } else {
        Patch::Set(cur.signature_spell.clone().map(|c| c.scryfall_data.id))
    };
    let format_update = if cur.selected_format == orig.selected_format {
        Patch::Unchanged
    } else {
        Patch::Set(cur.selected_format.map(|f| f.to_legality_key().to_string()))
    };
    let tags_update = if cur.selected_tags == orig.selected_tags {
        Patch::Unchanged
    } else {
        Patch::Set(Some(cur.selected_tags.clone()))
    };
    let land_target_update = if cur.land_target == orig.land_target {
        Patch::Unchanged
    } else {
        Patch::Set(cur.land_target)
    };
    let price_target_update = {
        let c: Option<f64> = cur.price_target.parse().ok().filter(|v: &f64| *v > 0.0);
        let o: Option<f64> = orig.price_target.parse().ok().filter(|v: &f64| *v > 0.0);
        if c == o {
            Patch::Unchanged
        } else {
            Patch::Set(c)
        }
    };
    let price_currency_update = if cur.price_target_currency == orig.price_target_currency {
        Patch::Unchanged
    } else {
        Patch::Set(Some(cur.price_target_currency))
    };
    let power_level_update = if cur.power_level == orig.power_level {
        Patch::Unchanged
    } else {
        Patch::Set(cur.power_level.map(|p| p.to_string()))
    };
    let other_tags_update = if cur.other_tags == orig.other_tags {
        Patch::Unchanged
    } else {
        Patch::Set(Some(
            cur.other_tags
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        ))
    };
    let oracle_tags_update = if cur.oracle_tags == orig.oracle_tags {
        Patch::Unchanged
    } else {
        Patch::Set(Some(cur.oracle_tags.clone()))
    };
    let has_made_changes = deck_name_update.is_some()
        || commander_id_update.is_changed()
        || partner_id_update.is_changed()
        || background_id_update.is_changed()
        || signature_spell_id_update.is_changed()
        || format_update.is_changed()
        || tags_update.is_changed()
        || land_target_update.is_changed()
        || price_target_update.is_changed()
        || price_currency_update.is_changed()
        || power_level_update.is_changed()
        || other_tags_update.is_changed()
        || oracle_tags_update.is_changed();
    if !has_made_changes {
        return None;
    }
    Some(
        HttpUpdateDeckProfile::builder()
            .name(deck_name_update.as_deref())
            .commander_id(commander_id_update)
            .partner_commander_id(partner_id_update)
            .background_id(background_id_update)
            .signature_spell_id(signature_spell_id_update)
            .format(format_update)
            .tags(tags_update)
            .land_target(land_target_update)
            .price_target(price_target_update)
            .price_target_currency(price_currency_update)
            .power_level(power_level_update)
            .other_tags(other_tags_update)
            .oracle_tags(oracle_tags_update)
            .build(),
    )
}

fn to_json(value: &impl Wire) -> Value {
    value.wire()
}

/// A request as JSON, the form it goes on the wire in.
trait Wire {
    fn wire(&self) -> Value;
}

impl Wire for HttpCreateDeckProfile {
    fn wire(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

impl Wire for HttpUpdateDeckProfile {
    fn wire(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|&s| s.to_owned()).collect()
}

/// A card that can take a partner of any kind.
fn partner_commander(name: &str) -> Card {
    let mut c = card(name);
    c.scryfall_data.keywords = Some(strings(&["Partner"]));
    c
}

/// A card that can choose a background.
fn background_commander(name: &str) -> Card {
    let mut c = card(name);
    c.scryfall_data.oracle_text = Some("Choose a Background".to_owned());
    c
}

/// A full Edit form: every field holds something.
fn full() -> AppForm {
    AppForm {
        deck_name: "Krenko".to_owned(),
        selected_format: Some(Format::Commander),
        selected_tags: strings(&["tokens", "aristocrats"]),
        commander: Some(partner_commander("Krenko")),
        partner_commander: Some(card("Partner")),
        background: Some(card("Background")),
        signature_spell: Some(card("Spell")),
        land_target: Some(36),
        price_target: "50".to_owned(),
        price_target_currency: PriceCurrency::Eur,
        power_level: Some(PowerLevel::Core),
        other_tags: vec![DeckOtherTag::Budget],
        oracle_tags: strings(&["ramp"]),
    }
}

/// An app form edit: its JSON key and how it changes the form.
type Edit = (&'static str, fn(&mut AppForm));

/// Each field set to a new value: the JSON key and the edit.
fn set_edits() -> Vec<Edit> {
    vec![
        ("name", |f| f.deck_name = "Goblins".to_owned()),
        ("commander_id", |f| f.commander = Some(card("Other"))),
        ("partner_commander_id", |f| {
            f.partner_commander = Some(card("Other"));
        }),
        ("background_id", |f| f.background = Some(card("Other"))),
        ("signature_spell_id", |f| {
            f.signature_spell = Some(card("Other"));
        }),
        ("format", |f| f.selected_format = Some(Format::Brawl)),
        ("tags", |f| {
            f.selected_tags = strings(&["aristocrats", "tokens"]);
        }),
        ("land_target", |f| f.land_target = Some(37)),
        ("price_target", |f| f.price_target = "75.5".to_owned()),
        ("price_target_currency", |f| {
            f.price_target_currency = PriceCurrency::Usd;
        }),
        ("power_level", |f| f.power_level = Some(PowerLevel::Cedh)),
        ("other_tags", |f| f.other_tags = vec![DeckOtherTag::Jank]),
        ("oracle_tags", |f| {
            f.oracle_tags = strings(&["ramp", "draw"]);
        }),
    ]
}

/// A clearing edit: its JSON key, the edit and what goes on the wire.
type Clear = (&'static str, fn(&mut AppForm), Value);

/// Each clearable field emptied.
fn clear_edits() -> Vec<Clear> {
    vec![
        ("commander_id", |f| f.commander = None, Value::Null),
        (
            "partner_commander_id",
            |f| f.partner_commander = None,
            Value::Null,
        ),
        ("background_id", |f| f.background = None, Value::Null),
        (
            "signature_spell_id",
            |f| f.signature_spell = None,
            Value::Null,
        ),
        ("format", |f| f.selected_format = None, Value::Null),
        ("tags", |f| f.selected_tags.clear(), json!([])),
        ("land_target", |f| f.land_target = None, Value::Null),
        ("price_target", |f| f.price_target.clear(), Value::Null),
        ("power_level", |f| f.power_level = None, Value::Null),
        ("other_tags", |f| f.other_tags.clear(), json!([])),
        ("oracle_tags", |f| f.oracle_tags.clear(), json!([])),
    ]
}

#[test]
fn nothing_changed_sends_nothing() {
    for form in [AppForm::default(), full()] {
        let values = form.values();
        assert!(values.update_request(&values).is_none());
        assert!(app_update(&form, &form).is_none());
    }
}

#[test]
fn a_set_field_goes_on_the_wire_alone() {
    let saved = full();
    for (key, edit) in set_edits() {
        let mut current = saved.clone();
        edit(&mut current);
        let got = current.values().update_request(&saved.values());
        let got = to_json(&got.unwrap_or_else(|| panic!("{key} changed")));
        let fields: Vec<&String> = got.as_object().unwrap().keys().collect();
        assert_eq!(fields, [key], "only {key} goes");
        assert_ne!(got[key], Value::Null, "{key} is set");
        assert_eq!(
            got,
            to_json(&app_update(&current, &saved).unwrap()),
            "{key} as the app sends it"
        );
    }
}

#[test]
fn a_cleared_field_goes_as_null_or_empty() {
    let saved = full();
    for (key, edit, want) in clear_edits() {
        let mut current = saved.clone();
        edit(&mut current);
        let got = to_json(&current.values().update_request(&saved.values()).unwrap());
        assert_eq!(got, json!({ key: want }), "clear {key}");
        assert_eq!(
            got,
            to_json(&app_update(&current, &saved).unwrap()),
            "clear {key} as the app sends it"
        );
    }
}

#[test]
fn set_values_are_the_wire_forms() {
    let saved = full();
    let mut current = saved.clone();
    for (_, edit) in set_edits() {
        edit(&mut current);
    }
    let got = to_json(&current.values().update_request(&saved.values()).unwrap());
    assert_eq!(got["name"], "Goblins");
    assert_eq!(got["format"], "brawl");
    assert_eq!(got["tags"], json!(["aristocrats", "tokens"]));
    assert_eq!(got["land_target"], 37);
    assert_eq!(got["price_target"], 75.5);
    assert_eq!(got["price_target_currency"], "Usd");
    assert_eq!(got["power_level"], "cedh");
    assert_eq!(got["other_tags"], json!(["jank"]));
    assert_eq!(got["oracle_tags"], json!(["ramp", "draw"]));
    assert_eq!(got, to_json(&app_update(&current, &saved).unwrap()));
}

#[test]
fn the_budget_compares_as_parsed() {
    let saved = full();
    for (text, changed) in [
        ("50", false),
        ("50.0", false),
        ("050", false),
        ("51", true),
        ("0", true),
        ("", true),
        ("1.2.3", true),
    ] {
        let mut current = saved.clone();
        current.price_target = text.to_owned();
        let got = current.values().update_request(&saved.values());
        assert_eq!(got.is_some(), changed, "price box {text:?}");
        assert_eq!(
            got.map(|r| to_json(&r)),
            app_update(&current, &saved).map(|r| to_json(&r)),
            "price box {text:?} as the app sends it"
        );
    }
    let mut empty = AppForm::default();
    let saved_empty = empty.clone();
    empty.price_target = "0".to_owned();
    assert!(
        empty
            .values()
            .update_request(&saved_empty.values())
            .is_none(),
        "zero and empty are both no budget"
    );
}

#[test]
fn the_command_zone_compares_whole_cards() {
    let saved = full();
    let mut current = saved.clone();
    let mut same_printing = saved.commander.clone().unwrap();
    same_printing.scryfall_data.name = "Krenko, refreshed".to_owned();
    current.commander = Some(same_printing);
    let got = current.values().update_request(&saved.values()).unwrap();
    let id = saved.commander.as_ref().unwrap().scryfall_data.id;
    assert_eq!(to_json(&got), json!({ "commander_id": id }));
    assert_eq!(
        to_json(&got),
        to_json(&app_update(&current, &saved).unwrap())
    );
}

#[test]
fn every_change_at_once_matches_the_app() {
    let saved = full();
    let current = AppForm {
        deck_name: "Other".to_owned(),
        ..AppForm::default()
    };
    let got = to_json(&current.values().update_request(&saved.values()).unwrap());
    assert_eq!(got, to_json(&app_update(&current, &saved).unwrap()));
    let fields = got.as_object().unwrap().len();
    assert_eq!(fields, 13, "every field differs");
}

#[test]
fn create_leaves_out_empties() {
    let empty = AppForm {
        deck_name: "New".to_owned(),
        ..AppForm::default()
    };
    let got = to_json(&empty.values().create_request());
    assert_eq!(
        got,
        json!({
            "name": "New",
            "commander_id": null,
            "partner_commander_id": null,
            "background_id": null,
            "signature_spell_id": null,
            "format": null,
        })
    );
    assert_eq!(got, to_json(&app_create(&empty)));

    let form = full();
    let got = to_json(&form.values().create_request());
    assert_eq!(got["tags"], json!(["tokens", "aristocrats"]));
    assert_eq!(got["format"], "commander");
    assert_eq!(got["power_level"], "core");
    assert_eq!(got["other_tags"], json!(["budget"]));
    assert_eq!(got["price_target"], 50.0);
    assert_eq!(got["price_target_currency"], "Eur");
    assert_eq!(got, to_json(&app_create(&form)));
}

#[test]
fn create_sends_a_budget_only_above_zero() {
    for (text, want) in [
        ("0", None),
        ("0.0", None),
        ("-5", None),
        ("", None),
        ("1.2.3", None),
        ("0.01", Some(0.01)),
        ("25", Some(25.0)),
    ] {
        let form = AppForm {
            deck_name: "New".to_owned(),
            price_target: text.to_owned(),
            price_target_currency: PriceCurrency::Tix,
            ..AppForm::default()
        };
        let got = form.values().create_request();
        assert_eq!(got.price_target, want, "price box {text:?}");
        assert_eq!(
            got.price_target_currency,
            want.map(|_| PriceCurrency::Tix),
            "currency rides with the budget, {text:?}"
        );
        assert_eq!(to_json(&got), to_json(&app_create(&form)), "{text:?}");
    }
}

#[test]
fn saved_reads_the_profile_without_its_command_zone() {
    let mut profile = deck_profile("Saved");
    profile.format = Some(Format::Commander);
    profile.price_target = Some(50.0);
    profile.land_target = Some(36);
    profile.tags = strings(&["tokens"]);
    profile.commander_id = Some(Uuid::new_v4());
    let saved = FormValues::saved(&profile);
    assert_eq!(saved.name, "Saved");
    assert_eq!(saved.price_text, "50");
    assert_eq!(
        saved.currency,
        PriceCurrency::Usd,
        "missing currency is USD"
    );
    assert_eq!(saved.land_target, Some(36));
    assert!(Slot::ALL.iter().all(|&s| saved.slot(s).is_none()));
    assert_eq!(FormValues::saved(&deck_profile("Bare")).price_text, "");
}

/// A Commander form with every slot filled, the commander able to take both mates.
fn zone() -> FormValues {
    let mut commander = partner_commander("Leader");
    commander.scryfall_data.oracle_text = Some("Choose a Background".to_owned());
    FormValues {
        format: Some(Format::Commander),
        commander: Some(commander),
        partner: Some(card("Partner")),
        background: Some(card("Background")),
        signature_spell: Some(card("Spell")),
        ..FormValues::default()
    }
}

#[test]
fn picking_a_format_clears_the_commander_and_its_mates() {
    let mut values = zone();
    let cascade = values.pick_format(Format::Modern);
    assert_eq!(values.format, Some(Format::Modern));
    assert!(values.commander.is_none());
    assert!(values.partner.is_none());
    assert!(values.background.is_none());
    assert!(values.signature_spell.is_none());
    assert_eq!(
        cascade,
        Cascade {
            cleared: vec![Slot::Commander, Slot::SignatureSpell],
            reset: vec![Slot::Partner, Slot::Background],
            filters_on: true,
        }
    );
}

#[test]
fn a_format_with_a_signature_spell_keeps_it() {
    let mut values = zone();
    let spell = values.signature_spell.clone();
    let cascade = values.pick_format(Format::Oathbreaker);
    assert!(values.commander.is_none());
    assert_eq!(values.signature_spell, spell);
    assert_eq!(cascade.cleared, [Slot::Commander]);
    assert!(cascade.filters_on);
}

#[test]
fn picking_a_format_with_no_commander_keeps_the_mates() {
    let mut values = zone();
    values.commander = None;
    let cascade = values.pick_format(Format::Commander);
    assert!(values.partner.is_some(), "no commander change, no clear");
    assert!(values.background.is_some());
    assert_eq!(cascade.reset, []);
    assert!(cascade.filters_on);
}

#[test]
fn clearing_the_format_clears_both() {
    let mut values = zone();
    values.format = Some(Format::Oathbreaker);
    let cascade = values.clear_format();
    assert!(values.format.is_none());
    assert!(values.commander.is_none());
    assert!(values.signature_spell.is_none());
    assert!(values.partner.is_none());
    assert_eq!(
        cascade,
        Cascade {
            cleared: vec![Slot::Commander, Slot::SignatureSpell],
            reset: vec![Slot::Partner, Slot::Background],
            filters_on: true,
        }
    );
}

#[test]
fn a_commander_change_clears_the_mates() {
    let mut values = zone();
    let cascade = values.set_commander(Some(card("Another")));
    assert!(values.partner.is_none());
    assert!(values.background.is_none());
    assert!(values.signature_spell.is_some(), "the spell is not a mate");
    assert_eq!(cascade.reset, [Slot::Partner, Slot::Background]);
    assert!(!cascade.filters_on, "only a format change turns them on");

    let mut values = zone();
    values.set_commander(None);
    assert!(values.partner.is_none(), "to none clears too");
}

#[test]
fn the_first_load_and_the_same_commander_keep_the_mates() {
    let mut values = zone();
    let commander = values.commander.take();
    let cascade = values.set_commander(commander.clone());
    assert!(values.partner.is_some(), "None to Some is the first load");
    assert_eq!(cascade, Cascade::default());

    let cascade = values.set_commander(commander);
    assert!(values.partner.is_some(), "the same commander again");
    assert_eq!(cascade, Cascade::default());
}

#[test]
fn clears_mates_follows_the_apps_previous_id() {
    let a = Some(Uuid::new_v4());
    let b = Some(Uuid::new_v4());
    let cases = [
        (None, None, false),
        (None, a, false),
        (a, a, false),
        (a, b, true),
        (a, None, true),
    ];
    for (previous, current, want) in cases {
        assert_eq!(
            clears_mates(previous, current),
            want,
            "{previous:?} to {current:?}"
        );
        // deck_fields.rs before the move: `if previous.is_none() || previous == current { return; }`
        let app = !(previous.is_none() || previous == current);
        assert_eq!(app, want, "the app's guard, {previous:?} to {current:?}");
    }
}

#[test]
fn slots_show_per_format_and_commander() {
    let plain = card("Plain");
    let partner = partner_commander("Partner");
    let background = background_commander("Background");
    for &format in Format::all() {
        let has = format.has_commander();
        let spell = format.has_signature_spell();
        assert_eq!(
            shows(Slot::Commander, Some(format), None),
            has,
            "{format:?}"
        );
        assert_eq!(shows(Slot::SignatureSpell, Some(format), None), spell);
        for (commander, partners, backgrounds) in [
            (None, false, false),
            (Some(&plain), false, false),
            (Some(&partner), true, false),
            (Some(&background), false, true),
        ] {
            assert_eq!(
                shows(Slot::Partner, Some(format), commander),
                has && partners,
                "partner in {format:?}"
            );
            assert_eq!(
                shows(Slot::Background, Some(format), commander),
                has && backgrounds,
                "background in {format:?}"
            );
        }
    }
    for slot in Slot::ALL {
        assert!(
            !shows(slot, None, Some(&partner)),
            "{slot:?} with no format"
        );
    }
    assert!(zone().shows(Slot::Partner));
    assert!(!zone().shows(Slot::SignatureSpell));
}

#[test]
fn the_command_zone_ids_are_every_picked_slot() {
    let tymna = card("Tymna the Weaver");
    let thrasios = card("Thrasios, Triton Hero");
    let mut values = FormValues {
        commander: Some(tymna.clone()),
        ..FormValues::default()
    };
    let ids = |cards: &[&Card]| -> HashSet<Uuid> {
        cards
            .iter()
            .filter_map(|c| c.scryfall_data.oracle_id)
            .collect()
    };
    assert_eq!(values.command_zone_oracle_ids(), ids(&[&tymna]));
    values.partner = Some(thrasios.clone());
    assert_eq!(values.command_zone_oracle_ids(), ids(&[&tymna, &thrasios]));
    assert!(FormValues::default().command_zone_oracle_ids().is_empty());
}
