use crossterm::event::KeyCode;

use super::*;
use crate::domain::theme::Palette;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{render, screen_text};

fn on_privacy() -> Harness {
    let mut harness = Harness::new();
    harness.open(open);
    harness
}

#[test]
fn the_policy_opens_on_its_first_card_and_scrolls_to_contact() {
    let mut harness = on_privacy();
    let top = screen_text(&harness.app);
    assert!(top.contains("Privacy policy"));
    assert!(top.contains("Last updated October 2026"));
    assert!(top.contains("Magic: The Gathering (https://magic.wizards.com/en)"));
    for _ in 0..30 {
        harness.press(KeyCode::PageDown);
    }
    let bottom = screen_text(&harness.app);
    assert!(bottom.contains("Contact"));
    assert!(bottom.contains("support@zwipe.net"));
    harness.press(KeyCode::Home);
    assert!(matches!(harness.screen(), Screen::Privacy(view) if view.scroll == 0));
}

#[test]
fn every_section_has_its_heading() {
    let text: String = draw::lines(&Palette::terminal(), 60)
        .iter()
        .map(ToString::to_string)
        .collect();
    for heading in ["Data we collect", "Security", "Fan content", "Contact"] {
        assert!(text.contains(heading), "{heading}");
    }
    assert!(text.contains("• Account data: email address"));
}

#[test]
fn esc_goes_back_and_the_screen_fits_a_tiny_terminal() {
    let mut harness = on_privacy();
    for (width, height) in [(12, 5), (1, 1)] {
        render(&harness.app, width, height);
    }
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
}

#[test]
fn privacy_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&on_privacy().app, "Esc back");
}
