use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{DeckOtherTag, DeckTagView, Format, MAX_DECK_TAGS, PowerLevel};
use zwipe_core::http::helpers::Patch;

use super::*;
use crate::test_support::{card, deck_profile};
use crate::toast::{NORMAL, QUICK};

/// A commander whose oracle text names `mate` as its partner.
fn partner_with(name: &str, mate: &str) -> Card {
    let mut card = card(name);
    card.scryfall_data.oracle_text = Some(format!("Partner with {mate}"));
    card
}

/// An open Create form with no seed.
fn create() -> State {
    let mut state = State::new();
    let _ = state.update(Action::Load(Open::Create { seed: None }));
    state
}

/// An Edit form whose deck fetch is out, with the ticket it holds.
fn edit_loading(deck_id: Uuid) -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load(Open::Edit(deck_id)));
    let [
        Effect::OfferHint(OneTimeHint::EditDeck),
        Effect::FetchDeck {
            ticket,
            deck_id: id,
        },
    ] = effects.as_slice()
    else {
        panic!("Edit fetches its deck: {effects:?}");
    };
    assert_eq!(*id, deck_id);
    (state, *ticket)
}

/// An Edit form loaded with `profile` and every command-zone card it names.
fn edit_loaded(profile: DeckProfile, cards: &[(Slot, Card)]) -> State {
    let (mut state, ticket) = edit_loading(profile.id);
    let effects = state.update(Action::Loaded {
        ticket,
        profile: Box::new(profile),
    });
    for effect in effects {
        let Effect::FetchCard { ticket, slot, .. } = effect else {
            panic!("only card fetches follow a load: {effect:?}");
        };
        let card = cards
            .iter()
            .find(|(s, _)| *s == slot)
            .map(|(_, c)| c.clone())
            .unwrap();
        let _ = state.update(Action::CardLoaded {
            ticket,
            card: Box::new(card),
        });
    }
    state
}

/// Types `text` into `slot` and returns the debounce's ticket.
fn type_search(state: &mut State, slot: Slot, text: &str) -> Ticket {
    let effects = state.update(Action::Type {
        slot,
        text: text.to_owned(),
    });
    let [Effect::Debounce { ticket, wait }] = effects.as_slice() else {
        panic!("a long enough query waits: {effects:?}");
    };
    assert_eq!(*wait, timing::COMMAND_ZONE_SEARCH_DEBOUNCE);
    *ticket
}

/// No tags.
const NONE: &[String] = &[];

fn toasts(effects: &[Effect]) -> Vec<Toast> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Toast(t) => Some(t.clone()),
            _ => None,
        })
        .collect()
}

fn navigation(effects: &[Effect]) -> Vec<Destination> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Navigate(to) => Some(*to),
            _ => None,
        })
        .collect()
}

fn names(state: &State, slot: Slot) -> Option<String> {
    state
        .values()
        .slot(slot)
        .map(|c| c.scryfall_data.name.clone())
}

#[test]
fn create_opens_ready_and_offers_its_hint() {
    let mut state = State::new();
    let effects = state.update(Action::Load(Open::Create { seed: None }));
    assert!(matches!(
        effects.as_slice(),
        [Effect::OfferHint(OneTimeHint::CreateDeck)]
    ));
    assert_eq!(state.status(), Status::Ready);
    assert_eq!(state.mode().title(), "Create deck");
    assert_eq!(state.save_label(), Some("Save"));
    assert_eq!(state.name_error(), None, "an untouched name shows no error");
}

#[test]
fn a_seeded_commander_comes_in_under_commander_with_no_partner_search() {
    let seed = partner_with("Pir, Imaginative Rascal", "Toothy, Imaginary Friend");
    let mut state = State::new();
    let effects = state.update(Action::Load(Open::Create {
        seed: Some(Box::new(seed)),
    }));
    assert_eq!(effects.len(), 1, "only the hint: {effects:?}");
    assert_eq!(state.values().format, Some(Format::Commander));
    assert_eq!(state.display(Slot::Commander), "Pir, Imaginative Rascal");
    assert_eq!(
        names(&state, Slot::Commander).as_deref(),
        Some("Pir, Imaginative Rascal")
    );
    assert!(state.shows(Slot::Partner));
}

#[test]
fn edit_loads_the_deck_then_its_command_zone() {
    let mut profile = deck_profile("Pirates");
    profile.format = Some(Format::Commander);
    profile.tags = vec!["ramp".to_owned()];
    let commander = card("Malcolm");
    let partner = card("Tymna");
    profile.commander_id = Some(commander.scryfall_data.id);
    profile.partner_commander_id = Some(partner.scryfall_data.id);
    let (mut state, ticket) = edit_loading(profile.id);
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.save_label(), None);

    let effects = state.update(Action::Loaded {
        ticket,
        profile: Box::new(profile),
    });
    let fetched: Vec<(Slot, Uuid)> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::FetchCard { slot, card_id, .. } => Some((*slot, *card_id)),
            _ => None,
        })
        .collect();
    assert_eq!(
        fetched,
        [
            (Slot::Commander, commander.scryfall_data.id),
            (Slot::Partner, partner.scryfall_data.id)
        ]
    );
    assert_eq!(state.status(), Status::Ready);
    assert_eq!(state.values().name, "Pirates");
    assert_eq!(state.save_label(), None, "nothing changed");

    // The partner lands before the commander, and the commander keeps it.
    let tickets: Vec<Ticket> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::FetchCard { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .collect();
    let _ = state.update(Action::CardLoaded {
        ticket: tickets[1],
        card: Box::new(partner),
    });
    let _ = state.update(Action::CardLoaded {
        ticket: tickets[0],
        card: Box::new(commander),
    });
    assert_eq!(names(&state, Slot::Partner).as_deref(), Some("Tymna"));
    assert_eq!(state.display(Slot::Commander), "Malcolm");
    assert_eq!(state.display(Slot::Partner), "Tymna");
    assert!(!state.has_changes(), "loaded cards are saved cards");
}

#[test]
fn a_gone_deck_is_replaced_by_the_deck_list() {
    let (mut state, ticket) = edit_loading(Uuid::new_v4());
    assert!(matches!(
        state.update(Action::Gone { ticket }).as_slice(),
        [Effect::Replace(Destination::DeckList)]
    ));
    assert_eq!(state.status(), Status::Failed);
}

#[test]
fn a_failed_deck_load_shows_the_failure_line() {
    let (mut state, ticket) = edit_loading(Uuid::new_v4());
    assert!(state.update(Action::Failed { ticket }).is_empty());
    assert_eq!(state.status(), Status::Failed);
    assert!(
        state.update(Action::Save).is_empty(),
        "no save off a failed load"
    );
}

#[test]
fn a_late_deck_reply_is_ignored() {
    let (mut state, ticket) = edit_loading(Uuid::new_v4());
    let _ = state.update(Action::Failed { ticket });
    let effects = state.update(Action::Loaded {
        ticket,
        profile: Box::new(deck_profile("Late")),
    });
    assert!(effects.is_empty());
    assert_eq!(state.status(), Status::Failed);
    assert_eq!(state.values().name, "");
}

#[test]
fn a_failed_card_load_leaves_the_slot_empty() {
    let mut profile = deck_profile("Deck");
    profile.format = Some(Format::Commander);
    profile.commander_id = Some(Uuid::new_v4());
    let (mut state, ticket) = edit_loading(profile.id);
    let effects = state.update(Action::Loaded {
        ticket,
        profile: Box::new(profile),
    });
    let [Effect::FetchCard { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let ticket = *ticket;
    assert!(state.update(Action::Failed { ticket }).is_empty());
    assert_eq!(state.status(), Status::Ready);
    let late = state.update(Action::CardLoaded {
        ticket,
        card: Box::new(card("Late")),
    });
    assert!(late.is_empty());
    assert_eq!(
        names(&state, Slot::Commander),
        None,
        "a late card is ignored"
    );
}

#[test]
fn the_name_error_shows_once_the_name_held_text() {
    let mut state = create();
    let _ = state.update(Action::SetName(String::new()));
    assert_eq!(state.name_error(), None, "never held text");
    let _ = state.update(Action::SetName("Burn".to_owned()));
    assert_eq!(state.name_error(), None);
    let _ = state.update(Action::SetName(String::new()));
    assert_eq!(state.name_error(), Some("Deck name is required"));
}

#[test]
fn save_refuses_a_bad_name_inline() {
    let mut state = create();
    assert!(state.update(Action::Save).is_empty());
    assert_eq!(state.name_error(), Some("Deck name is required"));
    assert!(!state.is_saving());
}

#[test]
fn create_sends_the_form_and_opens_the_new_deck() {
    let mut state = create();
    let _ = state.update(Action::SetName("Burn".to_owned()));
    let effects = state.update(Action::Save);
    let [Effect::CreateDeck { ticket, request }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(request.name, "Burn");
    assert!(state.is_saving());
    assert_eq!(state.save_label(), Some("Saving..."));
    assert!(state.update(Action::Save).is_empty(), "one save at a time");
    assert!(
        state.update(Action::Back).is_empty(),
        "Back is off while saving"
    );

    let deck_id = Uuid::new_v4();
    let effects = state.update(Action::Created {
        ticket: *ticket,
        deck_id,
    });
    assert_eq!(navigation(&effects), [Destination::Deck(deck_id)]);
    assert!(toasts(&effects).is_empty(), "no success toast");
}

#[test]
fn a_failed_create_turns_save_back_on() {
    let mut state = create();
    let _ = state.update(Action::SetName("Burn".to_owned()));
    let effects = state.update(Action::Save);
    let [Effect::CreateDeck { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let ticket = *ticket;
    assert!(state.update(Action::Failed { ticket }).is_empty());
    assert!(!state.is_saving());
    assert_eq!(state.save_label(), Some("Save"));
    let late = state.update(Action::Created {
        ticket,
        deck_id: Uuid::new_v4(),
    });
    assert!(late.is_empty(), "a late reply is ignored");
}

#[test]
fn edit_sends_only_what_changed_and_returns_to_the_deck() {
    let profile = deck_profile("Burn");
    let deck_id = profile.id;
    let mut state = edit_loaded(profile, &[]);
    let _ = state.update(Action::SetName("Burn 2".to_owned()));
    assert_eq!(state.save_label(), Some("Save"));
    let effects = state.update(Action::Save);
    let [
        Effect::UpdateDeck {
            ticket,
            deck_id: id,
            request,
        },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert_eq!(*id, deck_id);
    assert!(format!("{:?}", request.name).contains("Burn 2"));
    assert_eq!(request.format, Patch::Unchanged);
    let effects = state.update(Action::Updated { ticket: *ticket });
    assert!(!state.is_saving());
    assert_eq!(navigation(&effects), [Destination::Deck(deck_id)]);
    assert!(toasts(&effects).is_empty(), "no success toast");
}

#[test]
fn edit_with_nothing_changed_says_so() {
    let mut state = edit_loaded(deck_profile("Burn"), &[]);
    let effects = state.update(Action::Save);
    assert_eq!(
        toasts(&effects),
        [Toast::new(
            ToastKind::Error,
            "must update at least one field",
            NORMAL
        )]
    );
    assert!(!state.is_saving());
}

#[test]
fn back_goes_where_each_mode_came_from() {
    let mut state = create();
    assert_eq!(
        navigation(&state.update(Action::Back)),
        [Destination::Previous]
    );
    let profile = deck_profile("Burn");
    let deck_id = profile.id;
    let mut state = edit_loaded(profile, &[]);
    assert_eq!(
        navigation(&state.update(Action::Back)),
        [Destination::Deck(deck_id)]
    );
}

#[test]
fn a_short_query_closes_the_results_and_a_long_one_searches_after_the_debounce() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Commander));
    let effects = state.update(Action::Type {
        slot: Slot::Commander,
        text: "kr".to_owned(),
    });
    assert!(effects.is_empty());
    assert_eq!(state.results(Slot::Commander), None);
    assert_eq!(state.display(Slot::Commander), "kr");

    let ticket = type_search(&mut state, Slot::Commander, "kre");
    assert_eq!(state.results(Slot::Commander), Some(Results::Searching));
    let effects = state.update(Action::Debounced { ticket });
    let [
        Effect::SearchCards {
            ticket: sent,
            query,
        },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert_eq!(*sent, ticket);
    assert_eq!(query.limit(), 5);

    let found = card("Krenko");
    let _ = state.update(Action::Searched {
        ticket,
        cards: vec![found.clone()],
    });
    assert_eq!(
        state.results(Slot::Commander),
        Some(Results::Found(vec![found]))
    );
}

#[test]
fn partner_and_background_search_from_one_character() {
    let mut state = create();
    let _ = type_search(&mut state, Slot::Partner, "a");
    let _ = type_search(&mut state, Slot::Background, "b");
    let _ = type_search(&mut state, Slot::SignatureSpell, "abc");
}

#[test]
fn a_late_search_reply_is_ignored() {
    let mut state = create();
    let older = type_search(&mut state, Slot::Partner, "a");
    let newer = type_search(&mut state, Slot::Partner, "ab");
    assert!(state.update(Action::Debounced { ticket: older }).is_empty());
    let _ = state.update(Action::Searched {
        ticket: older,
        cards: vec![card("Old")],
    });
    assert_eq!(state.results(Slot::Partner), Some(Results::Searching));
    let _ = state.update(Action::Searched {
        ticket: newer,
        cards: Vec::new(),
    });
    assert_eq!(state.results(Slot::Partner), Some(Results::Empty));
}

#[test]
fn a_failed_search_closes_the_results() {
    let mut state = create();
    let ticket = type_search(&mut state, Slot::Background, "x");
    assert!(state.update(Action::Failed { ticket }).is_empty());
    assert_eq!(state.results(Slot::Background), None);
}

#[test]
fn the_filter_narrows_the_search_the_debounce_sends() {
    let mut state = create();
    let ticket = type_search(&mut state, Slot::Partner, "a");
    assert!(state.update(Action::ToggleFilter(Slot::Partner)).is_empty());
    assert!(!state.filter_on(Slot::Partner));
    let effects = state.update(Action::Debounced { ticket });
    let [Effect::SearchCards { query, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let sent = serde_json::to_value(&**query).unwrap();
    assert!(
        sent.get("is_partner").is_none(),
        "the Filter was off when it fired: {sent}"
    );
}

#[test]
fn typing_keeps_the_card_and_tapping_the_input_clears_it() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Commander));
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Commander,
        card: Box::new(card("Krenko")),
    });
    let _ = state.update(Action::Type {
        slot: Slot::Commander,
        text: "z".to_owned(),
    });
    assert_eq!(names(&state, Slot::Commander).as_deref(), Some("Krenko"));
    assert_eq!(state.display(Slot::Commander), "z");
    let _ = state.update(Action::TapInput(Slot::Commander));
    assert_eq!(names(&state, Slot::Commander), None);
    assert_eq!(state.display(Slot::Commander), "");
    assert!(!state.has_query(Slot::Commander));
}

#[test]
fn the_input_clear_keeps_the_card_and_the_label_clear_drops_it() {
    let mut state = create();
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Partner,
        card: Box::new(card("Tymna")),
    });
    let _ = type_search(&mut state, Slot::Partner, "ty");
    assert!(state.has_query(Slot::Partner));
    let _ = state.update(Action::ClearSearch(Slot::Partner));
    assert_eq!(names(&state, Slot::Partner).as_deref(), Some("Tymna"));
    assert_eq!(state.display(Slot::Partner), "");
    assert_eq!(state.results(Slot::Partner), None);
    let _ = state.update(Action::ClearSlot(Slot::Partner));
    assert_eq!(names(&state, Slot::Partner), None);
}

#[test]
fn a_result_pick_closes_the_results_and_a_swipe_pick_leaves_them() {
    let mut state = create();
    let ticket = type_search(&mut state, Slot::Background, "a");
    let _ = state.update(Action::Searched {
        ticket,
        cards: vec![card("Agent")],
    });
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Background,
        card: Box::new(card("Swiped")),
    });
    assert!(state.results(Slot::Background).is_some());
    assert_eq!(state.display(Slot::Background), "Swiped");
    let _ = state.update(Action::PickResult {
        slot: Slot::Background,
        card: Box::new(card("Agent")),
    });
    assert_eq!(state.results(Slot::Background), None);
    assert_eq!(state.display(Slot::Background), "Agent");
    assert!(state.has_query(Slot::Background), "the typed text stays");
}

#[test]
fn a_new_commander_clears_the_partner_and_background() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Commander));
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Commander,
        card: Box::new(card("First")),
    });
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Partner,
        card: Box::new(card("Mate")),
    });
    let _ = type_search(&mut state, Slot::Background, "bg");
    let _ = state.update(Action::PickResult {
        slot: Slot::Commander,
        card: Box::new(card("Second")),
    });
    assert_eq!(names(&state, Slot::Partner), None);
    assert_eq!(state.display(Slot::Partner), "");
    assert!(!state.has_query(Slot::Background));
    assert_eq!(state.results(Slot::Background), None);
}

#[test]
fn a_partner_with_commander_fills_its_partner_and_says_so() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Commander));
    let pir = partner_with("Pir, Imaginative Rascal", "Toothy, Imaginary Friend");
    let picked = pir.scryfall_data.id;
    let effects = state.update(Action::PickResult {
        slot: Slot::Commander,
        card: Box::new(pir),
    });
    let [Effect::FindPartner { mate, query, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(mate, "Toothy, Imaginary Friend");
    assert_eq!(query.limit(), 5);
    let effects = state.update(Action::PartnerSearched {
        picked,
        mate: mate.clone(),
        cards: vec![card("Toothy, Imaginary Friend")],
    });
    assert_eq!(
        toasts(&effects),
        [Toast::new(
            ToastKind::Info,
            "Partner found and selected: Toothy, Imaginary Friend",
            NORMAL
        )]
    );
    assert_eq!(state.display(Slot::Partner), "Toothy, Imaginary Friend");
}

#[test]
fn a_partner_reply_for_an_older_commander_is_ignored() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Commander));
    let pir = partner_with("Pir, Imaginative Rascal", "Toothy, Imaginary Friend");
    let picked = pir.scryfall_data.id;
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Commander,
        card: Box::new(pir),
    });
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Commander,
        card: Box::new(card("Other")),
    });
    let effects = state.update(Action::PartnerSearched {
        picked,
        mate: "Toothy, Imaginary Friend".to_owned(),
        cards: vec![card("Toothy, Imaginary Friend")],
    });
    assert!(effects.is_empty());
    assert_eq!(names(&state, Slot::Partner), None);
}

#[test]
fn a_format_change_clears_the_command_zone_and_turns_every_filter_on() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Oathbreaker));
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Commander,
        card: Box::new(card("Walker")),
    });
    let _ = state.update(Action::PickSwiped {
        slot: Slot::SignatureSpell,
        card: Box::new(card("Spell")),
    });
    let _ = state.update(Action::ToggleFilter(Slot::SignatureSpell));
    assert_eq!(state.label(Slot::Commander), "Oathbreaker");

    let _ = state.update(Action::TapFormat(Format::Oathbreaker));
    assert_eq!(
        state.values().format,
        None,
        "tapping the picked format clears it"
    );
    assert_eq!(names(&state, Slot::Commander), None);
    assert_eq!(names(&state, Slot::SignatureSpell), None);
    assert_eq!(state.display(Slot::SignatureSpell), "");
    assert!(state.filter_on(Slot::SignatureSpell));
    assert!(!state.shows(Slot::Commander));

    let _ = state.update(Action::TapFormat(Format::Commander));
    let _ = state.update(Action::ClearFormat);
    assert_eq!(state.values().format, None);
}

#[test]
fn the_format_picker_puts_the_command_zone_back_on_back() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Commander));
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Commander,
        card: Box::new(card("Krenko")),
    });
    let _ = state.update(Action::OpenPicker(Picker::Format));
    assert_eq!(state.picker(), Some(Picker::Format));
    assert!(state.picker_unchanged());
    let _ = state.update(Action::TapFormat(Format::Brawl));
    assert!(!state.picker_unchanged());
    assert_eq!(names(&state, Slot::Commander), None);

    let effects = state.update(Action::BackPicker);
    assert_eq!(
        toasts(&effects),
        [Toast::new(ToastKind::Info, "Format unchanged", QUICK)]
    );
    assert_eq!(state.picker(), None);
    assert_eq!(state.values().format, Some(Format::Commander));
    assert_eq!(names(&state, Slot::Commander).as_deref(), Some("Krenko"));
    assert_eq!(state.display(Slot::Commander), "Krenko");
}

#[test]
fn format_back_with_the_same_format_still_restores_quietly() {
    let mut state = create();
    let _ = state.update(Action::TapFormat(Format::Commander));
    let _ = state.update(Action::PickSwiped {
        slot: Slot::Commander,
        card: Box::new(card("Krenko")),
    });
    let _ = state.update(Action::OpenPicker(Picker::Format));
    let _ = state.update(Action::TapFormat(Format::Commander));
    let _ = state.update(Action::TapFormat(Format::Commander));
    assert!(state.picker_unchanged());
    assert!(state.update(Action::BackPicker).is_empty());
    assert_eq!(names(&state, Slot::Commander).as_deref(), Some("Krenko"));
}

#[test]
fn format_save_keeps_the_pick() {
    let mut state = create();
    let _ = state.update(Action::OpenPicker(Picker::Format));
    let _ = state.update(Action::TapFormat(Format::Modern));
    assert!(state.update(Action::SavePicker).is_empty());
    assert_eq!(state.picker(), None);
    assert_eq!(state.values().format, Some(Format::Modern));
}

fn catalog() -> Vec<DeckTagView> {
    vec![DeckTagView {
        slug: "ramp".to_owned(),
        display_name: "Ramp".to_owned(),
        description: String::new(),
        seed_otags: vec!["mana-rock".to_owned()],
    }]
}

#[test]
fn saving_deck_tags_seeds_oracle_tags_and_says_so() {
    let mut state = create();
    let _ = state.update(Action::Catalog(catalog()));
    let _ = state.update(Action::OpenPicker(Picker::DeckTags));
    let _ = state.update(Action::ToggleTag {
        picker: TagPicker::DeckTags,
        slug: "ramp".to_owned(),
    });
    let effects = state.update(Action::SavePicker);
    assert_eq!(
        toasts(&effects),
        [Toast::new(
            ToastKind::Info,
            "Oracle tags derived from your deck tags",
            NORMAL
        )]
    );
    assert_eq!(state.tags(TagPicker::OracleTags), ["mana-rock"]);

    // Taking the deck tag off and closing again drops its seed.
    let _ = state.update(Action::OpenPicker(Picker::DeckTags));
    let _ = state.update(Action::ClearTags(TagPicker::DeckTags));
    assert!(state.update(Action::SavePicker).is_empty());
    assert_eq!(state.tags(TagPicker::OracleTags), NONE);
}

#[test]
fn deck_tag_back_reverts_says_so_and_reseeds() {
    let mut state = create();
    let _ = state.update(Action::Catalog(catalog()));
    let _ = state.update(Action::OpenPicker(Picker::DeckTags));
    let _ = state.update(Action::ToggleTag {
        picker: TagPicker::DeckTags,
        slug: "ramp".to_owned(),
    });
    let effects = state.update(Action::BackPicker);
    assert_eq!(
        toasts(&effects),
        [Toast::new(ToastKind::Info, "Deck tags unchanged", QUICK)]
    );
    assert_eq!(state.tags(TagPicker::DeckTags), NONE);
    assert_eq!(state.tags(TagPicker::OracleTags), NONE);
}

#[test]
fn clearing_deck_tags_on_the_form_does_not_reseed() {
    let mut state = create();
    let _ = state.update(Action::Catalog(catalog()));
    let _ = state.update(Action::OpenPicker(Picker::DeckTags));
    let _ = state.update(Action::ToggleTag {
        picker: TagPicker::DeckTags,
        slug: "ramp".to_owned(),
    });
    let _ = state.update(Action::SavePicker);
    let _ = state.update(Action::RemoveTag {
        picker: TagPicker::DeckTags,
        slug: "ramp".to_owned(),
    });
    assert_eq!(state.tags(TagPicker::DeckTags), NONE);
    assert_eq!(state.tags(TagPicker::OracleTags), ["mana-rock"]);
}

#[test]
fn edit_reseeds_from_the_saved_tags() {
    let mut profile = deck_profile("Ramp");
    profile.tags = vec!["ramp".to_owned()];
    profile.oracle_tags = zwipe_core::domain::deck::seed_oracle_tags(&profile.tags);
    profile.oracle_tags.push("manual".to_owned());
    let mut state = edit_loaded(profile, &[]);
    let _ = state.update(Action::OpenPicker(Picker::DeckTags));
    let _ = state.update(Action::ClearTags(TagPicker::DeckTags));
    let _ = state.update(Action::SavePicker);
    assert_eq!(
        state.tags(TagPicker::OracleTags),
        ["manual"],
        "the saved seed goes, the manual pick stays"
    );
}

#[test]
fn a_tag_tap_past_the_cap_warns() {
    let mut state = create();
    let _ = state.update(Action::OpenPicker(Picker::DeckTags));
    for i in 0..MAX_DECK_TAGS {
        let effects = state.update(Action::ToggleTag {
            picker: TagPicker::DeckTags,
            slug: format!("t{i}"),
        });
        assert!(effects.is_empty());
    }
    let effects = state.update(Action::ToggleTag {
        picker: TagPicker::DeckTags,
        slug: "more".to_owned(),
    });
    assert_eq!(toasts(&effects), [TagPicker::DeckTags.cap_toast()]);
    assert_eq!(state.tags(TagPicker::DeckTags).len(), MAX_DECK_TAGS);
}

#[test]
fn the_oracle_tag_picker_reverts_on_back_and_adopts_from_the_dictionary() {
    let mut state = create();
    let _ = state.update(Action::OpenPicker(Picker::OracleTags));
    assert_eq!(state.picker(), Some(Picker::OracleTags));
    let effects = state.update(Action::AdoptOracleTag("ramp".to_owned()));
    assert_eq!(
        toasts(&effects),
        [Toast::new(ToastKind::Success, "Tag added to deck", QUICK)]
    );
    let effects = state.update(Action::AdoptOracleTag("ramp".to_owned()));
    assert_eq!(
        toasts(&effects),
        [Toast::new(ToastKind::Info, "Tag already added", QUICK)]
    );
    assert!(!state.picker_unchanged());
    let effects = state.update(Action::BackPicker);
    assert_eq!(
        toasts(&effects),
        [Toast::new(ToastKind::Info, "Oracle tags unchanged", QUICK)]
    );
    assert_eq!(state.tags(TagPicker::OracleTags), NONE);
}

#[test]
fn the_plain_fields_follow_their_chips_and_buttons() {
    let mut state = create();
    let _ = state.update(Action::TogglePowerLevel(PowerLevel::Core));
    assert_eq!(state.values().power_level, Some(PowerLevel::Core));
    let _ = state.update(Action::TogglePowerLevel(PowerLevel::Core));
    assert_eq!(state.values().power_level, None, "tapping it again unpicks");
    let _ = state.update(Action::TogglePowerLevel(PowerLevel::Core));
    let _ = state.update(Action::ClearPowerLevel);
    assert_eq!(state.values().power_level, None);

    let _ = state.update(Action::TapFormat(Format::Commander));
    let _ = state.update(Action::StepLand(Step::Up));
    assert_eq!(state.values().land_target, Some(37));
    let _ = state.update(Action::StepLand(Step::Down));
    assert_eq!(state.values().land_target, Some(36));
    let _ = state.update(Action::ClearLand);
    assert_eq!(state.values().land_target, None);

    let _ = state.update(Action::TypePrice("$12.5a".to_owned()));
    assert_eq!(state.values().price_text, "12.5");
    let _ = state.update(Action::ClearPrice);
    assert_eq!(state.values().price_text, "");
    let _ = state.update(Action::PickCurrency(PriceCurrency::Eur));
    assert_eq!(state.values().currency, PriceCurrency::Eur);

    let _ = state.update(Action::ToggleOtherTag(DeckOtherTag::Budget));
    assert_eq!(state.values().other_tags, [DeckOtherTag::Budget]);
    let _ = state.update(Action::ToggleOtherTag(DeckOtherTag::Budget));
    assert_eq!(state.values().other_tags, []);
    let _ = state.update(Action::ToggleOtherTag(DeckOtherTag::Budget));
    let _ = state.update(Action::ClearOtherTags);
    assert_eq!(state.values().other_tags, []);
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [CREATE_TITLE, EDIT_TITLE, LOAD_FAILED, BACK, SAVE, SAVING],
        [
            "Create deck",
            "Edit deck",
            "Could not load deck",
            "Back",
            "Save",
            "Saving..."
        ]
    );
    assert_eq!([PROFILE, BUDGET, TAGS], ["Profile", "Budget", "Tags"]);
    assert_eq!(
        [
            DECK_NAME,
            FORMAT,
            POWER_LEVEL,
            LAND_TARGET,
            PRICE_TARGET,
            DECK_TAGS,
            ORACLE_TAGS,
            OTHER_TAGS
        ],
        [
            "Deck name",
            "Format",
            "Power level",
            "Land target",
            "Price target",
            "Deck tags",
            "Oracle tags",
            "Other tags"
        ]
    );
    assert_eq!(
        [NOT_SET, FILTER, SWIPE, SEARCHING, NO_RESULTS],
        ["Not set", "Filter", "Swipe", "Searching...", "No results"]
    );
    assert_eq!(
        ORACLE_TAGS_SEEDED,
        "Oracle tags derived from your deck tags"
    );
    assert_eq!(Mode::Edit(Uuid::nil()).hint(), OneTimeHint::EditDeck);
}
