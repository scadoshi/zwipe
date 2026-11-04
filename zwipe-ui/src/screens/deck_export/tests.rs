use zwipe_core::domain::deck::Board as DeckBoard;

use super::*;
use crate::test_support::{card, deck, deck_profile};
use crate::toast::QUICK;

/// No effects, for comparing against.
const NONE: [Effect; 0] = [];

/// A deck with Sol Ring in the deck, Opt in the maybeboard and Duress in the sideboard.
fn built() -> Deck {
    let mut built = deck(
        deck_profile("Krenko"),
        vec![card("Sol Ring"), card("Opt"), card("Duress")],
    );
    built.entries[1].deck_card.board = DeckBoard::Maybeboard;
    built.entries[2].deck_card.board = DeckBoard::Sideboard;
    built
}

/// A state after Load, with the fetch's ticket.
fn loading() -> (State, Uuid, Ticket) {
    let deck_id = Uuid::new_v4();
    let mut state = State::new();
    let effects = state.update(Action::Load(deck_id));
    let Some(Effect::FetchDeck { ticket, .. }) = effects.last() else {
        panic!("expected a fetch, got {effects:?}");
    };
    (state, deck_id, *ticket)
}

fn loaded() -> (State, Uuid) {
    let (mut state, deck_id, ticket) = loading();
    let _ = state.update(Action::Loaded {
        ticket,
        deck: Box::new(built()),
    });
    (state, deck_id)
}

#[test]
fn the_screen_opens_loading_with_only_the_deck_on() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.boards(), Boards::default());
    assert!(state.is_on(Board::Deck));
    assert!(!state.is_on(Board::Maybe));
    assert!(!state.is_on(Board::Side));
    assert_eq!(state.text(), None);
    assert_eq!(state.deck_id(), None);
}

#[test]
fn load_offers_the_hint_then_fetches() {
    let deck_id = Uuid::new_v4();
    let mut state = State::new();
    let effects = state.update(Action::Load(deck_id));
    let [
        Effect::OfferHint(OneTimeHint::Export),
        Effect::FetchDeck {
            deck_id: fetched, ..
        },
    ] = effects.as_slice()
    else {
        panic!("expected the hint then a fetch, got {effects:?}");
    };
    assert_eq!(*fetched, deck_id);
    assert_eq!(state.deck_id(), Some(deck_id));
}

#[test]
fn a_second_load_fetches_without_the_hint() {
    let (mut state, deck_id, _) = loading();
    let effects = state.update(Action::Load(deck_id));
    assert!(
        matches!(effects.as_slice(), [Effect::FetchDeck { .. }]),
        "got {effects:?}"
    );
}

#[test]
fn the_loaded_deck_gives_the_decklist() {
    let (state, _) = loaded();
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.text().as_deref(), Some("// Deck\n1 Sol Ring"));
}

#[test]
fn an_empty_deck_loads_with_an_empty_decklist() {
    let (mut state, _, ticket) = loading();
    let _ = state.update(Action::Loaded {
        ticket,
        deck: Box::new(deck(deck_profile("Empty"), Vec::new())),
    });
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.text().as_deref(), Some(""));
}

#[test]
fn a_gone_deck_is_replaced_by_the_deck_list() {
    let (mut state, _, ticket) = loading();
    assert_eq!(
        state.update(Action::Gone { ticket }),
        [Effect::Replace(Destination::DeckList)]
    );
    assert_eq!(state.status(), Status::Failed);
}

#[test]
fn a_failed_fetch_shows_the_failed_line() {
    let (mut state, _, ticket) = loading();
    assert_eq!(state.update(Action::Failed { ticket }), NONE);
    assert_eq!(state.status(), Status::Failed);
    assert_eq!(state.text(), None);
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, deck_id, older) = loading();
    let _ = state.update(Action::Load(deck_id));
    let _ = state.update(Action::Loaded {
        ticket: older,
        deck: Box::new(built()),
    });
    assert_eq!(
        state.status(),
        Status::Loading,
        "the older reply landed over the newer fetch"
    );
    let _ = state.update(Action::Failed { ticket: older });
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn the_toggles_change_the_decklist() {
    let (mut state, _) = loaded();
    assert_eq!(state.update(Action::Toggle(Board::Maybe)), NONE);
    assert_eq!(state.update(Action::Toggle(Board::Side)), NONE);
    assert_eq!(
        state.text().as_deref(),
        Some("// Deck\n1 Sol Ring\n\n// Maybeboard\n1 Opt\n\n// Sideboard\n1 Duress")
    );
    let _ = state.update(Action::Toggle(Board::Deck));
    assert!(!state.is_on(Board::Deck));
    assert_eq!(
        state.text().as_deref(),
        Some("\n// Maybeboard\n1 Opt\n\n// Sideboard\n1 Duress")
    );
}

#[test]
fn the_last_board_on_stays_on() {
    let (mut state, _) = loaded();
    let _ = state.update(Action::Toggle(Board::Deck));
    assert!(state.is_on(Board::Deck), "the only board turned off");
    let _ = state.update(Action::Toggle(Board::Side));
    let _ = state.update(Action::Toggle(Board::Deck));
    let _ = state.update(Action::Toggle(Board::Side));
    assert!(state.is_on(Board::Deck), "the deck came back");
    assert!(!state.is_on(Board::Side));
}

#[test]
fn the_toggles_work_before_the_deck_loads() {
    let (mut state, _, _) = loading();
    let _ = state.update(Action::Toggle(Board::Maybe));
    assert!(state.is_on(Board::Maybe));
}

#[test]
fn copy_copies_the_decklist_and_toasts() {
    let (mut state, _) = loaded();
    assert_eq!(
        state.update(Action::Copy),
        [Effect::Copy {
            text: "// Deck\n1 Sol Ring".to_owned(),
            done: Toast::new(ToastKind::Info, "Copied to clipboard", QUICK),
        }]
    );
}

#[test]
fn copy_does_nothing_before_the_deck_loads_or_after_it_fails() {
    let (mut state, _, ticket) = loading();
    assert_eq!(state.update(Action::Copy), NONE);
    let _ = state.update(Action::Failed { ticket });
    assert_eq!(state.update(Action::Copy), NONE);
}

#[test]
fn back_goes_to_the_deck() {
    let (mut state, deck_id) = loaded();
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Deck(deck_id))]
    );
    assert_eq!(State::new().update(Action::Back), NONE);
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!((TITLE, BOARDS), ("Export", "Export"));
    assert_eq!(DECKLIST, "Decklist");
    assert_eq!(LOAD_FAILED, "Could not load deck");
    assert_eq!((BACK, COPY), ("Back", "Copy"));
    assert_eq!(COPIED, "Copied to clipboard");
    let boards: Vec<&str> = Board::ALL.into_iter().map(Board::label).collect();
    assert_eq!(boards, ["Deck", "Maybe", "Side"]);
}
