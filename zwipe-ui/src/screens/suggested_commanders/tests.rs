use zwipe_core::http::contracts::suggestions::HttpSuggestionReasons;

use super::*;
use crate::test_support::card;

/// A suggestion of a commander named `name`.
fn suggestion(name: &str) -> HttpCommanderSuggestion {
    HttpCommanderSuggestion {
        card: card(name),
        reasons: HttpSuggestionReasons {
            tags: Vec::new(),
            colors: Vec::new(),
        },
    }
}

/// `n` suggestions named `prefix` 1 to `n`.
fn page(prefix: &str, n: usize) -> Vec<HttpCommanderSuggestion> {
    (1..=n)
        .map(|i| suggestion(&format!("{prefix} {i}")))
        .collect()
}

/// The ticket and request of the one deal among `effects`.
fn deal_of(effects: &[Effect]) -> (Ticket, HttpCommanderDeal) {
    let deals: Vec<(Ticket, HttpCommanderDeal)> = effects
        .iter()
        .filter_map(|e| {
            if let Effect::Deal {
                ticket, request, ..
            } = e
            {
                Some((*ticket, (**request).clone()))
            } else {
                None
            }
        })
        .collect();
    let [one] = deals.as_slice() else {
        panic!("one deal: {effects:?}");
    };
    one.clone()
}

/// A full page landed for `ticket`.
fn landed(ticket: Ticket, rows: Vec<HttpCommanderSuggestion>) -> Action {
    Action::Page {
        ticket,
        reply: Ok(HttpCommanderDealPage {
            suggestions: rows,
            ranked_by: RankedBy::Playstyle,
        }),
    }
}

/// A list loaded with a full first page of ten.
fn loaded() -> State {
    let mut state = State::new();
    let (ticket, _) = deal_of(&state.update(Action::Load(Box::default())));
    let _ = state.update(landed(ticket, page("First", 10)));
    state
}

/// The names of the list's rows.
fn names(state: &State) -> Vec<String> {
    state
        .entries()
        .iter()
        .map(|s| s.card.scryfall_data.name.clone())
        .collect()
}

#[test]
fn it_starts_loading_and_deals_ten_from_the_top() {
    let mut state = State::new();
    assert!(state.is_loading());
    let (_, request) = deal_of(&state.update(Action::Load(Box::default())));
    assert_eq!(request.limit, LIST_PAGE);
    assert_eq!(request.exclude, Vec::<Uuid>::new());
    assert!(
        !request.keep_maybeboard,
        "Suggested leaves the maybeboard out"
    );
    let state = loaded();
    assert!(!state.is_loading());
    assert_eq!(state.entries().len(), 10);
    assert!(state.offers_more());
}

#[test]
fn show_more_deals_the_next_page_once_per_deal() {
    let mut state = loaded();
    let held = deal::oracle_ids(state.entries());
    let (ticket, request) = deal_of(&state.update(Action::ShowMore));
    assert_eq!(request.exclude, held, "leaves out what the list holds");
    assert!(!state.offers_more(), "used once, it goes");
    assert_eq!(state.update(Action::ShowMore), Vec::new());
    let _ = state.update(landed(ticket, page("Second", 10)));
    assert_eq!(state.entries().len(), 20);
    assert!(!state.offers_more(), "still gone until a fresh deal");
    let (ticket, _) = deal_of(&state.update(Action::Redeal));
    let _ = state.update(landed(ticket, page("Again", 10)));
    assert!(state.offers_more(), "a fresh deal offers it again");
}

#[test]
fn a_short_page_ends_the_ranking() {
    let mut state = State::new();
    let (ticket, _) = deal_of(&state.update(Action::Load(Box::default())));
    let _ = state.update(landed(ticket, page("Only", 4)));
    assert!(!state.offers_more(), "four of ten was the last page");
}

#[test]
fn only_the_newest_deal_lands() {
    let mut state = State::new();
    let (old, _) = deal_of(&state.update(Action::Load(Box::default())));
    let mut narrowed = CardQueryBuilder::new();
    narrowed.set_name_contains("kr");
    let (new, _) = deal_of(&state.update(Action::Load(Box::new(narrowed))));
    let _ = state.update(landed(old, page("Stale", 10)));
    assert!(state.entries().is_empty(), "the filter changed since");
    let _ = state.update(landed(new, page("Krenko", 3)));
    assert_eq!(names(&state), ["Krenko 1", "Krenko 2", "Krenko 3"]);
}

#[test]
fn a_saved_or_skipped_row_leaves_and_the_list_tops_back_up() {
    let mut state = loaded();
    let first = state.entries()[0].card.scryfall_data.oracle_id.unwrap();
    assert_eq!(
        state.update(Action::Maybeboard(first)),
        vec![Effect::AddMaybeboard(first)]
    );
    let effects = state.update(Action::Saved(first));
    assert_eq!(state.entries().len(), 9);
    let (_, request) = deal_of(&effects);
    assert_eq!(
        request.exclude.len(),
        9,
        "the top-up leaves out the rows left"
    );
    assert!(effects.contains(&Effect::Toast(Toast::quick(ToastKind::Success, SAVED))));
    assert!(state.offers_more(), "a top-up isn't the user's Show more");

    let mut state = loaded();
    let first = state.entries()[0].card.scryfall_data.oracle_id.unwrap();
    assert_eq!(
        state.update(Action::Dismiss(first)),
        vec![Effect::Dismiss(first)]
    );
    let effects = state.update(Action::Dismissed(first));
    assert!(effects.contains(&Effect::Toast(Toast::quick(ToastKind::Info, SKIPPED))));
}

#[test]
fn clear_skips_deals_again_from_the_top() {
    let mut state = loaded();
    assert_eq!(
        state.update(Action::ClearSkips),
        vec![Effect::ClearDismissals]
    );
    let effects = state.update(Action::Cleared);
    assert_eq!(
        effects.first(),
        Some(&Effect::Toast(Toast::quick(ToastKind::Info, SKIPS_CLEARED)))
    );
    let (_, request) = deal_of(&effects);
    assert_eq!(request.exclude, Vec::<Uuid>::new());
}

#[test]
fn a_failed_first_page_fails_only_an_empty_list() {
    let mut state = State::new();
    let (ticket, _) = deal_of(&state.update(Action::Load(Box::default())));
    let _ = state.update(Action::Page {
        ticket,
        reply: Err(()),
    });
    assert!(state.load_failed());
    assert!(!state.is_loading());

    let mut state = loaded();
    let (ticket, _) = deal_of(&state.update(Action::Redeal));
    let _ = state.update(Action::Page {
        ticket,
        reply: Err(()),
    });
    assert!(!state.load_failed(), "the rows it had stay");
    assert_eq!(state.entries().len(), 10);
}

#[test]
fn the_empty_line_says_whether_the_filter_matched_nothing() {
    let mut state = State::new();
    let (ticket, _) = deal_of(&state.update(Action::Load(Box::default())));
    let _ = state.update(landed(ticket, Vec::new()));
    assert_eq!(state.empty_line(), Some(NO_SUGGESTIONS));
    let mut narrowed = CardQueryBuilder::new();
    narrowed.set_name_contains("zzqx");
    let (ticket, _) = deal_of(&state.update(Action::Load(Box::new(narrowed))));
    let _ = state.update(landed(ticket, Vec::new()));
    assert_eq!(state.empty_line(), Some(NO_MATCH));
    assert_eq!(loaded().empty_line(), None);
}

#[test]
fn a_popular_ranking_says_so_over_the_rows() {
    let mut state = State::new();
    let (ticket, _) = deal_of(&state.update(Action::Load(Box::default())));
    let _ = state.update(Action::Page {
        ticket,
        reply: Ok(HttpCommanderDealPage {
            suggestions: page("Popular", 10),
            ranked_by: RankedBy::Popularity,
        }),
    });
    assert_eq!(state.ranked_line(), Some(deal::POPULAR_LINE));
    assert_eq!(loaded().ranked_line(), None);
}

#[test]
fn a_picked_printing_swaps_into_its_row() {
    let mut state = loaded();
    let mut picked = state.entries()[2].card.clone();
    picked.scryfall_data.id = Uuid::new_v4();
    assert!(state.swap_printing(&picked));
    assert_eq!(
        state.entries()[2].card.scryfall_data.id,
        picked.scryfall_data.id
    );
    assert!(!state.swap_printing(&card("Elsewhere")), "a card on no row");
}

#[test]
fn a_redeal_after_a_failed_load_ghosts_the_list_again_while_it_is_out() {
    let mut state = State::new();
    let (ticket, _) = deal_of(&state.update(Action::Load(Box::default())));
    let _ = state.update(Action::Page {
        ticket,
        reply: Err(()),
    });
    assert!(state.load_failed());
    let (ticket, _) = deal_of(&state.update(Action::Redeal));
    assert!(
        state.is_loading(),
        "Retry ghosts the list while the deal is out"
    );
    assert!(!state.load_failed());
    let _ = state.update(landed(ticket, page("Back", 10)));
    assert!(!state.is_loading());
    assert_eq!(state.entries().len(), 10);
}

#[test]
fn show_more_eases_ghosts_in_and_its_page_lands_under_them() {
    use crate::commander::more_ghosts::{ENTER_ANIMATION, Phase};
    let mut state = loaded();
    let (ticket, _) = deal_of(&state.update(Action::ShowMore));
    assert_eq!(state.ghosts().phase(), Phase::Entering, "at once");
    let _ = state.update(landed(ticket, page("More", 10)));
    assert_eq!(
        rows_around_ghosts(&state),
        (10, 0),
        "the page waits on the ghosts"
    );
    let _ = state.update(Action::GhostsEnded(ENTER_ANIMATION.to_owned()));
    assert_eq!(state.ghosts().phase(), Phase::Leaving);
    assert_eq!(rows_around_ghosts(&state), (10, 10));
}

#[test]
fn a_fresh_deal_takes_the_ghosts_away_and_a_failed_page_sends_them_off() {
    use crate::commander::more_ghosts::{ENTER_ANIMATION, LEAVE_ANIMATION, Phase};
    let mut state = loaded();
    let _ = state.update(Action::ShowMore);
    let _ = state.update(Action::Redeal);
    assert_eq!(state.ghosts().phase(), Phase::Off);

    let mut state = loaded();
    let (ticket, _) = deal_of(&state.update(Action::ShowMore));
    let _ = state.update(Action::Page {
        ticket,
        reply: Err(()),
    });
    let _ = state.update(Action::GhostsEnded(ENTER_ANIMATION.to_owned()));
    let _ = state.update(Action::GhostsEnded(LEAVE_ANIMATION.to_owned()));
    assert_eq!(state.ghosts().phase(), Phase::Off, "gone over no rows");
}

/// How many rows the list draws before Show more's ghosts and after them.
fn rows_around_ghosts(state: &State) -> (usize, usize) {
    use crate::commander::more_ghosts::MoreItem;
    let layout = state.layout();
    let at = layout
        .iter()
        .position(|item| matches!(item, MoreItem::Ghosts))
        .unwrap_or(layout.len());
    (at, layout.len().saturating_sub(at).saturating_sub(1))
}

#[test]
fn show_more_spends_itself_only_when_its_page_goes_out() {
    use crate::commander::more_ghosts::Phase;
    let mut state = loaded();
    // A row leaves and the list tops itself back up, so a deal is out.
    let oracle = state.entries()[0].card.scryfall_data.oracle_id.unwrap();
    let topped = state.update(Action::Saved(oracle));
    assert!(topped.iter().any(|e| matches!(e, Effect::Deal { .. })));
    assert!(
        state.update(Action::ShowMore).is_empty(),
        "no second deal while one is out"
    );
    assert!(state.offers_more(), "the press isn't spent");
    assert_eq!(
        state.ghosts().phase(),
        Phase::Off,
        "and no ghosts fade over nothing"
    );
}
