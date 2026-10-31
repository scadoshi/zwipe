use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::printings::{self, PrintingsView};
use crate::test_support::{card, render, screen_text, user};

const SLUG: &str = "mana-dork";

/// Home with the examples for [`SLUG`] open over it, the hint already seen, and nothing queued.
fn on_examples() -> Harness {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut me = user(true);
    me.hints_shown.insert(HINT_OTAG_EXAMPLES.to_owned(), true);
    harness.app.shared.user = Some(me);
    harness.open(|cx| open(cx, SLUG));
    harness.clear();
    harness
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "a test fixture: card names count up from a small offset"
)]
fn page(offset: u32, count: usize) -> Update {
    Update::Page {
        slug: SLUG.to_owned(),
        offset,
        cards: (0..count)
            .map(|i| card(&format!("Card {}", usize::try_from(offset).unwrap() + i)))
            .collect(),
    }
}

fn view(harness: &Harness) -> &ExamplesView {
    match harness.screen() {
        Screen::OtagExamples(view) => view,
        other => panic!("not the examples: {other:?}"),
    }
}

fn asked(offset: u32) -> screens::Request {
    Request::Page {
        slug: SLUG.to_owned(),
        offset,
    }
    .into()
}

#[test]
fn opening_asks_for_the_first_page_and_shows_the_hint_once() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(|cx| open(cx, SLUG));
    assert_eq!(
        harness.pending(),
        [
            asked(0),
            screens::Request::MarkHintShown(HINT_OTAG_EXAMPLES.to_owned())
        ]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.content.title, "Example cards");
    assert_eq!(
        hint.content.text(),
        "• Swipe left for the next card\n• Swipe down to go back one"
    );
}

#[test]
fn the_query_is_the_apps() {
    let query = query_for(SLUG, 50).unwrap();
    assert_eq!(query.limit(), 25);
    assert_eq!(query.offset(), 50);
    assert_eq!(query.sort(), Some(CardSortKey::EdhrecRank));
    let wire = serde_json::to_string(&query).unwrap();
    assert!(wire.contains(SLUG), "{wire}");
    assert!(wire.contains("\"is_token\":false"), "{wire}");
}

#[test]
fn left_moves_on_and_the_next_page_loads_near_the_end() {
    let mut harness = on_examples();
    harness.apply(page(0, 25));
    assert_eq!(
        view(&harness).current().unwrap().scryfall_data.name,
        "Card 0"
    );
    for _ in 0..18 {
        harness.press(KeyCode::Left);
    }
    assert!(harness.pending().is_empty(), "six or more still ahead");
    harness.press(KeyCode::Char('h'));
    assert_eq!(view(&harness).index, 19);
    assert_eq!(harness.pending(), [asked(25)]);
    harness.press(KeyCode::Left);
    assert_eq!(harness.pending().len(), 1, "one page at a time");
    harness.apply(page(25, 25));
    assert_eq!(view(&harness).cards.len(), 50);
    assert_eq!(view(&harness).offset, 50);
}

#[test]
fn down_steps_back_and_says_so_at_the_first_card() {
    let mut harness = on_examples();
    harness.apply(page(0, 3));
    harness.press(KeyCode::Down);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Already at the first card"
    );
    harness.press(KeyCode::Left);
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).index, 0);
}

#[test]
fn an_empty_page_ends_the_browse_and_down_comes_back() {
    let mut harness = on_examples();
    harness.apply(page(0, 2));
    harness.press(KeyCode::Left);
    assert_eq!(harness.pending(), [asked(25)]);
    harness.apply(page(25, 0));
    assert!(view(&harness).exhausted);
    harness.press(KeyCode::Left);
    assert_eq!(view(&harness).index, 2);
    // The last card is still flying off over the line; let it land.
    view_mut(&mut harness).motion = None;
    assert!(screen_text(&harness.app).contains("No more cards. Swipe down to go back."));
    harness.press(KeyCode::Left);
    assert_eq!(view(&harness).index, 2, "nothing past the end");
    harness.press(KeyCode::Down);
    view_mut(&mut harness).motion = None;
    assert!(screen_text(&harness.app).contains("Card 1"));
}

#[test]
fn a_tag_with_no_cards_says_so() {
    let mut harness = on_examples();
    assert!(
        screen_text(&harness.app).contains('░'),
        "a ghost card first"
    );
    harness.apply(page(0, 0));
    assert!(screen_text(&harness.app).contains("No example cards for this tag yet."));
}

#[test]
fn paging_stops_at_the_cap() {
    let mut harness = on_examples();
    if let Screen::OtagExamples(view) = harness.screen_mut() {
        view.cards = (0..MAX_CARDS).map(|i| card(&format!("Card {i}"))).collect();
        view.index = MAX_CARDS - 2;
        view.offset = 500;
        view.loading = false;
    }
    harness.press(KeyCode::Left);
    assert!(harness.pending().is_empty(), "no page past the cap");
    assert!(view(&harness).exhausted);
}

#[test]
fn a_late_page_for_another_tag_or_offset_is_ignored() {
    let mut harness = on_examples();
    harness.apply(Update::Page {
        slug: "other".to_owned(),
        offset: 0,
        cards: vec![card("Stray")],
    });
    harness.apply(page(25, 1));
    assert!(view(&harness).cards.is_empty(), "no stray cards");
    assert!(view(&harness).loading);
}

#[test]
fn a_failed_page_toasts_and_stops_loading() {
    let mut harness = on_examples();
    harness.apply(Update::PageFailed {
        slug: SLUG.to_owned(),
        message: "Server unavailable".to_owned(),
    });
    assert!(!view(&harness).loading);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Server unavailable"
    );
    assert!(screen_text(&harness.app).contains("No example cards for this tag yet."));
}

#[test]
fn d_opens_details_and_p_browses_printings_read_only() {
    let mut harness = on_examples();
    harness.apply(page(0, 1));
    harness.press(KeyCode::Char('d'));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("p Printings"));
    assert!(screen.contains("Card 0"));
    harness.press(KeyCode::Char('p'));
    let Screen::Printings(printings) = harness.screen() else {
        panic!("no printings");
    };
    let printings: &PrintingsView = printings;
    assert_eq!(printings.mode, printings::Mode::ReadOnly);
    harness.press(KeyCode::Esc);
    assert!(view(&harness).details.is_some(), "the sheet stays under");
    harness.press(KeyCode::Esc);
    assert!(view(&harness).details.is_none());
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn question_mark_shows_the_hint_again() {
    let mut harness = on_examples();
    harness.press(KeyCode::Char('?'));
    assert_eq!(
        harness
            .app
            .shared
            .hint
            .as_ref()
            .map(|h| h.content.title.as_str()),
        Some("Example cards")
    );
}

#[test]
fn the_examples_fit_a_tiny_terminal() {
    let mut harness = on_examples();
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
    harness.apply(page(0, 1));
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
    harness.press(KeyCode::Char('d'));
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
}

#[test]
fn the_next_card_flies_off_left_and_a_step_back_flies_it_in() {
    use crate::domain::stack::Direction;
    use crate::inbound::widgets::card_motion::Phase;

    let mut harness = on_examples();
    harness.apply(page(0, 3));
    harness.press(KeyCode::Left);
    let motion = view(&harness).motion.clone().expect("the card flies off");
    assert_eq!(motion.direction(), Direction::Left);
    assert!(matches!(motion.phase(), Phase::Exit(c) if c.scryfall_data.name == "Card 0"));
    assert!(harness.app.nav.top().animating());
    render(&harness.app, 80, 30);

    harness.press(KeyCode::Down);
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(
        (motion.direction(), motion.phase()),
        (Direction::Left, &Phase::Enter)
    );
    assert_eq!(
        view(&harness)
            .under()
            .map(|c| c.scryfall_data.name.as_str()),
        Some("Card 1")
    );
    render(&harness.app, 80, 30);

    view_mut(&mut harness).motion = None;
    harness.press(KeyCode::Down);
    assert!(
        view(&harness).motion.is_none(),
        "nothing flies at the first card"
    );
}

#[test]
fn the_swipe_keys_read_skip_and_undo_in_the_apps_swipe_colors() {
    use crate::test_support::word_colors;

    let mut harness = on_examples();
    harness.apply(page(0, 3));
    let palette = harness.app.shared.palette;
    let row = "← skip  ↓ undo";
    assert!(screen_text(&harness.app).contains(row));
    assert_eq!(
        word_colors(&harness.app, row, &["skip", "undo"]),
        [palette.error.fg, palette.tertiary.fg]
    );
}

fn view_mut(harness: &mut Harness) -> &mut ExamplesView {
    match harness.screen_mut() {
        Screen::OtagExamples(view) => view,
        other => panic!("not the examples: {other:?}"),
    }
}
