use crossterm::event::KeyCode;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_core::domain::deck::{DeckOtherTag, Format, MAX_DECK_ORACLE_TAGS};
use zwipe_core::http::helpers::Patch;
use zwipe_ui::screens::deck_form::Mode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::widgets::toast::Toasts;
use crate::test_support::{
    assert_about_marker, card, deck, deck_profile, find_all, render, screen_text, ticket, user,
    word_colors,
};

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

/// Sends `action` to the form on top, as a key handler would.
fn act(harness: &mut Harness, action: Action) {
    let mut open = Box::new(form(harness).clone());
    harness.open(|cx| dispatch(&mut open, action, cx));
    *harness.screen_mut() = Screen::DeckForm(open);
}

/// Moves the focus to `field`.
fn focus(harness: &mut Harness, field: Field) {
    form_mut(harness).focus = field;
}

/// The newest of the form's requests.
fn last(harness: &Harness) -> &Request {
    match harness.pending().iter().rev().find_map(|r| match r {
        screens::Request::DeckForm(request) => Some(request),
        _ => None,
    }) {
        Some(request) => request,
        None => panic!("the form asked for nothing: {:?}", harness.pending()),
    }
}

/// The ticket of the newest debounce.
fn debounce_ticket(harness: &Harness) -> Ticket {
    match last(harness) {
        Request::Debounce { ticket, .. } => *ticket,
        other => panic!("no debounce: {other:?}"),
    }
}

/// Types `text` into the focused command-zone box and lands `cards` as its search.
fn search(harness: &mut Harness, text: &str, cards: Vec<Card>) {
    harness.type_text(text);
    let ticket = debounce_ticket(harness);
    harness.apply(Update::Debounced { ticket });
    assert!(
        matches!(last(harness), Request::Search { ticket: t, .. } if *t == ticket),
        "the pause sends the search"
    );
    harness.apply(Update::Searched { ticket, cards });
}

fn tag(slug: &str, seeds: &[&str]) -> DeckTagView {
    DeckTagView {
        slug: slug.to_owned(),
        display_name: deck_tag_label(slug),
        description: format!("{slug} decks"),
        seed_otags: seeds.iter().map(|&s| s.to_owned()).collect(),
    }
}

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

fn toast_texts(harness: &Harness) -> Vec<String> {
    harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| t.message.clone())
        .collect()
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
    focus(&mut harness, Field::Land);
    harness.press(KeyCode::F(2));
    assert_eq!(harness.pending(), []);
    assert!(form(&harness).name.error().is_some());
    assert_eq!(
        form(&harness).focus,
        Field::Name,
        "the refused name takes the focus"
    );
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
    let Request::Create { ticket, body } = last(&harness).clone() else {
        panic!("no create: {:?}", harness.pending());
    };
    let sent = &body.0;
    assert_eq!(sent.name, "Tokens");
    assert_eq!(sent.power_level.as_deref(), Some("core"));
    assert_eq!(sent.other_tags, Some(vec!["jank".to_owned()]));
    assert_eq!(sent.land_target, Some(1));
    assert_eq!(sent.price_target, Some(12.5));
    assert_eq!(sent.price_target_currency, Some(PriceCurrency::Usd));
    assert_eq!(sent.format, None);
    assert!(form(&harness).model.is_saving());
    harness.apply(Update::Failed {
        ticket,
        message: Some("deck limit reached".to_owned()),
    });
    assert!(!form(&harness).model.is_saving());
    assert_eq!(toast_texts(&harness), ["deck limit reached"]);

    harness.press(KeyCode::F(2));
    let Request::Create { ticket: again, .. } = last(&harness).clone() else {
        panic!("no second create");
    };
    let created = deck_profile("Tokens");
    let id = created.id;
    harness.apply(Update::Created {
        ticket,
        profile: Box::new(deck_profile("Stale")),
    });
    assert!(
        matches!(harness.screen(), Screen::DeckForm(_)),
        "a reply to the failed save changes nothing"
    );
    harness.apply(Update::Created {
        ticket: again,
        profile: Box::new(created),
    });
    assert!(matches!(harness.screen(), Screen::Deck(view) if view.deck_id() == Some(id)));
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
    let chosen = form(&harness).model.values().format;
    assert!(chosen.is_some_and(|f| f.has_commander()));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Pool:"));
    assert!(screen.contains("Life:"));
    harness.press(KeyCode::Esc);
    assert_eq!(
        form(&harness).model.values().format,
        None,
        "Back reverts the pick"
    );
    assert!(form(&harness).overlay.is_none());
    assert_eq!(toast_texts(&harness), ["Format unchanged"]);

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

/// Create with the Commander format picked, the focus on the commander, and nothing queued.
fn commander_form() -> Harness {
    let mut harness = create();
    act(&mut harness, Action::TapFormat(Format::Commander));
    focus(&mut harness, Field::Card(Slot::Commander));
    harness.clear();
    harness
}

#[test]
fn the_commander_search_waits_for_three_letters_and_the_pause() {
    let mut harness = commander_form();
    harness.type_text("kr");
    assert_eq!(form(&harness).model.results(Slot::Commander), None);
    assert_eq!(harness.pending(), []);
    harness.type_text("e");
    assert_eq!(
        form(&harness).model.results(Slot::Commander),
        Some(Results::Searching)
    );
    let first = debounce_ticket(&harness);
    harness.clear();
    harness.apply(Update::Debounced { ticket: ticket(0) });
    assert_eq!(harness.pending(), [], "a stale pause is dropped");
    harness.apply(Update::Debounced { ticket: first });
    assert!(matches!(
        harness.pending(),
        [screens::Request::DeckForm(Request::Search { .. })]
    ));

    harness.apply(Update::Searched {
        ticket: first,
        cards: Vec::new(),
    });
    assert!(screen_text(&harness.app).contains("No results"));
    harness.apply(Update::Searched {
        ticket: first,
        cards: vec![card("Krenko, Mob Boss")],
    });
    assert_eq!(
        form(&harness).model.results(Slot::Commander),
        Some(Results::Empty),
        "a repeated reply changes nothing"
    );

    search(
        &mut harness,
        "n",
        vec![card("Krenko, Mob Boss"), card("Krenko, Tin Street Kingpin")],
    );
    harness.press(KeyCode::Down);
    harness.press(KeyCode::Enter);
    let view = form(&harness);
    assert_eq!(
        view.model
            .values()
            .commander
            .as_ref()
            .map(|c| c.scryfall_data.name.as_str()),
        Some("Krenko, Tin Street Kingpin")
    );
    assert_eq!(
        view.inputs.get(Slot::Commander).input.value(),
        "Krenko, Tin Street Kingpin"
    );
    assert_eq!(view.model.results(Slot::Commander), None);
}

#[test]
fn a_named_partner_fills_itself_in() {
    let mut harness = commander_form();
    let mut pir = card("Pir, Imaginative Rascal");
    pir.scryfall_data.oracle_text = Some("Partner with Toothy, Imaginary Friend".to_owned());
    let pir_id = pir.scryfall_data.id;
    search(&mut harness, "pir", vec![pir]);
    harness.clear();
    harness.press(KeyCode::Enter);
    let Request::FindPartner { picked, mate, .. } = last(&harness).clone() else {
        panic!("no partner search: {:?}", harness.pending());
    };
    assert_eq!(
        (picked, mate.as_str()),
        (pir_id, "Toothy, Imaginary Friend")
    );
    assert!(
        form(&harness)
            .fields()
            .contains(&Field::Card(Slot::Partner))
    );
    harness.apply(Update::PartnerSearched {
        picked,
        mate,
        cards: vec![card("Toothy, Imaginary Friend")],
    });
    assert_eq!(
        toast_texts(&harness),
        ["Partner found and selected: Toothy, Imaginary Friend"]
    );
    assert!(form(&harness).model.values().partner.is_some());
    assert_eq!(
        form(&harness).inputs.get(Slot::Partner).input.value(),
        "Toothy, Imaginary Friend"
    );

    harness.press(KeyCode::Backspace);
    assert!(form(&harness).model.values().commander.is_none());
    assert!(
        form(&harness).model.values().partner.is_none(),
        "changing the commander clears the partner"
    );
}

#[test]
fn the_land_stepper_starts_at_the_formats_count() {
    let mut harness = commander_form();
    focus(&mut harness, Field::Land);
    harness.press(KeyCode::Char('-'));
    assert_eq!(form(&harness).model.values().land_target, Some(37));
    harness.press(KeyCode::Char('l'));
    assert_eq!(form(&harness).model.values().land_target, Some(38));
    harness.press(KeyCode::F(8));
    assert_eq!(form(&harness).model.values().land_target, None);
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
    let deck_tags = |harness: &Harness| form(harness).model.tags(TagPicker::DeckTags).len();
    let oracle_tags = |harness: &Harness| form(harness).model.tags(TagPicker::OracleTags).len();
    assert_eq!(deck_tags(&harness), 5);
    assert_eq!(toast_texts(&harness), ["You may only choose up to 5 tags"]);
    // The toast sits over the picker's top row, where the count is.
    harness.app.shared.toasts = Toasts::default();
    let screen = screen_text(&harness.app);
    assert!(screen.contains("5/5"));
    harness.press(KeyCode::F(2));
    assert!(form(&harness).overlay.is_none());
    assert_eq!(oracle_tags(&harness), 5);
    assert_eq!(
        toast_texts(&harness),
        ["Oracle tags derived from your deck tags"]
    );

    harness.press(KeyCode::Enter);
    harness.press(KeyCode::F(8));
    harness.press(KeyCode::Esc);
    assert_eq!(deck_tags(&harness), 5, "Back reverts");
    assert!(
        toast_texts(&harness)
            .iter()
            .any(|t| t == "Deck tags unchanged")
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
    assert_eq!(
        form(&harness).model.tags(TagPicker::OracleTags),
        ["obscure"]
    );
    let screen = screen_text(&harness.app);
    assert!(screen.contains("No description for this tag."));
    assert!(
        screen.contains("obscure"),
        "picked tags join the default grid"
    );
    harness.press(KeyCode::Esc);
    assert_eq!(
        form(&harness).model.tags(TagPicker::OracleTags).to_vec(),
        Vec::<String>::new()
    );
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
    let newest = |harness: &Harness| {
        harness
            .app
            .shared
            .toasts
            .items()
            .last()
            .map(|t| (t.kind, t.message.clone()))
    };

    use_from_dictionary(&mut harness, "ramp");
    assert_eq!(form(&harness).model.tags(TagPicker::OracleTags), ["ramp"]);
    assert!(matches!(
        form(&harness).overlay,
        Some(Overlay::Tags(TagPicker::OracleTags, _))
    ));
    assert_eq!(
        newest(&harness),
        Some((ToastKind::Success, "Tag added to deck".to_owned()))
    );
    assert!(harness.app.shared.caches.handoff.oracle_tag.is_none());

    use_from_dictionary(&mut harness, "ramp");
    assert_eq!(form(&harness).model.tags(TagPicker::OracleTags), ["ramp"]);
    assert_eq!(
        newest(&harness),
        Some((ToastKind::Info, "Tag already added".to_owned()))
    );

    for i in 1..MAX_DECK_ORACLE_TAGS {
        act(
            &mut harness,
            Action::ToggleTag {
                picker: TagPicker::OracleTags,
                slug: format!("t{i}"),
            },
        );
    }
    use_from_dictionary(&mut harness, "more");
    assert!(
        !form(&harness)
            .model
            .tags(TagPicker::OracleTags)
            .contains(&"more".to_owned())
    );
    assert_eq!(
        newest(&harness),
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
    let generation = picker.generation.current();
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
    assert_eq!(toast_texts(&harness), ["Nothing to undo"]);
    harness.press(KeyCode::Left);
    harness.press(KeyCode::Down);
    assert_eq!(
        toast_texts(&harness).last().map(String::as_str),
        Some("Undid skip")
    );
    harness.clear();
    harness.press(KeyCode::Up);
    assert!(matches!(
        harness.pending().first(),
        Some(screens::Request::DeckForm(Request::MaybeboardAdd(_)))
    ));
    harness.press(KeyCode::Right);
    assert_eq!(
        toast_texts(&harness).last().map(String::as_str),
        Some("Commander selected")
    );
    assert!(form(&harness).overlay.is_none());
    assert_eq!(
        form(&harness)
            .model
            .values()
            .commander
            .as_ref()
            .map(|c| c.scryfall_data.name.as_str()),
        Some("Krenko")
    );
    assert_eq!(
        form(&harness).inputs.get(Slot::Commander).input.value(),
        "Krenko"
    );
}

#[test]
fn the_swipe_pickers_filter_opens_the_filter_sheet_and_narrows_the_pile() {
    use zwipe_ui::filter::sections::Section as FilterSection;

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
        sheet.filter.draft.set_name_contains("elf");
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
                if applied.generation.is_current(*generation) && query.criteria().name_contains() == Some("elf")
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

/// Edit for a deck named Pile with a Modern format, one other tag and a budget, loaded, with nothing queued.
fn edit() -> (Harness, Uuid) {
    let mut harness = Harness::on(Screen::Decks(Box::default()));
    let mut profile = deck_profile("Pile");
    profile.format = Some(Format::Modern);
    profile.other_tags = vec![DeckOtherTag::Budget];
    profile.price_target = Some(50.0);
    let deck_id = profile.id;
    harness.open(|cx| open_edit(cx, deck_id));
    let Request::LoadDeck { ticket, .. } = *last(&harness) else {
        panic!("Edit loads its deck");
    };
    harness.apply(Update::Loaded {
        ticket,
        deck: Box::new(deck(profile, Vec::new())),
    });
    harness.clear();
    (harness, deck_id)
}

#[test]
fn edit_fills_the_form_and_saves_only_what_changed() {
    let (mut harness, deck_id) = edit();
    assert_eq!(form(&harness).name.value(), "Pile");
    assert_eq!(form(&harness).price.value(), "50");
    assert!(!form(&harness).model.has_changes());
    assert!(!screen_text(&harness.app).contains("Save changes"));
    harness.press(KeyCode::F(2));
    assert_eq!(harness.pending(), [], "nothing to save");
    assert_eq!(toast_texts(&harness), Vec::<String>::new());

    harness.type_text("s");
    assert!(screen_text(&harness.app).contains("Save changes"));
    harness.press(KeyCode::F(2));
    let Request::Update {
        ticket,
        deck_id: id,
        body,
    } = last(&harness).clone()
    else {
        panic!("no update: {:?}", harness.pending());
    };
    assert_eq!(id, deck_id);
    assert_eq!(body.0.name, Patch::Set(Some("Piles".to_owned())));
    assert_eq!(body.0.format, Patch::Unchanged);
    assert_eq!(body.0.price_target, Patch::Unchanged);
    harness.apply(Update::Updated { ticket });
    assert!(
        matches!(harness.screen(), Screen::Decks(_)),
        "back to the screen under"
    );
}

#[test]
fn a_late_deck_load_changes_nothing() {
    let (mut harness, _) = edit();
    harness.type_text("s");
    let mut profile = deck_profile("Elsewhere");
    profile.format = Some(Format::Modern);
    harness.apply(Update::Loaded {
        ticket: ticket(1),
        deck: Box::new(deck(profile, Vec::new())),
    });
    assert_eq!(
        form(&harness).name.value(),
        "Piles",
        "a repeated load leaves the edit"
    );
}

#[test]
fn clearing_the_price_clears_the_budget() {
    let (mut harness, _) = edit();
    focus(&mut harness, Field::Price);
    harness.press(KeyCode::F(8));
    harness.press(KeyCode::F(2));
    let Request::Update { body, .. } = last(&harness) else {
        panic!("no update: {:?}", harness.pending());
    };
    assert_eq!(body.0.price_target, Patch::Set(None));
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
    let Request::LoadDeck { ticket, .. } = *last(&harness) else {
        panic!("Edit loads its deck");
    };
    harness.clear();
    harness.apply(Update::Loaded {
        ticket,
        deck: Box::new(deck(profile, Vec::new())),
    });
    let fetches: Vec<(Slot, Ticket)> = harness
        .pending()
        .iter()
        .filter_map(|r| match r {
            screens::Request::DeckForm(Request::FetchCard { ticket, slot, .. }) => {
                Some((*slot, *ticket))
            }
            _ => None,
        })
        .collect();
    assert_eq!(fetches.len(), 2, "one fetch per card");
    for (slot, ticket) in fetches.into_iter().rev() {
        let card = match slot {
            Slot::Commander => commander.clone(),
            _ => partner.clone(),
        };
        harness.apply(Update::CardLoaded {
            ticket,
            card: Box::new(card),
        });
    }
    assert!(form(&harness).model.values().partner.is_some());
    assert_eq!(
        form(&harness).inputs.get(Slot::Partner).input.value(),
        "Toothy"
    );
    assert!(!form(&harness).model.has_changes());
}

#[test]
fn a_failed_edit_load_says_so() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open_edit(cx, Uuid::new_v4()));
    let Request::LoadDeck { ticket, .. } = *last(&harness) else {
        panic!("Edit loads its deck");
    };
    assert!(screen_text(&harness.app).contains("░"));
    harness.apply(Update::Failed {
        ticket,
        message: Some("deck not found".to_owned()),
    });
    assert_eq!(toast_texts(&harness), ["deck not found"]);
    harness.app.shared.toasts = Toasts::default();
    assert!(screen_text(&harness.app).contains("Could not load deck"));
}

#[test]
fn i_explains_the_focused_field_and_f6_does_nothing() {
    let mut harness = create();
    focus(&mut harness, Field::Land);
    harness.press(KeyCode::F(6));
    assert!(harness.app.shared.hint.is_none(), "F6 is gone");
    assert!(!screen_text(&harness.app).contains("F6"));
    harness.press(KeyCode::Char('i'));
    assert!(screen_text(&harness.app).contains("Not set uses the format's usual count"));
}

#[test]
fn every_field_has_an_about_marker_lit_while_i_opens_it() {
    let mut harness = commander_form();
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    let rows = crate::test_support::rows(&harness.app, 100, 40);
    let bare = crate::test_support::bare_about_markers(&rows);
    assert!(bare.is_empty(), "bare ? at {bare:?}:\n{}", rows.join("\n"));
    let marker_after = |harness: &Harness, label: &str| {
        let buffer = render(&harness.app, 100, 40).backend().buffer().clone();
        let Some(&(x, y)) = find_all(&buffer, &format!("{label} [?]")).first() else {
            panic!("no marker after {label}:\n{}", screen_text(&harness.app));
        };
        let width = u16::try_from(label.chars().count()).unwrap();
        (buffer, (x + width + 1, y))
    };
    for label in [
        "Deck name",
        "Format",
        "Commander",
        "Power level",
        "Land target",
        "Price target",
    ] {
        marker_after(&harness, label);
    }
    focus(&mut harness, Field::Land);
    let (buffer, at) = marker_after(&harness, "Land target");
    assert_about_marker(&buffer, &palette, at, true, "the focused field");
    let (buffer, at) = marker_after(&harness, "Format");
    assert_about_marker(&buffer, &palette, at, false, "a field at rest");
    focus(&mut harness, Field::Card(Slot::Commander));
    let (buffer, at) = marker_after(&harness, "Commander");
    assert_about_marker(&buffer, &palette, at, false, "a box that takes letters");
}

#[test]
fn enter_in_a_text_field_saves_nothing() {
    let mut harness = create();
    harness.type_text("Tokens");
    harness.press(KeyCode::Enter);
    focus(&mut harness, Field::Price);
    harness.type_text("12");
    harness.press(KeyCode::Enter);
    act(&mut harness, Action::TapFormat(Format::Commander));
    focus(&mut harness, Field::Card(Slot::Commander));
    harness.press(KeyCode::Enter);
    assert!(
        !harness
            .pending()
            .iter()
            .any(|r| matches!(r, screens::Request::DeckForm(Request::Create { .. }))),
        "only F2 saves, as the app's Save button does"
    );
    assert!(!form(&harness).model.is_saving());
    harness.press(KeyCode::F(2));
    assert!(form(&harness).model.is_saving());
}

#[test]
fn create_seeded_with_a_commander_starts_in_commander() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open_create_with_commander(cx, card("Krenko")));
    let view = form(&harness);
    assert_eq!(view.model.mode(), Mode::Create);
    assert_eq!(view.model.values().format, Some(Format::Commander));
    assert!(view.model.values().commander.is_some());
    assert_eq!(view.inputs.get(Slot::Commander).input.value(), "Krenko");
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
    use zwipe_ui::stack::Direction;

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
    let generation = picker.generation.current();
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
