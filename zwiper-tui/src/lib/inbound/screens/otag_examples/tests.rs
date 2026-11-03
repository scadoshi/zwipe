use crossterm::event::KeyCode;
use zwipe_ui::generation::Generation;
use zwipe_ui::screens::oracle_tag_examples::{Status, query_for};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::printings::PrintingsView;
use crate::test_support::{card, render, screen_text, user};

const SLUG: &str = "mana-dork";

/// The ticket on the last page request queued.
fn last_ticket(harness: &Harness) -> Ticket {
    match harness.pending().last() {
        Some(screens::Request::OtagExamples(Request::Page { ticket, .. })) => *ticket,
        other => panic!("no page went out: {other:?}"),
    }
}

/// Home with the examples for [`SLUG`] open over it, the hint already seen, and nothing queued, with the visit's ticket. Every page of one visit carries it.
fn on_examples() -> (Harness, Ticket) {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut me = user(true);
    me.hints_shown.insert(HINT_OTAG_EXAMPLES.to_owned(), true);
    harness.app.shared.user = Some(me);
    harness.open(|cx| open(cx, SLUG));
    let ticket = last_ticket(&harness);
    harness.clear();
    (harness, ticket)
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "a test fixture: card names count up from a small offset"
)]
fn page(ticket: Ticket, offset: u32, count: usize) -> Update {
    Update::Page {
        ticket,
        cards: (0..count)
            .map(|i| {
                let mut card = card(&format!("Card {}", usize::try_from(offset).unwrap() + i));
                // A card with no type line or rules text shows no face, as in the app.
                card.scryfall_data.type_line = Some("Creature".to_owned());
                card
            })
            .collect(),
    }
}

fn view(harness: &Harness) -> &ExamplesView {
    match harness.screen() {
        Screen::OtagExamples(view) => view,
        other => panic!("not the examples: {other:?}"),
    }
}

fn asked(ticket: Ticket, offset: u32) -> screens::Request {
    Request::Page {
        ticket,
        query: Box::new(query_for(SLUG, offset).unwrap()),
    }
    .into()
}

#[test]
fn opening_asks_for_the_first_page_and_shows_the_hint_once() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(|cx| open(cx, SLUG));
    let ticket = last_ticket(&harness);
    assert_eq!(
        harness.pending(),
        [
            screens::Request::MarkHintShown(HINT_OTAG_EXAMPLES.to_owned()),
            asked(ticket, 0),
        ]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.content.title, "Example cards");
    assert_eq!(
        hint::text(&hint.content),
        "• Swipe left for the next card\n• Swipe down to go back one"
    );
}

#[test]
fn left_moves_on_and_the_next_page_loads_near_the_end() {
    let (mut harness, ticket) = on_examples();
    harness.apply(page(ticket, 0, 25));
    assert_eq!(
        view(&harness).current().unwrap().scryfall_data.name,
        "Card 0"
    );
    for _ in 0..18 {
        harness.press(KeyCode::Left);
    }
    assert!(harness.pending().is_empty(), "six or more still ahead");
    harness.press(KeyCode::Char('h'));
    assert_eq!(view(&harness).model.index(), 19);
    assert_eq!(harness.pending(), [asked(ticket, 25)]);
    harness.press(KeyCode::Left);
    assert_eq!(harness.pending().len(), 1, "one page at a time");
    harness.apply(page(ticket, 25, 25));
    assert_eq!(view(&harness).model.cards().len(), 50);
}

#[test]
fn down_steps_back_and_says_so_at_the_first_card() {
    let (mut harness, ticket) = on_examples();
    harness.apply(page(ticket, 0, 3));
    harness.press(KeyCode::Down);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Already at the first card"
    );
    harness.press(KeyCode::Left);
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).model.index(), 0);
}

#[test]
fn an_empty_page_ends_the_browse_and_down_comes_back() {
    let (mut harness, ticket) = on_examples();
    harness.apply(page(ticket, 0, 2));
    harness.press(KeyCode::Left);
    assert_eq!(harness.pending(), [asked(ticket, 25)]);
    harness.apply(page(ticket, 25, 0));
    harness.press(KeyCode::Left);
    assert_eq!(view(&harness).model.index(), 2);
    assert_eq!(view(&harness).model.status(), Status::End);
    // The last card is still flying off over the line; let it land.
    view_mut(&mut harness).motion = None;
    assert!(screen_text(&harness.app).contains("No more cards. Swipe down to go back."));
    harness.press(KeyCode::Left);
    assert_eq!(view(&harness).model.index(), 2, "nothing past the end");
    harness.press(KeyCode::Down);
    view_mut(&mut harness).motion = None;
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Card 1"));
    assert!(screen.contains("? hint") && !screen.contains("? Help"));
}

#[test]
fn a_tag_with_no_cards_says_so() {
    let (mut harness, ticket) = on_examples();
    assert!(
        screen_text(&harness.app).contains('░'),
        "a ghost card first"
    );
    harness.apply(page(ticket, 0, 0));
    assert!(screen_text(&harness.app).contains("No example cards for this tag yet."));
}

#[test]
fn a_page_on_a_ticket_the_visit_did_not_issue_changes_nothing() {
    let (mut harness, ticket) = on_examples();
    let mut other = Generation::default();
    other.next_ticket();
    let stale = other.next_ticket();
    assert_ne!(stale, ticket);
    harness.apply(Update::Page {
        ticket: stale,
        cards: vec![card("Stray")],
    });
    harness.apply(Update::PageFailed {
        ticket: stale,
        message: "Server unavailable".to_owned(),
    });
    assert!(view(&harness).model.cards().is_empty(), "no stray cards");
    assert_eq!(view(&harness).model.status(), Status::Loading);
    harness.apply(page(ticket, 0, 1));
    assert_eq!(view(&harness).model.cards().len(), 1);
}

#[test]
fn a_failed_page_toasts_and_stops_loading() {
    let (mut harness, ticket) = on_examples();
    harness.apply(Update::PageFailed {
        ticket,
        message: "Server unavailable".to_owned(),
    });
    assert_eq!(view(&harness).model.status(), Status::Empty);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Server unavailable"
    );
    assert!(screen_text(&harness.app).contains("No example cards for this tag yet."));
}

#[test]
fn d_opens_details_and_p_browses_printings_read_only() {
    let (mut harness, ticket) = on_examples();
    harness.apply(page(ticket, 0, 1));
    harness.press(KeyCode::Char('d'));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("p Printings"));
    assert!(screen.contains("Card 0"));
    harness.press(KeyCode::Char('p'));
    let Screen::Printings(printings) = harness.screen() else {
        panic!("no printings");
    };
    let printings: &PrintingsView = printings;
    assert!(!printings.model.shows_save(), "read only");
    harness.press(KeyCode::Esc);
    assert!(view(&harness).details.is_some(), "the sheet stays under");
    harness.press(KeyCode::Esc);
    assert!(view(&harness).details.is_none());
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn question_mark_shows_the_hint_again() {
    let (mut harness, _) = on_examples();
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
    let (mut harness, ticket) = on_examples();
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
    harness.apply(page(ticket, 0, 1));
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
    harness.press(KeyCode::Char('d'));
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
}

#[test]
fn the_next_card_flies_off_left_and_a_step_back_flies_it_in() {
    use zwipe_ui::stack::Direction;

    use crate::inbound::widgets::card_motion::Phase;

    let (mut harness, ticket) = on_examples();
    harness.apply(page(ticket, 0, 3));
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

    let (mut harness, ticket) = on_examples();
    harness.apply(page(ticket, 0, 3));
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

#[test]
fn the_examples_keep_every_key_on_a_narrow_terminal() {
    let (mut harness, ticket) = on_examples();
    crate::test_support::assert_key_bar_fits(&harness.app, "? hint");
    harness.apply(page(ticket, 0, 2));
    crate::test_support::assert_key_bar_fits(&harness.app, "? hint");
}

#[test]
fn a_leaving_card_keeps_its_words_whole_and_the_next_card_shows_under_it() {
    use std::time::{Duration, Instant};

    use zwipe_ui::stack::Direction;

    use crate::inbound::widgets::card_motion::{CardMotion, Phase};

    let (mut harness, ticket) = on_examples();
    let rules = [
        "Protean Hydra enters with X +1/+1 counters on it. If damage would be dealt to Protean Hydra, prevent that damage and remove that many +1/+1 counters from it.",
        "When this enters, put a +1/+1 counter on each creature you control and draw a card.",
    ];
    let cards = ["Protean Hydra", "Next Card"]
        .into_iter()
        .zip(rules)
        .map(|(name, text)| {
            let mut c = card(name);
            c.scryfall_data.type_line = Some("Creature — Hydra".to_owned());
            c.scryfall_data.oracle_text = Some(text.to_owned());
            c
        })
        .collect();
    harness.apply(Update::Page { ticket, cards });
    harness.press(KeyCode::Left);
    let Some(Phase::Exit(leaving)) = view(&harness).motion.as_ref().map(|m| m.phase().clone())
    else {
        panic!("the card flies off");
    };
    let at = |harness: &mut Harness, millis: u64| {
        let started = Instant::now()
            .checked_sub(Duration::from_millis(millis))
            .unwrap();
        view_mut(harness).motion =
            Some(CardMotion::exit(Direction::Left, leaving.clone(), started));
    };
    for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
        at(&mut harness, 30);
        let rows = crate::test_support::rows(&harness.app, width, height);
        let text = rows.join("\n");
        let name = rows
            .iter()
            .find_map(|r| {
                r.split('│')
                    .map(str::trim)
                    .find(|cell| cell.starts_with("Pro"))
            })
            .unwrap_or_else(|| panic!("the leaving name at {width}x{height}:\n{text}"));
        assert!(
            "Protean Hydra".starts_with(name),
            "the name is clipped, never thinned, at {width}x{height}: {name:?}\n{text}"
        );
        at(&mut harness, 120);
        let text = crate::test_support::rows(&harness.app, width, height).join("\n");
        for part in ["Next Card", "When this enters"] {
            assert!(
                text.contains(part),
                "{part} under the fading card at {width}x{height}:\n{text}"
            );
        }
        assert!(
            !text.contains("Protean"),
            "past halfway the leaving card is an outline:\n{text}"
        );
    }
}
