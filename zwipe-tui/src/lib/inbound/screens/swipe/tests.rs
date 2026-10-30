use std::collections::HashSet;

use crossterm::event::KeyCode;
use ratatui::style::Color;
use ratatui_image::picker::Picker;
use zwipe_core::domain::card::Card;

use super::*;
use crate::domain::swipe::Swipe;
use crate::domain::theme::Palette;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck_profile, render, screen_text};

fn send(deck_id: Uuid, effect: Effect) -> screens::Request {
    Request::Send { deck_id, effect }.into()
}

fn page(cards: Vec<Card>) -> HttpDeckCardSearch {
    HttpDeckCardSearch {
        cards,
        synergy_applied: false,
        related_from: None,
    }
}

#[test]
fn swipes_send_their_effects_and_undo_reverses_the_count() {
    let (a, b) = (card("A"), card("B"));
    let (mut harness, deck_id) = Harness::swiping(vec![a.clone(), b.clone()]);

    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('h'));
    assert_eq!(harness.swipe_view().added, 1);
    assert!(matches!(
        harness.pending(),
        [
            screens::Request::Swipe(Request::Send {
                effect: Effect::Add {
                    maybeboard: false,
                    ..
                },
                ..
            }),
            screens::Request::Swipe(Request::Send {
                effect: Effect::Skip { .. },
                ..
            }),
        ]
    ));

    harness.clear();
    harness.press(KeyCode::Char('u'));
    harness.press(KeyCode::Char('u'));
    assert_eq!(harness.swipe_view().added, 0);
    assert_eq!(
        harness.pending(),
        [
            send(
                deck_id,
                Effect::Unskip {
                    oracle_id: b.scryfall_data.oracle_id.unwrap()
                }
            ),
            send(
                deck_id,
                Effect::Remove {
                    scryfall_data_id: a.scryfall_data.id
                }
            ),
        ]
    );
}

#[test]
fn a_short_page_asks_for_the_next_one_once() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A")]);
    assert_eq!(
        harness.pending(),
        [],
        "the helper clears the follow-up fetch"
    );

    if let Screen::Swipe(view) = harness.screen_mut() {
        view.fetching = false;
    }
    harness.apply(Update::Cards {
        deck_id,
        page: page(vec![card("B")]),
    });
    assert_eq!(
        harness.pending().len(),
        1,
        "two cards still leave the stack low"
    );

    harness.clear();
    harness.apply(Update::Cards {
        deck_id,
        page: page(Vec::new()),
    });
    assert_eq!(harness.pending(), [], "an empty page ends the stack");
    assert!(harness.swipe_view().stack.as_ref().unwrap().is_exhausted());
}

#[test]
fn swiping_with_no_card_does_nothing() {
    let (mut harness, _) = Harness::swiping(Vec::new());
    harness.press(KeyCode::Char('l'));
    assert_eq!(harness.swipe_view().added, 0);
    assert!(
        !harness
            .pending()
            .iter()
            .any(|r| matches!(r, screens::Request::Swipe(Request::Send { .. })))
    );
}

#[test]
fn images_load_around_the_current_card_and_drop_behind_it() {
    let cards: Vec<Card> = (0..6).map(|i| card(&i.to_string())).collect();
    let ids: Vec<Uuid> = cards.iter().map(|c| c.scryfall_data.id).collect();
    let (mut harness, deck_id) = Harness::swiping(Vec::new());
    harness.app.shared.images = Some(Picker::halfblocks());
    harness.apply(Update::Cards {
        deck_id,
        page: page(cards),
    });
    let requested = |harness: &Harness| -> Vec<Uuid> {
        harness
            .pending()
            .iter()
            .filter_map(|r| match r {
                screens::Request::Swipe(Request::Image { card_id, url }) => {
                    assert!(images::allowed(url), "{url}");
                    Some(*card_id)
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(
        requested(&harness),
        ids[..4],
        "the card on screen and three ahead"
    );

    harness.clear();
    for _ in 0..3 {
        harness.press(KeyCode::Char('h'));
    }
    assert_eq!(requested(&harness), ids[4..6]);
    let held: HashSet<Uuid> = harness.swipe_view().images.keys().copied().collect();
    assert_eq!(
        held,
        ids[2..6].iter().copied().collect(),
        "one behind, the rest dropped"
    );

    let image = DynamicImage::new_rgb8(8, 11);
    harness.apply(Update::Image {
        card_id: ids[3],
        image: Some(Box::new(image)),
    });
    harness.apply(Update::Image {
        card_id: ids[4],
        image: None,
    });
    let view = harness.swipe_view();
    assert!(matches!(
        view.images.get(&ids[3]),
        Some(CardImage::Ready(_))
    ));
    assert!(matches!(view.images.get(&ids[4]), Some(CardImage::Failed)));
}

#[test]
fn without_images_no_image_is_requested_and_i_still_toggles() {
    let (mut harness, _) = Harness::swiping(vec![card("A")]);
    harness.press(KeyCode::Char('h'));
    assert!(
        !harness
            .pending()
            .iter()
            .any(|r| matches!(r, screens::Request::Swipe(Request::Image { .. })))
    );
    harness.press(KeyCode::Char('i'));
    assert!(harness.swipe_view().show_image);
}

/// The swipe screen for a deck called Simic Ramp with `card` up, in gruvbox dark.
fn swipe_app(card: Card) -> Harness {
    let mut stack = Stack::new(HashSet::default());
    stack.push_page(HttpDeckCardSearch {
        cards: vec![card],
        synergy_applied: true,
        related_from: None,
    });
    let mut view = SwipeView::opening(deck_profile("Simic Ramp"));
    view.stack = Some(stack);
    let mut harness = Harness::on(Screen::Swipe(Box::new(view)));
    harness.app.shared.palette = Palette::for_account("gruvbox", true);
    harness
}

#[test]
fn the_card_shows_its_text_and_colors_its_mana_by_theme() {
    let mut elves = card("Llanowar Elves");
    elves.scryfall_data.mana_cost = Some("{G}".to_owned());
    elves.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
    elves.scryfall_data.oracle_text = Some("{T}: Add {G}.".to_owned());
    elves.scryfall_data.power = Some("1".to_owned());
    elves.scryfall_data.toughness = Some("1".to_owned());
    let harness = swipe_app(elves);

    let terminal = render(&harness.app, 100, 40);
    let buffer = terminal.backend().buffer();
    let screen: String = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    for expected in [
        "Simic Ramp",
        "Llanowar Elves",
        "Creature — Elf Druid",
        "{T}: Add {G}.",
        "1/1",
        "1 left",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }

    let green = harness.app.shared.palette.mana[4].fg.unwrap();
    let cost = buffer
        .content()
        .iter()
        .position(|c| c.symbol() == "{" && c.fg == green)
        .is_some();
    assert!(
        cost,
        "the {{G}} cost is drawn in the theme's green mana color"
    );
}

#[test]
fn a_theme_paints_every_cell() {
    let harness = swipe_app(card("Anything"));
    let background = Color::Rgb(0x28, 0x28, 0x28);
    assert!(
        render(&harness.app, 80, 30)
            .backend()
            .buffer()
            .content()
            .iter()
            .all(|c| c.bg == background)
    );
}

#[test]
fn an_exhausted_deck_says_so() {
    let mut harness = swipe_app(card("Last"));
    if let Screen::Swipe(view) = harness.screen_mut() {
        let stack = view.stack.as_mut().unwrap();
        stack.swipe(Swipe::Pass);
        stack.push_page(HttpDeckCardSearch {
            cards: Vec::new(),
            synergy_applied: true,
            related_from: None,
        });
    }
    assert!(screen_text(&harness.app).contains("No more cards match this deck."));
}

#[test]
fn the_swipe_screen_fits_a_tiny_terminal() {
    render(&swipe_app(card("Tiny")).app, 12, 5);
}

#[test]
fn a_loaded_image_draws_beside_the_card_text_on_a_wide_terminal() {
    let elves = card("Llanowar Elves");
    let id = elves.scryfall_data.id;
    let mut harness = swipe_app(elves);
    let picker = Picker::halfblocks();
    let red = image::RgbImage::from_pixel(40, 56, image::Rgb([255, 0, 0]));
    let protocol = picker.new_resize_protocol(DynamicImage::ImageRgb8(red));
    harness.app.shared.images = Some(picker);
    if let Screen::Swipe(view) = harness.screen_mut() {
        view.images
            .insert(id, CardImage::Ready(Box::new(RefCell::new(protocol))));
    }
    let terminal = render(&harness.app, 160, 40);
    let buffer = terminal.backend().buffer();
    let screen: String = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(screen.contains("Llanowar Elves"), "text beside the image");
    let red = Color::Rgb(255, 0, 0);
    assert!(
        buffer.content().iter().any(|c| c.fg == red || c.bg == red),
        "the image's pixels are drawn"
    );

    let narrow = screen_text(&harness.app);
    assert!(
        narrow.contains("Llanowar Elves"),
        "a narrow terminal shows the text first"
    );

    if let Screen::Swipe(view) = harness.screen_mut() {
        view.show_image = true;
        view.images.insert(id, CardImage::Loading);
    }
    let narrow_image = screen_text(&harness.app);
    assert!(narrow_image.contains("Loading image..."));
    assert!(
        !narrow_image.contains("Llanowar Elves"),
        "i swaps the text for the image"
    );
}
