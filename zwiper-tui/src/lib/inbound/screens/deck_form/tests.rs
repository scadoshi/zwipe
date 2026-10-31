use crossterm::event::KeyCode;
use zwipe_core::domain::deck::{DeckOtherTag, MAX_DECK_ORACLE_TAGS};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck, deck_profile, render, screen_text, user, word_colors};

/// Create over the deck list, with its hint closed and nothing queued.
fn create() -> Harness {
    let mut harness = Harness::on(Screen::Decks(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(open_create);
    harness.app.shared.hint = None;
    harness.clear();
    harness
}

fn form(harness: &Harness) -> &DeckForm {
    match harness.screen() {
        Screen::DeckForm(form) => form,
        other => panic!("not on the deck form: {other:?}"),
    }
}

fn form_mut(harness: &mut Harness) -> &mut DeckForm {
    match harness.screen_mut() {
        Screen::DeckForm(form) => form,
        other => panic!("not on the deck form: {other:?}"),
    }
}

/// Moves the focus to `field`.
fn focus(harness: &mut Harness, field: Field) {
    form_mut(harness).focus = field;
}

fn tag(slug: &str, seeds: &[&str]) -> DeckTagView {
    DeckTagView {
        slug: slug.to_owned(),
        display_name: deck_tag_label(slug),
        description: format!("{slug} decks"),
        seed_otags: seeds.iter().map(|&s| s.to_owned()).collect(),
    }
}

use zwipe_core::domain::deck::deck_tag::deck_tag_label;

fn oracle(slug: &str, curated: bool) -> OracleTag {
    serde_json::from_value(serde_json::json!({
        "slug": slug,
        "label": slug,
        "description": null,
        "parent_slugs": [],
        "curated": curated,
    }))
    .unwrap()
}

#[test]
fn opening_create_loads_both_catalogs_and_shows_the_hint_once() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(open_create);
    assert_eq!(
        harness.pending(),
        [
            screens::Request::DeckForm(Request::DeckTagCatalog),
            screens::Request::DeckForm(Request::OracleCatalog),
            screens::Request::MarkHintShown("create_deck".to_owned()),
        ]
    );
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Building a deck"));
}

#[test]
fn the_name_is_checked_once_it_has_held_text() {
    let mut harness = create();
    harness.type_text("A");
    assert!(form(&harness).name.error().is_none());
    harness.press(KeyCode::Backspace);
    assert_eq!(
        form(&harness).name.error(),
        Some("deck name minimum length is 1 character")
    );
    harness.type_text(&"x".repeat(65));
    assert_eq!(
        form(&harness).name.error(),
        Some("deck name maximum length is 64 characters")
    );
}

#[test]
fn saving_create_checks_the_name_then_sends_the_deck() {
    let mut harness = create();
    harness.press(KeyCode::F(2));
    assert_eq!(harness.pending(), []);
    assert!(form(&harness).name.error().is_some());
    harness.type_text("Tokens");
    focus(&mut harness, Field::Land);
    harness.press(KeyCode::Char('+'));
    harness.press(KeyCode::Char('+'));
    focus(&mut harness, Field::Price);
    harness.type_text("$12a.5");
    focus(&mut harness, Field::OtherTags);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char(' '));
    focus(&mut harness, Field::Power);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::F(2));
    let Some(screens::Request::DeckForm(Request::Create(deck))) = harness.pending().first() else {
        panic!("no create: {:?}", harness.pending());
    };
    assert_eq!(
        **deck,
        NewDeck {
            name: "Tokens".to_owned(),
            commander: None,
            partner: None,
            background: None,
            signature_spell: None,
            format: None,
            tags: None,
            power: Some("core".to_owned()),
            other_tags: Some(vec!["jank".to_owned()]),
            oracle_tags: None,
            land_target: Some(1),
            price_target: Some(12.5),
            currency: Some(PriceCurrency::Usd),
        }
    );
    assert!(form(&harness).saving);
    harness.apply(Update::SaveFailed("deck limit reached".to_owned()));
    assert!(!form(&harness).saving);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "deck limit reached"
    );
    let created = deck_profile("Tokens");
    let id = created.id;
    harness.apply(Update::Created(Box::new(created)));
    assert!(matches!(harness.screen(), Screen::Deck(view) if view.profile.id == id));
    assert_eq!(harness.app.nav.depth(), 2, "the deck replaces the form");
}

#[test]
fn the_format_picker_cascades_and_back_reverts() {
    let mut harness = create();
    focus(&mut harness, Field::Format);
    harness.press(KeyCode::Enter);
    harness.type_text("commander");
    let Some(Overlay::Format(picker)) = &form(&harness).overlay else {
        panic!("the picker is not open");
    };
    assert!(
        picker
            .formats()
            .iter()
            .any(|f| f.display_name() == "Commander")
    );
    harness.press(KeyCode::Right);
    harness.press(KeyCode::Enter);
    let chosen = form(&harness).format;
    assert!(chosen.is_some_and(|f| f.has_commander()));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Pool:"));
    assert!(screen.contains("Life:"));
    harness.press(KeyCode::Esc);
    assert_eq!(form(&harness).format, None, "Back reverts the pick");
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Format unchanged"
    );

    harness.press(KeyCode::Enter);
    harness.type_text("Commander");
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::F(2));
    assert!(form(&harness).overlay.is_none());
    assert!(
        form(&harness)
            .fields()
            .contains(&Field::Card(Slot::Commander))
    );
}

/// Create with the Commander format picked and nothing queued.
fn commander_form() -> Harness {
    let mut harness = create();
    form_mut(&mut harness).set_format(Some(Format::Commander));
    focus(&mut harness, Field::Card(Slot::Commander));
    harness
}

#[test]
fn the_commander_search_waits_for_three_letters_and_the_pause() {
    let mut harness = commander_form();
    harness.type_text("kr");
    assert_eq!(
        form(&harness).card(Slot::Commander).dropdown,
        Dropdown::Hidden
    );
    assert_eq!(harness.pending(), []);
    harness.type_text("e");
    assert_eq!(
        form(&harness).card(Slot::Commander).dropdown,
        Dropdown::Searching
    );
    let generation = form(&harness).card(Slot::Commander).generation;
    assert_eq!(
        harness.pending().last(),
        Some(&screens::Request::DeckForm(Request::Debounce {
            slot: Slot::Commander,
            generation
        }))
    );
    harness.clear();
    harness.apply(Update::Debounced {
        slot: Slot::Commander,
        generation: generation - 1,
    });
    assert_eq!(harness.pending(), [], "a stale pause is dropped");
    harness.apply(Update::Debounced {
        slot: Slot::Commander,
        generation,
    });
    assert!(matches!(
        harness.pending(),
        [screens::Request::DeckForm(Request::Search {
            slot: Slot::Commander,
            ..
        })]
    ));

    harness.apply(Update::Results {
        slot: Slot::Commander,
        generation,
        cards: Vec::new(),
    });
    assert!(screen_text(&harness.app).contains("No results"));
    harness.apply(Update::Results {
        slot: Slot::Commander,
        generation,
        cards: vec![card("Krenko, Mob Boss"), card("Krenko, Tin Street Kingpin")],
    });
    harness.press(KeyCode::Down);
    harness.press(KeyCode::Enter);
    let field = form(&harness).card(Slot::Commander);
    assert_eq!(
        field.card.as_ref().map(|c| c.scryfall_data.name.as_str()),
        Some("Krenko, Tin Street Kingpin")
    );
    assert_eq!(field.input.value(), "Krenko, Tin Street Kingpin");
    assert_eq!(field.dropdown, Dropdown::Hidden);
}

#[test]
fn a_named_partner_fills_itself_in() {
    let mut harness = commander_form();
    let mut pir = card("Pir, Imaginative Rascal");
    pir.scryfall_data.oracle_text = Some("Partner with Toothy, Imaginary Friend".to_owned());
    let pir_id = pir.scryfall_data.id;
    form_mut(&mut harness).card_mut(Slot::Commander).dropdown = Dropdown::Results(vec![pir]);
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [screens::Request::DeckForm(Request::FindPartner {
            commander_id: pir_id,
            mate: "Toothy, Imaginary Friend".to_owned()
        })]
    );
    assert!(
        form(&harness)
            .fields()
            .contains(&Field::Card(Slot::Partner))
    );
    harness.apply(Update::PartnerHits {
        commander_id: pir_id,
        mate: "Toothy, Imaginary Friend".to_owned(),
        cards: vec![card("Toothy, Imaginary Friend")],
    });
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Partner found and selected: Toothy, Imaginary Friend"
    );
    assert!(form(&harness).card(Slot::Partner).card.is_some());

    harness.press(KeyCode::Backspace);
    assert!(form(&harness).card(Slot::Commander).card.is_none());
    assert!(
        form(&harness).card(Slot::Partner).card.is_none(),
        "changing the commander clears the partner"
    );
}

#[test]
fn the_land_stepper_starts_at_the_formats_count() {
    let mut harness = commander_form();
    focus(&mut harness, Field::Land);
    harness.press(KeyCode::Char('-'));
    assert_eq!(form(&harness).land_target, Some(37));
    harness.press(KeyCode::Char('l'));
    assert_eq!(form(&harness).land_target, Some(38));
    harness.press(KeyCode::F(8));
    assert_eq!(form(&harness).land_target, None);
}

#[test]
fn deck_tags_cap_at_five_and_seed_oracle_tags_on_save() {
    let mut harness = create();
    harness.apply(Update::DeckTagCatalog(
        ["tokens", "aristocrats", "voltron", "burn", "mill", "ramp"]
            .iter()
            .map(|s| tag(s, &[&format!("{s}-otag")]))
            .collect(),
    ));
    focus(&mut harness, Field::DeckTags);
    harness.press(KeyCode::Enter);
    for _ in 0..6 {
        harness.press(KeyCode::Enter);
        harness.press(KeyCode::Right);
    }
    assert_eq!(form(&harness).tags.len(), 5);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "You may only choose up to 5 tags"
    );
    // The toast sits over the picker's top row, where the count is.
    harness.app.shared.toasts = crate::inbound::widgets::toast::Toasts::default();
    let screen = screen_text(&harness.app);
    assert!(screen.contains("5/5"));
    harness.press(KeyCode::F(2));
    assert!(form(&harness).overlay.is_none());
    assert_eq!(form(&harness).oracle_tags.len(), 5);
    assert_eq!(
        harness.app.shared.toasts.items().last().unwrap().message,
        "Oracle tags derived from your deck tags"
    );

    harness.press(KeyCode::Enter);
    harness.press(KeyCode::F(8));
    harness.press(KeyCode::Esc);
    assert_eq!(form(&harness).tags.len(), 5, "Back reverts");
    assert!(
        harness
            .app
            .shared
            .toasts
            .items()
            .iter()
            .any(|t| t.message == "Deck tags unchanged")
    );
}

#[test]
fn the_oracle_picker_shows_curated_tags_then_search_hits() {
    let mut harness = create();
    focus(&mut harness, Field::OracleTags);
    harness.press(KeyCode::Enter);
    assert!(
        screen_text(&harness.app).contains("░"),
        "skeleton chips while the catalog loads"
    );
    harness.apply(Update::OracleCatalog(vec![
        oracle("ramp", true),
        oracle("removal", true),
        oracle("obscure", false),
    ]));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("ramp") && !screen.contains("obscure"));
    harness.type_text("obsc");
    assert!(screen_text(&harness.app).contains("obscure"));
    harness.press(KeyCode::Enter);
    assert_eq!(form(&harness).oracle_tags, ["obscure"]);
    let screen = screen_text(&harness.app);
    assert!(screen.contains("No description for this tag."));
    assert!(
        screen.contains("obscure"),
        "picked tags join the default grid"
    );
    harness.press(KeyCode::Esc);
    assert_eq!(form(&harness).oracle_tags, Vec::<String>::new());
}

/// Opens the dictionary from the oracle tag picker with F3 and comes back from it as Use does, with `slug`.
fn use_from_dictionary(harness: &mut Harness, slug: &str) {
    harness.press(KeyCode::F(3));
    assert!(
        matches!(harness.screen(), Screen::OracleTags(_)),
        "F3 opens the dictionary"
    );
    harness.app.shared.hint = None;
    harness.app.shared.caches.handoff.oracle_tag = Some(slug.to_owned());
    harness.press(KeyCode::Esc);
}

#[test]
fn the_dictionarys_use_adds_a_tag_to_the_deck_and_keeps_the_picker_open() {
    let mut harness = create();
    focus(&mut harness, Field::OracleTags);
    harness.press(KeyCode::Enter);
    let last = |harness: &Harness| {
        harness
            .app
            .shared
            .toasts
            .items()
            .last()
            .map(|t| (t.kind, t.message.clone()))
    };

    use_from_dictionary(&mut harness, "ramp");
    assert_eq!(form(&harness).oracle_tags, ["ramp"]);
    assert!(matches!(
        form(&harness).overlay,
        Some(Overlay::OracleTags(_))
    ));
    assert_eq!(
        last(&harness),
        Some((ToastKind::Success, "Tag added to deck".to_owned()))
    );
    assert!(harness.app.shared.caches.handoff.oracle_tag.is_none());

    use_from_dictionary(&mut harness, "ramp");
    assert_eq!(form(&harness).oracle_tags, ["ramp"]);
    assert_eq!(
        last(&harness),
        Some((ToastKind::Info, "Tag already added".to_owned()))
    );

    form_mut(&mut harness).oracle_tags =
        (0..MAX_DECK_ORACLE_TAGS).map(|i| format!("t{i}")).collect();
    use_from_dictionary(&mut harness, "ramp");
    assert!(!form(&harness).oracle_tags.contains(&"ramp".to_owned()));
    assert_eq!(
        last(&harness),
        Some((
            ToastKind::Warning,
            "You may only choose up to 30 oracle tags".to_owned()
        ))
    );
}

#[test]
fn the_swipe_picker_skips_picks_saves_and_undoes() {
    let mut harness = commander_form();
    harness.press(KeyCode::F(4));
    assert!(screen_text(&harness.app).contains("Swipe right picks that commander"));
    harness.app.shared.hint = None;
    let Some(Overlay::Swipe(picker)) = &form(&harness).overlay else {
        panic!("the swipe picker is not open");
    };
    let generation = picker.generation;
    assert!(matches!(
        harness.pending(),
        [screens::Request::DeckForm(Request::SwipePage { .. }), ..]
    ));
    harness.apply(Update::SwipePage {
        generation,
        cards: vec![card("Atraxa"), card("Krenko")],
    });
    assert!(screen_text(&harness.app).contains("Atraxa"));
    harness.press(KeyCode::Down);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Nothing to undo"
    );
    harness.press(KeyCode::Left);
    harness.press(KeyCode::Down);
    assert_eq!(
        harness.app.shared.toasts.items().last().unwrap().message,
        "Undid skip"
    );
    harness.clear();
    harness.press(KeyCode::Up);
    assert!(matches!(
        harness.pending().first(),
        Some(screens::Request::DeckForm(Request::MaybeboardAdd(_)))
    ));
    harness.press(KeyCode::Right);
    assert_eq!(
        harness.app.shared.toasts.items().last().unwrap().message,
        "Commander selected"
    );
    assert!(form(&harness).overlay.is_none());
    assert_eq!(
        form(&harness)
            .commander()
            .map(|c| c.scryfall_data.name.as_str()),
        Some("Krenko")
    );
}

#[test]
fn the_swipe_pickers_filter_opens_the_filter_sheet_and_narrows_the_pile() {
    use crate::domain::filter::sections::Section as FilterSection;

    let picker = |harness: &Harness| -> SwipePicker {
        let Some(Overlay::Swipe(picker)) = &form(harness).overlay else {
            panic!("the swipe picker is not open");
        };
        (**picker).clone()
    };
    let mut harness = commander_form();
    harness.press(KeyCode::F(4));
    harness.app.shared.hint = None;
    harness.press(KeyCode::F(3));
    harness.app.shared.hint = None;
    let Some(sheet) = picker(&harness).sheet else {
        panic!("F3 did not open the filter sheet");
    };
    assert!(
        !sheet.sections().contains(&FilterSection::Format),
        "the swipe picker's sheet has no format section"
    );
    assert!(screen_text(&harness.app).contains(" Filter "));
    render(&harness.app, 12, 5);

    if let Some(Overlay::Swipe(open)) = &mut form_mut(&mut harness).overlay
        && let Some(sheet) = &mut open.sheet
    {
        sheet.draft.set_name_contains("elf");
    }
    harness.clear();
    harness.press(KeyCode::Char('a'));
    let applied = picker(&harness);
    assert!(applied.sheet.is_none(), "Apply closes the sheet");
    assert_eq!(applied.filter.name_contains(), Some("elf"));
    assert!(applied.filter_dot());
    assert!(
        harness.pending().iter().any(|r| matches!(
            r,
            screens::Request::DeckForm(Request::SwipePage { generation, query })
                if *generation == applied.generation && query.criteria().name_contains() == Some("elf")
        )),
        "the pile starts over under the filter: {:?}",
        harness.pending()
    );

    harness.press(KeyCode::Esc);
    harness.press(KeyCode::F(4));
    assert_eq!(
        picker(&harness).filter.name_contains(),
        Some("elf"),
        "the slot keeps its filter while the picker is closed"
    );
}

/// Edit for a deck named Pile with a Modern format and one other tag, loaded.
fn edit() -> (Harness, Uuid) {
    let mut harness = Harness::on(Screen::Decks(Box::default()));
    let mut profile = deck_profile("Pile");
    profile.format = Some(Format::Modern);
    profile.other_tags = vec![DeckOtherTag::Budget];
    profile.price_target = Some(50.0);
    let deck_id = profile.id;
    harness.open(|cx| open_edit(cx, deck_id));
    harness.apply(Update::DeckLoaded(Box::new(deck(profile, Vec::new()))));
    harness.clear();
    (harness, deck_id)
}

#[test]
fn edit_fills_the_form_and_saves_only_what_changed() {
    let (mut harness, deck_id) = edit();
    assert_eq!(form(&harness).name.value(), "Pile");
    assert_eq!(form(&harness).price.value(), "50");
    assert!(!form(&harness).has_changes());
    assert!(!screen_text(&harness.app).contains("Save changes"));
    harness.press(KeyCode::F(2));
    assert_eq!(harness.pending(), [], "nothing to save");

    harness.type_text("s");
    assert!(screen_text(&harness.app).contains("Save changes"));
    harness.press(KeyCode::F(2));
    let Some(screens::Request::DeckForm(Request::Update {
        deck_id: id,
        changes,
    })) = harness.pending().first()
    else {
        panic!("no update: {:?}", harness.pending());
    };
    assert_eq!(*id, deck_id);
    assert_eq!(changes.name.as_deref(), Some("Piles"));
    assert_eq!(changes.format, Patch::Unchanged);
    assert_eq!(changes.price_target, Patch::Unchanged);
    harness.apply(Update::Updated);
    assert!(
        matches!(harness.screen(), Screen::Decks(_)),
        "back to the screen under"
    );
}

#[test]
fn clearing_the_price_clears_the_budget() {
    let (mut harness, _) = edit();
    focus(&mut harness, Field::Price);
    harness.press(KeyCode::F(8));
    let changes = form(&harness).changes().unwrap();
    assert_eq!(changes.price_target, Patch::Set(None));
}

#[test]
fn edit_loads_each_command_zone_card_without_cascading() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut profile = deck_profile("Pair");
    profile.format = Some(Format::Commander);
    let (commander, partner) = (card("Pir"), card("Toothy"));
    profile.commander_id = Some(commander.scryfall_data.id);
    profile.partner_commander_id = Some(partner.scryfall_data.id);
    let deck_id = profile.id;
    harness.open(|cx| open_edit(cx, deck_id));
    harness.clear();
    harness.apply(Update::DeckLoaded(Box::new(deck(profile, Vec::new()))));
    assert_eq!(harness.pending().len(), 2, "one fetch per card");
    harness.apply(Update::CardLoaded {
        slot: Slot::Partner,
        card: Box::new(partner),
    });
    harness.apply(Update::CardLoaded {
        slot: Slot::Commander,
        card: Box::new(commander),
    });
    assert!(form(&harness).card(Slot::Partner).card.is_some());
    assert!(!form(&harness).has_changes());
}

#[test]
fn a_failed_edit_load_says_so() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open_edit(cx, Uuid::new_v4()));
    assert!(screen_text(&harness.app).contains("░"));
    harness.apply(Update::LoadFailed("deck not found".to_owned()));
    assert!(screen_text(&harness.app).contains("Could not load deck"));
}

#[test]
fn f1_explains_the_focused_field() {
    let mut harness = create();
    focus(&mut harness, Field::Land);
    harness.press(KeyCode::F(6));
    assert!(screen_text(&harness.app).contains("Not set uses the format's usual count"));
    harness.press(KeyCode::Esc);
    assert!(form(&harness).overlay.is_none());
}

#[test]
fn enter_in_a_text_field_saves_nothing() {
    let mut harness = create();
    harness.type_text("Tokens");
    harness.press(KeyCode::Enter);
    focus(&mut harness, Field::Price);
    harness.type_text("12");
    harness.press(KeyCode::Enter);
    form_mut(&mut harness).set_format(Some(Format::Commander));
    focus(&mut harness, Field::Card(Slot::Commander));
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [],
        "only F2 saves, as the app's Save button does"
    );
    assert!(!form(&harness).saving);
    harness.press(KeyCode::F(2));
    assert!(form(&harness).saving);
}

#[test]
fn create_seeded_with_a_commander_starts_in_commander() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open_create_with_commander(cx, card("Krenko")));
    assert_eq!(form(&harness).format, Some(Format::Commander));
    assert!(form(&harness).commander().is_some());
}

#[test]
fn the_form_and_its_pickers_fit_a_tiny_terminal() {
    let mut harness = commander_form();
    render(&harness.app, 12, 5);
    render(&harness.app, 1, 1);
    for field in [Field::Format, Field::DeckTags, Field::OracleTags] {
        focus(&mut harness, field);
        harness.press(KeyCode::Enter);
        render(&harness.app, 12, 5);
        harness.press(KeyCode::Esc);
    }
    focus(&mut harness, Field::Card(Slot::Commander));
    harness.press(KeyCode::F(4));
    render(&harness.app, 12, 5);
}

#[test]
fn the_swipe_pickers_keys_use_the_apps_verbs_in_its_hint_colors() {
    let mut harness = commander_form();
    harness.press(KeyCode::F(4));
    harness.app.shared.hint = None;
    let palette = harness.app.shared.palette;
    let row = "→ add  ← skip  ↑ maybeboard  ↓ undo";
    assert!(screen_text(&harness.app).contains(row));
    assert_eq!(
        word_colors(&harness.app, row, &["add", "skip", "maybeboard", "undo"]),
        [
            palette.add.fg,
            palette.error.fg,
            palette.warning.fg,
            palette.tertiary.fg
        ]
    );
    for word in ["pick", "save", "pass"] {
        assert!(
            !screen_text(&harness.app).contains(&format!(" {word} ")),
            "{word}"
        );
    }
}

#[test]
fn the_swipe_picker_flies_skipped_and_saved_cards_off_and_undo_brings_them_back() {
    use crate::domain::stack::Direction;
    use crate::inbound::widgets::card_motion::Phase;

    let motion = |harness: &Harness| {
        let Some(Overlay::Swipe(picker)) = &form(harness).overlay else {
            panic!("the swipe picker is not open");
        };
        picker.motion.clone()
    };
    let mut harness = commander_form();
    harness.press(KeyCode::F(4));
    harness.app.shared.hint = None;
    let Some(Overlay::Swipe(picker)) = &form(&harness).overlay else {
        panic!("the swipe picker is not open");
    };
    let generation = picker.generation;
    harness.apply(Update::SwipePage {
        generation,
        cards: vec![card("Atraxa"), card("Krenko"), card("Edgar")],
    });
    assert!(!harness.screen().animating());
    harness.press(KeyCode::Left);
    let exit = motion(&harness).unwrap();
    assert_eq!(exit.direction(), Direction::Left);
    assert!(matches!(exit.phase(), Phase::Exit(c) if c.scryfall_data.name == "Atraxa"));
    assert!(harness.screen().animating());
    let Some(Overlay::Swipe(picker)) = &form(&harness).overlay else {
        panic!("the swipe picker is not open");
    };
    assert_eq!(
        picker.current().map(|c| c.scryfall_data.name.as_str()),
        Some("Krenko"),
        "the next card is up at once"
    );

    harness.press(KeyCode::Up);
    assert_eq!(motion(&harness).map(|m| m.direction()), Some(Direction::Up));
    harness.press(KeyCode::Down);
    let enter = motion(&harness).unwrap();
    assert_eq!(
        (enter.phase(), enter.direction()),
        (&Phase::Enter, Direction::Up)
    );
    render(&harness.app, 100, 40);
    render(&harness.app, 12, 5);
}

#[test]
fn the_form_and_its_pickers_keep_every_key_on_a_narrow_terminal() {
    use crate::test_support::{assert_key_bar_fits, rows};

    let mut harness = commander_form();
    assert_key_bar_fits(&harness.app, "F2 Save");
    focus(&mut harness, Field::Power);
    let narrow = rows(&harness.app, 60, 20).join("\n");
    assert!(
        narrow.contains("Optimized (4)") && narrow.contains("cEDH (5)"),
        "the power chips wrap instead of running off the box:\n{narrow}"
    );
    for field in [Field::Format, Field::DeckTags, Field::OracleTags] {
        focus(&mut harness, field);
        assert_key_bar_fits(&harness.app, "F2 Save");
        harness.press(KeyCode::Enter);
        assert_key_bar_fits(&harness.app, "F2 Save");
        harness.press(KeyCode::Esc);
    }
    focus(&mut harness, Field::Card(Slot::Commander));
    harness.press(KeyCode::F(4));
    assert_key_bar_fits(&harness.app, "Esc Back");
    let hint = rows(&harness.app, 80, 24).join("\n");
    assert!(
        hint.contains("Swipe right picks that commander") && !hint.contains("commander• Swipe"),
        "each hint bullet starts its own line:\n{hint}"
    );
}
