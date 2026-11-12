//! A deck page splits into its profile and its cards on a phone, under a switch under its head,
//! for a host that asks; a page that doesn't ask draws as it always has.

use std::cell::Cell;

use components::DeckPage;
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_ui::deck::view::DeckArea;

thread_local! {
    static AREA: Cell<Option<Signal<DeckArea>>> = const { Cell::new(None) };
}

fn page(areas: Option<Signal<DeckArea>>) -> Element {
    rsx! {
        DeckPage {
            head: rsx! { h1 { "Head" } },
            controls: rsx! { p { "Controls" } },
            bands: rsx! { p { "Bands" } },
            stats: rsx! { p { "Stats" } },
            foot: rsx! { p { "Foot" } },
            areas,
        }
    }
}

fn with_areas() -> Element {
    let area = use_signal(DeckArea::default);
    AREA.with(|a| a.set(Some(area)));
    page(Some(area))
}

fn render(dom: &mut VirtualDom) -> String {
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(dom)
}

#[test]
fn a_page_with_areas_opens_on_its_profile_with_a_switch_under_its_head() {
    let mut dom = VirtualDom::new(with_areas);
    dom.rebuild_in_place();
    let html = render(&mut dom);
    assert!(
        html.contains(r#"class="deck-page deck-page-areas area-profile""#),
        "{html}"
    );
    let head = html.find("Head").unwrap();
    let switch = html.find("deck-area-switch").unwrap();
    let controls = html.find("Controls").unwrap();
    assert!(
        head < switch && switch < controls,
        "the switch sits under the head: {html}"
    );
    assert!(html.contains(r#"class="chip selected""#), "{html}");
    for part in ["Profile", "Cards", "Bands", "Stats", "Foot"] {
        assert!(
            html.contains(part),
            "every part stays mounted: {part} in {html}"
        );
    }
}

#[test]
fn picking_cards_marks_the_page_so_a_phone_shows_the_cards() {
    let mut dom = VirtualDom::new(with_areas);
    dom.rebuild_in_place();
    render(&mut dom);
    let mut area = AREA.with(Cell::get).unwrap();
    dom.in_runtime(|| area.set(DeckArea::Cards));
    let html = render(&mut dom);
    assert!(
        html.contains(r#"class="deck-page deck-page-areas area-cards""#),
        "{html}"
    );
}

#[test]
fn a_page_without_areas_draws_as_it_always_has() {
    let mut dom = VirtualDom::new(|| page(None));
    dom.rebuild_in_place();
    let html = render(&mut dom);
    assert!(
        html.contains(r#"<div class="deck-page" data-settle-scope="">"#),
        "{html}"
    );
    assert!(!html.contains("deck-area-switch"), "{html}");
}

#[test]
fn a_phone_hides_the_area_not_picked_and_a_wide_window_shows_both() {
    assert!(
        components::MAGIC_CSS
            .contains(".deck-page-areas > .deck-area-switch {\n    display: none;"),
        "the wide layout hides the switch at a .chip-row's strength and more"
    );
    let block = components::MAGIC_CSS
        .split("@media (max-width: 899px)")
        .skip(1)
        .filter_map(|rest| rest.get(..rest.find("\n}\n")?))
        .find(|block| block.contains(".deck-page-areas"))
        .unwrap();
    for rule in [
        ".deck-page-areas > .deck-area-switch",
        ".deck-page-areas.area-profile > .deck-controls",
        ".deck-page-areas.area-profile > .deck-bands",
        ".deck-page-areas.area-cards > .deck-stats",
        ".deck-page-areas.area-cards > .deck-foot",
    ] {
        assert!(block.contains(rule), "{rule} in {block}");
    }
}
