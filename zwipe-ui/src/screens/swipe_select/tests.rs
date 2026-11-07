use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::http::contracts::suggestions::HttpSuggestionReasons;

use super::*;
use crate::test_support::card;

/// The window of an overlay with no card to show.
const NO_CARDS: &[Card] = &[];

/// Cards named `names`.
fn cards(names: &[&str]) -> Vec<Card> {
    names.iter().map(|n| card(n)).collect()
}

/// The names of `cards`, for readable assertions.
fn names(cards: &[Card]) -> Vec<&str> {
    cards
        .iter()
        .map(|c| c.scryfall_data.name.as_str())
        .collect()
}

/// `Open` for `mode` with an empty filter, no pile and no exclusions.
fn open(mode: Mode) -> Action {
    Action::Open {
        mode: Some(mode),
        filter: CardQueryBuilder::new(),
        pile: SuggestedPile::default(),
        excluded: HashSet::new(),
    }
}

/// The ticket and query of the one search among `effects`.
fn search_of(effects: &[Effect]) -> (Ticket, CardQuery) {
    let found: Vec<(Ticket, CardQuery)> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::SearchCommanders { ticket, query } | Effect::LoadMore { ticket, query } => {
                Some((*ticket, (**query).clone()))
            }
            _ => None,
        })
        .collect();
    let [one] = found.as_slice() else {
        panic!("one search: {effects:?}");
    };
    one.clone()
}

/// The ticket of the one search among `effects`.
fn ticket_of(effects: &[Effect]) -> Ticket {
    search_of(effects).0
}

/// The search `mode` sends for the empty filter at `offset`.
fn want_query(mode: &Mode, offset: u32) -> CardQuery {
    let mut builder = CardQueryBuilder::new();
    mode.apply(&mut builder);
    builder.set_is_token(false);
    builder.set_limit(PAGE_SIZE);
    builder.set_offset(offset);
    builder.build().unwrap()
}

/// A commander overlay holding `pile` from its first page.
fn loaded(pile: &[Card]) -> State {
    let mut state = State::new();
    let ticket = ticket_of(&state.update(open(Mode::Commander(Format::Commander))));
    let effects = state.update(Action::Found {
        ticket,
        cards: pile.to_vec(),
    });
    if pile.is_empty() {
        assert_eq!(
            effects,
            [Effect::Toast(Toast::quick(
                ToastKind::Info,
                "No commanders match your filters"
            ))]
        );
    } else {
        assert!(
            effects.is_empty(),
            "a full page asks for nothing: {effects:?}"
        );
    }
    state
}

/// Twenty-five commanders, a full first page.
fn full_page() -> Vec<Card> {
    (0..25).map(|i| card(&format!("Commander {i}"))).collect()
}

/// A suggestion of a card named `name`.
fn suggestion(name: &str) -> HttpCommanderSuggestion {
    HttpCommanderSuggestion {
        card: card(name),
        reasons: HttpSuggestionReasons {
            tags: Vec::new(),
            colors: Vec::new(),
        },
    }
}

/// A suggested pile of `suggestions`.
fn pile_of(suggestions: Vec<HttpCommanderSuggestion>) -> SuggestedPile {
    SuggestedPile {
        suggestions,
        deck_count: 3,
        identity: None,
    }
}

/// A suggested overlay opened on `pile`.
fn suggested(pile: SuggestedPile) -> State {
    let mut state = State::new();
    let effects = state.update(Action::Open {
        mode: Some(Mode::Suggested),
        filter: CardQueryBuilder::new(),
        pile,
        excluded: HashSet::new(),
    });
    assert_eq!(
        effects,
        vec![Effect::OfferHint(OneTimeHint::SwipeSelect)],
        "the suggested pile never searches"
    );
    state
}

fn toast(effect: &Effect) -> &Toast {
    match effect {
        Effect::Toast(toast) | Effect::Confirm(toast) => toast,
        other => panic!("not a toast: {other:?}"),
    }
}

#[test]
fn the_overlay_starts_empty_with_no_mode() {
    let state = State::new();
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.mode(), None);
    assert_eq!(state.title(), TITLE);
    assert_eq!(state.noun(), "card");
    assert!(!state.can_save());
    assert_eq!(state.window(), NO_CARDS);
    assert_eq!(state.message(), None);
}

#[test]
fn opening_offers_the_hint_and_searches_the_first_page() {
    let mut state = State::new();
    let mode = Mode::Commander(Format::Commander);
    let effects = state.update(open(mode.clone()));
    assert!(matches!(
        effects.as_slice(),
        [
            Effect::OfferHint(OneTimeHint::SwipeSelect),
            Effect::SearchCommanders { .. }
        ]
    ));
    assert_eq!(search_of(&effects).1, want_query(&mode, 0));
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn the_hint_is_offered_on_the_first_open_only() {
    let mut state = loaded(&cards(&["Krenko"]));
    let effects = state.update(open(Mode::Partner));
    assert!(
        !effects.iter().any(|e| matches!(e, Effect::OfferHint(_))),
        "{effects:?}"
    );
}

#[test]
fn each_mode_adds_its_constraint_and_noun() {
    let colors: Colors = [Color::Blue].into_iter().collect();
    let cases = [
        (Mode::Commander(Format::Commander), "commander", true),
        (Mode::Commander(Format::Oathbreaker), "oathbreaker", true),
        (Mode::Partner, "partner", true),
        (Mode::Background, "background", false),
        (Mode::SignatureSpell(colors), "signature spell", false),
    ];
    for (mode, noun, saves) in cases {
        let mut state = State::new();
        let effects = state.update(open(mode.clone()));
        assert_eq!(search_of(&effects).1, want_query(&mode, 0), "{mode:?}");
        assert_eq!(state.noun(), noun, "{mode:?}");
        assert_eq!(state.can_save(), saves, "{mode:?}");
    }
    assert!(Mode::Suggested.saves());
    assert_eq!(Mode::Suggested.noun(), "commander");
}

#[test]
fn a_page_loads_the_stack() {
    let state = loaded(&cards(&["Krenko", "Narset"]));
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(names(state.window()), ["Krenko", "Narset"]);
    assert_eq!(
        state.current().map(|c| c.scryfall_data.name.as_str()),
        Some("Krenko")
    );
}

#[test]
fn an_empty_first_page_is_empty_and_spent() {
    let mut state = loaded(&[]);
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.update(Action::Skip), Vec::new(), "no card to skip");
}

#[test]
fn a_failed_search_shows_failed_and_a_reopen_tries_again() {
    let mut state = State::new();
    let mode = Mode::Commander(Format::Commander);
    let ticket = ticket_of(&state.update(open(mode.clone())));
    assert_eq!(state.update(Action::Failed { ticket }), Vec::new());
    assert_eq!(state.status(), Status::Failed);
    let effects = state.update(open(mode));
    let _ = ticket_of(&effects);
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn a_reopen_with_the_same_mode_keeps_the_pile_and_its_place() {
    let mut state = loaded(&cards(&["Krenko", "Narset"]));
    let _ = state.update(Action::Skip);
    let effects = state.update(open(Mode::Commander(Format::Commander)));
    assert_eq!(effects, Vec::new());
    assert_eq!(names(state.window()), ["Narset"]);
}

#[test]
fn a_reopen_with_another_mode_deals_afresh() {
    let mut state = loaded(&cards(&["Krenko"]));
    let effects = state.update(open(Mode::Commander(Format::Oathbreaker)));
    let _ = ticket_of(&effects);
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.window(), NO_CARDS);
}

#[test]
fn opening_with_no_mode_deals_nothing() {
    let mut state = State::new();
    let effects = state.update(Action::Open {
        mode: None,
        filter: CardQueryBuilder::new(),
        pile: SuggestedPile::default(),
        excluded: HashSet::new(),
    });
    assert_eq!(effects, vec![Effect::OfferHint(OneTimeHint::SwipeSelect)]);
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn a_late_reply_is_ignored() {
    let mut state = State::new();
    let older = ticket_of(&state.update(open(Mode::Partner)));
    let newer = ticket_of(&state.update(Action::Refresh(CardQueryBuilder::new())));
    assert_eq!(
        state.update(Action::Found {
            ticket: older,
            cards: cards(&["Stale"])
        }),
        Vec::new()
    );
    assert_eq!(state.update(Action::Failed { ticket: older }), Vec::new());
    assert_eq!(
        state.status(),
        Status::Loading,
        "the newer search is still out"
    );
    let _ = state.update(Action::Found {
        ticket: newer,
        cards: cards(&["Fresh"]),
    });
    assert_eq!(names(state.window()), ["Fresh"]);
}

#[test]
fn a_late_page_of_an_older_pile_is_ignored() {
    let mut pile = full_page();
    pile.truncate(6);
    let mut state = loaded(&pile);
    let more = ticket_of(&state.update(Action::Skip));
    let _ = state.update(Action::Filter(CardQueryBuilder::new()));
    assert_eq!(
        state.update(Action::Found {
            ticket: more,
            cards: cards(&["Stale"])
        }),
        Vec::new()
    );
    assert_eq!(state.window(), NO_CARDS);
}

#[test]
fn refresh_toasts_then_searches_again_from_the_top() {
    let mut state = loaded(&cards(&["Krenko", "Narset"]));
    let _ = state.update(Action::Skip);
    let mut filter = CardQueryBuilder::new();
    filter.set_sort(CardSortKey::Name);
    let effects = state.update(Action::Refresh(filter));
    assert_eq!(
        *toast(&effects[0]),
        Toast::quick(ToastKind::Info, STACK_REFRESHED)
    );
    let (ticket, query) = search_of(&effects);
    assert_eq!(
        query.sort(),
        Some(CardSortKey::Name),
        "the new filter searches"
    );
    let _ = state.update(Action::Found {
        ticket,
        cards: cards(&["Talrand"]),
    });
    assert_eq!(names(state.window()), ["Talrand"]);
    assert_eq!(
        state.update(Action::Undo),
        vec![Effect::Toast(Toast::normal(
            ToastKind::Info,
            NOTHING_TO_UNDO
        ))],
        "a fresh deal forgets the swipes"
    );
}

#[test]
fn an_applied_filter_searches_again() {
    let mut state = loaded(&cards(&["Krenko"]));
    let effects = state.update(Action::Filter(CardQueryBuilder::new()));
    assert!(matches!(
        effects.as_slice(),
        [Effect::SearchCommanders { .. }]
    ));
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn excluded_oracle_ids_stay_out_of_the_pile() {
    let pile = cards(&["Krenko", "Narset"]);
    let mut state = State::new();
    let ticket = ticket_of(&state.update(Action::Open {
        mode: Some(Mode::Commander(Format::Commander)),
        filter: CardQueryBuilder::new(),
        pile: SuggestedPile::default(),
        excluded: pile[0].scryfall_data.oracle_id.into_iter().collect(),
    }));
    let _ = state.update(Action::Found {
        ticket,
        cards: pile,
    });
    assert_eq!(names(state.window()), ["Narset"]);
}

#[test]
fn a_first_page_all_excluded_pulls_the_next() {
    let pile = cards(&["Krenko"]);
    let mut state = State::new();
    let ticket = ticket_of(&state.update(Action::Open {
        mode: Some(Mode::Partner),
        filter: CardQueryBuilder::new(),
        pile: SuggestedPile::default(),
        excluded: pile[0].scryfall_data.oracle_id.into_iter().collect(),
    }));
    let effects = state.update(Action::Found {
        ticket,
        cards: pile,
    });
    assert!(matches!(effects.as_slice(), [Effect::LoadMore { .. }]));
    assert_eq!(search_of(&effects).1, want_query(&Mode::Partner, PAGE_SIZE));
    assert_eq!(state.status(), Status::Loading, "the next page is out");
}

#[test]
fn a_swipe_near_the_end_loads_the_next_page() {
    let mut state = loaded(&full_page());
    for _ in 0..19 {
        let effects = state.update(Action::Skip);
        assert!(
            !effects.iter().any(|e| matches!(e, Effect::LoadMore { .. })),
            "too early: {effects:?}"
        );
    }
    let effects = state.update(Action::Skip);
    let (ticket, query) = search_of(&effects);
    assert_eq!(
        query,
        want_query(&Mode::Commander(Format::Commander), PAGE_SIZE)
    );
    assert!(
        !state
            .update(Action::Skip)
            .iter()
            .any(|e| matches!(e, Effect::LoadMore { .. })),
        "one page at a time"
    );
    let _ = state.update(Action::Found {
        ticket,
        cards: cards(&["Talrand"]),
    });
    assert_eq!(
        state.window().len(),
        5,
        "four left of the first page and the new one"
    );
}

#[test]
fn a_page_of_cards_already_dealt_pulls_again_up_to_four_times() {
    let first = full_page();
    let mut state = loaded(&first);
    let mut ticket = (0..20)
        .filter_map(|_| {
            let effects = state.update(Action::Skip);
            effects
                .iter()
                .any(|e| matches!(e, Effect::LoadMore { .. }))
                .then(|| ticket_of(&effects))
        })
        .last()
        .unwrap();
    let repeat = vec![first[0].clone()];
    for pull in 2..=PAGE_PULLS {
        let effects = state.update(Action::Found {
            ticket,
            cards: repeat.clone(),
        });
        let (next, query) = search_of(&effects);
        let offset = u32::try_from(pull).unwrap() * PAGE_SIZE;
        assert_eq!(
            query,
            want_query(&Mode::Commander(Format::Commander), offset)
        );
        ticket = next;
    }
    assert_eq!(
        state.update(Action::Found {
            ticket,
            cards: repeat
        }),
        Vec::new(),
        "the fourth pull is the last"
    );
    assert_eq!(state.status(), Status::Loaded);
    let effects = state.update(Action::Skip);
    let _ = ticket_of(&effects);
}

#[test]
fn an_empty_next_page_spends_the_pile() {
    let mut state = loaded(&full_page());
    let ticket = (0..20)
        .filter_map(|_| {
            let effects = state.update(Action::Skip);
            effects
                .iter()
                .any(|e| matches!(e, Effect::LoadMore { .. }))
                .then(|| ticket_of(&effects))
        })
        .last()
        .unwrap();
    let _ = state.update(Action::Found {
        ticket,
        cards: Vec::new(),
    });
    for _ in 0..5 {
        let effects = state.update(Action::Skip);
        assert!(!effects.iter().any(|e| matches!(e, Effect::LoadMore { .. })));
    }
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn a_failed_next_page_stops_loading() {
    let mut state = loaded(&cards(&["Krenko"]));
    let ticket = ticket_of(&state.update(Action::Skip));
    assert_eq!(state.status(), Status::Loading);
    let _ = state.update(Action::Failed { ticket });
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn a_left_swipe_skips() {
    let mut state = loaded(&full_page());
    assert_eq!(
        state.update(Action::Skip),
        vec![Effect::Confirm(Toast::quick(ToastKind::Info, SKIPPED))]
    );
    assert_eq!(state.window()[0].scryfall_data.name, "Commander 1");
}

#[test]
fn a_right_swipe_toasts_and_hands_the_card_over() {
    let pile = cards(&["Krenko"]);
    let mut state = loaded(&pile);
    assert_eq!(
        state.update(Action::Choose),
        vec![
            Effect::Confirm(Toast::quick(ToastKind::Success, "Commander selected")),
            Effect::Pick(Box::new(pile[0].clone())),
        ]
    );
    let mut oathbreaker = State::new();
    let ticket = ticket_of(&oathbreaker.update(open(Mode::Commander(Format::Oathbreaker))));
    let _ = oathbreaker.update(Action::Found {
        ticket,
        cards: pile,
    });
    assert_eq!(
        *toast(&oathbreaker.update(Action::Choose)[0]),
        Toast::quick(ToastKind::Success, "Oathbreaker selected")
    );
}

#[test]
fn an_up_swipe_saves_and_toasts_once_it_lands() {
    let pile = full_page();
    let mut state = loaded(&pile);
    let effects = state.update(Action::Save);
    let [Effect::AddMaybeboardCard { ticket, oracle_id }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(Some(*oracle_id), pile[0].scryfall_data.oracle_id);
    assert_eq!(
        state.window()[0].scryfall_data.name,
        "Commander 1",
        "the pile moves on"
    );
    assert_eq!(
        state.update(Action::Landed { ticket: *ticket }),
        vec![Effect::Confirm(Toast::new(
            ToastKind::Success,
            SAVED,
            MAYBEBOARD_TOAST
        ))]
    );
    assert_eq!(
        state.update(Action::Landed { ticket: *ticket }),
        Vec::new(),
        "once"
    );
}

#[test]
fn a_failed_save_raises_nothing_more() {
    let mut state = loaded(&full_page());
    let effects = state.update(Action::Save);
    let [Effect::AddMaybeboardCard { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(state.update(Action::Failed { ticket: *ticket }), Vec::new());
}

#[test]
fn a_save_still_toasts_after_a_fresh_deal() {
    let mut state = loaded(&full_page());
    let effects = state.update(Action::Save);
    let [Effect::AddMaybeboardCard { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let _ = state.update(Action::Refresh(CardQueryBuilder::new()));
    assert_eq!(state.update(Action::Landed { ticket: *ticket }).len(), 1);
}

#[test]
fn an_up_swipe_without_an_oracle_id_saves_nothing_and_moves_on() {
    let mut pile = full_page();
    pile[0].scryfall_data.oracle_id = None;
    let mut state = loaded(&pile);
    assert_eq!(
        state.update(Action::Save),
        vec![Effect::Toast(Toast::normal(ToastKind::Error, CANT_SAVE))]
    );
    assert_eq!(state.window()[0].scryfall_data.name, "Commander 1");
    assert_eq!(
        state.update(Action::Undo),
        vec![
            Effect::Enter(Direction::Left),
            Effect::Confirm(Toast::quick(ToastKind::Info, UNDID_SKIP)),
        ],
        "undo treats it as a skip"
    );
}

#[test]
fn modes_without_a_maybeboard_ignore_an_up_swipe() {
    let mut state = State::new();
    let ticket = ticket_of(&state.update(open(Mode::Background)));
    let _ = state.update(Action::Found {
        ticket,
        cards: cards(&["Raised by Giants"]),
    });
    assert_eq!(state.update(Action::Save), Vec::new());
    assert_eq!(state.window().len(), 1);
}

#[test]
fn undo_with_nothing_swiped_says_so() {
    let mut state = loaded(&cards(&["Krenko"]));
    assert_eq!(
        state.update(Action::Undo),
        vec![Effect::Toast(Toast::normal(
            ToastKind::Info,
            NOTHING_TO_UNDO
        ))]
    );
}

#[test]
fn undo_brings_a_skipped_card_back_from_the_left() {
    let mut state = loaded(&full_page());
    let _ = state.update(Action::Skip);
    assert_eq!(
        state.update(Action::Undo),
        vec![
            Effect::Enter(Direction::Left),
            Effect::Confirm(Toast::quick(ToastKind::Info, UNDID_SKIP)),
        ]
    );
    assert_eq!(state.window()[0].scryfall_data.name, "Commander 0");
}

#[test]
fn undoing_a_save_removes_it_and_toasts_once_it_lands() {
    let pile = full_page();
    let mut state = loaded(&pile);
    let _ = state.update(Action::Save);
    let effects = state.update(Action::Undo);
    let [
        Effect::Enter(Direction::Left),
        Effect::RemoveMaybeboardCard { ticket, oracle_id },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert_eq!(Some(*oracle_id), pile[0].scryfall_data.oracle_id);
    assert_eq!(
        state.update(Action::Landed { ticket: *ticket }),
        vec![Effect::Confirm(Toast::quick(ToastKind::Info, UNSAVED))]
    );
}

#[test]
fn a_printing_pick_swaps_the_card_on_top() {
    let mut state = loaded(&cards(&["Krenko", "Narset"]));
    let reprint = card("Krenko reprint");
    let _ = state.update(Action::PickPrinting(Box::new(reprint.clone())));
    assert_eq!(state.current(), Some(&reprint));
    assert_eq!(state.window().len(), 2);
}

#[test]
fn back_closes_the_overlay() {
    let mut state = loaded(&cards(&["Krenko"]));
    assert_eq!(
        state.update(Action::Back),
        vec![Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn the_suggested_pile_deals_the_host_s_ranking() {
    let state = suggested(pile_of(vec![suggestion("Krenko"), suggestion("Narset")]));
    assert_eq!(state.title(), SUGGESTED_TITLE);
    assert!(state.is_suggested());
    assert!(state.can_save());
    assert_eq!(names(state.window()), ["Krenko", "Narset"]);
    assert_eq!(state.deck_count(), 3);
    assert!(state.reasons().is_some());
}

#[test]
fn a_suggested_left_swipe_dismisses_and_undo_lifts_it() {
    let pile = pile_of(vec![suggestion("Krenko"), suggestion("Narset")]);
    let oracle_id = pile.suggestions[0].card.scryfall_data.oracle_id.unwrap();
    let mut state = suggested(pile);
    assert_eq!(
        state.update(Action::Skip),
        vec![
            Effect::DismissCommander(oracle_id),
            Effect::Confirm(Toast::quick(ToastKind::Info, SKIPPED)),
        ],
        "a dealt pile never loads more"
    );
    assert_eq!(
        state.update(Action::Undo),
        vec![
            Effect::Enter(Direction::Left),
            Effect::UndismissCommander(oracle_id),
            Effect::Confirm(Toast::quick(ToastKind::Info, UNDID_SKIP)),
        ]
    );
}

#[test]
fn a_suggested_reopen_deals_the_host_s_pile_afresh() {
    let mut state = suggested(pile_of(vec![suggestion("Krenko"), suggestion("Narset")]));
    let _ = state.update(Action::Skip);
    let effects = state.update(Action::Open {
        mode: Some(Mode::Suggested),
        filter: CardQueryBuilder::new(),
        pile: pile_of(vec![suggestion("Talrand")]),
        excluded: HashSet::new(),
    });
    assert_eq!(effects, Vec::new());
    assert_eq!(names(state.window()), ["Talrand"]);
}

#[test]
fn a_redeal_keeps_the_swiped_behind_the_top_card() {
    let krenko = suggestion("Krenko");
    let mut state = suggested(pile_of(vec![krenko.clone(), suggestion("Narset")]));
    let _ = state.update(Action::Skip);
    let talrand = suggestion("Talrand");
    let _ = state.update(Action::Redeal(SuggestedPile {
        suggestions: vec![krenko, talrand],
        deck_count: 5,
        identity: Some(vec![Color::Blue]),
    }));
    assert_eq!(names(state.window()), ["Talrand"]);
    assert_eq!(state.deck_count(), 5);
    assert_eq!(state.identity(), Some([Color::Blue].as_slice()));
    let _ = state.update(Action::Undo);
    assert_eq!(names(state.window()), ["Krenko", "Talrand"]);
    assert!(
        state.reasons().is_some(),
        "the swiped card keeps its reasons"
    );
}

#[test]
fn a_redeal_outside_the_suggested_mode_changes_nothing() {
    let mut state = loaded(&cards(&["Krenko"]));
    let before = state.clone();
    let _ = state.update(Action::Redeal(pile_of(vec![suggestion("Talrand")])));
    assert_eq!(state, before);
}

#[test]
fn an_empty_identity_says_so_in_the_host_s_words() {
    let mut state = suggested(SuggestedPile {
        suggestions: Vec::new(),
        deck_count: 3,
        identity: Some(vec![Color::White, Color::Blue, Color::Red]),
    });
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(
        state.message().as_deref(),
        Some("No Jeskai commanders in your top 100")
    );
    let _ = state.update(Action::Redeal(pile_of(Vec::new())));
    assert_eq!(state.message(), None, "All has no line");
}

#[test]
fn the_filter_dot_lights_for_a_filter_or_a_sort() {
    let mut filter = CardQueryBuilder::new();
    assert!(!filter_lit(&filter));
    filter.set_sort(CardSortKey::Name);
    assert!(filter_lit(&filter));
    let mut colors = CardQueryBuilder::new();
    colors.set_is_partner(true);
    assert!(filter_lit(&colors));
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Swipe select");
    assert_eq!(SUGGESTED_TITLE, "Suggested commanders");
    assert_eq!(BACK, "Back");
    assert_eq!(FILTER, "Filter");
    assert_eq!(REFRESH, "Refresh");
    assert_eq!(SKIPPED, "Skipped");
    assert_eq!(CANT_SAVE, "This card can't be saved");
    assert_eq!(SAVED, "Added to your commander maybeboard");
    assert_eq!(UNSAVED, "Removed from your commander maybeboard");
    assert_eq!(NOTHING_TO_UNDO, "Nothing to undo");
    assert_eq!(UNDID_SKIP, "Undid skip");
    assert_eq!(STACK_REFRESHED, "Stack refreshed");
    assert_eq!(FALLBACK_NOUN, "card");
    assert_eq!(MAYBEBOARD_TOAST, std::time::Duration::from_millis(1500));
    assert_eq!(PAGE_SIZE, 25);
    assert_eq!(LOAD_MORE_AT, 5);
}

#[test]
fn swiping_the_last_card_says_the_pile_is_spent() {
    let mut state = loaded(&cards(&["Krenko"]));
    let ticket = ticket_of(&state.update(Action::Skip));
    assert_eq!(
        state.status(),
        Status::Loading,
        "the next page is asked for"
    );
    let effects = state.update(Action::Found {
        ticket,
        cards: Vec::new(),
    });
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(
        effects,
        [Effect::Toast(Toast::quick(ToastKind::Info, NO_CARDS_LEFT))]
    );
}

/// A maybeboard overlay holding `pile` from its first page.
fn collecting(pile: &[Card]) -> State {
    let mut state = State::new();
    let effects = state.update(open(Mode::Maybeboard));
    assert_eq!(
        search_of(&effects).1,
        want_query(&Mode::Commander(Format::Commander), 0),
        "the maybeboard deals Commander's commanders"
    );
    let ticket = search_of(&effects).0;
    let _ = state.update(Action::Found {
        ticket,
        cards: pile.to_vec(),
    });
    state
}

#[test]
fn from_the_maybeboard_right_saves_and_the_swiping_goes_on() {
    let pile = full_page();
    let mut state = collecting(&pile);
    assert!(state.right_saves());
    assert!(!state.can_save(), "up has nothing to do");
    assert_eq!(state.noun(), "commander");
    let effects = state.update(Action::Choose);
    let [Effect::AddMaybeboardCard { ticket, oracle_id }] = effects.as_slice() else {
        panic!("a right swipe saves, picks nothing and stays open: {effects:?}");
    };
    assert_eq!(Some(*oracle_id), pile[0].scryfall_data.oracle_id);
    assert_eq!(
        state.window()[0].scryfall_data.name,
        "Commander 1",
        "the pile moves on"
    );
    assert_eq!(
        state.update(Action::Landed { ticket: *ticket }),
        vec![Effect::Confirm(Toast::new(
            ToastKind::Success,
            SAVED,
            MAYBEBOARD_TOAST
        ))]
    );
}

#[test]
fn from_the_maybeboard_up_does_nothing() {
    let mut state = collecting(&full_page());
    assert_eq!(state.update(Action::Save), Vec::new());
    assert_eq!(
        state.window()[0].scryfall_data.name,
        "Commander 0",
        "the card stays on top"
    );
}

#[test]
fn from_the_maybeboard_undo_takes_a_right_save_back() {
    let mut state = collecting(&full_page());
    let _ = state.update(Action::Choose);
    let effects = state.update(Action::Undo);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RemoveMaybeboardCard { .. })),
        "{effects:?}"
    );
    assert_eq!(state.window()[0].scryfall_data.name, "Commander 0");
}
