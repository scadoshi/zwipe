use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;

use super::*;
use crate::deck::VERIFY_TO_CREATE;
use crate::test_support::card;
use crate::toast::{LONG, QUICK};

/// A commander named `name` with `identity`.
fn commander(name: &str, identity: &[Color]) -> Card {
    let mut card = card(name);
    card.scryfall_data.color_identity = identity.iter().copied().collect();
    card
}

fn oracle(card: &Card) -> Uuid {
    card.scryfall_data.oracle_id.unwrap()
}

fn names(cards: &[Card]) -> Vec<String> {
    cards.iter().map(|c| c.scryfall_data.name.clone()).collect()
}

fn blank() -> CardQueryBuilder {
    CardQueryBuilder::new()
}

fn shown(state: &State) -> Vec<String> {
    names(&state.shown(&blank()))
}

/// A state whose first load is out, with the ticket it holds.
fn loading() -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load);
    let [
        Effect::OfferHint(OneTimeHint::CommanderMaybeboard),
        Effect::FetchMaybeboard(ticket),
        Effect::FetchDeckProfiles,
    ] = effects.as_slice()
    else {
        panic!("Load offers the hint and fetches: {effects:?}");
    };
    (state, *ticket)
}

/// A state with `cards` loaded.
fn loaded(cards: Vec<Card>) -> State {
    let (mut state, ticket) = loading();
    assert_eq!(state.update(Action::Loaded { ticket, cards }), []);
    state
}

/// Atraxa (WUBG), Krenko (R) and Kenrith (WUBRG), newest save first.
fn library() -> Vec<Card> {
    use Color::{Black, Blue, Green, Red, White};
    vec![
        commander("Atraxa", &[White, Blue, Black, Green]),
        commander("Krenko", &[Red]),
        commander("Kenrith", &[White, Blue, Black, Red, Green]),
    ]
}

/// The ticket a refetch effect holds.
fn fetch_ticket(effect: &Effect) -> Ticket {
    let Effect::FetchMaybeboard(ticket) = effect else {
        panic!("expected a refetch: {effect:?}");
    };
    *ticket
}

/// Types `query` into a state and lands its search with `cards`.
fn searched(state: &mut State, query: &str, cards: Vec<Card>) {
    let effects = state.update(Action::Type(query.to_owned()));
    let [Effect::Debounce { ticket, .. }] = effects.as_slice() else {
        panic!("a long query debounces: {effects:?}");
    };
    let ticket = *ticket;
    let effects = state.update(Action::Debounced { ticket });
    assert!(
        matches!(effects.as_slice(), [Effect::SearchCommanders { .. }]),
        "{effects:?}"
    );
    assert_eq!(state.update(Action::Searched { ticket, cards }), []);
}

/// Sends `action` for an edit and returns its ticket.
fn edit_ticket(state: &mut State, action: Action) -> Ticket {
    let effects = state.update(action);
    match effects.as_slice() {
        [
            Effect::AddCommander { ticket, .. }
            | Effect::RemoveCommander { ticket, .. }
            | Effect::ClearMaybeboard(ticket),
        ] => *ticket,
        other => panic!("expected one edit request: {other:?}"),
    }
}

#[test]
fn a_new_state_is_loading_with_nothing_picked() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.message(&blank()), None);
    assert_eq!(state.entries(), []);
    assert!(state.shows_all());
    assert_eq!(state.query(), "");
    assert_eq!(state.quick_add(), None);
    assert_eq!(state.highlight(), None);
}

#[test]
fn the_hint_is_offered_on_the_first_load_only() {
    let (mut state, _) = loading();
    let effects = state.update(Action::Load);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::FetchMaybeboard(_), Effect::FetchDeckProfiles]
        ),
        "{effects:?}"
    );
}

#[test]
fn a_load_shows_the_list_in_saved_order() {
    let state = loaded(library());
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.message(&blank()), None);
    assert_eq!(shown(&state), ["Atraxa", "Krenko", "Kenrith"]);
}

#[test]
fn an_empty_load_says_no_commanders() {
    let state = loaded(Vec::new());
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.message(&blank()), Some(NO_COMMANDERS));
}

#[test]
fn a_failed_first_load_says_so() {
    let (mut state, ticket) = loading();
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert_eq!(state.status(), Status::Failed);
    assert_eq!(state.message(&blank()), Some(LOAD_FAILED));
}

#[test]
fn a_failed_refetch_keeps_the_list() {
    let mut state = loaded(library());
    let ticket = fetch_ticket(&state.update(Action::SwipeClosed)[0]);
    let _ = state.update(Action::Failed { ticket });
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(shown(&state).len(), 3);
}

#[test]
fn a_refetch_is_silent() {
    let mut state = loaded(library());
    let _ = state.update(Action::SwipeClosed);
    assert_eq!(state.status(), Status::Loaded, "a refetch ghosted the list");
}

#[test]
fn a_late_load_is_ignored() {
    let (mut state, first) = loading();
    let second = fetch_ticket(&state.update(Action::SwipeClosed)[0]);
    let _ = state.update(Action::Loaded {
        ticket: first,
        cards: library(),
    });
    assert_eq!(state.status(), Status::Loading, "the older load landed");
    let _ = state.update(Action::Failed { ticket: first });
    assert_eq!(state.status(), Status::Loading, "the older failure landed");
    let _ = state.update(Action::Loaded {
        ticket: second,
        cards: Vec::new(),
    });
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn a_load_after_a_failure_clears_it() {
    let (mut state, ticket) = loading();
    let _ = state.update(Action::Failed { ticket });
    let ticket = fetch_ticket(&state.update(Action::SwipeClosed)[0]);
    let _ = state.update(Action::Loaded {
        ticket,
        cards: library(),
    });
    assert_eq!(state.status(), Status::Loaded);
}

#[test]
fn the_pips_keep_identities_holding_every_picked_color() {
    let mut state = loaded(library());
    let _ = state.update(Action::ToggleColor(Color::Red));
    assert!(!state.shows_all());
    assert!(state.is_picked(Color::Red));
    assert_eq!(shown(&state), ["Krenko", "Kenrith"]);
    let _ = state.update(Action::ToggleColor(Color::White));
    assert_eq!(shown(&state), ["Kenrith"]);
    let _ = state.update(Action::ToggleColor(Color::White));
    assert!(!state.is_picked(Color::White), "a second tap unpicks");
    assert_eq!(shown(&state), ["Krenko", "Kenrith"]);
    let _ = state.update(Action::ShowAll);
    assert!(state.shows_all());
    assert_eq!(shown(&state).len(), 3);
}

#[test]
fn nothing_passing_says_no_match() {
    let mut state = loaded(vec![commander("Krenko", &[Color::Red])]);
    let _ = state.update(Action::ToggleColor(Color::Green));
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.shown(&blank()), []);
    assert_eq!(state.message(&blank()), Some(NO_MATCH));
}

#[test]
fn the_filter_sort_reorders_and_saved_order_stands_without_one() {
    let state = loaded(library());
    let mut filter = blank();
    filter.set_sort(CardSortKey::Name);
    filter.set_direction(SortDirection::Ascending);
    assert_eq!(
        names(&state.shown(&filter)),
        ["Atraxa", "Kenrith", "Krenko"]
    );
    assert_eq!(shown(&state), ["Atraxa", "Krenko", "Kenrith"]);
}

#[test]
fn a_short_query_clears_the_results_and_searches_nothing() {
    let mut state = loaded(Vec::new());
    searched(&mut state, "kre", vec![commander("Krenko", &[Color::Red])]);
    assert_eq!(state.update(Action::Type(" k ".to_owned())), []);
    assert_eq!(state.query(), " k ");
    assert!(!state.searching_catalog());
    assert_eq!(state.quick_add(), None);
    let _ = state.update(Action::Type("kr".to_owned()));
    assert_eq!(
        state.quick_add(),
        Some(QuickAddResults::Searching),
        "the old results came back"
    );
}

#[test]
fn a_long_query_debounces_then_searches_by_name() {
    let mut state = loaded(Vec::new());
    let effects = state.update(Action::Type("kren".to_owned()));
    let [Effect::Debounce { ticket, wait }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(*wait, timing::QUICK_ADD_DEBOUNCE);
    assert_eq!(state.quick_add(), Some(QuickAddResults::Searching));
    let effects = state.update(Action::Debounced { ticket: *ticket });
    let want = search_query("kren").build().unwrap();
    assert_eq!(
        effects,
        [Effect::SearchCommanders {
            ticket: *ticket,
            query: Box::new(want),
        }]
    );
}

#[test]
fn the_search_asks_for_named_commanders() {
    let mut want = CardQueryBuilder::with_name_contains("atraxa");
    want.set_is_commander_in_format(Format::Commander)
        .set_is_token(false)
        .set_sort(CardSortKey::Name)
        .set_limit(8);
    assert_eq!(search_query("  atraxa "), want, "the app's search");
}

#[test]
fn a_debounce_overtaken_by_typing_is_ignored() {
    let mut state = loaded(Vec::new());
    let effects = state.update(Action::Type("kre".to_owned()));
    let [Effect::Debounce { ticket: old, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let old = *old;
    let _ = state.update(Action::Type("kren".to_owned()));
    assert_eq!(state.update(Action::Debounced { ticket: old }), []);
}

#[test]
fn a_late_search_is_ignored() {
    let mut state = loaded(Vec::new());
    let effects = state.update(Action::Type("kre".to_owned()));
    let [Effect::Debounce { ticket: old, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let old = *old;
    let _ = state.update(Action::Type("atr".to_owned()));
    let _ = state.update(Action::Searched {
        ticket: old,
        cards: vec![commander("Krenko", &[Color::Red])],
    });
    assert_eq!(state.quick_add(), Some(QuickAddResults::Searching));
    let _ = state.update(Action::Failed { ticket: old });
    assert_eq!(state.quick_add(), Some(QuickAddResults::Searching));
}

#[test]
fn a_failed_search_keeps_the_old_results() {
    let mut state = loaded(Vec::new());
    searched(&mut state, "kre", vec![commander("Krenko", &[Color::Red])]);
    let effects = state.update(Action::Type("kren".to_owned()));
    let [Effect::Debounce { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let _ = state.update(Action::Failed { ticket: *ticket });
    let Some(QuickAddResults::Found(found)) = state.quick_add() else {
        panic!("{:?}", state.quick_add());
    };
    assert_eq!(names(&found), ["Krenko"]);
}

#[test]
fn saved_commanders_are_left_out_of_the_results() {
    let atraxa = commander("Atraxa", &[]);
    let mut oracle_less = commander("Nobody", &[]);
    oracle_less.scryfall_data.oracle_id = None;
    let mut state = loaded(vec![atraxa.clone()]);
    searched(
        &mut state,
        "atr",
        vec![atraxa, oracle_less, commander("Atog", &[])],
    );
    let Some(QuickAddResults::Found(found)) = state.quick_add() else {
        panic!("{:?}", state.quick_add());
    };
    assert_eq!(names(&found), ["Atog"]);
}

#[test]
fn a_search_with_nothing_addable_says_no_results() {
    let atraxa = commander("Atraxa", &[]);
    let mut state = loaded(vec![atraxa.clone()]);
    searched(&mut state, "atr", vec![atraxa]);
    assert_eq!(state.quick_add(), Some(QuickAddResults::Empty));
}

#[test]
fn an_add_leaves_the_results_at_once_and_joins_the_top_when_it_lands() {
    let krenko = commander("Krenko", &[Color::Red]);
    let mut state = loaded(library());
    searched(
        &mut state,
        "kre",
        vec![krenko.clone(), commander("Kresh", &[])],
    );
    let effects = state.update(Action::Add(oracle(&krenko)));
    let [Effect::AddCommander { ticket, oracle_id }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(*oracle_id, oracle(&krenko));
    let Some(QuickAddResults::Found(found)) = state.quick_add() else {
        panic!("{:?}", state.quick_add());
    };
    assert_eq!(names(&found), ["Kresh"], "the chip stayed up");
    assert_eq!(
        shown(&state).len(),
        3,
        "the row joined before the server said so"
    );

    let effects = state.update(Action::Added { ticket: *ticket });
    let [refetch, Effect::Toast(toast)] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let _ = fetch_ticket(refetch);
    assert_eq!(*toast, Toast::new(ToastKind::Info, "Added Krenko", QUICK));
    assert_eq!(shown(&state)[0], "Krenko");
}

#[test]
fn an_add_already_on_the_list_is_not_doubled() {
    let krenko = commander("Krenko", &[Color::Red]);
    let mut state = loaded(Vec::new());
    searched(&mut state, "kre", vec![krenko.clone()]);
    let ticket = edit_ticket(&mut state, Action::Add(oracle(&krenko)));
    // A refetch lands with Krenko before the add's reply.
    let refetch = fetch_ticket(&state.update(Action::SwipeClosed)[0]);
    let _ = state.update(Action::Loaded {
        ticket: refetch,
        cards: vec![krenko],
    });
    let _ = state.update(Action::Added { ticket });
    assert_eq!(shown(&state), ["Krenko"]);
}

#[test]
fn an_add_not_in_the_results_does_nothing() {
    let mut state = loaded(Vec::new());
    assert_eq!(state.update(Action::Add(Uuid::new_v4())), []);
}

#[test]
fn a_failed_add_raises_nothing_more_and_its_late_success_is_ignored() {
    let krenko = commander("Krenko", &[Color::Red]);
    let mut state = loaded(Vec::new());
    searched(&mut state, "kre", vec![krenko.clone()]);
    let ticket = edit_ticket(&mut state, Action::Add(oracle(&krenko)));
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert_eq!(state.update(Action::Added { ticket }), []);
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn the_highlight_steps_through_the_results_and_adds() {
    let krenko = commander("Krenko", &[Color::Red]);
    let kresh = commander("Kresh", &[]);
    let mut state = loaded(Vec::new());
    searched(&mut state, "kre", vec![krenko, kresh.clone()]);
    assert_eq!(state.update(Action::AddHighlighted), [], "nothing is lit");
    let _ = state.update(Action::StepHighlight(Step::Down));
    let _ = state.update(Action::StepHighlight(Step::Down));
    assert_eq!(state.highlight(), Some(1));
    let effects = state.update(Action::AddHighlighted);
    assert!(
        matches!(effects.as_slice(), [Effect::AddCommander { oracle_id, .. }] if *oracle_id == oracle(&kresh)),
        "{effects:?}"
    );
    assert_eq!(state.highlight(), None, "an add clears the highlight");
}

#[test]
fn typing_clears_the_highlight() {
    let mut state = loaded(Vec::new());
    searched(&mut state, "kre", vec![commander("Krenko", &[])]);
    let _ = state.update(Action::StepHighlight(Step::Up));
    assert_eq!(state.highlight(), Some(0));
    let _ = state.update(Action::Type("kren".to_owned()));
    assert_eq!(state.highlight(), None);
}

#[test]
fn a_remove_drops_the_row_when_it_lands() {
    let cards = library();
    let atraxa = oracle(&cards[0]);
    let mut state = loaded(cards);
    let ticket = edit_ticket(&mut state, Action::Remove(atraxa));
    assert_eq!(
        shown(&state).len(),
        3,
        "the row left before the server said so"
    );
    let effects = state.update(Action::Removed { ticket });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(ToastKind::Info, "Removed", QUICK))],
        "a settled list fetched again"
    );
    assert_eq!(shown(&state), ["Krenko", "Kenrith"]);
}

#[test]
fn a_load_sent_before_a_remove_does_not_bring_the_row_back() {
    let cards = library();
    let atraxa = oracle(&cards[0]);
    let mut state = loaded(cards.clone());
    let older_load = fetch_ticket(&state.update(Action::SwipeClosed)[0]);
    let remove = edit_ticket(&mut state, Action::Remove(atraxa));
    let effects = state.update(Action::Removed { ticket: remove });
    let [refetch, Effect::Toast(_)] = effects.as_slice() else {
        panic!("the older_load load was not sent again: {effects:?}");
    };
    let fresh = fetch_ticket(refetch);
    let _ = state.update(Action::Loaded {
        ticket: older_load,
        cards,
    });
    assert_eq!(
        shown(&state),
        ["Krenko", "Kenrith"],
        "the older load put the row back"
    );
    let _ = state.update(Action::Loaded {
        ticket: fresh,
        cards: library().split_off(1),
    });
    assert_eq!(shown(&state).len(), 2);
}

#[test]
fn a_clear_empties_the_list_when_it_lands() {
    let mut state = loaded(library());
    let ticket = edit_ticket(&mut state, Action::Clear);
    assert_eq!(state.status(), Status::Loaded);
    let effects = state.update(Action::Cleared { ticket });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(
            ToastKind::Info,
            "Maybeboard cleared",
            QUICK
        ))]
    );
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.message(&blank()), Some(NO_COMMANDERS));
}

#[test]
fn a_clear_with_a_load_out_sends_it_again() {
    let mut state = loaded(library());
    let _ = state.update(Action::SwipeClosed);
    let ticket = edit_ticket(&mut state, Action::Clear);
    let effects = state.update(Action::Cleared { ticket });
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::FetchMaybeboard(_), Effect::Toast(_)]
        ),
        "{effects:?}"
    );
}

#[test]
fn a_reply_of_the_wrong_kind_is_ignored() {
    let cards = library();
    let atraxa = oracle(&cards[0]);
    let mut state = loaded(cards);
    let ticket = edit_ticket(&mut state, Action::Remove(atraxa));
    assert_eq!(state.update(Action::Cleared { ticket }), []);
    assert_eq!(shown(&state).len(), 3);
    let effects = state.update(Action::Removed { ticket });
    assert_eq!(effects.len(), 1, "the remove still waits: {effects:?}");
}

#[test]
fn create_seeds_the_commander_and_opens_create_deck() {
    let card = Box::new(commander("Krenko", &[Color::Red]));
    let mut state = loaded(Vec::new());
    let _ = state.update(Action::DeckProfilesLoaded { count: 3 });
    let effects = state.update(Action::Create {
        card: card.clone(),
        verified: true,
    });
    assert_eq!(
        effects,
        [
            Effect::SeedCommander(card),
            Effect::Navigate(Destination::CreateDeck)
        ]
    );
}

#[test]
fn an_unverified_user_with_a_deck_is_warned() {
    let card = Box::new(commander("Krenko", &[Color::Red]));
    let mut state = loaded(Vec::new());
    let _ = state.update(Action::DeckProfilesLoaded { count: 1 });
    let effects = state.update(Action::Create {
        card,
        verified: false,
    });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(
            ToastKind::Warning,
            VERIFY_TO_CREATE,
            LONG
        ))]
    );
}

#[test]
fn the_gate_counts_no_deck_before_the_decks_arrive() {
    let card = Box::new(commander("Krenko", &[Color::Red]));
    let mut state = loaded(Vec::new());
    let effects = state.update(Action::Create {
        card,
        verified: false,
    });
    assert!(
        matches!(
            effects.last(),
            Some(Effect::Navigate(Destination::CreateDeck))
        ),
        "{effects:?}"
    );
}

#[test]
fn a_picked_printing_swaps_the_entry_sharing_its_oracle() {
    let cards = library();
    let mut swapped = commander("Atraxa, Praetors' Voice", &[]);
    swapped.scryfall_data.oracle_id = cards[0].scryfall_data.oracle_id;
    let mut state = loaded(cards);
    assert_eq!(
        state.update(Action::PickPrinting(Box::new(swapped.clone()))),
        []
    );
    assert_eq!(state.entries()[0], swapped);

    let mut stray = commander("Stray", &[]);
    stray.scryfall_data.oracle_id = None;
    let before = state.clone();
    let _ = state.update(Action::PickPrinting(Box::new(stray)));
    assert_eq!(state, before, "a card with no oracle id changed the list");
}

#[test]
fn the_swipe_closing_fetches_again() {
    let mut state = loaded(library());
    let effects = state.update(Action::SwipeClosed);
    assert!(
        matches!(effects.as_slice(), [Effect::FetchMaybeboard(_)]),
        "{effects:?}"
    );
}

#[test]
fn saved_oracles_lists_every_saved_commander() {
    let cards = library();
    let want: HashSet<Uuid> = cards.iter().map(oracle).collect();
    assert_eq!(loaded(cards).saved_oracles(), want);
}

#[test]
fn back_goes_to_the_deck_list() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::DeckList)]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Commander maybeboard");
    assert_eq!(QUICK_ADD, "Quick add");
    assert_eq!(QUICK_ADD_PLACEHOLDER, "Search a commander to add");
    assert_eq!(SEARCHING, "Searching...");
    assert_eq!(NO_RESULTS, "No results");
    assert_eq!(SHOW, "Show");
    assert_eq!(SHOW_ALL, "All");
    assert_eq!(ART, "Art");
    assert_eq!(CREATE_DECK, "Create deck");
    assert_eq!(REMOVE, "Remove");
    assert_eq!(FILTER, "Filter");
    assert_eq!(SWIPE, "Swipe");
    assert_eq!(CLEAR_MAYBEBOARD, "Clear maybeboard");
    assert_eq!(
        CLEAR_BODY,
        "Every commander on your maybeboard will be removed. This can't be undone."
    );
    assert_eq!(CANCEL, "Cancel");
    assert_eq!(CLEAR, "Clear");
    assert_eq!(REMOVED, "Removed");
    assert_eq!(CLEARED, "Maybeboard cleared");
    assert_eq!(added("Krenko, Mob Boss"), "Added Krenko, Mob Boss");
    assert_eq!(LIST_TITLE, "Commanders");
    assert_eq!(NO_COMMANDERS, "No commanders");
    assert_eq!(NO_MATCH, "No commanders match");
    assert_eq!(LOAD_FAILED, "Couldn't load your commander maybeboard");
}
