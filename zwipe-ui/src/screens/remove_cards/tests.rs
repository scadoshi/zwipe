use zwipe_core::domain::deck::{DeckProfile, Quantity};
use zwipe_core::http::helpers::Patch;

use super::*;
use crate::test_support::{card, deck, deck_profile};
use crate::toast::{NORMAL, QUICK};

/// No effects.
const NOTHING: [Effect; 0] = [];

/// A state that has sent `Load` for `profile`'s deck, with the fetch's ticket.
fn loading(profile: &DeckProfile) -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id: profile.id,
        filter: CardQueryBuilder::default(),
    });
    let [
        Effect::OfferHint(OneTimeHint::RemoveDeckCards),
        Effect::FetchDeck { ticket, deck_id },
    ] = effects.as_slice()
    else {
        panic!("Load offers the hint and fetches the deck: {effects:?}");
    };
    assert_eq!(*deck_id, profile.id);
    (state, *ticket)
}

/// A state with `built` loaded.
fn loaded(built: Deck) -> State {
    let (mut state, ticket) = loading(&built.deck_profile);
    let effects = state.update(Action::Loaded {
        ticket,
        deck: Box::new(built),
    });
    let expected = if state.status() == Status::Empty {
        vec![Effect::Toast(Toast::quick(
            ToastKind::Info,
            NO_CARDS_ON_BOARDS,
        ))]
    } else {
        Vec::new()
    };
    assert_eq!(effects, expected, "loading asks for nothing more");
    state
}

/// A deck of `names`, one copy each on the main board.
fn deck_of(names: &[&str]) -> Deck {
    deck(
        deck_profile("Trim"),
        names.iter().map(|name| card(name)).collect(),
    )
}

/// The names of the stack's cards, in order.
fn names(state: &State) -> Vec<String> {
    state
        .cards()
        .iter()
        .map(|c| c.scryfall_data.name.clone())
        .collect()
}

/// The name of the card on top.
fn top(state: &State) -> Option<String> {
    state.current().map(|c| c.scryfall_data.name.clone())
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

/// The ticket of the first request in `effects`.
fn ticket_of(effects: &[Effect]) -> Ticket {
    effects
        .iter()
        .find_map(|e| match e {
            Effect::DeleteDeckCard { ticket, .. }
            | Effect::MoveDeckCard { ticket, .. }
            | Effect::TakeUndo { ticket, .. }
            | Effect::RestoreDeckCard { ticket, .. }
            | Effect::RestoreBoard { ticket, .. }
            | Effect::FetchDeck { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no request in {effects:?}"))
}

/// Swipes `action`, lands its request, and returns the undo log entry it recorded.
fn swipe_and_land(state: &mut State, action: Action) -> UndoAction {
    let effects = state.update(action);
    let ticket = ticket_of(&effects);
    match state.update(Action::Landed { ticket }).as_slice() {
        [Effect::RecordUndo { action, .. }] => action.clone(),
        other => panic!("a landed swipe records its undo entry: {other:?}"),
    }
}

/// Undoes the newest swipe and answers the claim with `taken`.
fn undo_with(state: &mut State, taken: Option<UndoAction>) -> Vec<Effect> {
    let effects = state.update(Action::Undo);
    let ticket = ticket_of(&effects);
    state.update(Action::Taken { ticket, taken })
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Remove");
    assert_eq!(HINT_TITLE, "Swipe to trim");
    assert_eq!(BOARDS, "Boards");
    assert_eq!([BACK, FILTER, REFRESH], ["Back", "Filter", "Refresh"]);
    assert_eq!(
        BoardFilter::ALL.map(BoardFilter::label),
        ["Main", "Maybe", "Side", "All"]
    );
    assert_eq!(
        [
            KEPT,
            REMOVED,
            MOVED_TO_MAYBEBOARD,
            MOVED_TO_MAIN,
            NOTHING_TO_UNDO,
            UNDID_KEEP,
            UNDID_REMOVE,
            UNDID_MOVE,
            ALREADY_CHANGED,
            STACK_REFRESHED,
        ],
        [
            "Kept",
            "Removed from deck",
            "Moved to maybeboard",
            "Moved to main",
            "Nothing to undo",
            "Undid keep",
            "Undid remove",
            "Undid move",
            "Already changed",
            "Stack refreshed",
        ]
    );
}

#[test]
fn a_new_screen_is_loading_on_the_main_board() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.board(), BoardFilter::Deck);
    assert!(state.current().is_none());
    assert_eq!(state.window().len(), 0, "no card draws");
}

#[test]
fn load_offers_the_hint_once_per_visit() {
    let profile = deck_profile("Trim");
    let (mut state, _) = loading(&profile);
    let effects = state.update(Action::Load {
        deck_id: profile.id,
        filter: CardQueryBuilder::default(),
    });
    assert!(
        matches!(effects.as_slice(), [Effect::FetchDeck { .. }]),
        "a second Load only fetches: {effects:?}"
    );
    assert_eq!(state.deck_id(), Some(profile.id));
}

#[test]
fn a_loaded_deck_shows_its_main_board() {
    let mut built = deck_of(&["Opt", "Sol Ring", "Shock"]);
    built.entries[2].deck_card.board = Board::Maybeboard;
    let state = loaded(built);
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(names(&state), ["Opt", "Sol Ring"]);
    assert_eq!(top(&state).as_deref(), Some("Opt"));
    assert_eq!(state.window().len(), 2);
}

#[test]
fn a_gone_deck_is_replaced_by_the_deck_list() {
    let (mut state, ticket) = loading(&deck_profile("Trim"));
    assert_eq!(
        state.update(Action::Gone { ticket }),
        [Effect::Replace(Destination::DeckList)]
    );
    assert_eq!(state.status(), Status::Failed);
}

#[test]
fn a_failed_load_says_so() {
    let (mut state, ticket) = loading(&deck_profile("Trim"));
    assert_eq!(state.update(Action::Failed { ticket }), NOTHING);
    assert_eq!(state.status(), Status::Failed);
}

#[test]
fn a_deck_with_nothing_on_the_board_is_empty() {
    let state = loaded(deck_of(&[]));
    assert_eq!(state.status(), Status::Empty);
    assert!(state.current().is_none());
}

#[test]
fn a_late_load_reply_is_ignored() {
    let built = deck_of(&["Opt"]);
    let (mut state, old) = loading(&built.deck_profile);
    let _ = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: CardQueryBuilder::default(),
    });
    let _ = state.update(Action::Loaded {
        ticket: old,
        deck: Box::new(built),
    });
    assert_eq!(state.status(), Status::Loading, "the old reply landed");
    let _ = state.update(Action::Failed { ticket: old });
    assert_eq!(state.status(), Status::Loading, "the old failure landed");
}

#[test]
fn the_board_chips_pick_what_shows() {
    let mut built = deck_of(&["Opt", "Shock", "Duress"]);
    built.entries[1].deck_card.board = Board::Maybeboard;
    built.entries[2].deck_card.board = Board::Sideboard;
    let mut state = loaded(built);
    let _ = state.update(Action::Keep);
    for (board, want) in [
        (BoardFilter::Maybeboard, vec!["Shock"]),
        (BoardFilter::Sideboard, vec!["Duress"]),
        (BoardFilter::All, vec!["Opt", "Shock", "Duress"]),
        (BoardFilter::Deck, vec!["Opt"]),
    ] {
        assert_eq!(state.update(Action::PickBoard(board)), NOTHING);
        assert_eq!(state.board(), board);
        assert_eq!(names(&state), want, "board {board:?}");
        assert_eq!(state.index(), 0, "a new pick starts at the first card");
    }
}

#[test]
fn a_board_picked_before_the_deck_loads_applies_once_it_does() {
    let mut built = deck_of(&["Opt", "Shock"]);
    built.entries[1].deck_card.board = Board::Maybeboard;
    let (mut state, ticket) = loading(&built.deck_profile);
    let _ = state.update(Action::PickBoard(BoardFilter::Maybeboard));
    let _ = state.update(Action::Loaded {
        ticket,
        deck: Box::new(built),
    });
    assert_eq!(names(&state), ["Shock"]);
}

#[test]
fn the_filter_narrows_and_sorts_the_stack() {
    let mut state = loaded(deck_of(&["Shock", "Opt", "Opportunity"]));
    let mut filter = CardQueryBuilder::default();
    filter.set_name_contains("Opp");
    let _ = state.update(Action::Filter(filter.clone()));
    assert_eq!(names(&state), ["Opportunity"]);
    assert_eq!(state.filter(), &filter);

    let mut by_name = CardQueryBuilder::default();
    by_name.set_sort(CardSortKey::Name);
    let _ = state.update(Action::Filter(by_name));
    assert_eq!(names(&state), ["Opportunity", "Opt", "Shock"]);
}

#[test]
fn the_filter_given_on_load_builds_the_first_stack() {
    let built = deck_of(&["Shock", "Opt"]);
    let mut filter = CardQueryBuilder::default();
    filter.set_name_contains("Opt");
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter,
    });
    let _ = state.update(Action::Loaded {
        ticket: ticket_of(&effects),
        deck: Box::new(built),
    });
    assert_eq!(names(&state), ["Opt"]);
}

#[test]
fn keep_cycles_past_the_card_and_wraps() {
    let mut state = loaded(deck_of(&["Opt", "Shock"]));
    let effects = state.update(Action::Keep);
    assert_eq!(
        effects,
        [Effect::Confirm(Toast::quick(ToastKind::Info, KEPT))]
    );
    assert_eq!(top(&state).as_deref(), Some("Shock"));
    let _ = state.update(Action::Keep);
    assert_eq!(top(&state).as_deref(), Some("Opt"), "the stack cycles");
}

#[test]
fn a_swipe_on_an_empty_stack_does_nothing() {
    let mut state = loaded(deck_of(&[]));
    for action in [Action::Keep, Action::Remove, Action::Move] {
        assert_eq!(state.update(action.clone()), NOTHING, "{action:?}");
    }
}

#[test]
fn remove_drops_the_card_and_records_undo_once_it_lands() {
    let mut built = deck_of(&["Opt", "Shock"]);
    built.entries[0].deck_card.quantity = Quantity::new(3).unwrap();
    let opt = built.entries[0].clone();
    let mut state = loaded(built.clone());

    let effects = state.update(Action::Remove);
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        [
            Effect::DeleteDeckCard {
                ticket,
                deck_id: built.deck_profile.id,
                card_id: opt.card.scryfall_data.id,
            },
            Effect::Confirm(Toast::quick(ToastKind::Success, REMOVED)),
        ]
    );
    assert_eq!(names(&state), ["Shock"]);
    assert_eq!(state.entries().len(), 1, "the entry left with the card");

    assert_eq!(
        state.update(Action::Landed { ticket }),
        [Effect::RecordUndo {
            deck_id: built.deck_profile.id,
            action: UndoAction::Removed {
                entry: Box::new(opt),
                baseline: 3,
            },
        }]
    );
}

#[test]
fn a_failed_remove_records_nothing_and_keeps_the_card_out() {
    let mut state = loaded(deck_of(&["Opt", "Shock"]));
    let ticket = ticket_of(&state.update(Action::Remove));
    assert_eq!(state.update(Action::Failed { ticket }), NOTHING);
    assert_eq!(names(&state), ["Shock"]);
}

#[test]
fn a_late_remove_reply_is_ignored() {
    let built = deck_of(&["Opt"]);
    let mut state = loaded(built.clone());
    let ticket = ticket_of(&state.update(Action::Remove));
    let _ = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: CardQueryBuilder::default(),
    });
    assert_eq!(state.update(Action::Landed { ticket }), NOTHING);
}

#[test]
fn move_toggles_between_main_and_maybeboard() {
    let mut built = deck_of(&["Opt", "Shock", "Duress"]);
    built.entries[1].deck_card.board = Board::Maybeboard;
    built.entries[2].deck_card.board = Board::Sideboard;
    let deck_id = built.deck_profile.id;
    let ids: Vec<Uuid> = built
        .entries
        .iter()
        .map(|e| e.card.scryfall_data.id)
        .collect();
    let mut state = loaded(built);
    let _ = state.update(Action::PickBoard(BoardFilter::All));

    for (at, from, to, message) in [
        (0, Board::Deck, Board::Maybeboard, MOVED_TO_MAYBEBOARD),
        (1, Board::Maybeboard, Board::Deck, MOVED_TO_MAIN),
        (2, Board::Sideboard, Board::Maybeboard, MOVED_TO_MAYBEBOARD),
    ] {
        let effects = state.update(Action::Move);
        let ticket = ticket_of(&effects);
        // The last move empties the pile, whose toast stands in for its own.
        let said = if at == 2 {
            Effect::Toast(Toast::quick(ToastKind::Info, NOTHING_LEFT))
        } else {
            Effect::Confirm(Toast::quick(ToastKind::Info, message))
        };
        assert_eq!(
            effects[..2],
            [
                Effect::MoveDeckCard {
                    ticket,
                    deck_id,
                    card_id: ids[at],
                    to,
                },
                said,
            ],
            "card {at}"
        );
        assert_eq!(state.entries()[at].deck_card.board, to);
        let recorded = state.update(Action::Landed { ticket });
        let [
            Effect::RecordUndo {
                action:
                    UndoAction::MovedBoard {
                        card_id,
                        from: left,
                        ..
                    },
                ..
            },
        ] = recorded.as_slice()
        else {
            panic!("a landed move records its undo entry: {recorded:?}");
        };
        assert_eq!((*card_id, *left), (ids[at], from));
    }
    assert!(state.is_empty(), "each moved card left the stack");
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn removing_a_land_below_the_target_warns() {
    let mut built = deck_of(&["Forest", "Island"]);
    built.deck_profile.land_target = Some(2);
    for entry in &mut built.entries {
        entry.card.scryfall_data.type_line = Some("Basic Land".to_owned());
    }
    let mut state = loaded(built);
    let effects = state.update(Action::Remove);
    assert_eq!(
        toasts(&effects),
        [Toast::new(
            ToastKind::Warning,
            "Below land target (2)",
            NORMAL
        )],
        "the warning stands in for the swipe's own line: one toast per action"
    );
    assert!(
        toasts(&state.update(Action::Remove))
            .iter()
            .all(|toast| toast.kind != ToastKind::Warning),
        "the warning fires once per crossing"
    );
}

#[test]
fn no_land_target_means_no_land_toast() {
    let mut built = deck_of(&["Forest"]);
    built.entries[0].card.scryfall_data.type_line = Some("Basic Land".to_owned());
    let mut state = loaded(built);
    assert!(
        toasts(&state.update(Action::Remove))
            .iter()
            .all(|toast| toast.kind != ToastKind::Warning)
    );
}

#[test]
fn moving_a_card_to_main_can_cross_the_budget() {
    let mut built = deck_of(&["Opt", "Jewel"]);
    built.deck_profile.price_target = Some(10.0);
    built.entries[0].card.scryfall_data.prices.usd = Some("1.00".to_owned());
    built.entries[1].card.scryfall_data.prices.usd = Some("9.50".to_owned());
    built.entries[1].deck_card.board = Board::Maybeboard;
    let mut state = loaded(built);
    let _ = state.update(Action::PickBoard(BoardFilter::Maybeboard));
    let effects = state.update(Action::Move);
    assert_eq!(
        toasts(&effects),
        [Toast::new(
            ToastKind::Warning,
            "Deck at 105.00% of your $10.00 budget",
            NORMAL
        )],
        "the budget warning outranks the emptied pile's line: one toast per action"
    );
}

#[test]
fn an_emptied_pile_says_why_once() {
    let mut state = loaded(deck_of(&["Opt"]));
    assert_eq!(
        toasts(&state.update(Action::Keep)),
        [Toast::quick(ToastKind::Info, KEPT)],
        "keeping cycles the pile, so it never runs out"
    );
    assert_eq!(
        toasts(&state.update(Action::Remove)),
        [Toast::quick(ToastKind::Info, NOTHING_LEFT)],
        "the run-out toast stands in for Removed from deck"
    );
    assert_eq!(state.status(), Status::Empty);
    assert!(
        toasts(&state.update(Action::Remove)).is_empty(),
        "an already empty pile says nothing more"
    );
}

#[test]
fn undo_with_no_swipe_says_so() {
    let mut state = loaded(deck_of(&["Opt"]));
    assert_eq!(
        state.update(Action::Undo),
        [Effect::Toast(Toast::new(
            ToastKind::Info,
            NOTHING_TO_UNDO,
            NORMAL
        ))]
    );
}

#[test]
fn undo_steps_back_over_a_keep() {
    let mut state = loaded(deck_of(&["Opt", "Shock"]));
    let _ = state.update(Action::Keep);
    assert_eq!(
        state.update(Action::Undo),
        [
            Effect::Enter(Direction::Left),
            Effect::Confirm(Toast::new(ToastKind::Info, UNDID_KEEP, QUICK)),
        ]
    );
    assert_eq!(top(&state).as_deref(), Some("Opt"));
}

#[test]
fn undo_puts_a_removed_card_back() {
    let built = deck_of(&["Opt", "Shock"]);
    let deck_id = built.deck_profile.id;
    let opt = built.entries[0].clone();
    let mut state = loaded(built);
    let recorded = swipe_and_land(&mut state, Action::Remove);

    let effects = state.update(Action::Undo);
    assert_eq!(
        effects,
        [Effect::TakeUndo {
            ticket: ticket_of(&effects),
            deck_id,
            claim: UndoClaim::Removed {
                card_id: opt.card.scryfall_data.id,
            },
        }]
    );
    let effects = state.update(Action::Taken {
        ticket: ticket_of(&effects),
        taken: Some(recorded),
    });
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        [
            Effect::Enter(Direction::Right),
            Effect::RestoreDeckCard {
                ticket,
                deck_id,
                card: Box::new(opt.card.clone()),
                board: Board::Deck,
                quantity: 1,
            },
        ]
    );
    assert_eq!(
        top(&state).as_deref(),
        Some("Opt"),
        "the card is back on top"
    );
    assert_eq!(state.entries().len(), 1, "the entry waits for the server");

    assert_eq!(
        state.update(Action::Restored {
            ticket,
            deck_card: Box::new(opt.deck_card.clone()),
        }),
        [Effect::Confirm(Toast::quick(
            ToastKind::Success,
            UNDID_REMOVE
        ))]
    );
    assert!(state.entries().contains(&opt));
}

#[test]
fn undo_of_a_remove_the_deck_cards_undo_reversed_sends_nothing() {
    let built = deck_of(&["Opt", "Shock"]);
    let opt = built.entries[0].clone();
    let mut state = loaded(built);
    let ticket = ticket_of(&state.update(Action::Remove));
    let _ = state.update(Action::Landed { ticket });
    // The card is back in the deck and its log entry is gone.
    let undo = ticket_of(&state.update(Action::Undo));
    state.entries.push(opt);
    assert_eq!(
        state.update(Action::Taken {
            ticket: undo,
            taken: None,
        }),
        [Effect::Toast(Toast::quick(
            ToastKind::Info,
            ALREADY_CHANGED
        ))]
    );
    assert_eq!(names(&state), ["Shock"], "the card stays out of the stack");
}

#[test]
fn undo_of_a_remove_whose_entry_aged_out_still_restores() {
    let mut state = loaded(deck_of(&["Opt"]));
    let _ = swipe_and_land(&mut state, Action::Remove);
    let effects = undo_with(&mut state, None);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::Enter(_), Effect::RestoreDeckCard { .. }]
        ),
        "the card is gone, so the undo goes ahead: {effects:?}"
    );
}

#[test]
fn a_failed_undo_gives_its_entry_back() {
    let built = deck_of(&["Opt"]);
    let deck_id = built.deck_profile.id;
    let mut state = loaded(built);
    let recorded = swipe_and_land(&mut state, Action::Remove);
    let ticket = ticket_of(&undo_with(&mut state, Some(recorded.clone())));
    assert_eq!(
        state.update(Action::Failed { ticket }),
        [Effect::RecordUndo {
            deck_id,
            action: recorded,
        }]
    );
    assert_eq!(
        state.entries(),
        Vec::<DeckEntry>::new(),
        "the removal still stands"
    );
}

#[test]
fn a_failed_undo_with_no_entry_records_nothing() {
    let mut state = loaded(deck_of(&["Opt"]));
    let _ = swipe_and_land(&mut state, Action::Remove);
    let ticket = ticket_of(&undo_with(&mut state, None));
    assert_eq!(state.update(Action::Failed { ticket }), NOTHING);
}

#[test]
fn undo_moves_a_card_back() {
    let built = deck_of(&["Opt", "Shock"]);
    let deck_id = built.deck_profile.id;
    let card_id = built.entries[0].card.scryfall_data.id;
    let mut state = loaded(built);
    let recorded = swipe_and_land(&mut state, Action::Move);

    let effects = state.update(Action::Undo);
    assert!(matches!(
        effects.as_slice(),
        [Effect::TakeUndo {
            claim: UndoClaim::MovedBoard {
                from: Board::Deck,
                ..
            },
            ..
        }]
    ));
    let effects = state.update(Action::Taken {
        ticket: ticket_of(&effects),
        taken: Some(recorded),
    });
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        [
            Effect::Enter(Direction::Up),
            Effect::RestoreBoard {
                ticket,
                deck_id,
                card_id,
                board: Board::Deck,
            },
        ]
    );
    assert_eq!(top(&state).as_deref(), Some("Opt"));
    assert_eq!(
        state.update(Action::Landed { ticket }),
        [Effect::Confirm(Toast::quick(
            ToastKind::Success,
            UNDID_MOVE
        ))]
    );
    assert_eq!(state.entries()[0].deck_card.board, Board::Deck);
}

#[test]
fn undo_of_a_move_already_moved_back_sends_nothing() {
    let mut state = loaded(deck_of(&["Opt", "Shock"]));
    let _ = swipe_and_land(&mut state, Action::Move);
    state.entries[0].deck_card.board = Board::Deck;
    assert_eq!(
        undo_with(&mut state, None),
        [Effect::Toast(Toast::quick(
            ToastKind::Info,
            ALREADY_CHANGED
        ))]
    );
}

#[test]
fn a_late_undo_reply_is_ignored() {
    let built = deck_of(&["Opt"]);
    let mut state = loaded(built.clone());
    let recorded = swipe_and_land(&mut state, Action::Remove);
    let ticket = ticket_of(&undo_with(&mut state, Some(recorded)));
    let _ = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: CardQueryBuilder::default(),
    });
    assert_eq!(
        state.update(Action::Restored {
            ticket,
            deck_card: Box::new(built.entries[0].deck_card.clone()),
        }),
        NOTHING
    );
    assert_eq!(state.update(Action::Failed { ticket }), NOTHING);
}

#[test]
fn a_late_answer_to_a_claim_is_ignored() {
    let built = deck_of(&["Opt"]);
    let mut state = loaded(built.clone());
    let _ = swipe_and_land(&mut state, Action::Remove);
    let ticket = ticket_of(&state.update(Action::Undo));
    let _ = state.update(Action::Load {
        deck_id: built.deck_profile.id,
        filter: CardQueryBuilder::default(),
    });
    assert_eq!(
        state.update(Action::Taken {
            ticket,
            taken: None
        }),
        NOTHING
    );
}

#[test]
fn refresh_rewinds_and_forgets_the_swipes() {
    let mut state = loaded(deck_of(&["Opt", "Shock"]));
    let _ = state.update(Action::Keep);
    assert_eq!(
        state.update(Action::Refresh),
        [Effect::Toast(Toast::quick(
            ToastKind::Info,
            STACK_REFRESHED
        ))]
    );
    assert_eq!(state.index(), 0);
    assert_eq!(
        state.update(Action::Undo),
        [Effect::Toast(Toast::normal(
            ToastKind::Info,
            NOTHING_TO_UNDO
        ))],
        "refresh drops the history"
    );
}

#[test]
fn refresh_on_a_random_sort_deals_again() {
    let mut state = loaded(deck_of(&["Opt", "Shock", "Duress"]));
    let mut random = CardQueryBuilder::default();
    random.set_sort(CardSortKey::Random);
    let _ = state.update(Action::Filter(random));
    let _ = state.update(Action::Remove);
    let _ = state.update(Action::Refresh);
    assert_eq!(state.cards().len(), 2, "the dealt stack holds what is left");
}

#[test]
fn back_goes_where_the_user_came_from() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn a_claim_matches_only_its_own_entry() {
    let opt = deck_of(&["Opt"]).entries.remove(0);
    let id = opt.card.scryfall_data.id;
    let removed = UndoAction::Removed {
        entry: Box::new(opt),
        baseline: 1,
    };
    let moved = UndoAction::MovedBoard {
        card_id: id,
        card_name: "Opt".to_owned(),
        from: Board::Deck,
    };
    assert!(UndoClaim::Removed { card_id: id }.matches(&removed));
    assert!(!UndoClaim::Removed { card_id: id }.matches(&moved));
    assert!(
        !UndoClaim::Removed {
            card_id: Uuid::new_v4()
        }
        .matches(&removed)
    );
    assert!(
        UndoClaim::MovedBoard {
            card_id: id,
            from: Board::Deck
        }
        .matches(&moved)
    );
    assert!(
        !UndoClaim::MovedBoard {
            card_id: id,
            from: Board::Maybeboard
        }
        .matches(&moved)
    );
}

#[test]
fn the_requests_put_the_card_back_where_it_was() {
    let opt = card("Opt");
    let on_side = restore_request(&opt, Board::Sideboard, 3);
    assert_eq!(
        (on_side.quantity, on_side.board.as_deref()),
        (3, Some("sideboard")),
        "all removed copies come back on their old board"
    );
    assert_eq!(
        restore_request(&opt, Board::Deck, 2).board,
        None,
        "the main deck is the server default"
    );
    assert_eq!(
        board_request(Board::Maybeboard).board,
        Patch::Set(Some("maybeboard".to_owned()))
    );
}
