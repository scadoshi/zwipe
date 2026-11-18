use std::time::Duration;

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::models::lookup::LOOKUP_LIMIT;

use super::*;
use crate::test_support::card;

/// Types `text` and answers the debounce, returning the lookup the model sends and its ticket.
fn typed(state: &mut State, text: &str) -> (Ticket, HttpCardLookup) {
    let effects = state.update(Action::Type(text.to_owned()));
    let [Effect::Debounce { ticket, wait }] = effects.as_slice() else {
        panic!("typing waits: {effects:?}");
    };
    assert_eq!(*wait, timing::CARD_SEARCH_DEBOUNCE);
    let ticket = *ticket;
    let effects = state.update(Action::Debounced(ticket));
    let [
        Effect::Lookup {
            ticket: sent,
            lookup,
        },
    ] = effects.as_slice()
    else {
        panic!("the rest sends the lookup: {effects:?}");
    };
    assert_eq!(*sent, ticket);
    (ticket, lookup.clone())
}

fn cards(names: &[&str]) -> Vec<Card> {
    names.iter().map(|name| card(name)).collect()
}

#[test]
fn it_starts_idle() {
    let state = State::new();
    assert_eq!(state.status(), Status::Idle);
    assert_eq!(state.results().len(), 0);
}

#[test]
fn typing_waits_then_looks_up_the_trimmed_name() {
    let mut state = State::new();
    let (_, lookup) = typed(&mut state, "  bolt ");
    assert_eq!(lookup.name().as_str(), "bolt");
    assert!(lookup.types().is_empty() && lookup.colors().is_empty());
    assert_eq!(lookup.card_roles(), []);
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn only_the_last_keystroke_s_wait_sends() {
    let mut state = State::new();
    let first = state.update(Action::Type("bo".to_owned()));
    let [Effect::Debounce { ticket: early, .. }] = first.as_slice() else {
        panic!("{first:?}");
    };
    let early = *early;
    let _ = state.update(Action::Type("bol".to_owned()));
    assert_eq!(
        state.update(Action::Debounced(early)),
        [],
        "a newer keystroke owns the wait"
    );
}

#[test]
fn an_answer_shows_its_cards_and_none_says_so() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "bolt");
    let _ = state.update(Action::Loaded {
        ticket,
        cards: cards(&["Lightning Bolt", "Bolt Bend"]),
    });
    assert_eq!(state.status(), Status::Results);
    assert_eq!(state.results().len(), 2);

    let (ticket, _) = typed(&mut state, "zzqx");
    let _ = state.update(Action::Loaded {
        ticket,
        cards: Vec::new(),
    });
    assert_eq!(state.status(), Status::NoMatch);
    assert_eq!(state.results().len(), 0);
}

#[test]
fn results_stop_at_the_lookup_limit() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "a");
    let many: Vec<Card> = (0..LOOKUP_LIMIT + 5)
        .map(|i| card(&format!("Card {i}")))
        .collect();
    let _ = state.update(Action::Loaded {
        ticket,
        cards: many,
    });
    assert_eq!(state.results().len(), 30);
}

#[test]
fn a_stale_answer_is_dropped() {
    let mut state = State::new();
    let (old, _) = typed(&mut state, "bo");
    let (new, _) = typed(&mut state, "bolt");
    let _ = state.update(Action::Loaded {
        ticket: old,
        cards: cards(&["Bog Wraith"]),
    });
    assert_eq!(
        state.status(),
        Status::Loading,
        "the old answer is not shown"
    );
    let _ = state.update(Action::Failed {
        ticket: old,
        failure: Failure::Other,
    });
    assert_eq!(state.status(), Status::Loading, "nor its failure");
    let _ = state.update(Action::Loaded {
        ticket: new,
        cards: cards(&["Lightning Bolt"]),
    });
    assert_eq!(state.status(), Status::Results);
}

#[test]
fn emptying_the_box_goes_idle_and_drops_what_is_out() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "bolt");
    assert_eq!(state.update(Action::Type("   ".to_owned())), []);
    assert_eq!(state.status(), Status::Idle);
    let _ = state.update(Action::Loaded {
        ticket,
        cards: cards(&["Lightning Bolt"]),
    });
    assert_eq!(state.status(), Status::Idle);
    assert_eq!(state.results().len(), 0);

    let (ticket, _) = typed(&mut state, "bolt");
    assert_eq!(state.update(Action::Clear), []);
    assert_eq!(state.query(), "");
    let _ = state.update(Action::Loaded {
        ticket,
        cards: cards(&["Lightning Bolt"]),
    });
    assert_eq!(state.status(), Status::Idle);
}

#[test]
fn a_name_no_card_could_have_matches_nothing_without_asking() {
    let mut state = State::new();
    let long = "a".repeat(101);
    assert_eq!(state.update(Action::Type(long)), []);
    assert_eq!(state.status(), Status::NoMatch);
}

#[test]
fn a_name_of_only_punctuation_matches_nothing_without_asking() {
    let mut state = State::new();
    assert_eq!(state.update(Action::Type("%%%".to_owned())), []);
    assert_eq!(state.status(), Status::NoMatch);
}

#[test]
fn a_failure_offers_retry_which_asks_again() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "bolt");
    assert_eq!(
        state.update(Action::Failed {
            ticket,
            failure: Failure::Other,
        }),
        []
    );
    assert_eq!(state.status(), Status::Failed);
    let effects = state.update(Action::Retry);
    let [Effect::Lookup { lookup, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(lookup.name().as_str(), "bolt");
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn rate_limited_waits_the_server_s_time_then_looks_up_the_latest_text() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "bol");
    let effects = state.update(Action::Failed {
        ticket,
        failure: Failure::RateLimited {
            retry_after: Some(Duration::from_secs(3)),
        },
    });
    let [Effect::Cooldown { ticket: cool, wait }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let cool = *cool;
    assert_eq!(*wait, Duration::from_secs(3));
    assert_eq!(state.status(), Status::RateLimited);

    assert_eq!(
        state.update(Action::Type("bolt".to_owned())),
        [],
        "typing during the wait sends nothing"
    );
    assert_eq!(state.status(), Status::RateLimited);

    let effects = state.update(Action::CooledDown(cool));
    let [Effect::Lookup { lookup, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(lookup.name().as_str(), "bolt", "the text as it stands");
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn rate_limited_without_a_wait_takes_the_fallback() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "bolt");
    let effects = state.update(Action::Failed {
        ticket,
        failure: Failure::RateLimited { retry_after: None },
    });
    let [Effect::Cooldown { wait, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(*wait, timing::CARD_SEARCH_RATE_LIMIT_WAIT);
}

#[test]
fn a_cooldown_with_the_box_emptied_goes_idle() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "bolt");
    let effects = state.update(Action::Failed {
        ticket,
        failure: Failure::RateLimited { retry_after: None },
    });
    let [Effect::Cooldown { ticket: cool, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let cool = *cool;
    assert_eq!(state.update(Action::Type(String::new())), []);
    assert_eq!(state.update(Action::CooledDown(cool)), []);
    assert_eq!(state.status(), Status::Idle);
}

#[test]
fn picking_a_result_opens_the_card_by_its_slug() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "fire");
    let fire_ice = card("Fire // Ice");
    let id = fire_ice.scryfall_data.id;
    let _ = state.update(Action::Loaded {
        ticket,
        cards: vec![card("Fireball"), fire_ice],
    });
    assert_eq!(
        state.update(Action::Pick(id)),
        [Effect::Navigate(Destination::Card(
            "fire-ice".parse().unwrap()
        ))]
    );
    assert_eq!(
        state.update(Action::Pick(uuid::Uuid::nil())),
        [],
        "a card not in the results"
    );
}

#[test]
fn a_name_with_no_slug_opens_its_printing() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "a");
    let odd = card("_____");
    let id = odd.scryfall_data.id;
    let _ = state.update(Action::Loaded {
        ticket,
        cards: vec![odd],
    });
    assert_eq!(
        state.update(Action::Pick(id)),
        [Effect::Navigate(Destination::Printing {
            path: slug::printing_path("_____", id),
        })]
    );
}

#[test]
fn text_typed_after_clear_during_a_cooldown_is_looked_up_when_it_ends() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "bol");
    let effects = state.update(Action::Failed {
        ticket,
        failure: Failure::RateLimited { retry_after: None },
    });
    let [Effect::Cooldown { ticket: cool, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let cool = *cool;
    assert_eq!(state.update(Action::Clear), []);
    assert_eq!(state.update(Action::Type("bolt".to_owned())), []);
    let effects = state.update(Action::CooledDown(cool));
    let [Effect::Lookup { lookup, .. }] = effects.as_slice() else {
        panic!("the box's text is looked up: {effects:?}");
    };
    assert_eq!(lookup.name().as_str(), "bolt");
    assert_eq!(state.status(), Status::Loading);
}
