//! The swipe card's ghost while a pile loads, or where it has run out, is a card in a stack of
//! its own, so the card's own sizing gives it the top card's box at every width.

use components::CardGhost;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_ghost_is_a_stack_card_in_a_stack_of_its_own() {
    let html = render(|| rsx! { CardGhost {} });
    assert!(
        html.starts_with(r#"<div class="swipe-stack web-swipe-stack card-ghost-stack" aria-hidden="true"><div class="swipe-stack-card swipe-stack-ghost-card" aria-hidden="true"><div class="flippable-card-wrapper card-image"><div class="flip-face"><img class="img-loaded card-ghost""#),
        "{html}"
    );
}

#[test]
fn an_empty_pile_s_ghost_says_why_inside_the_card_s_box() {
    let html = render(|| {
        rsx! {
            CardGhost { note: "No cards match your filters", detail: "Synergy is on" }
        }
    });
    let (_, face) = html
        .split_once(r#"<div class="flip-face">"#)
        .expect("the card's face");
    assert!(
        face.contains(r#"<div class="card-ghost-note"><span>No cards match your filters</span><span class="skeleton-note-detail">Synergy is on</span></div>"#),
        "{html}"
    );
}
