use zwipe_core::domain::deck::{ImportedCard, UnresolvedCard};

use super::*;
use crate::toast::{NORMAL, QUICK};

/// No effects, for comparing against.
const NONE: [Effect; 0] = [];

fn opened() -> (State, Uuid) {
    let deck_id = Uuid::new_v4();
    let mut state = State::new();
    let _ = state.update(Action::Open(deck_id));
    (state, deck_id)
}

fn submit(state: &mut State, input: &str) -> Vec<Effect> {
    state.update(Action::Submit {
        input: input.to_owned(),
    })
}

/// Submits `input` and returns the import's ticket.
fn send(state: &mut State, input: &str) -> Ticket {
    match submit(state, input).as_slice() {
        [Effect::ImportDeckCards { ticket, .. } | Effect::ImportArchidektDeck { ticket, .. }] => {
            *ticket
        }
        other => panic!("expected one import, got {other:?}"),
    }
}

fn result(imported: &[&str], unresolved: &[&str]) -> ImportDeckCardsResult {
    ImportDeckCardsResult {
        imported: imported
            .iter()
            .map(|name| ImportedCard {
                name: (*name).to_owned(),
                quantity: 1,
            })
            .collect(),
        unresolved: unresolved
            .iter()
            .map(|name| UnresolvedCard {
                name: (*name).to_owned(),
                reason: "not found".to_owned(),
            })
            .collect(),
    }
}

fn names(state: &State) -> Option<(Vec<String>, Vec<String>)> {
    state.result().map(|r| {
        (
            r.imported.iter().map(|c| c.name.clone()).collect(),
            r.unresolved.iter().map(|c| c.name.clone()).collect(),
        )
    })
}

#[test]
fn the_screen_opens_on_text_add_and_main_with_nothing_sent() {
    let state = State::new();
    assert_eq!(state.deck_id(), None);
    assert_eq!(state.source(), Source::Text);
    assert_eq!(state.mode(), ImportMode::Add);
    assert_eq!(state.board(), Board::Main);
    assert!(!state.importing());
    assert!(state.result().is_none());
    assert_eq!(state.submit_label(), IMPORT);
}

#[test]
fn opening_offers_the_hint_once_per_visit() {
    let deck_id = Uuid::new_v4();
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Open(deck_id)),
        [Effect::OfferHint(OneTimeHint::Import)]
    );
    assert_eq!(state.deck_id(), Some(deck_id));
    assert_eq!(state.update(Action::Open(deck_id)), NONE);
}

#[test]
fn the_picks_change_what_shows() {
    let (mut state, _) = opened();
    assert_eq!(state.update(Action::PickSource(Source::Archidekt)), NONE);
    assert_eq!(state.update(Action::PickMode(ImportMode::Replace)), NONE);
    assert_eq!(state.update(Action::PickBoard(Board::Side)), NONE);
    assert_eq!(state.source(), Source::Archidekt);
    assert_eq!(state.mode(), ImportMode::Replace);
    assert_eq!(state.board(), Board::Side);
    assert_eq!(state.submit_label(), REPLACE);
}

#[test]
fn the_explainer_follows_the_mode_and_board() {
    let (mut state, _) = opened();
    assert_eq!(
        state.explainer(),
        Explainer {
            before: "Adds imported cards into the ",
            board: "mainboard",
            after: ". Existing cards stay but take new quantities if applicable.",
        }
    );
    let _ = state.update(Action::PickMode(ImportMode::Replace));
    let _ = state.update(Action::PickBoard(Board::Maybe));
    assert_eq!(
        state.explainer(),
        Explainer {
            before: "Removes all existing cards in the ",
            board: "maybeboard",
            after: ", replacing with imported cards.",
        }
    );
}

#[test]
fn main_sends_no_board() {
    let wires: Vec<Option<&str>> = Board::ALL.into_iter().map(Board::wire).collect();
    assert_eq!(wires, [None, Some("maybeboard"), Some("sideboard")]);
}

#[test]
fn a_text_import_sends_the_decklist_as_typed() {
    let (mut state, deck_id) = opened();
    let _ = state.update(Action::PickBoard(Board::Side));
    let effects = submit(&mut state, " 1 Sol Ring\n");
    let [
        Effect::ImportDeckCards {
            deck_id: sent,
            text,
            board,
            mode,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("expected a text import, got {effects:?}");
    };
    assert_eq!(
        (*sent, text.as_str(), *board, *mode),
        (deck_id, " 1 Sol Ring\n", Some("sideboard"), ImportMode::Add)
    );
    assert!(state.importing());
    assert_eq!(state.submit_label(), IMPORTING);
}

#[test]
fn an_archidekt_import_sends_the_link() {
    let (mut state, deck_id) = opened();
    let _ = state.update(Action::PickSource(Source::Archidekt));
    let _ = state.update(Action::PickMode(ImportMode::Replace));
    let effects = submit(&mut state, "https://archidekt.com/decks/1");
    let [
        Effect::ImportArchidektDeck {
            deck_id: sent,
            url,
            board,
            mode,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("expected an Archidekt import, got {effects:?}");
    };
    assert_eq!(
        (*sent, url.as_str(), *board, *mode),
        (
            deck_id,
            "https://archidekt.com/decks/1",
            None,
            ImportMode::Replace
        )
    );
}

#[test]
fn a_blank_input_sends_nothing() {
    let (mut state, _) = opened();
    assert!(!state.can_submit("  \n "));
    assert_eq!(submit(&mut state, "  \n "), NONE);
    assert!(!state.importing());
}

#[test]
fn nothing_sends_before_the_screen_opens() {
    let mut state = State::new();
    assert_eq!(submit(&mut state, "1 Sol Ring"), NONE);
    assert_eq!(state.update(Action::Back), NONE);
}

#[test]
fn a_second_submit_waits_for_the_first() {
    let (mut state, _) = opened();
    let _ = send(&mut state, "1 Sol Ring");
    assert!(!state.can_submit("1 Sol Ring"));
    assert_eq!(submit(&mut state, "1 Sol Ring"), NONE);
}

#[test]
fn an_import_with_cards_clears_undo_then_toasts_success() {
    let (mut state, deck_id) = opened();
    let ticket = send(&mut state, "2 Sol Ring");
    let effects = state.update(Action::Imported {
        ticket,
        result: result(&["Sol Ring", "Opt"], &[]),
    });
    assert_eq!(
        effects,
        [
            Effect::ClearUndo(deck_id),
            Effect::Toast(Toast::new(ToastKind::Success, "Imported 2 cards", QUICK)),
        ]
    );
    assert!(!state.importing());
    assert_eq!(
        names(&state),
        Some((vec!["Sol Ring".to_owned(), "Opt".to_owned()], Vec::new()))
    );
}

#[test]
fn some_unresolved_lines_toast_info() {
    let (mut state, deck_id) = opened();
    let ticket = send(&mut state, "x");
    let effects = state.update(Action::Imported {
        ticket,
        result: result(&["Opt"], &["Nope", "Nada"]),
    });
    assert_eq!(
        effects,
        [
            Effect::ClearUndo(deck_id),
            Effect::Toast(Toast::new(
                ToastKind::Info,
                "Imported 1, 2 unresolved",
                QUICK
            )),
        ]
    );
}

#[test]
fn only_unresolved_lines_toast_an_error_and_keep_undo() {
    let (mut state, _) = opened();
    let ticket = send(&mut state, "x");
    let effects = state.update(Action::Imported {
        ticket,
        result: result(&[], &["Nope"]),
    });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(
            ToastKind::Error,
            "1 card unresolved",
            NORMAL
        ))]
    );
    assert_eq!(names(&state), Some((Vec::new(), vec!["Nope".to_owned()])));
}

#[test]
fn an_empty_import_toasts_no_cards_found() {
    let (mut state, _) = opened();
    let ticket = send(&mut state, "// nothing");
    let effects = state.update(Action::Imported {
        ticket,
        result: result(&[], &[]),
    });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(
            ToastKind::Info,
            "No cards found",
            QUICK
        ))]
    );
}

#[test]
fn a_new_import_clears_the_last_result() {
    let (mut state, _) = opened();
    let ticket = send(&mut state, "x");
    let _ = state.update(Action::Imported {
        ticket,
        result: result(&["Opt"], &[]),
    });
    let _ = send(&mut state, "y");
    assert!(state.result().is_none());
}

#[test]
fn a_gone_deck_is_replaced_by_the_deck_list() {
    let (mut state, _) = opened();
    let ticket = send(&mut state, "x");
    assert_eq!(
        state.update(Action::Gone { ticket }),
        [Effect::Replace(Destination::DeckList)]
    );
    assert!(!state.importing());
}

#[test]
fn a_failed_import_leaves_no_result_and_takes_another_submit() {
    let (mut state, _) = opened();
    let ticket = send(&mut state, "x");
    assert_eq!(state.update(Action::Failed { ticket }), NONE);
    assert!(!state.importing());
    assert!(state.result().is_none());
    assert!(state.can_submit("x"));
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, _) = opened();
    let older = send(&mut state, "x");
    let _ = state.update(Action::Failed { ticket: older });
    let newer = send(&mut state, "y");
    let late = state.update(Action::Imported {
        ticket: older,
        result: result(&["Opt"], &[]),
    });
    assert_eq!(late, NONE);
    assert!(state.importing(), "the older reply ended the newer import");
    assert_eq!(state.update(Action::Failed { ticket: older }), NONE);
    assert!(state.importing());
    let _ = state.update(Action::Failed { ticket: newer });
    assert!(!state.importing());
}

#[test]
fn a_reply_after_it_landed_is_ignored() {
    let (mut state, _) = opened();
    let ticket = send(&mut state, "x");
    let _ = state.update(Action::Imported {
        ticket,
        result: result(&["Opt"], &[]),
    });
    assert_eq!(state.update(Action::Failed { ticket }), NONE);
    assert!(
        state.result().is_some(),
        "a repeat reply cleared the result"
    );
}

#[test]
fn back_goes_to_the_deck() {
    let (mut state, deck_id) = opened();
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Deck(deck_id))]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Import");
    assert_eq!((FROM, MODE, BOARD), ("From", "Mode", "Board"));
    assert_eq!(TEXT_LABEL, "Paste decklist");
    assert_eq!(
        TEXT_PLACEHOLDER,
        "1 Krenko, Mob Boss\n1 Urza, Lord High Artificer\n1 Sol Ring"
    );
    assert_eq!(URL_LABEL, "Paste an Archidekt deck URL");
    assert_eq!(URL_PLACEHOLDER, "https://archidekt.com/decks/...");
    assert_eq!((IMPORTED, UNRESOLVED), ("Imported", "Unresolved"));
    assert_eq!(BACK, "Back");
    assert_eq!(
        (IMPORT, REPLACE, IMPORTING),
        ("Import", "Replace", "Importing...")
    );
    let sources: Vec<&str> = Source::ALL.into_iter().map(Source::label).collect();
    assert_eq!(sources, ["Text", "Archidekt"]);
    let modes: Vec<&str> = MODES.into_iter().map(mode_label).collect();
    assert_eq!(modes, ["Add", "Replace"]);
    let boards: Vec<&str> = Board::ALL.into_iter().map(Board::label).collect();
    assert_eq!(boards, ["Main", "Maybe", "Side"]);
    let words: Vec<&str> = Board::ALL.into_iter().map(Board::word).collect();
    assert_eq!(words, ["mainboard", "maybeboard", "sideboard"]);
}
