use zwipe_core::domain::card::{Card, slug};

use super::*;
use crate::test_support::card;

fn cards(names: &[&str]) -> Vec<Card> {
    names.iter().map(|name| card(name)).collect()
}

/// Loads `query`, returning the lookup's ticket and what it asks for.
fn loading(state: &mut State, query: &str) -> (Ticket, HttpCardLookup) {
    let effects = state.update(Action::Load(query.to_owned()));
    let [Effect::Lookup { ticket, lookup }] = effects.as_slice() else {
        panic!("a load looks the text up: {effects:?}");
    };
    (*ticket, lookup.clone())
}

fn card_page(name: &str) -> Destination {
    Destination::Card(slug::card_slug(name).unwrap())
}

#[test]
fn a_load_looks_up_the_trimmed_text_and_shows_loading() {
    let mut state = State::new();
    let (_, lookup) = loading(&mut state, "  solemn ");
    assert_eq!(lookup.name().as_str(), "solemn");
    assert_eq!(state.query(), "solemn");
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.title(), "Cards matching “solemn”");
}

#[test]
fn several_matches_show_in_order() {
    let mut state = State::new();
    let (ticket, _) = loading(&mut state, "solemn");
    let effects = state.update(Action::Loaded {
        ticket,
        cards: cards(&["Solemn Simulacrum", "Solemn Offering"]),
    });
    assert_eq!(effects, []);
    assert_eq!(state.status(), Status::Several);
    let names: Vec<&str> = state
        .cards()
        .iter()
        .map(|c| c.scryfall_data.name.as_str())
        .collect();
    assert_eq!(names, ["Solemn Simulacrum", "Solemn Offering"]);
}

#[test]
fn one_match_opens_its_card_in_place_of_the_page() {
    let mut state = State::new();
    let (ticket, _) = loading(&mut state, "simulacrum");
    let effects = state.update(Action::Loaded {
        ticket,
        cards: cards(&["Solemn Simulacrum"]),
    });
    assert_eq!(effects, [Effect::Replace(card_page("Solemn Simulacrum"))]);
}

#[test]
fn no_match_says_so() {
    let mut state = State::new();
    let (ticket, _) = loading(&mut state, "zzqx");
    let _ = state.update(Action::Loaded {
        ticket,
        cards: Vec::new(),
    });
    assert_eq!(state.status(), Status::NoMatch);
    assert_eq!(state.cards().len(), 0);
}

#[test]
fn text_no_card_could_have_matches_nothing_without_asking() {
    let mut state = State::new();
    assert_eq!(state.update(Action::Load(" ?! ".to_owned())), []);
    assert_eq!(state.status(), Status::NoMatch);
    assert_eq!(state.update(Action::Load(String::new())), []);
    assert_eq!(state.status(), Status::NoMatch);
}

#[test]
fn a_stale_answer_is_dropped() {
    let mut state = State::new();
    let (old, _) = loading(&mut state, "solemn");
    let (new, _) = loading(&mut state, "goblin");
    let _ = state.update(Action::Loaded {
        ticket: old,
        cards: cards(&["Solemn Simulacrum", "Solemn Offering"]),
    });
    assert_eq!(state.status(), Status::Loading, "the old answer is dropped");
    let _ = state.update(Action::Loaded {
        ticket: new,
        cards: cards(&["Goblin Guide", "Goblin Bombardment"]),
    });
    assert_eq!(state.cards().len(), 2);
    assert_eq!(state.query(), "goblin");
}

#[test]
fn a_failure_offers_retry_which_asks_again() {
    let mut state = State::new();
    let (ticket, _) = loading(&mut state, "solemn");
    let _ = state.update(Action::Failed {
        ticket,
        failure: Failure::Other,
    });
    assert_eq!(state.status(), Status::Failed);
    let effects = state.update(Action::Retry);
    assert!(
        matches!(effects.as_slice(), [Effect::Lookup { lookup, .. }] if lookup.name().as_str() == "solemn"),
        "{effects:?}"
    );
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn rate_limited_says_so_and_retry_asks_again() {
    let mut state = State::new();
    let (ticket, _) = loading(&mut state, "solemn");
    let _ = state.update(Action::Failed {
        ticket,
        failure: Failure::RateLimited { retry_after: None },
    });
    assert_eq!(state.status(), Status::RateLimited);
    assert_eq!(state.update(Action::Retry).len(), 1);
}

#[test]
fn retry_does_nothing_unless_a_lookup_failed() {
    let mut state = State::new();
    let _ = loading(&mut state, "solemn");
    assert_eq!(state.update(Action::Retry), []);
}

#[test]
fn picking_a_card_opens_its_card_page() {
    let mut state = State::new();
    let (ticket, _) = loading(&mut state, "solemn");
    let found = cards(&["Solemn Simulacrum", "Solemn Offering"]);
    let id = found[1].scryfall_data.id;
    let _ = state.update(Action::Loaded {
        ticket,
        cards: found,
    });
    assert_eq!(
        state.update(Action::Pick(id)),
        [Effect::Navigate(card_page("Solemn Offering"))]
    );
    assert_eq!(state.update(Action::Pick(uuid::Uuid::new_v4())), []);
}
