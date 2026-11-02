use super::*;
use crate::test_support::card;
use crate::toast::QUICK;

fn page(count: usize) -> Vec<Card> {
    (0..count).map(|i| card(&format!("Card {i}"))).collect()
}

fn names(state: &State) -> Vec<String> {
    state
        .cards()
        .iter()
        .map(|c| c.scryfall_data.name.clone())
        .collect()
}

/// The ticket and offset of a page search, or a panic naming what came back.
fn searched(effects: &[Effect]) -> (Ticket, CardQuery) {
    let Some(Effect::SearchCards { ticket, query }) = effects.last() else {
        panic!("expected a page search: {effects:?}");
    };
    (*ticket, (**query).clone())
}

/// A browse for `ramp` whose first page is out, with its ticket.
fn loading() -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load {
        slug: "ramp".to_owned(),
    });
    assert_eq!(
        effects.first(),
        Some(&Effect::OfferHint(OneTimeHint::OtagExamples))
    );
    assert_eq!(effects.len(), 2, "Load offers the hint and searches");
    let (ticket, _) = searched(&effects);
    (state, ticket)
}

/// A browse whose first page brought `count` cards.
fn landed(count: usize) -> (State, Ticket) {
    let (mut state, ticket) = loading();
    let effects = state.update(Action::Loaded {
        ticket,
        cards: page(count),
    });
    assert_eq!(effects, [], "a reply asks for nothing");
    (state, ticket)
}

#[test]
fn a_new_state_is_loading_with_no_card() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.message(), None);
    assert_eq!(state.current(), None);
    assert_eq!(state.window(), []);
    assert_eq!(state.index(), 0);
}

#[test]
fn load_searches_the_first_page_by_edhrec_rank() {
    let (state, _) = loading();
    let mut state = state;
    assert_eq!(state.slug(), "ramp");
    let effects = state.update(Action::Load {
        slug: "draw".to_owned(),
    });
    let [Effect::SearchCards { query, .. }] = effects.as_slice() else {
        panic!("a second Load only searches: {effects:?}");
    };
    assert_eq!(**query, query_for("draw", 0).unwrap());
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn the_query_is_the_apps() {
    let mut builder = CardQueryBuilder::new();
    builder
        .set_oracle_tags_contains_any(vec!["ramp".to_owned()])
        .set_is_token(false)
        .set_sort(CardSortKey::EdhrecRank)
        .set_limit(25)
        .set_offset(50);
    assert_eq!(query_for("ramp", 50).unwrap(), builder.build().unwrap());
}

#[test]
fn a_page_shows_its_first_card() {
    let (state, _) = landed(3);
    assert_eq!(state.status(), Status::Showing);
    assert_eq!(state.message(), None);
    assert_eq!(
        state.current().map(|c| c.scryfall_data.name.as_str()),
        Some("Card 0")
    );
    assert_eq!(state.window().len(), 3);
}

#[test]
fn an_empty_first_page_is_empty() {
    let (state, _) = landed(0);
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.message(), Some(EMPTY));
}

#[test]
fn a_failed_first_page_reads_as_empty() {
    let (mut state, ticket) = loading();
    let effects = state.update(Action::Failed { ticket });
    assert_eq!(effects, [], "the client's runner raises the error toast");
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.message(), Some(EMPTY));
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, older) = loading();
    let _ = state.update(Action::Load {
        slug: "ramp".to_owned(),
    });
    let _ = state.update(Action::Loaded {
        ticket: older,
        cards: page(2),
    });
    assert_eq!(state.status(), Status::Loading, "a stale page landed");
    assert_eq!(state.cards(), []);
    let _ = state.update(Action::Failed { ticket: older });
    assert_eq!(state.status(), Status::Loading, "a stale failure landed");
}

#[test]
fn next_moves_on_without_paging_far_from_the_end() {
    let (mut state, _) = landed(25);
    let effects = state.update(Action::Next);
    assert_eq!(effects, []);
    assert_eq!(state.index(), 1);
    assert_eq!(
        state.current().map(|c| c.scryfall_data.name.as_str()),
        Some("Card 1")
    );
}

#[test]
fn next_near_the_end_asks_for_the_next_page_once() {
    let (mut state, first) = landed(25);
    for _ in 0..18 {
        assert_eq!(state.update(Action::Next), []);
    }
    let effects = state.update(Action::Next);
    let (ticket, query) = searched(&effects);
    assert_eq!(ticket, first, "a next page continues the generation");
    assert_eq!(query, query_for("ramp", 25).unwrap());
    assert_eq!(state.update(Action::Next), [], "one page out at a time");

    let _ = state.update(Action::Loaded {
        ticket,
        cards: page(25),
    });
    assert_eq!(state.cards().len(), 50);
    assert_eq!(state.status(), Status::Showing);
}

#[test]
fn a_failed_page_lets_the_next_swipe_ask_again() {
    let (mut state, _) = landed(6);
    let effects = state.update(Action::Next);
    let (ticket, _) = searched(&effects);
    let _ = state.update(Action::Failed { ticket });
    let effects = state.update(Action::Next);
    assert_eq!(searched(&effects).1, query_for("ramp", 25).unwrap());
}

#[test]
fn past_the_last_card_the_end_shows_or_ghosts_while_a_page_is_out() {
    let (mut state, _) = landed(1);
    let effects = state.update(Action::Next);
    let (ticket, _) = searched(&effects);
    assert_eq!(state.current(), None);
    assert_eq!(state.status(), Status::Loading, "a page is out");
    let _ = state.update(Action::Loaded {
        ticket,
        cards: Vec::new(),
    });
    assert_eq!(state.status(), Status::End);
    assert_eq!(state.message(), Some(END));
    assert_eq!(state.update(Action::Next), [], "an empty page ends paging");
}

#[test]
fn paging_stops_at_the_cap() {
    let (mut state, ticket) = loading();
    let _ = state.update(Action::Loaded {
        ticket,
        cards: page(timing::EXAMPLES_MAX_CARDS),
    });
    for _ in 0..timing::EXAMPLES_MAX_CARDS {
        let effects = state.update(Action::Next);
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SearchCards { .. })),
            "paged past the cap"
        );
    }
    assert_eq!(state.status(), Status::End);
}

#[test]
fn back_one_brings_the_card_back_from_the_left() {
    let (mut state, _) = landed(3);
    let _ = state.update(Action::Next);
    let effects = state.update(Action::BackOne);
    assert_eq!(effects, [Effect::Enter(Direction::Left)]);
    assert_eq!(state.index(), 0);
    assert_eq!(names(&state).len(), 3, "stepping back keeps every card");
}

#[test]
fn back_one_from_the_end_returns_to_the_last_card() {
    let (mut state, _) = landed(1);
    let effects = state.update(Action::Next);
    let (ticket, _) = searched(&effects);
    let _ = state.update(Action::Loaded {
        ticket,
        cards: Vec::new(),
    });
    let _ = state.update(Action::BackOne);
    assert_eq!(state.status(), Status::Showing);
}

#[test]
fn back_one_at_the_first_card_says_so() {
    let (mut state, _) = landed(3);
    assert_eq!(
        state.update(Action::BackOne),
        [Effect::Toast(Toast::new(ToastKind::Info, AT_FIRST, QUICK))]
    );
    assert_eq!(state.index(), 0);
}

#[test]
fn back_returns_to_the_screen_that_opened_it() {
    let (mut state, _) = landed(3);
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [TITLE, HINT_TITLE, END, EMPTY, AT_FIRST, BACK],
        [
            "Examples",
            "Example cards",
            "No more cards. Swipe down to go back.",
            "No example cards for this tag yet.",
            "Already at the first card",
            "Back"
        ]
    );
}
