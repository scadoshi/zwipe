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
fn the_empty_box_reads_card_search() {
    assert_eq!(PLACEHOLDER, "Card search");
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
fn the_first_ten_of_an_answer_show_and_enter_opens_them_all() {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, "a");
    let many: Vec<Card> = (0..LOOKUP_LIMIT + 5)
        .map(|i| card(&format!("Card {i}")))
        .collect();
    let _ = state.update(Action::Loaded {
        ticket,
        cards: many,
    });
    let shown: Vec<&str> = state
        .results()
        .iter()
        .map(|c| c.scryfall_data.name.as_str())
        .collect();
    let first_ten: Vec<String> = (0..10).map(|i| format!("Card {i}")).collect();
    assert_eq!(
        shown, first_ten,
        "the chips are the answer's first ten, in order"
    );
    assert_eq!(CHIP_LIMIT, 10);
    assert_eq!(state.update(Action::Enter), opens_results("a"));
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

/// A bar holding the answer `names` for `text`.
fn answered(text: &str, names: &[&str]) -> State {
    let mut state = State::new();
    let (ticket, _) = typed(&mut state, text);
    let _ = state.update(Action::Loaded {
        ticket,
        cards: cards(names),
    });
    state
}

fn opens(name: &str) -> Vec<Effect> {
    vec![Effect::Navigate(Destination::Card(
        slug::card_slug(name).unwrap(),
    ))]
}

fn opens_results(query: &str) -> Vec<Effect> {
    vec![Effect::Navigate(Destination::Results {
        query: query.to_owned(),
    })]
}

#[test]
fn a_single_match_is_highlighted_and_enter_opens_its_card() {
    let mut state = answered("krenko, mob", &["Krenko, Mob Boss"]);
    assert_eq!(state.highlight(), Some(0));
    assert_eq!(state.update(Action::Enter), opens("Krenko, Mob Boss"));
}

#[test]
fn several_matches_highlight_none_and_enter_opens_the_results_page() {
    let mut state = answered(" krenko ", &["Krenko, Mob Boss", "Krenko's Command"]);
    assert_eq!(state.highlight(), None);
    assert_eq!(state.update(Action::Enter), opens_results("krenko"));
}

#[test]
fn the_arrows_move_the_highlight_and_stop_at_either_end() {
    let mut state = answered("krenko", &["Krenko, Mob Boss", "Krenko's Command"]);
    let _ = state.update(Action::Step(Step::Down));
    assert_eq!(
        state.highlight(),
        Some(0),
        "the first step lights the top one"
    );
    let _ = state.update(Action::Step(Step::Down));
    let _ = state.update(Action::Step(Step::Down));
    assert_eq!(state.highlight(), Some(1), "stops at the last");
    assert_eq!(
        state.clone().update(Action::Enter),
        opens("Krenko's Command")
    );
    let _ = state.update(Action::Step(Step::Up));
    let _ = state.update(Action::Step(Step::Up));
    assert_eq!(state.highlight(), Some(0), "stops at the first");
    assert_eq!(state.update(Action::Enter), opens("Krenko, Mob Boss"));
}

#[test]
fn enter_with_nothing_typed_or_nothing_matching_opens_nothing() {
    let mut state = State::new();
    assert_eq!(state.update(Action::Enter), vec![]);
    let mut empty = answered("zzzz", &[]);
    assert_eq!(empty.highlight(), None);
    assert_eq!(empty.update(Action::Enter), vec![]);
    let mut blank = State::new();
    let _ = blank.update(Action::Type("   ".to_owned()));
    assert_eq!(blank.update(Action::Enter), vec![]);
}

#[test]
fn enter_before_the_answer_opens_the_results_page_for_the_box() {
    let mut state = State::new();
    let _ = typed(&mut state, "krenko");
    assert_eq!(state.highlight(), None, "nothing while the lookup is out");
    assert_eq!(state.update(Action::Enter), opens_results("krenko"));

    let mut waiting = State::new();
    let _ = waiting.update(Action::Type("goblin".to_owned()));
    assert_eq!(
        waiting.update(Action::Enter),
        opens_results("goblin"),
        "still in the typing wait"
    );
}

#[test]
fn typing_again_drops_the_highlight_and_enter_looks_up_the_new_text() {
    let mut state = answered("krenko", &["Krenko, Mob Boss"]);
    let _ = state.update(Action::Type("krenko m".to_owned()));
    assert_eq!(state.highlight(), None);
    assert_eq!(state.update(Action::Enter), opens_results("krenko m"));
}

#[test]
fn typing_past_a_no_match_answer_lets_enter_look_again() {
    let mut state = answered("krenkp", &[]);
    assert_eq!(state.update(Action::Enter), vec![]);
    let _ = state.update(Action::Type("krenk".to_owned()));
    assert_eq!(state.update(Action::Enter), opens_results("krenk"));
}

#[test]
fn opening_a_card_or_the_results_empties_the_box() {
    let mut picked = answered("krenko", &["Krenko, Mob Boss", "Krenko's Command"]);
    let id = picked.results()[1].scryfall_data.id;
    assert_eq!(picked.update(Action::Pick(id)), opens("Krenko's Command"));
    assert_eq!(picked.query(), "");
    assert_eq!(picked.status(), Status::Idle);

    let mut entered = answered("krenko", &["Krenko, Mob Boss", "Krenko's Command"]);
    assert_eq!(entered.update(Action::Enter), opens_results("krenko"));
    assert_eq!(entered.query(), "");
    assert_eq!(entered.status(), Status::Idle);
}

#[test]
fn each_destination_has_its_path() {
    assert_eq!(
        Destination::Card(slug::card_slug("Sol Ring").unwrap()).path(),
        "/card/sol-ring"
    );
    let id = Uuid::nil();
    let printing = slug::printing_path("_____", id);
    assert_eq!(
        Destination::Printing {
            path: printing.clone()
        }
        .path(),
        printing
    );
    assert_eq!(
        Destination::Results {
            query: "solemn sim".to_owned()
        }
        .path(),
        "/search?q=solemn%20sim"
    );
    assert_eq!(
        Destination::Results {
            query: "Jötun & co?/#+".to_owned()
        }
        .path(),
        "/search?q=J%C3%B6tun%20%26%20co%3F%2F%23%2B"
    );
}
