use std::cell::RefCell;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use image::DynamicImage;
use ratatui::style::Color;
use ratatui_image::picker::{Picker, ProtocolType};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{DeckCard, Format, Quantity};

use super::*;
use crate::domain::stack::Direction;
use crate::domain::theme::Palette;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::widgets::card_motion::{CardMotion, Phase, Snapshot};
use crate::inbound::widgets::toast::ToastKind;
use crate::test_support::{card, deck, deck_profile, render, screen_text};

fn toasts(harness: &Harness) -> Vec<(ToastKind, String)> {
    harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| (t.kind, t.message.clone()))
        .collect()
}

fn last_toast(harness: &Harness) -> (ToastKind, String) {
    toasts(harness).last().cloned().unwrap()
}

fn requests(harness: &Harness) -> Vec<&Request> {
    harness
        .pending()
        .iter()
        .filter_map(|r| match r {
            screens::Request::Swipe(r) => Some(r),
            _ => None,
        })
        .collect()
}

fn view(harness: &Harness) -> &SwipeView {
    harness.swipe_view()
}

fn view_mut(harness: &mut Harness) -> &mut SwipeView {
    match harness.screen_mut() {
        Screen::Swipe(view) => view,
        other => panic!("not on Add: {other:?}"),
    }
}

fn deck_card(deck_id: Uuid, card: &Card, board: Board) -> DeckCard {
    DeckCard {
        deck_id,
        scryfall_data_id: card.scryfall_data.id,
        oracle_id: card.scryfall_data.oracle_id.unwrap(),
        quantity: Quantity::new(1).unwrap(),
        board,
        mvp_at: None,
    }
}

/// Answers every create the screen sent.
fn answer_creates(harness: &mut Harness, deck_id: Uuid) {
    let creates: Vec<(Box<Card>, Board)> = requests(harness)
        .into_iter()
        .filter_map(|r| match r {
            Request::Create { card, board, .. } => Some((card.clone(), *board)),
            _ => None,
        })
        .collect();
    harness.clear();
    for (card, board) in creates {
        let result = Ok(Box::new(deck_card(deck_id, &card, board)));
        harness.apply(Update::Created {
            deck_id,
            card,
            board,
            result,
        });
    }
}

#[test]
fn swipes_send_their_calls_and_toast() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A"), card("B"), card("C")]);
    harness.press(KeyCode::Char('l'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Success, "Added to deck".to_owned())
    );
    harness.press(KeyCode::Char('h'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Info, "Skipped".to_owned())
    );
    harness.press(KeyCode::Char('m'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Info, "Added to maybeboard".to_owned())
    );
    let sent: Vec<&Request> = requests(&harness)
        .into_iter()
        .filter(|r| !matches!(r, Request::Search { .. }))
        .collect();
    assert!(
        matches!(sent[0], Request::Create { board: Board::Deck, deck_id: d, .. } if *d == deck_id)
    );
    assert!(matches!(sent[1], Request::Skip { .. }));
    assert!(matches!(
        sent[2],
        Request::Create {
            board: Board::Maybeboard,
            ..
        }
    ));
    assert_eq!(
        view(&harness).stack.history(),
        [AddAction::Add, AddAction::Skip, AddAction::Maybe]
    );
}

#[test]
fn an_undo_before_the_add_answers_waits_and_deletes_after() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A"), card("B")]);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('u'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Success, "Undid add".to_owned())
    );
    assert!(
        !requests(&harness)
            .iter()
            .any(|r| matches!(r, Request::UndoAdd { .. })),
        "nothing to delete yet"
    );
    answer_creates(&mut harness, deck_id);
    assert!(matches!(
        requests(&harness)[..],
        [Request::DeleteUndone { .. }]
    ));
    assert!(harness.app.shared.cards.undo.is_empty(deck_id));
}

#[test]
fn an_undo_after_the_add_answers_takes_its_log_entry_and_deletes() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A"), card("B")]);
    harness.press(KeyCode::Char('l'));
    answer_creates(&mut harness, deck_id);
    assert_eq!(harness.app.shared.cards.undo.log(deck_id).len(), 1);
    harness.press(KeyCode::Down);
    assert!(harness.app.shared.cards.undo.is_empty(deck_id));
    let Some(Request::UndoAdd { card, taken, .. }) = requests(&harness).first().copied() else {
        panic!("no undo sent");
    };
    let (card, taken) = (card.clone(), taken.clone());
    assert!(taken.is_some());
    harness.clear();
    harness.apply(Update::UndidAdd {
        deck_id,
        card,
        board: Board::Deck,
        taken,
        result: Err(UndoFailure::Call("offline".to_owned())),
    });
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Error, "Failed to undo: offline".to_owned())
    );
    assert_eq!(
        harness.app.shared.cards.undo.log(deck_id).len(),
        1,
        "the entry goes back"
    );
    assert_eq!(view(&harness).stack.index(), 0, "the cursor stays");
}

#[test]
fn an_add_the_deck_view_already_undid_is_not_deleted_twice() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A"), card("B")]);
    harness.press(KeyCode::Char('l'));
    answer_creates(&mut harness, deck_id);
    harness.app.shared.cards.undo.pop(deck_id);
    let oracle = view(&harness).stack.cards()[0]
        .scryfall_data
        .oracle_id
        .unwrap();
    view_mut(&mut harness).deck.in_deck.remove(&oracle);
    harness.press(KeyCode::Char('u'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Success, "Undid add".to_owned())
    );
    assert_eq!(requests(&harness).len(), 0);
}

#[test]
fn undo_says_when_there_is_nothing_and_skips_unskip() {
    let (mut harness, _) = Harness::swiping(vec![card("A"), card("B")]);
    harness.press(KeyCode::Char('u'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Info, "Nothing to undo".to_owned())
    );
    harness.press(KeyCode::Char('h'));
    harness.clear();
    harness.press(KeyCode::Backspace);
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Info, "Undid skip".to_owned())
    );
    assert!(matches!(requests(&harness)[..], [Request::Unskip { .. }]));
    assert_eq!(
        view(&harness).stack.entering(),
        Some(crate::domain::stack::Direction::Left)
    );
}

#[test]
fn a_deck_without_a_commander_or_filter_is_refused() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let pile = deck(deck_profile("Pile"), Vec::new());
    harness.open(|cx| open(cx, &pile));
    assert_eq!(
        last_toast(&harness),
        (
            ToastKind::Warning,
            "Try a filter to start swiping".to_owned()
        )
    );
    assert_eq!(requests(&harness).len(), 0);
    assert!(screen_text(&harness.app).contains("No cards"));
}

#[test]
fn the_end_of_results_says_so_and_low_stacks_fetch_more() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A"), card("B")]);
    harness.press(KeyCode::Char('h'));
    assert!(
        requests(&harness).iter().any(|r| matches!(
            r,
            Request::Search {
                page: Page::More,
                ..
            }
        )),
        "two cards is low"
    );
    let generation = view(&harness).generation;
    let filter = Box::new(view(&harness).filter.clone());
    harness.apply(Update::Searched {
        deck_id,
        generation,
        filter,
        page: Page::More,
        result: Ok(HttpDeckCardSearch {
            cards: Vec::new(),
            synergy_applied: true,
            related_from: None,
        }),
    });
    assert!(view(&harness).exhausted);
    harness.press(KeyCode::Char('h'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Warning, "End of results".to_owned())
    );
    assert!(screen_text(&harness.app).contains("u steps back into the pile"));
    harness.press(KeyCode::Char('u'));
    assert!(
        view(&harness).current().is_some(),
        "undo steps back into the pile"
    );
}

#[test]
fn an_empty_first_page_explains_synergy_and_a_stale_answer_is_dropped() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A")]);
    harness.press(KeyCode::Char('y'));
    assert!(!view(&harness).filter.synergy());
    harness.press(KeyCode::Char('y'));
    let generation = view(&harness).generation;
    harness.apply(Update::Searched {
        deck_id,
        generation: generation - 1,
        filter: Box::new(view(&harness).filter.clone()),
        page: Page::First,
        result: Ok(HttpDeckCardSearch {
            cards: vec![card("Stale")],
            synergy_applied: true,
            related_from: None,
        }),
    });
    assert!(
        view(&harness).stack.is_empty(),
        "an older search is ignored"
    );
    harness.apply(Update::Searched {
        deck_id,
        generation,
        filter: Box::new(view(&harness).filter.clone()),
        page: Page::First,
        result: Ok(HttpDeckCardSearch {
            cards: Vec::new(),
            synergy_applied: false,
            related_from: None,
        }),
    });
    let messages: Vec<String> = toasts(&harness).into_iter().map(|t| t.1).collect();
    assert!(messages.contains(&"Synergy warming up; showing all cards for now".to_owned()));
    assert!(messages.contains(
        &"No results for this filter; Synergy is warming up, so all cards were searched".to_owned()
    ));
}

#[test]
fn the_first_related_card_is_announced_once() {
    let (mut harness, deck_id) = Harness::swiping(Vec::new());
    harness.press(KeyCode::Char('r'));
    let generation = view(&harness).generation;
    harness.apply(Update::Searched {
        deck_id,
        generation,
        filter: Box::new(view(&harness).filter.clone()),
        page: Page::Refresh,
        result: Ok(HttpDeckCardSearch {
            cards: vec![card("Synergy"), card("Related"), card("Also")],
            synergy_applied: true,
            related_from: Some(1),
        }),
    });
    let notice = "That's every synergy pick. Showing cards like your deck".to_owned();
    harness.press(KeyCode::Char('h'));
    assert_eq!(last_toast(&harness), (ToastKind::Info, notice.clone()));
    harness.press(KeyCode::Char('h'));
    assert_eq!(toasts(&harness).iter().filter(|t| t.1 == notice).count(), 1);
}

#[test]
fn the_maybeboard_promotes_cycles_and_undoes() {
    let mut profile = deck_profile("Maybe");
    profile.commander_id = Some(Uuid::new_v4());
    profile.format = Some(Format::Commander);
    let mut d = deck(profile, vec![card("M1"), card("M2")]);
    for entry in &mut d.entries {
        entry.deck_card.board = Board::Maybeboard;
    }
    let deck_id = d.deck_profile.id;
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open(cx, &d));
    harness.press(KeyCode::Tab);
    assert_eq!(view(&harness).source, Source::Maybeboard);
    assert_eq!(view(&harness).mb_stack.len(), 2);
    assert!(!view(&harness).synergy_shown());

    harness.press(KeyCode::Char('m'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Info, "Already on maybeboard".to_owned())
    );
    harness.press(KeyCode::Char('h'));
    harness.press(KeyCode::Char('u'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Info, "Undid skip".to_owned())
    );

    harness.clear();
    harness.press(KeyCode::Char('l'));
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Success, "Moved to deck".to_owned())
    );
    assert_eq!(view(&harness).mb_stack.len(), 1);
    let Some(Request::Promote { card, .. }) = requests(&harness).first().copied() else {
        panic!("no promote");
    };
    let card = card.clone();
    harness.apply(Update::Promoted {
        deck_id,
        card: card.clone(),
    });
    assert_eq!(harness.app.shared.cards.undo.log(deck_id).len(), 1);

    harness.clear();
    harness.press(KeyCode::Char('u'));
    assert!(matches!(requests(&harness)[..], [Request::Demote { .. }]));
    assert_eq!(view(&harness).mb_stack.len(), 2, "the card is back on top");
    harness.apply(Update::Demoted {
        deck_id,
        card: card.clone(),
        taken: None,
        result: Ok(Box::new(deck_card(deck_id, &card, Board::Deck))),
    });
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Success, "Undid move to deck".to_owned())
    );
    assert_eq!(view(&harness).deck.maybeboard.len(), 2);

    harness.press(KeyCode::Tab);
    assert_eq!(view(&harness).source, Source::Search);
}

#[test]
fn leaving_parks_the_stack_and_coming_back_resumes() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A"), card("B"), card("C")]);
    harness.press(KeyCode::Char('h'));
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Deck(_)));
    let mut profile = deck_profile("Pile");
    profile.id = deck_id;
    profile.commander_id = Some(Uuid::new_v4());
    profile.format = Some(Format::Commander);
    let d = deck(profile, Vec::new());
    harness.clear();
    harness.open(|cx| open(cx, &d));
    assert_eq!(requests(&harness).len(), 0, "no refetch");
    assert_eq!(view(&harness).stack.index(), 1);
    assert_eq!(view(&harness).stack.history(), [AddAction::Skip]);
}

#[test]
fn land_and_budget_crossings_toast_on_add() {
    let mut profile = deck_profile("Lands");
    profile.commander_id = Some(Uuid::new_v4());
    profile.format = Some(Format::Commander);
    profile.land_target = Some(1);
    profile.price_target = Some(1.0);
    let d = deck(profile, Vec::new());
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open(cx, &d));
    let mut forest = card("Forest");
    forest.scryfall_data.type_line = Some("Basic Land — Forest".to_owned());
    forest.scryfall_data.prices.usd = Some("2.00".to_owned());
    harness.serve(vec![forest, card("B")]);
    harness.press(KeyCode::Char('l'));
    let messages: Vec<(ToastKind, String)> = toasts(&harness);
    assert!(messages.contains(&(ToastKind::Info, "Land target reached (1)".to_owned())));
    assert!(messages.contains(&(
        ToastKind::Warning,
        "Deck at 200.00% of your $1.00 budget".to_owned()
    )));
    assert!(view(&harness).deck.lands_at_target());
}

#[test]
fn the_filter_sheet_opens_applies_and_refetches() {
    let (mut harness, _) = Harness::swiping(vec![card("A")]);
    harness.press(KeyCode::Char('f'));
    assert!(view(&harness).sheet.is_some());
    assert!(screen_text(&harness.app).contains("Oracle text"));
    view_mut(&mut harness)
        .sheet
        .as_mut()
        .unwrap()
        .draft
        .set_name_contains("elf");
    harness.clear();
    harness.press(KeyCode::Char('a'));
    assert!(view(&harness).sheet.is_none());
    assert_eq!(view(&harness).filter.name_contains(), Some("elf"));
    assert!(view(&harness).filter_dot());
    assert!(matches!(
        requests(&harness)[..],
        [Request::Search {
            page: Page::First,
            ..
        }]
    ));
}

#[test]
fn details_open_printings_and_a_saved_printing_replaces_the_card() {
    let a = card("A");
    let (mut harness, _) = Harness::swiping(vec![a.clone()]);
    harness.press(KeyCode::Char('e'));
    assert!(screen_text(&harness.app).contains("printings"));
    harness.clear();
    harness.press(KeyCode::Char('p'));
    let oracle_id = a.scryfall_data.oracle_id.unwrap();
    assert!(matches!(
        harness.pending(),
        [screens::Request::CardSheets(screens::card_sheets::Request::Printings { oracle_id: o })] if *o == oracle_id
    ));
    let mut other = a.clone();
    other.scryfall_data.id = Uuid::new_v4();
    other.scryfall_data.set_name = "Other Set".to_owned();
    harness.apply(screens::card_sheets::Update::Printings {
        oracle_id,
        printings: Ok(vec![a, other.clone()]),
    });
    harness.press(KeyCode::Char('s'));
    assert!(view(&harness).printing.is_some(), "Save waits for a change");
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('s'));
    assert!(view(&harness).printing.is_none() && view(&harness).details.is_none());
    assert_eq!(
        last_toast(&harness),
        (ToastKind::Info, "Printing swapped".to_owned())
    );
    assert_eq!(
        view(&harness).current().unwrap().scryfall_data.id,
        other.scryfall_data.id
    );
}

#[test]
fn images_load_for_the_top_card_and_the_next_three() {
    let cards: Vec<Card> = (0..6)
        .map(|i| {
            let mut c = card(&i.to_string());
            c.scryfall_data.image_uris.as_mut().unwrap().normal =
                Some(format!("https://cards.scryfall.io/normal/front/{i}.jpg"));
            c
        })
        .collect();
    let (mut harness, _) = Harness::swiping(Vec::new());
    harness.app.shared.images = Some(Picker::halfblocks());
    harness.serve(cards);
    let urls: Vec<String> = requests(&harness)
        .into_iter()
        .filter_map(|r| match r {
            Request::Image { url } => {
                assert!(images::allowed(url), "{url}");
                Some(url.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(urls.len(), 4);
    let first = urls[0].clone();
    harness.apply(Update::Image {
        url: first.clone(),
        image: Some(Box::new(DynamicImage::new_rgb8(8, 11))),
    });
    assert!(matches!(
        view(&harness).images.get(&first),
        Some(CardImage::Ready(_))
    ));
    harness.press(KeyCode::Char('h'));
    assert!(
        !view(&harness).images.contains_key(&first),
        "the swiped card's image drops"
    );
}

/// Add for a deck called Simic Ramp with `card` up, in gruvbox dark.
fn add_app(card: Card) -> Harness {
    let (mut harness, _) = Harness::swiping(vec![card]);
    view_mut(&mut harness).deck_name = "Simic Ramp".to_owned();
    harness.app.shared.palette = Palette::for_account("gruvbox", true);
    harness.app.shared.toasts = crate::inbound::widgets::toast::Toasts::default();
    harness
}

#[test]
fn the_card_shows_its_text_info_and_colors_its_mana_by_theme() {
    let mut elves = card("Llanowar Elves");
    elves.scryfall_data.mana_cost = Some("{G}".to_owned());
    elves.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
    elves.scryfall_data.oracle_text = Some("{T}: Add {G}.".to_owned());
    elves.scryfall_data.power = Some("1".to_owned());
    elves.scryfall_data.toughness = Some("1".to_owned());
    let harness = add_app(elves);
    let terminal = render(&harness.app, 100, 40);
    let buffer = terminal.backend().buffer();
    let screen: String = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    for expected in [
        "Add deck cards",
        "Simic Ramp",
        "From",
        "Search",
        "Maybeboard",
        "Synergy",
        "Llanowar Elves",
        "Creature — Elf Druid",
        "{T}: Add {G}.",
        "1/1",
        "Set: ",
        "Released: ",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    let green = harness.app.shared.palette.mana[4].fg.unwrap();
    assert!(
        buffer
            .content()
            .iter()
            .any(|c| c.symbol() == "{" && c.fg == green),
        "the {{G}} cost is drawn in the theme's green"
    );
}

#[test]
fn the_add_screen_fits_a_tiny_terminal() {
    let mut harness = add_app(card("Tiny"));
    render(&harness.app, 12, 5);
    harness.press(KeyCode::Char('f'));
    render(&harness.app, 12, 5);
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('e'));
    render(&harness.app, 12, 5);
}

#[test]
fn a_loaded_image_draws_beside_the_card_text_on_a_wide_terminal() {
    let elves = card("Llanowar Elves");
    let url = elves
        .scryfall_data
        .face_image_url(
            0,
            zwipe_core::domain::card::scryfall_data::ImageSize::Normal,
        )
        .unwrap()
        .to_owned();
    let mut harness = add_app(elves);
    let picker = Picker::halfblocks();
    let red = image::RgbImage::from_pixel(40, 56, image::Rgb([255, 0, 0]));
    let protocol = picker.new_resize_protocol(DynamicImage::ImageRgb8(red));
    harness.app.shared.images = Some(picker);
    view_mut(&mut harness).images.insert(
        url.clone(),
        CardImage::Ready(Box::new(RefCell::new(protocol))),
    );
    let terminal = render(&harness.app, 160, 40);
    let buffer = terminal.backend().buffer();
    let red = Color::Rgb(255, 0, 0);
    assert!(buffer.content().iter().any(|c| c.fg == red || c.bg == red));

    view_mut(&mut harness).show_image = true;
    view_mut(&mut harness)
        .images
        .insert(url, CardImage::Loading);
    assert!(
        screen_text(&harness.app).contains("Loading image..."),
        "i swaps the text for the image"
    );
}

/// A motion on `card` by `direction` that started `millis` ago.
fn exit_since(card: Card, direction: Direction, millis: u64) -> Motion {
    let started = Instant::now()
        .checked_sub(Duration::from_millis(millis))
        .unwrap();
    CardMotion::exit(direction, Snapshot { card, face: 0 }, started)
}

#[test]
fn a_swipe_sends_the_card_off_and_the_screen_animates_until_it_lands() {
    let (mut harness, _) = Harness::swiping(vec![card("A"), card("B"), card("C")]);
    assert_eq!(view(&harness).motion, None);
    harness.press(KeyCode::Char('l'));
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(motion.direction(), Direction::Right);
    assert!(
        matches!(motion.phase(), Phase::Exit(s) if s.card.scryfall_data.name == "A"),
        "the exit carries the card that left"
    );
    assert!(
        harness.screen().animating(),
        "the loop redraws while it flies"
    );
    view_mut(&mut harness).motion = Some(exit_since(card("A"), Direction::Right, 180));
    assert!(!harness.screen().animating(), "it stops at the end");
}

#[test]
fn a_key_during_an_exit_acts_on_the_next_card_and_replaces_the_exit() {
    let (mut harness, _) = Harness::swiping(vec![card("A"), card("B"), card("C")]);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('h'));
    assert_eq!(view(&harness).stack.index(), 2, "the skip took B at once");
    assert_eq!(
        view(&harness)
            .current()
            .map(|c| c.scryfall_data.name.as_str()),
        Some("C")
    );
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(motion.direction(), Direction::Left);
    assert!(matches!(motion.phase(), Phase::Exit(s) if s.card.scryfall_data.name == "B"));
    harness.press(KeyCode::Char('m'));
    assert_eq!(
        view(&harness).motion.as_ref().map(Motion::direction),
        Some(Direction::Up),
        "up flies off the top"
    );
}

#[test]
fn undo_brings_the_card_back_from_the_side_it_left() {
    let (mut harness, _) = Harness::swiping(vec![card("A"), card("B")]);
    harness.press(KeyCode::Char('h'));
    harness.press(KeyCode::Char('u'));
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(motion.phase(), &Phase::Enter);
    assert_eq!(motion.direction(), Direction::Left);
    assert_eq!(
        view(&harness)
            .under()
            .map(|c| c.scryfall_data.name.as_str()),
        Some("B"),
        "the card it covers rests under it"
    );
    harness.press(KeyCode::Char('u'));
    assert_eq!(
        view(&harness).motion.as_ref().map(Motion::phase),
        Some(&Phase::Enter),
        "nothing to undo leaves the entrance alone"
    );
}

/// Where `symbol` is drawn when `harness` renders at 100 by 40.
fn cells_with(harness: &Harness, symbol: &str) -> Vec<(u16, u16)> {
    let terminal = render(&harness.app, 100, 40);
    let buffer = terminal.backend().buffer();
    let mut found = Vec::new();
    for y in 0..40 {
        for x in 0..100 {
            if buffer.cell((x, y)).unwrap().symbol() == symbol {
                found.push((x, y));
            }
        }
    }
    found
}

#[test]
fn the_exiting_card_draws_offset_over_the_next_one() {
    let (mut harness, _) = Harness::swiping(vec![card("A"), card("B")]);
    let rest = cells_with(&harness, "┌");
    assert_eq!(rest.len(), 1, "one card at rest: {rest:?}");
    let (rest_x, _) = rest[0];
    harness.press(KeyCode::Char('l'));
    harness.app.shared.toasts = crate::inbound::widgets::toast::Toasts::default();
    view_mut(&mut harness).motion = Some(exit_since(card("A"), Direction::Right, 60));
    let moving = cells_with(&harness, "┌");
    assert_eq!(moving.len(), 2, "B at rest and A flying: {moving:?}");
    assert!(
        moving.iter().any(|(x, _)| *x == rest_x),
        "B rests where A was"
    );
    assert!(
        moving.iter().any(|(x, _)| *x > rest_x),
        "A is drawn right of its rest: {moving:?}"
    );
    view_mut(&mut harness).motion = None;
    assert_eq!(
        cells_with(&harness, "┌"),
        rest,
        "once it lands only B is left"
    );
}

#[test]
fn sixel_and_iterm2_images_wait_for_the_motion_to_settle() {
    let elves = card("Llanowar Elves");
    let url = elves
        .scryfall_data
        .face_image_url(
            0,
            zwipe_core::domain::card::scryfall_data::ImageSize::Normal,
        )
        .unwrap()
        .to_owned();
    let mut harness = add_app(elves);
    let red_drawn = |harness: &Harness| {
        let terminal = render(&harness.app, 160, 40);
        let red = Color::Rgb(255, 0, 0);
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .any(|c| c.fg == red || c.bg == red)
    };
    let mut picker = Picker::halfblocks();
    let red = image::RgbImage::from_pixel(40, 56, image::Rgb([255, 0, 0]));
    let protocol = picker.new_resize_protocol(DynamicImage::ImageRgb8(red));
    view_mut(&mut harness)
        .images
        .insert(url, CardImage::Ready(Box::new(RefCell::new(protocol))));
    harness.app.shared.images = Some(picker.clone());
    view_mut(&mut harness).motion = Some(exit_since(card("Gone"), Direction::Left, 30));
    assert!(
        red_drawn(&harness),
        "half blocks are cells and stay under the motion"
    );

    picker.set_protocol_type(ProtocolType::Sixel);
    harness.app.shared.images = Some(picker);
    assert!(
        !red_drawn(&harness),
        "a sixel image waits while the card flies"
    );
    assert!(
        screen_text(&harness.app).contains("Llanowar Elves"),
        "the text stays while the image waits"
    );
}
