//! A split deck page shows only its profile on a phone, for a host that draws the cards in a layer
//! of its own; a page that doesn't split draws as it always has.

use components::DeckPage;
use dioxus::prelude::*;

fn page(split: bool) -> Element {
    rsx! {
        DeckPage {
            head: rsx! { h1 { "Head" } },
            controls: rsx! { p { "Controls" } },
            bands: rsx! { p { "Bands" } },
            stats: rsx! { p { "Stats" } },
            foot: rsx! { p { "Foot" } },
            split,
        }
    }
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn a_split_page_keeps_every_part_mounted_and_marks_itself() {
    let html = render(|| page(true));
    assert!(
        html.contains(r#"<div class="deck-page deck-page-split" data-settle-scope="">"#),
        "{html}"
    );
    for part in ["Head", "Controls", "Bands", "Stats", "Foot"] {
        assert!(html.contains(part), "{part} stays mounted: {html}");
    }
}

#[test]
fn a_page_that_does_not_split_draws_as_it_always_has() {
    let html = render(|| page(false));
    assert!(
        html.contains(r#"<div class="deck-page" data-settle-scope="">"#),
        "{html}"
    );
}

#[test]
fn a_phone_hides_a_split_page_s_cards_and_a_wide_window_shows_them() {
    let block = components::MAGIC_CSS
        .split("@media (max-width: 899px)")
        .skip(1)
        .filter_map(|rest| rest.get(..rest.find("\n}\n")?))
        .find(|block| block.contains(".deck-page-split"))
        .unwrap();
    for rule in [
        ".deck-page-split > .deck-controls",
        ".deck-page-split > .deck-bands",
        ".deck-page-split > .deck-empty",
    ] {
        assert!(block.contains(rule), "{rule} in {block}");
    }
}
