use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::deck::Format;

use super::*;
use crate::test_support::{card, deck, deck_profile};
use crate::toast::{NORMAL, QUICK};

/// No effects.
const NOTHING: [Effect; 0] = [];

/// A commander deck holding `names` on the main board, with its commander in the command zone.
fn commander_deck(names: &[&str]) -> Deck {
    let commander = card("Commander");
    let mut profile = deck_profile("Build");
    profile.format = Some(Format::Commander);
    profile.commander_id = Some(commander.scryfall_data.id);
    let mut built = deck(profile, names.iter().map(|n| card(n)).collect());
    built.command_zone_cards = vec![commander];
    built
}

/// A deck with no format and no commander, which Search serves only with a filter.
fn pile_deck() -> Deck {
    deck(deck_profile("Pile"), Vec::new())
}

/// A land card.
fn land(name: &str) -> Card {
    let mut c = card(name);
    c.scryfall_data.type_line = Some("Basic Land — Forest".to_owned());
    c
}

/// A page of `cards`, synergy applied.
fn page(cards: Vec<Card>) -> HttpDeckCardSearch {
    HttpDeckCardSearch {
        cards,
        synergy_applied: true,
        related_from: None,
    }
}

/// The ticket of the first request in `effects`.
fn ticket_of(effects: &[Effect]) -> Ticket {
    effects
        .iter()
        .find_map(|e| match e {
            Effect::FetchDeck { ticket, .. }
            | Effect::RefreshMaybeboard { ticket, .. }
            | Effect::Search { ticket, .. }
            | Effect::LoadMore { ticket, .. }
            | Effect::AddCard { ticket, .. }
            | Effect::PromoteCard { ticket, .. }
            | Effect::TakeUndo { ticket, .. }
            | Effect::UndoAdd { ticket, .. }
            | Effect::UndoPromote { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no request in {effects:?}"))
}

/// Whether `effects` starts a search from the top.
fn has_search(effects: &[Effect]) -> bool {
    effects.iter().any(|e| matches!(e, Effect::Search { .. }))
}

/// The toasts in `effects`.
fn toasts(effects: &[Effect]) -> Vec<Toast> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Toast(toast) | Effect::Confirm(toast) => Some(toast.clone()),
            _ => None,
        })
        .collect()
}

/// The words of the toasts in `effects`.
fn words(effects: &[Effect]) -> Vec<String> {
    toasts(effects).into_iter().map(|t| t.text).collect()
}

/// The name of the card on top.
fn top(state: &State) -> Option<String> {
    state.current().map(|c| c.scryfall_data.name.clone())
}

/// A state that sent `Load` and took in `built`, with the effects of taking it in.
fn with_deck(built: &Deck) -> (State, Vec<Effect>) {
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: None,
        parked: None,
    });
    let ticket = ticket_of(&effects);
    let effects = state.update(Action::DeckLoaded {
        ticket,
        deck: Box::new(built.clone()),
    });
    (state, effects)
}

/// A commander deck's screen serving `served`, the client having applied the seeded filter.
fn serving(served: &[Card]) -> State {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let effects = state.update(Action::Filter(state.filter().clone()));
    let ticket = ticket_of(&effects);
    let effects = state.update(Action::Searched {
        ticket,
        page: page(served.to_vec()),
    });
    assert!(toasts(&effects).is_empty(), "{effects:?}");
    state
}

#[test]
fn load_offers_the_hint_fetches_the_deck_and_waits_for_a_filter() {
    let built = commander_deck(&[]);
    let mut state = State::new();
    assert_eq!(state.pile(), Pile::Empty);
    let effects = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: None,
        parked: None,
    });
    assert!(matches!(
        effects.as_slice(),
        [
            Effect::OfferHint(OneTimeHint::AddDeckCards),
            Effect::FetchDeck { .. },
            Effect::StopEnter(Source::Search),
            Effect::EaseImages,
        ]
    ));
    assert!(!state.is_deck_loaded());
    let again = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: None,
        parked: None,
    });
    assert!(
        !again.iter().any(|e| matches!(e, Effect::OfferHint(_))),
        "the hint is offered once per visit"
    );
}

#[test]
fn a_remembered_filter_searches_before_the_deck_is_in() {
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id: Uuid::new_v4(),
        filter: Some(CardQueryBuilder::with_name_contains("elf")),
        parked: None,
    });
    assert!(has_search(&effects));
    assert_eq!(state.pile(), Pile::Loading);
}

#[test]
fn the_deck_seeds_the_filter_and_serves_an_empty_stack() {
    let (state, effects) = with_deck(&commander_deck(&["Elf"]));
    let [Effect::SetFilter(filter), Effect::Rerun] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert!(filter.synergy(), "a commander deck starts in synergy");
    assert_eq!(
        filter.legalities_contains_any(),
        Some(&["commander".to_owned()][..])
    );
    assert!(state.shows_synergy());
    assert!(!state.shows_dot(filter), "deck context is no user filter");
    assert!(state.validates_apply());
}

#[test]
fn a_remembered_filter_keeps_its_synergy_pick() {
    let built = commander_deck(&[]);
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: Some(CardQueryBuilder::with_name_contains("elf")),
        parked: None,
    });
    let ticket = ticket_of(&effects);
    let _searching = state.update(Action::DeckLoaded {
        ticket,
        deck: Box::new(built),
    });
    assert!(!state.filter().synergy());
}

#[test]
fn lands_stop_once_the_deck_meets_its_target() {
    let mut built = commander_deck(&[]);
    built.deck_profile.land_target = Some(1);
    built = deck(built.deck_profile.clone(), vec![land("Forest")]);
    let (state, _) = with_deck(&built);
    assert_eq!(
        state.filter().card_type_excludes_any(),
        Some(&[CardType::Land][..])
    );
}

#[test]
fn search_asks_for_a_filter_when_it_has_nothing_to_serve() {
    let (mut state, _) = with_deck(&pile_deck());
    let effects = state.update(Action::Filter(state.filter().clone()));
    assert!(!has_search(&effects));
    assert_eq!(
        toasts(&effects),
        [Toast::normal(ToastKind::Warning, TRY_A_FILTER)]
    );
    assert_eq!(state.pile(), Pile::Empty);
}

#[test]
fn a_search_serves_cards_the_deck_does_not_hold() {
    let built = commander_deck(&["Elf"]);
    let held = built.entries[0].card.clone();
    let (mut state, _) = with_deck(&built);
    let effects = state.update(Action::Filter(state.filter().clone()));
    assert_eq!(state.pile(), Pile::Loading);
    let ticket = ticket_of(&effects);
    let _served = state.update(Action::Searched {
        ticket,
        page: page(vec![held, card("Bear")]),
    });
    assert_eq!(state.pile(), Pile::Cards);
    assert_eq!(top(&state).as_deref(), Some("Bear"));
    assert_eq!(state.window().len(), 1);
    assert_eq!(
        state.printing_card().map(|c| c.scryfall_data.name.as_str()),
        Some("Bear")
    );
}

#[test]
fn an_empty_search_says_why_on_the_ghost_alone() {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let effects = state.update(Action::Filter(state.filter().clone()));
    let ticket = ticket_of(&effects);
    let effects = state.update(Action::Searched {
        ticket,
        page: page(Vec::new()),
    });
    assert!(effects.iter().any(|e| matches!(
        e,
        Effect::PileEnd {
            kind: PileEndKind::Empty,
            synergy: true,
            ..
        }
    )));
    assert!(
        toasts(&effects).is_empty(),
        "the ghost says it, not a toast: {effects:?}"
    );
    assert_eq!(state.pile(), Pile::Empty);
    assert_eq!(
        state.ghost_note(),
        Some(NO_CARDS_MATCH),
        "the ghost says why"
    );
    assert_eq!(
        state.ghost_synergy_note(),
        Some("Synergy is on, turn it off to search everything"),
        "and that Synergy may be the reason"
    );

    let effects = state.update(Action::Filter(state.filter().clone()));
    assert_eq!(state.pile(), Pile::Loading);
    assert_eq!(state.ghost_note(), None, "no note while the search runs");
    let ticket = ticket_of(&effects);
    let _ = state.update(Action::Searched {
        ticket,
        page: page(vec![card("Bear")]),
    });
    assert_eq!(state.ghost_note(), None, "cards on show");
}

#[test]
fn the_ghost_has_no_note_before_a_search() {
    let (state, _) = with_deck(&commander_deck(&[]));
    assert_eq!(state.ghost_note(), None);
}

#[test]
fn a_config_only_filter_searches_and_an_empty_reply_says_no_cards_match() {
    let mut profile = deck_profile("Modern");
    profile.format = Some(Format::Modern);
    let (mut state, _) = with_deck(&deck(profile, Vec::new()));
    let mut promo = state.filter().clone();
    promo.set_promo(true);
    let effects = state.update(Action::Filter(promo));
    assert!(has_search(&effects), "Is promo alone searches: {effects:?}");
    let _ = state.update(Action::Searched {
        ticket: ticket_of(&effects),
        page: page(Vec::new()),
    });
    assert_eq!(state.ghost_note(), Some(NO_CARDS_MATCH));
    assert_eq!(
        state.ghost_synergy_note(),
        None,
        "no Synergy without a commander"
    );
}

#[test]
fn an_unsearchable_filter_says_try_a_filter_once_the_deck_is_in() {
    let (mut state, _) = with_deck(&pile_deck());
    let effects = state.update(Action::Filter(state.filter().clone()));
    assert!(!has_search(&effects));
    assert_eq!(
        toasts(&effects),
        [Toast::normal(ToastKind::Warning, TRY_A_FILTER)]
    );
}

#[test]
fn a_late_search_reply_is_ignored() {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let older = ticket_of(&state.update(Action::Filter(state.filter().clone())));
    let newer = ticket_of(&state.update(Action::Filter(state.filter().clone())));
    let effects = state.update(Action::Searched {
        ticket: older,
        page: page(vec![card("Old")]),
    });
    assert_eq!(effects, NOTHING);
    assert_eq!(state.pile(), Pile::Loading, "the newer search still runs");
    let _served = state.update(Action::Searched {
        ticket: newer,
        page: page(vec![card("New")]),
    });
    assert_eq!(top(&state).as_deref(), Some("New"));
}

#[test]
fn a_gone_deck_is_replaced_by_the_deck_list() {
    let built = commander_deck(&[]);
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: None,
        parked: None,
    });
    let Some(Effect::FetchDeck { ticket, .. }) = effects.get(1) else {
        panic!("the deck is fetched: {effects:?}");
    };
    assert_eq!(
        state.update(Action::Gone { ticket: *ticket }),
        [Effect::Replace(Destination::DeckList)]
    );
    assert!(!state.is_deck_loaded());
}

#[test]
fn a_failed_search_stops_loading() {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let ticket = ticket_of(&state.update(Action::Filter(state.filter().clone())));
    assert_eq!(state.update(Action::Failed { ticket }), NOTHING);
    assert_eq!(state.pile(), Pile::Empty);
    assert!(state.load_failed(), "the pile says it failed, with a Retry");
    assert_eq!(state.ghost_note(), None);

    let effects = state.update(Action::Retry);
    assert!(has_search(&effects), "Retry searches again: {effects:?}");
    assert!(!state.load_failed(), "the skeleton while it runs");
    let _ = state.update(Action::Searched {
        ticket: ticket_of(&effects),
        page: page(vec![card("Bear")]),
    });
    assert!(!state.load_failed());
    assert_eq!(state.update(Action::Retry), NOTHING, "nothing failed");
}

#[test]
fn a_failed_deck_fetch_comes_back_on_retry() {
    let deck_id = Uuid::new_v4();
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id,
        filter: None,
        parked: None,
    });
    let deck_ticket = effects
        .iter()
        .find_map(|e| match e {
            Effect::FetchDeck { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .unwrap();
    let search = effects.iter().find_map(|e| match e {
        Effect::Search { ticket, .. } => Some(*ticket),
        _ => None,
    });
    let _ = state.update(Action::Failed {
        ticket: deck_ticket,
    });
    if let Some(ticket) = search {
        let _ = state.update(Action::Failed { ticket });
    }
    assert!(state.load_failed(), "no deck and no cards");
    let effects = state.update(Action::Retry);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::FetchDeck { .. })),
        "the deck goes again: {effects:?}"
    );
    assert_eq!(
        has_search(&effects),
        search.is_some(),
        "and the search, if it had gone out"
    );
}

#[test]
fn warming_toasts_once_until_synergy_applies_again() {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let mut cold = page(vec![card("A")]);
    cold.synergy_applied = false;
    for expected in [1, 0] {
        let ticket = ticket_of(&state.update(Action::Filter(state.filter().clone())));
        let effects = state.update(Action::Searched {
            ticket,
            page: cold.clone(),
        });
        assert_eq!(
            words(&effects)
                .iter()
                .filter(|w| *w == SYNERGY_WARMING)
                .count(),
            expected
        );
    }
}

#[test]
fn the_first_related_card_on_top_says_synergy_is_spent() {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let ticket = ticket_of(&state.update(Action::Filter(state.filter().clone())));
    let effects = state.update(Action::Searched {
        ticket,
        page: HttpDeckCardSearch {
            cards: vec![card("Pick"), card("Like")],
            synergy_applied: true,
            related_from: Some(1),
        },
    });
    assert_eq!(words(&effects), Vec::<String>::new());
    let effects = state.update(Action::Swipe(Direction::Left));
    assert_eq!(
        words(&effects).last().map(String::as_str),
        Some(SYNERGY_SPENT)
    );
    assert!(effects.iter().any(|e| matches!(
        e,
        Effect::PileEnd {
            kind: PileEndKind::SynergyExhausted,
            ..
        }
    )));
    let back = state.update(Action::Undo);
    let again = state.update(Action::Swipe(Direction::Left));
    assert!(!words(&back).contains(&SYNERGY_SPENT.to_owned()));
    assert!(
        !words(&again).contains(&SYNERGY_SPENT.to_owned()),
        "once per stack"
    );
}

#[test]
fn a_left_swipe_skips_for_a_week() {
    let mut state = serving(&[card("A"), card("B")]);
    let effects = state.update(Action::Swipe(Direction::Left));
    assert!(matches!(effects.first(), Some(Effect::SkipCard { .. })));
    assert_eq!(toasts(&effects)[0], Toast::quick(ToastKind::Info, SKIPPED));
    assert_eq!(top(&state).as_deref(), Some("B"));
}

#[test]
fn a_right_swipe_adds_and_records_the_undo_once_it_lands() {
    let mut state = serving(&[card("A"), card("B")]);
    let effects = state.update(Action::Swipe(Direction::Right));
    let Some(Effect::AddCard { card, board, .. }) = effects.first() else {
        panic!("{effects:?}");
    };
    assert_eq!(*board, Board::Deck);
    assert_eq!(add_request(card, Board::Deck).board, None);
    assert_eq!(
        toasts(&effects)[0],
        Toast::quick(ToastKind::Success, ADDED_TO_DECK)
    );
    let card_id = card.scryfall_data.id;
    let ticket = ticket_of(&effects);
    let deck_card = Box::new(serving_deck_card(card));
    let effects = state.update(Action::Added { ticket, deck_card });
    assert!(matches!(
        effects.as_slice(),
        [Effect::RecordUndo {
            action: UndoAction::Added { card_id: id, .. },
            ..
        }] if *id == card_id
    ));
}

/// The deck card the server creates for `card`.
fn serving_deck_card(card: &Card) -> DeckCard {
    DeckCard {
        deck_id: Uuid::new_v4(),
        scryfall_data_id: card.scryfall_data.id,
        oracle_id: card.scryfall_data.oracle_id.unwrap_or_default(),
        quantity: Quantity::one(),
        board: Board::Deck,
        mvp_at: None,
    }
}

#[test]
fn an_up_swipe_adds_to_the_maybeboard_which_the_maybeboard_source_shows() {
    let mut state = serving(&[card("A"), card("B")]);
    let effects = state.update(Action::Swipe(Direction::Up));
    let Some(Effect::AddCard { card, board, .. }) = effects.first() else {
        panic!("{effects:?}");
    };
    assert_eq!(*board, Board::Maybeboard);
    assert_eq!(
        add_request(card, Board::Maybeboard).board.as_deref(),
        Some("maybeboard")
    );
    assert_eq!(
        toasts(&effects),
        [Toast::quick(ToastKind::Info, ADDED_TO_MAYBEBOARD)]
    );
    let deck_card = Box::new(serving_deck_card(card));
    let _recorded = state.update(Action::Added {
        ticket: ticket_of(&effects),
        deck_card,
    });
    let _picked = state.update(Action::PickSource {
        source: Source::Maybeboard,
        remembered: None,
    });
    let _filtered = state.update(Action::Filter(state.filter().clone()));
    assert_eq!(top(&state).as_deref(), Some("A"));
}

#[test]
fn adds_toast_the_land_target_and_the_budget_bands() {
    let mut built = commander_deck(&[]);
    built.deck_profile.land_target = Some(1);
    built.deck_profile.price_target = Some(0.3);
    let (mut state, _) = with_deck(&built);
    let mut forest = land("Forest");
    forest.scryfall_data.prices.usd = Some("0.08".to_owned());
    let ticket = ticket_of(&state.update(Action::Filter(state.filter().clone())));
    let _served = state.update(Action::Searched {
        ticket,
        page: page(vec![forest, land("Swamp")]),
    });
    let effects = state.update(Action::Swipe(Direction::Right));
    assert_eq!(
        toasts(&effects),
        [Toast::normal(ToastKind::Info, "Land target reached (1)")],
        "one toast per action: the first of two equal bands, in place of the swipe's own line"
    );
    let effects = state.update(Action::Swipe(Direction::Right));
    assert!(
        !words(&effects).iter().any(|w| w.starts_with("Land target")),
        "only the upward crossing toasts"
    );
}

#[test]
fn nearing_the_end_fetches_a_page_that_skips_what_the_stack_holds() {
    let cards: Vec<Card> = (0..6).map(|i| card(&format!("C{i}"))).collect();
    let mut state = serving(&cards);
    let effects = state.update(Action::Swipe(Direction::Left));
    let Some(Effect::LoadMore { ticket, .. }) = effects.last() else {
        panic!("{effects:?}");
    };
    let ticket = *ticket;
    let again = state.update(Action::Swipe(Direction::Left));
    assert!(
        !again.iter().any(|e| matches!(e, Effect::LoadMore { .. })),
        "one page at a time"
    );
    let _appended = state.update(Action::Searched {
        ticket,
        page: page(vec![cards[0].clone(), card("C6")]),
    });
    assert_eq!(state.window().len(), 5);
}

#[test]
fn an_empty_page_ends_the_results() {
    let mut state = serving(&[card("A")]);
    let effects = state.update(Action::Swipe(Direction::Left));
    assert_eq!(state.pile(), Pile::Loading, "the next page is on its way");
    let effects = state.update(Action::Searched {
        ticket: ticket_of(&effects[1..]),
        page: page(Vec::new()),
    });
    assert!(
        matches!(
            effects.as_slice(),
            [
                Effect::PileEnd {
                    kind: PileEndKind::Empty,
                    ..
                },
                Effect::Toast(toast),
            ] if *toast == Toast::quick(ToastKind::Info, NO_CARDS_LEFT)
        ),
        "the reply that ends the pile under the cursor says so: {effects:?}"
    );
    assert_eq!(state.pile(), Pile::End);
    let _back = state.update(Action::Undo);
    let effects = state.update(Action::Swipe(Direction::Left));
    assert_eq!(
        toasts(&effects).last(),
        Some(&Toast::quick(ToastKind::Info, NO_CARDS_LEFT))
    );
}

#[test]
fn a_late_page_is_ignored_but_ends_loading() {
    let mut state = serving(&[card("A")]);
    let effects = state.update(Action::Swipe(Direction::Left));
    let page_ticket = ticket_of(&effects[1..]);
    let _refreshed = state.update(Action::Refresh);
    let effects = state.update(Action::Searched {
        ticket: page_ticket,
        page: page(vec![card("Stale")]),
    });
    assert_eq!(effects, NOTHING);
    assert_eq!(state.window().len(), 0);
}

#[test]
fn a_failed_page_lets_the_next_one_go() {
    let cards: Vec<Card> = (0..7).map(|i| card(&format!("C{i}"))).collect();
    let mut state = serving(&cards);
    let effects = state.update(Action::Swipe(Direction::Left));
    let ticket = ticket_of(&effects[1..]);
    assert_eq!(state.update(Action::Failed { ticket }), NOTHING);
    let effects = state.update(Action::Swipe(Direction::Left));
    assert!(effects.iter().any(|e| matches!(e, Effect::LoadMore { .. })));
}

#[test]
fn the_stack_warns_near_its_cap_and_stops_at_it() {
    let cards: Vec<Card> = (0..MAX_CARDS_IN_STACK)
        .map(|i| card(&format!("C{i}")))
        .collect();
    let mut state = serving(&cards[..400]);
    let effects = state.update(Action::Swipe(Direction::Left));
    assert!(words(&effects).contains(&STACK_LIMIT_NEAR.to_owned()));
    let mut full = serving(&cards);
    for _ in 0..495 {
        let _swiped = full.update(Action::Swipe(Direction::Left));
    }
    let effects = full.update(Action::Swipe(Direction::Left));
    assert_eq!(
        toasts(&effects).last(),
        Some(&Toast::normal(ToastKind::Warning, STACK_LIMIT_REACHED))
    );
}

#[test]
fn undo_with_nothing_to_undo_says_so() {
    let mut state = serving(&[card("A")]);
    assert_eq!(
        toasts(&state.update(Action::Undo)),
        [Toast::normal(ToastKind::Info, NOTHING_TO_UNDO)]
    );
}

#[test]
fn undoing_a_skip_brings_the_card_back_from_the_left() {
    let mut state = serving(&[card("A"), card("B")]);
    let _skipped = state.update(Action::Swipe(Direction::Left));
    let effects = state.update(Action::Undo);
    assert!(matches!(
        effects.as_slice(),
        [
            Effect::Enter {
                pile: Source::Search,
                from: Direction::Left
            },
            Effect::UnskipCard { .. },
            Effect::Confirm(_),
        ]
    ));
    assert_eq!(
        toasts(&effects),
        [Toast::quick(ToastKind::Info, UNDID_SKIP)]
    );
    assert_eq!(top(&state).as_deref(), Some("A"));
}

#[test]
fn undoing_an_add_in_flight_parks_the_delete_for_its_reply() {
    let mut built = commander_deck(&[]);
    built.deck_profile.land_target = Some(1);
    let (mut state, _) = with_deck(&built);
    let ticket = ticket_of(&state.update(Action::Filter(state.filter().clone())));
    let _served = state.update(Action::Searched {
        ticket,
        page: page(vec![land("Forest"), card("B")]),
    });
    let effects = state.update(Action::Swipe(Direction::Right));
    let add = ticket_of(&effects);
    let undo = state.update(Action::Undo);
    assert_eq!(toasts(&undo), [Toast::quick(ToastKind::Success, UNDID_ADD)]);
    let effects = state.update(Action::Added {
        ticket: add,
        deck_card: Box::new(serving_deck_card(state.current().unwrap())),
    });
    assert!(matches!(
        effects.as_slice(),
        [Effect::DeleteUndoneAdd { .. }]
    ));
    let again = state.update(Action::Swipe(Direction::Right));
    assert!(
        words(&again).contains(&"Land target reached (1)".to_owned()),
        "the undo rewound the land count"
    );
}

/// A state that added its top card, the add landed, and the undo asked for the log entry.
fn claimed(board: Direction) -> (State, Ticket, Uuid) {
    let mut state = serving(&[card("A"), card("B")]);
    let effects = state.update(Action::Swipe(board));
    let Some(Effect::AddCard { card, .. }) = effects.first() else {
        panic!("{effects:?}");
    };
    let card_id = card.scryfall_data.id;
    let deck_card = Box::new(serving_deck_card(card));
    let _recorded = state.update(Action::Added {
        ticket: ticket_of(&effects),
        deck_card,
    });
    let effects = state.update(Action::Undo);
    let Some(Effect::TakeUndo { ticket, claim, .. }) = effects.last() else {
        panic!("{effects:?}");
    };
    assert_eq!(*claim, UndoClaim::Added { card_id });
    (state, *ticket, card_id)
}

#[test]
fn undoing_a_landed_add_deletes_it_with_its_log_entry() {
    let (mut state, ticket, card_id) = claimed(Direction::Right);
    let entry = UndoAction::Added {
        card_id,
        card_name: "A".to_owned(),
    };
    assert!(UndoClaim::Added { card_id }.matches(&entry));
    let effects = state.update(Action::Taken {
        ticket,
        taken: Some(entry),
    });
    let [
        Effect::UndoAdd {
            ticket,
            board: Board::Deck,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    let effects = state.update(Action::Landed { ticket: *ticket });
    assert_eq!(
        toasts(&effects),
        [Toast::quick(ToastKind::Success, UNDID_ADD)]
    );
}

#[test]
fn an_add_the_deck_cards_view_already_undid_sends_nothing() {
    let mut state = serving(&[card("A")]);
    let effects = state.update(Action::Swipe(Direction::Right));
    // The add fails, so the deck never holds the card and the log has no entry.
    let _failed = state.update(Action::Failed {
        ticket: ticket_of(&effects),
    });
    let undo = state.update(Action::Undo);
    let effects = state.update(Action::Taken {
        ticket: ticket_of(&undo),
        taken: None,
    });
    assert_eq!(
        toasts(&effects),
        [Toast::quick(ToastKind::Success, UNDID_ADD)]
    );
}

#[test]
fn an_add_without_its_entry_but_still_held_is_deleted_anyway() {
    let (mut state, ticket, _) = claimed(Direction::Right);
    let effects = state.update(Action::Taken {
        ticket,
        taken: None,
    });
    assert!(matches!(effects.as_slice(), [Effect::UndoAdd { .. }]));
}

#[test]
fn an_undo_that_never_went_out_steps_forward_and_puts_the_entry_back() {
    let (mut state, ticket, card_id) = claimed(Direction::Right);
    let entry = UndoAction::Added {
        card_id,
        card_name: "A".to_owned(),
    };
    let effects = state.update(Action::Taken {
        ticket,
        taken: Some(entry.clone()),
    });
    let effects = state.update(Action::NotSent {
        ticket: ticket_of(&effects),
    });
    assert!(matches!(
        effects.as_slice(),
        [Effect::StopEnter(Source::Search), Effect::RecordUndo { action, .. }] if *action == entry
    ));
    assert_eq!(top(&state).as_deref(), Some("B"));
}

#[test]
fn a_failed_undo_leaves_the_add_and_its_entry() {
    let (mut state, ticket, card_id) = claimed(Direction::Up);
    let entry = UndoAction::Added {
        card_id,
        card_name: "A".to_owned(),
    };
    let effects = state.update(Action::Taken {
        ticket,
        taken: Some(entry.clone()),
    });
    let effects = state.update(Action::Failed {
        ticket: ticket_of(&effects),
    });
    assert!(matches!(
        effects.as_slice(),
        [Effect::RecordUndo { action, .. }] if *action == entry
    ));
    assert_eq!(top(&state).as_deref(), Some("A"), "the cursor stays");
}

#[test]
fn undo_on_the_first_card_is_refused() {
    let mut state = serving(&[card("A"), card("B")]);
    let _skipped = state.update(Action::Swipe(Direction::Left));
    let mut parked = state.parked().unwrap();
    parked.index = 0;
    let mut back = State::new();
    let _resumed = back.update(Action::Load {
        deck_id: state.deck_id().unwrap(),
        filter: Some(state.filter().clone()),
        parked: Some(Box::new(parked)),
    });
    assert_eq!(
        toasts(&back.update(Action::Undo)),
        [Toast::normal(ToastKind::Warning, NO_PREVIOUS_CARD)]
    );
    assert_eq!(
        words(&back.update(Action::Undo)),
        [NO_PREVIOUS_CARD],
        "the swipe stays for another try"
    );
}

#[test]
fn a_parked_stack_resumes_when_its_filter_matches() {
    let mut state = serving(&[card("A"), card("B")]);
    let _skipped = state.update(Action::Swipe(Direction::Left));
    let parked = state.parked().unwrap();
    assert_eq!(parked.index, 1);
    let mut back = State::new();
    let effects = back.update(Action::Load {
        deck_id: state.deck_id().unwrap(),
        filter: Some(state.filter().clone()),
        parked: Some(Box::new(parked.clone())),
    });
    assert!(!has_search(&effects));
    assert_eq!(top(&back).as_deref(), Some("B"));
    let mut other = State::new();
    let effects = other.update(Action::Load {
        deck_id: state.deck_id().unwrap(),
        filter: Some(CardQueryBuilder::with_name_contains("other")),
        parked: Some(Box::new(parked)),
    });
    assert!(has_search(&effects), "another filter searches again");
    assert!(State::new().parked().is_none());
}

#[test]
fn the_maybeboard_source_cycles_promotes_and_undoes() {
    let mut built = commander_deck(&["Main", "Maybe A", "Maybe B"]);
    built.entries[1].deck_card.board = Board::Maybeboard;
    built.entries[2].deck_card.board = Board::Maybeboard;
    let (mut state, _) = with_deck(&built);
    let effects = state.update(Action::PickSource {
        source: Source::Maybeboard,
        remembered: None,
    });
    assert!(matches!(
        effects.as_slice(),
        [
            Effect::ParkFilter {
                source: Source::Search,
                ..
            },
            Effect::SetFilter(_),
            Effect::StopEnter(Source::Maybeboard),
            Effect::Rerun,
        ]
    ));
    assert!(!state.shows_synergy());
    assert!(!state.validates_apply());
    let _filtered = state.update(Action::Filter(state.filter().clone()));
    assert_eq!(state.pile(), Pile::Cards);
    assert_eq!(top(&state).as_deref(), Some("Maybe A"));

    assert_eq!(
        toasts(&state.update(Action::Swipe(Direction::Up))),
        [Toast::quick(ToastKind::Info, ALREADY_ON_MAYBEBOARD)]
    );
    assert_eq!(
        toasts(&state.update(Action::Swipe(Direction::Left))),
        [Toast::quick(ToastKind::Info, SKIPPED)]
    );
    assert_eq!(top(&state).as_deref(), Some("Maybe B"));
    let effects = state.update(Action::Undo);
    assert!(words(&effects).contains(&UNDID_SKIP.to_owned()));

    let effects = state.update(Action::Swipe(Direction::Right));
    let [
        Effect::PromoteCard {
            ticket, card_id, ..
        },
        Effect::Confirm(toast),
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert_eq!(*toast, Toast::quick(ToastKind::Success, MOVED_TO_DECK));
    let card_id = *card_id;
    assert_eq!(top(&state).as_deref(), Some("Maybe B"));
    let effects = state.update(Action::Landed { ticket: *ticket });
    let [Effect::RecordUndo { action, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert!(
        UndoClaim::MovedBoard {
            card_id,
            from: Board::Maybeboard
        }
        .matches(action)
    );

    let effects = state.update(Action::Undo);
    let effects = state.update(Action::Taken {
        ticket: ticket_of(&effects),
        taken: Some(action.clone()),
    });
    assert_eq!(top(&state).as_deref(), Some("Maybe A"));
    let effects = state.update(Action::Landed {
        ticket: ticket_of(&effects),
    });
    assert_eq!(
        toasts(&effects),
        [Toast::quick(ToastKind::Success, UNDID_MOVE_TO_DECK)]
    );
}

#[test]
fn an_undo_promote_still_standing_without_its_entry_moves_it_back() {
    let mut built = commander_deck(&["Maybe"]);
    built.entries[0].deck_card.board = Board::Maybeboard;
    let (mut state, _) = with_deck(&built);
    let _picked = state.update(Action::PickSource {
        source: Source::Maybeboard,
        remembered: None,
    });
    let _filtered = state.update(Action::Filter(state.filter().clone()));
    let _promoted = state.update(Action::Swipe(Direction::Right));
    // The entry aged out of the log, but the card is still off the maybeboard.
    let undo = state.update(Action::Undo);
    let effects = state.update(Action::Taken {
        ticket: ticket_of(&undo),
        taken: None,
    });
    assert!(matches!(effects.as_slice(), [Effect::UndoPromote { .. }]));
    let effects = state.update(Action::NotSent {
        ticket: ticket_of(&effects),
    });
    assert_eq!(effects, [Effect::StopEnter(Source::Maybeboard)]);
}

#[test]
fn an_undo_promote_finding_the_card_back_changes_nothing() {
    let mut built = commander_deck(&["Maybe"]);
    built.entries[0].deck_card.board = Board::Maybeboard;
    let (mut state, _) = with_deck(&built);
    let _picked = state.update(Action::PickSource {
        source: Source::Maybeboard,
        remembered: None,
    });
    let _filtered = state.update(Action::Filter(state.filter().clone()));
    let effects = state.update(Action::Swipe(Direction::Right));
    let promote = ticket_of(&effects);
    let undo = state.update(Action::Undo);
    let _reloaded = {
        let refresh = state.update(Action::Refresh);
        state.update(Action::DeckLoaded {
            ticket: ticket_of(&refresh),
            deck: Box::new(built),
        })
    };
    let _late = state.update(Action::Landed { ticket: promote });
    let effects = state.update(Action::Taken {
        ticket: ticket_of(&undo),
        taken: None,
    });
    assert_eq!(
        effects,
        [
            Effect::StopEnter(Source::Maybeboard),
            Effect::Toast(Toast::quick(ToastKind::Info, ALREADY_CHANGED)),
        ]
    );
}

#[test]
fn switching_back_to_search_takes_its_fresh_default() {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let _picked = state.update(Action::PickSource {
        source: Source::Maybeboard,
        remembered: None,
    });
    assert!(state.filter().is_empty());
    let effects = state.update(Action::PickSource {
        source: Source::Search,
        remembered: None,
    });
    assert!(matches!(
        effects.first(),
        Some(Effect::ParkFilter {
            source: Source::Maybeboard,
            ..
        })
    ));
    assert!(state.filter().synergy());
    assert!(
        state
            .update(Action::PickSource {
                source: Source::Search,
                remembered: None,
            })
            .is_empty(),
        "the live source changes nothing"
    );
}

#[test]
fn synergy_toggles_reruns_and_says_the_new_mode() {
    let (mut state, _) = with_deck(&commander_deck(&[]));
    let effects = state.update(Action::ToggleSynergy);
    let [
        Effect::SetFilter(filter),
        Effect::Rerun,
        Effect::Toast(said),
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert!(!filter.synergy());
    assert_eq!(said.text, SYNERGY_OFF);
    assert_eq!(said, &Toast::quick(ToastKind::Info, SYNERGY_OFF));

    let effects = state.update(Action::ToggleSynergy);
    let [
        Effect::SetFilter(filter),
        Effect::Rerun,
        Effect::Toast(said),
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert!(filter.synergy());
    assert_eq!(said.text, SYNERGY_ON);
}

#[test]
fn reset_stages_the_default_or_says_it_is_there() {
    let (state, _) = with_deck(&commander_deck(&[]));
    assert_eq!(
        state.update_copy(Action::ResetFilter(state.filter().clone())),
        [Effect::Toast(Toast::quick(
            ToastKind::Warning,
            FILTER_AT_DEFAULT
        ))]
    );
    let mut draft = state.filter().clone();
    draft.set_name_contains("elf");
    let effects = state.update_copy(Action::ResetFilter(draft));
    let [Effect::SetFilter(reset), Effect::Toast(toast)] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(reset, state.filter());
    assert_eq!(*toast, Toast::quick(ToastKind::Info, FILTER_RESET));
}

impl State {
    /// Applies `action` to a copy, for tests that don't need the state after.
    fn update_copy(&self, action: Action) -> Vec<Effect> {
        self.clone().update(action)
    }
}

#[test]
fn refresh_searches_again_without_explaining_an_empty_reply() {
    let mut state = serving(&[card("A")]);
    let effects = state.update(Action::Refresh);
    assert!(has_search(&effects));
    assert_eq!(
        toasts(&effects),
        [Toast::quick(ToastKind::Info, STACK_REFRESHED)]
    );
    assert_eq!(state.pile(), Pile::Loading);
    let effects = state.update(Action::Searched {
        ticket: ticket_of(&effects),
        page: page(Vec::new()),
    });
    assert_eq!(effects, NOTHING);

    let (mut pile, _) = with_deck(&pile_deck());
    assert_eq!(
        toasts(&pile.update(Action::Refresh)),
        [Toast::quick(ToastKind::Warning, FILTER_EMPTY)]
    );
}

#[test]
fn a_printing_pick_reskins_the_search_card() {
    let mut state = serving(&[card("A")]);
    let _swapped = state.update(Action::SwapPrinting(Box::new(card("A foil"))));
    assert_eq!(top(&state).as_deref(), Some("A foil"));
}

#[test]
fn back_returns_where_the_user_came_from() {
    assert_eq!(
        State::new().update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn a_late_deck_reply_is_ignored() {
    let built = commander_deck(&[]);
    let mut state = State::new();
    let first = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: None,
        parked: None,
    });
    let _again = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: None,
        parked: None,
    });
    let effects = state.update(Action::DeckLoaded {
        ticket: ticket_of(&first),
        deck: Box::new(built),
    });
    assert_eq!(effects, NOTHING);
    assert!(!state.is_deck_loaded());
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Add");
    assert_eq!(HINT_TITLE, "Swipe to build");
    assert_eq!(FROM, "From");
    assert_eq!(SYNERGY, "Synergy");
    assert_eq!([BACK, FILTER, REFRESH], ["Back", "Filter", "Refresh"]);
    assert_eq!(Source::ALL.map(Source::label), ["Search", "Maybe"]);
    assert_eq!(PRINTING_SWAPPED, "Printing swapped");
    assert_eq!(
        [
            SKIPPED,
            ADDED_TO_DECK,
            ADDED_TO_MAYBEBOARD,
            MOVED_TO_DECK,
            ALREADY_ON_MAYBEBOARD,
            NOTHING_TO_UNDO,
            NO_PREVIOUS_CARD,
            UNDID_SKIP,
            UNDID_ADD,
            UNDID_MAYBEBOARD,
            UNDID_MOVE_TO_DECK,
            ALREADY_CHANGED,
            TRY_A_FILTER,
            FILTER_AT_DEFAULT,
            FILTER_RESET,
            FILTER_EMPTY,
            STACK_REFRESHED,
            MAYBEBOARD_REFRESHED,
        ],
        [
            "Skipped",
            "Added to deck",
            "Added to maybeboard",
            "Moved to deck",
            "Already on maybeboard",
            "Nothing to undo",
            "No previous card",
            "Undid skip",
            "Undid add",
            "Undid maybeboard",
            "Undid move to deck",
            "Already changed",
            "Try a filter to start swiping",
            "Filter already at default",
            "Filter reset",
            "Filter is empty",
            "Stack refreshed",
            "Maybeboard refreshed",
        ]
    );
    assert_eq!(
        STACK_LIMIT_REACHED,
        "Card stack limit reached, start a new search to continue"
    );
    assert_eq!(
        STACK_LIMIT_NEAR,
        "Approaching the card stack limit, start a new search to continue"
    );
    assert_eq!(
        SYNERGY_WARMING,
        "Synergy warming up; showing all cards for now"
    );
    assert_eq!(
        SYNERGY_SPENT,
        "That's every synergy pick. Showing cards like your deck"
    );
    assert_eq!((QUICK, NORMAL), (crate::toast::QUICK, crate::toast::NORMAL));
}

#[test]
fn the_source_toggle_flips_between_the_two() {
    assert_eq!(Source::Search.toggled(), Source::Maybeboard);
    assert_eq!(Source::Maybeboard.toggled(), Source::Search);
}

#[test]
fn the_maybeboard_pile_running_out_under_a_swipe_says_no_cards_left_alone() {
    let mut built = commander_deck(&["Main", "Maybe A"]);
    built.entries[1].deck_card.board = Board::Maybeboard;
    let (mut state, _) = with_deck(&built);
    let _ = state.update(Action::PickSource {
        source: Source::Maybeboard,
        remembered: None,
    });
    let _filtered = state.update(Action::Filter(state.filter().clone()));
    assert_eq!(state.pile(), Pile::Cards);
    let effects = state.update(Action::Swipe(Direction::Right));
    assert_eq!(state.pile(), Pile::Empty);
    assert_eq!(
        toasts(&effects),
        [Toast::quick(ToastKind::Info, NO_CARDS_LEFT)],
        "the move's own line gives way"
    );
}
