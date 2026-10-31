use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, render, screen_text};

fn preview(harness: &Harness) -> &ImagePreview {
    match harness.screen() {
        Screen::ImagePreview(view) => view,
        other => panic!("not the preview: {other:?}"),
    }
}

#[test]
fn with_images_off_the_preview_shows_the_text_and_esc_closes_it() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut elves = card("Llanowar Elves");
    elves.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
    harness.open(|cx| open(cx, elves, 0));
    assert!(
        harness.pending().is_empty(),
        "no image to fetch with images off"
    );
    assert!(matches!(preview(&harness).image, FaceImage::None));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Llanowar Elves"));
    assert!(screen.contains("Esc close"));
    assert!(!screen.contains("f flip"), "a single face does not flip");
    harness.press(KeyCode::Char('f'));
    assert_eq!(preview(&harness).face, 0);
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn a_late_image_for_another_face_is_ignored() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open(cx, card("Sol Ring"), 0));
    harness.apply(Update::Image {
        face: 1,
        image: None,
    });
    assert!(matches!(preview(&harness).image, FaceImage::None));
}

#[test]
fn the_preview_fits_a_tiny_terminal() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open(cx, card("Sol Ring"), 0));
    render(&harness.app, 8, 3);
}
