use std::collections::HashSet;
use std::hash::RandomState;

use super::*;
use crate::test_support::card;

/// No effects.
const NONE: [Effect; 0] = [];

/// No printings.
const NO_CARDS: &[Card] = &[];

/// Three printings of one card, all sharing the first one's oracle id.
fn printings() -> Vec<Card> {
    let first = card("Sol Ring");
    let oracle_id = first.scryfall_data.oracle_id;
    let mut all = vec![first, card("Sol Ring"), card("Sol Ring")];
    for printing in &mut all {
        printing.scryfall_data.oracle_id = oracle_id;
    }
    all
}

fn open(state: &mut State, card: &Card) -> Vec<Effect> {
    state.update(Action::Open {
        printing_id: card.scryfall_data.id,
        oracle_id: card.scryfall_data.oracle_id,
    })
}

fn ticket_of(effects: &[Effect]) -> Ticket {
    match effects {
        [Effect::FetchPrintings { ticket, .. }] => *ticket,
        other => panic!("expected one fetch, got {other:?}"),
    }
}

/// A sheet opened on `all[at]` with every printing loaded.
fn loaded_on(mode: Mode, all: &[Card], at: usize) -> State {
    let mut state = State::new(mode, SAVED);
    let ticket = ticket_of(&open(&mut state, &all[at]));
    let effects = state.update(Action::Loaded {
        ticket,
        printings: all.to_vec(),
    });
    assert_eq!(effects, [Effect::Show(at)]);
    state
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Printings");
    assert_eq!(SAVED, "Printing saved");
    assert_eq!(UNCHANGED, "Printing unchanged");
    assert_eq!(BACK, "Back");
    assert_eq!(SAVE, "Save");
    assert_eq!(LOAD_FAILED, "Couldn't load printings");
}

#[test]
fn a_new_sheet_is_empty_and_not_loading() {
    let state = State::new(Mode::Pick, SAVED);
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.printings(), NO_CARDS);
    assert!(state.shows_save());
    assert!(!state.can_save(0));
    assert_eq!(State::default(), state);
}

#[test]
fn opening_fetches_the_cards_printings_and_loads() {
    let all = printings();
    let mut state = State::default();
    let effects = open(&mut state, &all[1]);
    assert!(matches!(
        effects.as_slice(),
        [Effect::FetchPrintings { oracle_id, .. }] if Some(*oracle_id) == all[1].scryfall_data.oracle_id
    ));
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn a_card_without_an_oracle_id_loads_nothing() {
    let mut lone = card("Token");
    lone.scryfall_data.oracle_id = None;
    let mut state = State::default();
    assert_eq!(open(&mut state, &lone), NONE);
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn loaded_shows_the_cards_own_printing() {
    let all = printings();
    let state = loaded_on(Mode::Pick, &all, 2);
    assert_eq!(state.status(), Status::Several);
    assert_eq!(state.initial(), 2);
    assert_eq!(state.visible(2), Some(&all[2]));
    assert!(!state.has_changed(2));
}

#[test]
fn a_list_without_the_cards_printing_opens_on_the_first() {
    let all = printings();
    let mut state = State::default();
    let mut elsewhere = all[0].clone();
    elsewhere.scryfall_data.id = Uuid::new_v4();
    let ticket = ticket_of(&open(&mut state, &elsewhere));
    let effects = state.update(Action::Loaded {
        ticket,
        printings: all,
    });
    assert_eq!(effects, [Effect::Show(0)]);
    assert_eq!(state.initial(), 0);
}

#[test]
fn one_printing_is_single() {
    let all = printings();
    let state = loaded_on(Mode::Pick, &all[..1], 0);
    assert_eq!(state.status(), Status::Single);
}

#[test]
fn an_empty_reply_is_empty() {
    let all = printings();
    let mut state = State::default();
    let ticket = ticket_of(&open(&mut state, &all[0]));
    let effects = state.update(Action::Loaded {
        ticket,
        printings: Vec::new(),
    });
    assert_eq!(effects, [Effect::Show(0)]);
    assert_eq!(state.status(), Status::Empty);
    assert!(
        !state.has_changed(1),
        "an empty list never counts as changed"
    );
}

#[test]
fn a_failed_fetch_says_so_instead_of_sitting_empty() {
    let all = printings();
    let mut state = State::default();
    let ticket = ticket_of(&open(&mut state, &all[0]));
    assert_eq!(state.update(Action::Failed { ticket }), NONE);
    assert_eq!(state.status(), Status::Failed);
}

#[test]
fn retry_sends_the_failed_fetch_again_and_the_sheet_recovers() {
    let all = printings();
    let mut state = State::default();
    let first = ticket_of(&open(&mut state, &all[1]));
    assert_eq!(state.update(Action::Failed { ticket: first }), NONE);

    let effects = state.update(Action::Retry);
    let again = ticket_of(&effects);
    assert_ne!(again, first, "a fresh ticket");
    assert!(
        matches!(&effects[..], [Effect::FetchPrintings { oracle_id, .. }] if Some(*oracle_id) == all[1].scryfall_data.oracle_id),
        "{effects:?}"
    );
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(
        state.update(Action::Failed { ticket: first }),
        NONE,
        "the first fetch answering late changes nothing"
    );
    assert_eq!(state.status(), Status::Loading);

    let effects = state.update(Action::Loaded {
        ticket: again,
        printings: all.clone(),
    });
    assert_eq!(
        effects,
        [Effect::Show(1)],
        "opens on the card's own printing"
    );
    assert_eq!(state.status(), Status::Several);
    assert_eq!(state.update(Action::Retry), NONE, "nothing failed");
}

#[test]
fn a_retry_can_fail_again_and_be_retried() {
    let all = printings();
    let mut state = State::default();
    let ticket = ticket_of(&open(&mut state, &all[0]));
    assert_eq!(state.update(Action::Failed { ticket }), NONE);
    let ticket = ticket_of(&state.update(Action::Retry));
    assert_eq!(state.update(Action::Failed { ticket }), NONE);
    assert_eq!(state.status(), Status::Failed);
    assert!(matches!(
        state.update(Action::Retry)[..],
        [Effect::FetchPrintings { .. }]
    ));
}

#[test]
fn opening_another_card_after_a_failure_fetches_that_card() {
    let all = printings();
    let mut state = State::default();
    let ticket = ticket_of(&open(&mut state, &all[0]));
    assert_eq!(state.update(Action::Failed { ticket }), NONE);
    let other = card("Arcane Signet");
    let effects = open(&mut state, &other);
    assert!(
        matches!(&effects[..], [Effect::FetchPrintings { oracle_id, .. }] if Some(*oracle_id) == other.scryfall_data.oracle_id),
        "{effects:?}"
    );
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn a_new_sheet_retries_nothing() {
    let mut state = State::default();
    assert_eq!(state.update(Action::Retry), NONE);
    assert_eq!(state.status(), Status::Empty);
}

#[test]
fn reopening_on_the_same_card_rewinds_without_a_fetch() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 1);
    assert_eq!(open(&mut state, &all[1]), [Effect::Rewind(1)]);
    assert_eq!(state.status(), Status::Several);
}

#[test]
fn reopening_after_a_printing_swap_rewinds_to_the_new_one() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 0);
    assert_eq!(open(&mut state, &all[2]), [Effect::Rewind(2)]);
    assert_eq!(state.initial(), 2);
}

#[test]
fn opening_on_another_card_fetches_again() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 0);
    let other = card("Arcane Signet");
    let ticket = ticket_of(&open(&mut state, &other));
    assert_eq!(state.status(), Status::Loading);
    let effects = state.update(Action::Loaded {
        ticket,
        printings: vec![other.clone()],
    });
    assert_eq!(effects, [Effect::Show(0)]);
    assert_eq!(state.printings(), [other]);
}

#[test]
fn a_late_reply_is_ignored() {
    let all = printings();
    let mut state = State::default();
    let older = ticket_of(&open(&mut state, &all[0]));
    let newer = ticket_of(&open(&mut state, &all[0]));
    assert_eq!(
        state.update(Action::Loaded {
            ticket: older,
            printings: vec![all[2].clone()],
        }),
        NONE
    );
    assert_eq!(state.update(Action::Failed { ticket: older }), NONE);
    assert_eq!(state.status(), Status::Loading);
    let effects = state.update(Action::Loaded {
        ticket: newer,
        printings: all.clone(),
    });
    assert_eq!(effects, [Effect::Show(0)]);
    assert_eq!(state.printings(), all.as_slice());
}

#[test]
fn back_on_the_opening_printing_closes_quietly() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 1);
    assert_eq!(state.update(Action::Back { at: 1 }), [Effect::Close]);
    assert_eq!(state.update(Action::Dismiss { at: 1 }), [Effect::Close]);
}

#[test]
fn back_on_another_printing_toasts_unchanged_as_info() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 1);
    assert_eq!(
        state.update(Action::Back { at: 0 }),
        [
            Effect::Toast(Toast::quick(ToastKind::Info, UNCHANGED)),
            Effect::Close
        ]
    );
}

#[test]
fn dismissing_on_another_printing_toasts_unchanged_as_a_warning() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 1);
    assert_eq!(
        state.update(Action::Dismiss { at: 2 }),
        [
            Effect::Toast(Toast::quick(ToastKind::Warning, UNCHANGED)),
            Effect::Close
        ]
    );
}

#[test]
fn read_only_never_toasts_or_saves() {
    let all = printings();
    let mut state = loaded_on(Mode::ReadOnly, &all, 0);
    assert!(!state.shows_save());
    assert!(!state.can_save(1));
    assert_eq!(state.update(Action::Back { at: 1 }), [Effect::Close]);
    assert_eq!(state.update(Action::Dismiss { at: 1 }), [Effect::Close]);
    assert_eq!(state.update(Action::Save { at: 1 }), NONE);
}

#[test]
fn save_picks_the_showing_printing_toasts_and_closes() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 0);
    assert!(state.can_save(2));
    assert_eq!(
        state.update(Action::Save { at: 2 }),
        [
            Effect::Pick(Box::new(all[2].clone())),
            Effect::Toast(Toast::quick(ToastKind::Info, SAVED)),
            Effect::Close
        ]
    );
}

#[test]
fn save_toasts_the_hosts_own_message() {
    let all = printings();
    let mut state = State::new(Mode::Pick, "Printing swapped");
    let ticket = ticket_of(&open(&mut state, &all[0]));
    let _ = state.update(Action::Loaded {
        ticket,
        printings: all.clone(),
    });
    let effects = state.update(Action::Save { at: 1 });
    assert_eq!(
        effects.get(1),
        Some(&Effect::Toast(Toast::quick(
            ToastKind::Info,
            "Printing swapped"
        )))
    );
}

#[test]
fn save_does_nothing_on_the_opening_printing_or_past_the_list() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 0);
    assert!(!state.can_save(0));
    assert_eq!(state.update(Action::Save { at: 0 }), NONE);
    assert!(!state.can_save(9));
    assert_eq!(state.update(Action::Save { at: 9 }), NONE);
}

#[test]
fn a_screen_reader_hears_whose_printings_these_are() {
    assert_eq!(label("Sol Ring"), "Printings of Sol Ring");
}

/// `card` with no image URL anywhere: a printing whose image is gone, which shows as its stock
/// card.
fn imageless(mut card: Card) -> Card {
    card.scryfall_data.image_uris = None;
    card.scryfall_data.card_faces = None;
    card
}

#[test]
fn printings_with_no_image_drop_out_of_the_choice() {
    let mut all = printings();
    all[1] = imageless(all[1].clone());
    let shown = shown_printings(&all, all[0].scryfall_data.id);
    let ids: Vec<Uuid> = shown.iter().map(|c| c.scryfall_data.id).collect();
    assert_eq!(
        ids,
        [all[0].scryfall_data.id, all[2].scryfall_data.id],
        "in their order, the imageless one gone"
    );
}

#[test]
fn the_printing_a_deck_has_stays_even_with_no_image() {
    let mut all = printings();
    all[1] = imageless(all[1].clone());
    let shown = shown_printings(&all, all[1].scryfall_data.id);
    assert_eq!(shown.len(), 3, "the selected stock card stays where it was");
    assert_eq!(shown[1].scryfall_data.id, all[1].scryfall_data.id);
}

#[test]
fn a_card_with_no_image_on_any_printing_keeps_them_all() {
    let all: Vec<Card> = printings().into_iter().map(imageless).collect();
    assert_eq!(shown_printings(&all, all[0].scryfall_data.id).len(), 3);
}

#[test]
fn the_sheet_hides_imageless_printings_but_the_one_it_opened_on_even_from_its_cache() {
    let mut all = printings();
    all[2] = imageless(all[2].clone());
    let mut state = loaded_on(Mode::Pick, &all, 0);
    assert_eq!(state.printings().len(), 2, "the stock card hidden");
    // Reopened on the stock card itself (the deck's printing), from the cache.
    let effects = open(&mut state, &all[2]);
    assert_eq!(
        effects,
        vec![Effect::Rewind(2)],
        "it opens on the stock card"
    );
    assert_eq!(state.printings().len(), 3, "which shows, being the deck's");
}

/// The ids of `all` at `at`, as a set of owned printings.
fn owned_of(all: &[Card], at: &[usize]) -> Vec<Uuid> {
    at.iter().map(|i| all[*i].scryfall_data.id).collect()
}

/// The ids the sheet lists.
fn listed_ids(state: &State) -> Vec<Uuid> {
    state
        .printings()
        .iter()
        .map(|c| c.scryfall_data.id)
        .collect()
}

#[test]
fn only_mine_is_offered_once_an_owned_printing_is_known_and_lists_just_those() {
    let all = printings();
    let oracle_id = all[0].scryfall_data.oracle_id.unwrap();
    let mut state = loaded_on(Mode::Pick, &all, 0);
    assert!(!state.offers_only_mine(), "nothing known yet");
    assert_eq!(
        state.update(Action::OnlyMine { on: true, at: 0 }),
        NONE,
        "not offered: no change"
    );
    assert_eq!(state.printings().len(), 3);

    let owned = owned_of(&all, &[1, 2]);
    assert_eq!(
        state.update(Action::OwnedLoaded {
            oracle_id,
            owned: owned.clone()
        }),
        NONE
    );
    assert!(state.offers_only_mine());
    assert!(!state.only_mine(), "offered, but off until tapped");

    // On, from the opening printing (not owned): the list narrows and shows its first.
    assert_eq!(
        state.update(Action::OnlyMine { on: true, at: 0 }),
        [Effect::Show(0)]
    );
    assert!(state.only_mine());
    assert_eq!(listed_ids(&state), owned);
    assert!(
        state.can_save(0),
        "an owned printing other than the deck's saves"
    );

    // Off again, from the second owned printing: the full list, on that same printing.
    assert_eq!(
        state.update(Action::OnlyMine { on: false, at: 1 }),
        [Effect::Show(2)]
    );
    assert_eq!(state.printings().len(), 3);
    assert!(!state.can_save(0), "the opening printing is unchanged");
}

#[test]
fn owned_printings_of_another_card_or_none_listed_never_offer_it() {
    let all = printings();
    let mut state = loaded_on(Mode::Pick, &all, 0);
    let elsewhere = card("Lightning Bolt");
    let _ = state.update(Action::OwnedLoaded {
        oracle_id: elsewhere.scryfall_data.oracle_id.unwrap(),
        owned: vec![all[1].scryfall_data.id],
    });
    assert!(
        !state.offers_only_mine(),
        "another card's owned ids never count"
    );
    let _ = state.update(Action::OwnedLoaded {
        oracle_id: all[0].scryfall_data.oracle_id.unwrap(),
        owned: vec![Uuid::nil()],
    });
    assert!(
        !state.offers_only_mine(),
        "owning none of the listed printings offers nothing"
    );
}

#[test]
fn reopening_starts_with_only_mine_off() {
    let all = printings();
    let oracle_id = all[0].scryfall_data.oracle_id.unwrap();
    let mut state = loaded_on(Mode::Pick, &all, 0);
    let _ = state.update(Action::OwnedLoaded {
        oracle_id,
        owned: owned_of(&all, &[1]),
    });
    let _ = state.update(Action::OnlyMine { on: true, at: 0 });
    let _ = open(&mut state, &all[0]);
    assert!(!state.only_mine());
    assert_eq!(state.printings().len(), 3);
}

#[test]
fn the_pure_listing_narrows_only_with_owned_ids() {
    let all = printings();
    let keep = all[0].scryfall_data.id;
    assert_eq!(listed::<RandomState>(&all, keep, None).len(), 3);
    let owned: HashSet<Uuid> = owned_of(&all, &[2]).into_iter().collect();
    let mine = listed(&all, keep, Some(&owned));
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].scryfall_data.id, all[2].scryfall_data.id);
    assert!(offers_only_mine(&all, keep, Some(&owned)));
    assert!(!offers_only_mine::<RandomState>(&all, keep, None));
}
