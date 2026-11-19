use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionCursor, CollectionQuantity, CollectionTags, Finish,
};
use zwipe_core::http::contracts::collection::{
    CollectionCard, HttpCollectionExport, HttpCollectionPage, HttpSetCollectionCardReply,
};

use super::*;
use crate::test_support::card;

fn row(name: &str, copies: i32) -> CollectionCard {
    CollectionCard {
        card: card(name),
        finish: Finish::Nonfoil,
        condition: Some(CardCondition::NearMint),
        language: Some(CardLanguage::En),
        quantity: CollectionQuantity::new(copies).unwrap(),
        tags: CollectionTags::default(),
        added_at: None,
    }
}

/// The cursor of the page after `page`, opaque hex as the server sends it.
fn cursor(page: &str) -> CollectionCursor {
    CollectionCursor::new(page).unwrap()
}

fn page(cards: Vec<CollectionCard>, total: i64, next: Option<&str>) -> HttpCollectionPage {
    HttpCollectionPage {
        cards,
        total,
        next: next.map(cursor),
    }
}

/// Opens the screen and answers its first page with `cards`.
fn opened(cards: Vec<CollectionCard>, next: Option<&str>) -> State {
    let mut state = State::new();
    let effects = state.update(Action::Open);
    let [
        Effect::FetchPage {
            ticket,
            after: None,
            search,
        },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert!(search.is_empty());
    let total = i64::try_from(cards.len()).unwrap();
    let _ = state.update(Action::PageLoaded {
        ticket: *ticket,
        page: Box::new(page(cards, total, next)),
    });
    state
}

fn key_of(state: &State, name: &str) -> RowKey {
    state
        .rows()
        .iter()
        .find(|r| r.card.card.scryfall_data.name == name)
        .map(|r| r.key)
        .unwrap()
}

fn shown(state: &State, name: &str) -> Option<i32> {
    state
        .rows()
        .iter()
        .find(|r| r.card.card.scryfall_data.name == name)
        .map(|r| r.copies)
}

/// Steps `key` by `delta` and answers the flush wait, returning the write it sends.
fn stepped(state: &mut State, key: RowKey, delta: i32) -> (Ticket, HttpSetCollectionCard) {
    let effects = state.update(Action::Step { key, delta });
    let [Effect::FlushAfter { ticket, wait }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(*wait, timing::QUANTITY_FLUSH);
    let effects = state.update(Action::Flush(*ticket));
    let [Effect::SetCopies { ticket, set }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    (*ticket, set.clone())
}

#[test]
fn opening_loads_the_first_page_and_lists_it() {
    let state = opened(vec![row("Sol Ring", 2), row("Lightning Bolt", 4)], None);
    assert_eq!(state.status(), Status::List);
    assert_eq!(state.rows().len(), 2);
    assert_eq!(state.total(), 2);
}

#[test]
fn an_empty_collection_points_to_import() {
    let state = opened(Vec::new(), None);
    assert_eq!(state.status(), Status::Empty);
}

/// An import on the same screen changes the collection under the list, so the list reads it again
/// from the first page for the search it holds.
#[test]
fn an_import_reloads_the_list_from_the_first_page() {
    let mut state = opened(Vec::new(), None);
    assert_eq!(state.status(), Status::Empty);
    let effects = state.update(Action::Imported);
    let [
        Effect::FetchPage {
            ticket,
            after: None,
            search,
        },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert!(search.is_empty());
    assert_eq!(state.status(), Status::Loading);
    let _ = state.update(Action::PageLoaded {
        ticket: *ticket,
        page: Box::new(page(vec![row("Sol Ring", 4)], 79, None)),
    });
    assert_eq!(state.status(), Status::List);
    assert_eq!(state.total(), 79);
}

/// A page the list asked for before the import holds the collection as it was, so it doesn't land.
#[test]
fn a_page_from_before_an_import_is_dropped() {
    let mut state = opened(vec![row("Sol Ring", 1)], Some("a1"));
    let effects = state.update(Action::NearEnd);
    let [Effect::FetchPage { ticket: old, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let old = *old;
    let _ = state.update(Action::Imported);
    let _ = state.update(Action::PageLoaded {
        ticket: old,
        page: Box::new(page(vec![row("Old", 1)], 9, None)),
    });
    assert_eq!(state.rows().len(), 0, "the page from before doesn't land");
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn the_first_page_failing_offers_retry() {
    let mut state = State::new();
    let effects = state.update(Action::Open);
    let [Effect::FetchPage { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let _ = state.update(Action::PageFailed { ticket: *ticket });
    assert_eq!(state.status(), Status::Failed);
    let effects = state.update(Action::Retry);
    assert!(
        matches!(effects.as_slice(), [Effect::FetchPage { after: None, .. }]),
        "{effects:?}"
    );
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn nearing_the_end_asks_for_the_next_page_once() {
    let mut state = opened(vec![row("A", 1)], Some("a1"));
    let effects = state.update(Action::NearEnd);
    let [
        Effect::FetchPage {
            ticket,
            after: Some(after),
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert_eq!(*after, cursor("a1"), "the last page's next");
    let ticket = *ticket;
    assert_eq!(state.update(Action::NearEnd), [], "one page out at a time");
    let _ = state.update(Action::PageLoaded {
        ticket,
        page: Box::new(page(vec![row("B", 1)], 2, None)),
    });
    assert_eq!(state.rows().len(), 2);
    assert!(!state.has_more());
    assert_eq!(state.update(Action::NearEnd), [], "the last page is in");
}

#[test]
fn a_later_page_failing_keeps_the_list_and_tries_again_on_the_next_scroll() {
    let mut state = opened(vec![row("A", 1)], Some("a1"));
    let effects = state.update(Action::NearEnd);
    let [Effect::FetchPage { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let effects = state.update(Action::PageFailed { ticket: *ticket });
    assert!(
        matches!(effects.as_slice(), [Effect::Toast(_)]),
        "{effects:?}"
    );
    assert_eq!(state.status(), Status::List);
    assert!(matches!(
        state.update(Action::NearEnd).as_slice(),
        [Effect::FetchPage { after: Some(after), .. }] if *after == cursor("a1")
    ));
}

#[test]
fn typing_searches_after_a_rest_from_the_first_page() {
    let mut state = opened(vec![row("Sol Ring", 1)], Some("a1"));
    let effects = state.update(Action::Type("bolt".to_owned()));
    let [Effect::SearchAfter { ticket, wait }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(*wait, timing::QUICK_ADD_DEBOUNCE);
    let effects = state.update(Action::Search(*ticket));
    let [
        Effect::FetchPage {
            ticket,
            after: None,
            search,
        },
    ] = effects.as_slice()
    else {
        panic!("{effects:?}");
    };
    assert_eq!(search.as_str(), "bolt");
    assert_eq!(state.status(), Status::Loading);
    let _ = state.update(Action::PageLoaded {
        ticket: *ticket,
        page: Box::new(page(Vec::new(), 0, None)),
    });
    assert_eq!(
        state.status(),
        Status::NoMatch,
        "a search with nothing is not an empty collection"
    );
}

#[test]
fn a_page_for_an_older_search_is_dropped() {
    let mut state = opened(vec![row("Sol Ring", 1)], Some("a1"));
    let effects = state.update(Action::NearEnd);
    let [Effect::FetchPage { ticket: old, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let old = *old;
    let effects = state.update(Action::Type("bolt".to_owned()));
    let [Effect::SearchAfter { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let _ = state.update(Action::Search(*ticket));
    let _ = state.update(Action::PageLoaded {
        ticket: old,
        page: Box::new(page(vec![row("Old", 1)], 9, None)),
    });
    assert_eq!(state.rows().len(), 0, "the old search's page doesn't land");
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn only_the_last_keystroke_searches() {
    let mut state = opened(vec![row("Sol Ring", 1)], None);
    let effects = state.update(Action::Type("bo".to_owned()));
    let [Effect::SearchAfter { ticket: early, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let early = *early;
    let _ = state.update(Action::Type("bol".to_owned()));
    assert_eq!(state.update(Action::Search(early)), []);
}

#[test]
fn a_search_too_long_matches_nothing_without_asking() {
    let mut state = opened(vec![row("Sol Ring", 1)], None);
    let effects = state.update(Action::Type("a".repeat(101)));
    let [Effect::SearchAfter { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(state.update(Action::Search(*ticket)), []);
    assert_eq!(state.status(), Status::NoMatch);
}

#[test]
fn stepping_shows_the_count_at_once_and_saves_the_row_after_a_rest() {
    let mut state = opened(vec![row("Sol Ring", 2)], None);
    let key = key_of(&state, "Sol Ring");
    let first = state.update(Action::Step { key, delta: 1 });
    let [Effect::FlushAfter { ticket: early, .. }] = first.as_slice() else {
        panic!("{first:?}");
    };
    let early = *early;
    assert_eq!(shown(&state, "Sol Ring"), Some(3));
    let _ = state.update(Action::Step { key, delta: 1 });
    assert_eq!(shown(&state, "Sol Ring"), Some(4));
    assert_eq!(
        state.update(Action::Flush(early)),
        [],
        "a newer step owns the wait"
    );

    let (ticket, set) = stepped(&mut state, key, 1);
    assert_eq!(set.quantity.get(), 5, "the burst goes as one write");
    assert_eq!(set.finish, Finish::Nonfoil);
    assert_eq!(set.condition, Some(CardCondition::NearMint));
    assert_eq!(set.language, Some(CardLanguage::En));
    let mut saved = row("Sol Ring", 5);
    saved.card.scryfall_data.id = key.printing;
    let effects = state.update(Action::Saved {
        ticket,
        reply: Box::new(HttpSetCollectionCardReply { card: Some(saved) }),
    });
    assert_eq!(effects, [Effect::CollectionChanged]);
    assert_eq!(shown(&state, "Sol Ring"), Some(5));
}

#[test]
fn copies_stay_between_zero_and_the_most() {
    let mut state = opened(
        vec![row("Sol Ring", 1), row("Plains", CollectionQuantity::MAX)],
        None,
    );
    let sol = key_of(&state, "Sol Ring");
    let _ = state.update(Action::Step {
        key: sol,
        delta: -5,
    });
    assert_eq!(shown(&state, "Sol Ring"), Some(0));
    let plains = key_of(&state, "Plains");
    let _ = state.update(Action::Step {
        key: plains,
        delta: 1,
    });
    assert_eq!(shown(&state, "Plains"), Some(CollectionQuantity::MAX));
}

#[test]
fn zero_removes_the_row_once_saved_and_the_next_page_keeps_its_cursor() {
    let mut state = opened(vec![row("Sol Ring", 1), row("Bolt", 1)], Some("a1"));
    let key = key_of(&state, "Sol Ring");
    let (ticket, set) = stepped(&mut state, key, -1);
    assert_eq!(set.quantity.get(), 0);
    assert_eq!(state.rows().len(), 2, "shown at 0 until the write lands");
    let effects = state.update(Action::Saved {
        ticket,
        reply: Box::new(HttpSetCollectionCardReply { card: None }),
    });
    assert_eq!(effects, [Effect::CollectionChanged]);
    assert_eq!(state.rows().len(), 1);
    assert_eq!(state.total(), 1);
    assert!(
        matches!(
            state.update(Action::NearEnd).as_slice(),
            [Effect::FetchPage { after: Some(after), .. }] if *after == cursor("a1")
        ),
        "a removal doesn't move the cursor"
    );
}

#[test]
fn a_failed_write_puts_the_saved_count_back() {
    let mut state = opened(vec![row("Sol Ring", 2)], None);
    let key = key_of(&state, "Sol Ring");
    let (ticket, _) = stepped(&mut state, key, -2);
    let effects = state.update(Action::SaveFailed { ticket });
    assert!(
        matches!(effects.as_slice(), [Effect::Toast(_)]),
        "{effects:?}"
    );
    assert_eq!(shown(&state, "Sol Ring"), Some(2));
    assert_eq!(state.rows().len(), 1);
}

#[test]
fn a_step_during_a_write_is_sent_when_it_lands() {
    let mut state = opened(vec![row("Sol Ring", 2)], None);
    let key = key_of(&state, "Sol Ring");
    let (ticket, _) = stepped(&mut state, key, 1);
    let effects = state.update(Action::Step { key, delta: 1 });
    let [Effect::FlushAfter { ticket: wait, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(
        state.update(Action::Flush(*wait)),
        [],
        "one write per row at a time"
    );
    let mut saved = row("Sol Ring", 3);
    saved.card.scryfall_data.id = key.printing;
    let effects = state.update(Action::Saved {
        ticket,
        reply: Box::new(HttpSetCollectionCardReply { card: Some(saved) }),
    });
    let [Effect::CollectionChanged, Effect::SetCopies { set, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(set.quantity.get(), 4);
    assert_eq!(shown(&state, "Sol Ring"), Some(4));
}

#[test]
fn an_edit_waiting_to_save_survives_a_new_search() {
    let mut state = opened(vec![row("Sol Ring", 2)], None);
    let key = key_of(&state, "Sol Ring");
    let effects = state.update(Action::Step { key, delta: 1 });
    let [Effect::FlushAfter { ticket: wait, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let wait = *wait;
    let effects = state.update(Action::Type("bolt".to_owned()));
    let [Effect::SearchAfter { ticket, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let _ = state.update(Action::Search(*ticket));
    let effects = state.update(Action::Flush(wait));
    let [Effect::SetCopies { set, .. }] = effects.as_slice() else {
        panic!("the edit still saves: {effects:?}");
    };
    assert_eq!(set.quantity.get(), 3);
}

#[test]
fn a_row_with_a_value_this_build_doesn_t_know_is_not_edited() {
    let mut odd = row("Sol Ring", 2);
    odd.language = Some(CardLanguage::Unknown);
    let mut state = opened(vec![odd, row("Bolt", 1)], None);
    let key = key_of(&state, "Sol Ring");
    assert!(!state.rows()[0].editable);
    assert_eq!(state.update(Action::Step { key, delta: 1 }), []);
    assert_eq!(shown(&state, "Sol Ring"), Some(2));
    assert!(state.rows()[1].editable);
}

#[test]
fn export_hands_the_file_to_the_client_once_at_a_time() {
    let mut state = opened(vec![row("Sol Ring", 1)], None);
    let effects = state.update(Action::Export);
    let [Effect::Export { ticket }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let ticket = *ticket;
    assert!(state.exporting());
    assert_eq!(state.update(Action::Export), [], "one export out at a time");
    let effects = state.update(Action::Exported {
        ticket,
        export: HttpCollectionExport {
            file_name: "zwipe-collection.csv".to_owned(),
            csv: "Name\nSol Ring\n".to_owned(),
        },
    });
    // The file alone: the client says it went once it has, since a share sheet opening is not
    // a file saved.
    let [Effect::Download { file_name, text }] = effects.as_slice() else {
        panic!("the file alone: {effects:?}");
    };
    assert_eq!(file_name, "zwipe-collection.csv");
    assert_eq!(text, "Name\nSol Ring\n");
    assert!(!state.exporting());
}

#[test]
fn a_failed_export_toasts() {
    let mut state = opened(vec![row("Sol Ring", 1)], None);
    let effects = state.update(Action::Export);
    let [Effect::Export { ticket }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let effects = state.update(Action::ExportFailed { ticket: *ticket });
    assert!(
        matches!(effects.as_slice(), [Effect::Toast(_)]),
        "{effects:?}"
    );
    assert!(!state.exporting());
}

/// A row removed while the next page is out: the page lands as sent, since a cursor doesn't move
/// with a removal, and a row already listed isn't listed twice.
#[test]
fn a_removal_while_a_page_is_out_lands_that_page_without_repeating_a_row() {
    let mut state = opened(vec![row("A", 1), row("B", 1)], Some("a1"));
    let key = key_of(&state, "A");
    let (write, _) = stepped(&mut state, key, -1);
    let effects = state.update(Action::NearEnd);
    let [Effect::FetchPage { ticket: out, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let out = *out;
    let _ = state.update(Action::Saved {
        ticket: write,
        reply: Box::new(HttpSetCollectionCardReply { card: None }),
    });
    let b = state.rows()[0].card.clone();
    let effects = state.update(Action::PageLoaded {
        ticket: out,
        page: Box::new(page(vec![b, row("C", 1)], 2, None)),
    });
    assert_eq!(effects, [], "nothing asked again");
    let names: Vec<&str> = state
        .rows()
        .iter()
        .map(|r| r.card.card.scryfall_data.name.as_str())
        .collect();
    assert_eq!(names, ["B", "C"], "a row already listed isn't listed twice");
}

/// A failed write restores the saved count, then replays the steps made while it was out.
#[test]
fn a_failed_write_keeps_the_step_queued_behind_it() {
    let mut state = opened(vec![row("Sol Ring", 2)], None);
    let key = key_of(&state, "Sol Ring");
    let (ticket, _) = stepped(&mut state, key, 1);
    let effects = state.update(Action::Step { key, delta: 1 });
    let [Effect::FlushAfter { ticket: wait, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(
        state.update(Action::Flush(*wait)),
        [],
        "waits on the write out"
    );
    assert_eq!(shown(&state, "Sol Ring"), Some(4));

    let effects = state.update(Action::SaveFailed { ticket });
    let [Effect::Toast(_), Effect::SetCopies { set, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(
        set.quantity.get(),
        3,
        "the saved 2 plus the one step made since"
    );
    assert_eq!(shown(&state, "Sol Ring"), Some(3));
}

/// A failed write with a step still resting leaves the send to that step's flush.
#[test]
fn a_failed_write_with_a_step_still_resting_sends_on_its_flush() {
    let mut state = opened(vec![row("Sol Ring", 2)], None);
    let key = key_of(&state, "Sol Ring");
    let (ticket, _) = stepped(&mut state, key, 1);
    let effects = state.update(Action::Step { key, delta: 1 });
    let [Effect::FlushAfter { ticket: wait, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let wait = *wait;
    let effects = state.update(Action::SaveFailed { ticket });
    assert!(
        matches!(effects.as_slice(), [Effect::Toast(_)]),
        "{effects:?}"
    );
    assert_eq!(shown(&state, "Sol Ring"), Some(3));
    let effects = state.update(Action::Flush(wait));
    let [Effect::SetCopies { set, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(set.quantity.get(), 3);
}

#[test]
fn a_row_id_is_the_printing_s_for_a_plain_row_and_differs_for_each_other_row_of_it() {
    let printing = uuid::Uuid::from_u128(0x1234);
    let key = |finish, condition, language| RowKey {
        printing,
        finish,
        condition,
        language,
    };
    let plain = key(Finish::Nonfoil, None, None);
    let rows = [
        plain,
        key(Finish::Foil, None, None),
        key(Finish::Nonfoil, Some(CardCondition::NearMint), None),
        key(Finish::Nonfoil, None, Some(CardLanguage::Ja)),
        key(
            Finish::Etched,
            Some(CardCondition::Played),
            Some(CardLanguage::De),
        ),
    ];
    let ids: std::collections::HashSet<uuid::Uuid> = rows.iter().map(|k| k.id()).collect();
    assert_eq!(
        ids.len(),
        rows.len(),
        "every row of one printing has its own id"
    );
    assert_eq!(
        plain.id(),
        printing,
        "a nonfoil row with nothing given is the printing's id"
    );
    assert_eq!(rows[1].id(), rows[1].id(), "the same row, the same id");
}

#[test]
fn a_row_s_tags_lead_with_its_count_then_what_sets_it_apart() {
    let mut foil = row("Sol Ring", 4);
    foil.finish = Finish::Foil;
    foil.condition = Some(CardCondition::LightlyPlayed);
    foil.language = Some(CardLanguage::Ja);
    let state = opened(vec![foil, row("Bolt", 2)], None);
    assert_eq!(row_tags(&state.rows()[0]), ["4", "Foil", "LP", "JA"]);
    assert_eq!(
        row_tags(&state.rows()[1]),
        ["2", "NM"],
        "nonfoil and English left out"
    );
}

#[test]
fn every_row_s_drawer_label_says_what_sets_it_apart() {
    let mut foil = row("Sol Ring", 3);
    foil.finish = Finish::Foil;
    foil.condition = Some(CardCondition::LightlyPlayed);
    let mut bare = row("Sol Ring", 1);
    bare.condition = None;
    let mut unknown = row("Sol Ring", 1);
    unknown.finish = Finish::Unknown;
    unknown.condition = None;
    let state = opened(vec![foil, row("Bolt", 2), bare, unknown], None);
    let labels: Vec<String> = state.rows().iter().map(row_label).collect();
    assert_eq!(labels, ["Foil LP", "NM", "Nonfoil", "Other finish"]);
}
