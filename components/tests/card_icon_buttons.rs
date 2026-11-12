//! The Image button is the card art icon, never the word: one button wherever a card's image
//! opens, the height of the text buttons beside it, named Image for a screen reader and a
//! tooltip.

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

#[test]
fn the_image_button_is_the_card_art_icon_named_image() {
    let html = render(|| rsx! { components::ImageButton { on_click: move |()| {} } });
    for part in [
        r#"class="card-action-btn card-action-icon""#,
        r#"type="button""#,
        r#"data-action="image""#,
        r#"aria-label="Image""#,
        r#"title="Image""#,
        r#"class="action-dock-icon""#,
        components::IMAGE_ICON,
    ] {
        assert!(html.contains(part), "{part} in {html}");
    }
    assert!(!html.contains(">Image<"), "no word on the button: {html}");
}

#[test]
fn its_icon_takes_the_text_buttons_line_so_the_heights_match() {
    let css = components::MAGIC_CSS;
    let at = css.find(".card-action-icon > .action-dock-icon {").unwrap();
    let rule = css.get(at..).unwrap();
    let rule = rule.get(..rule.find('}').unwrap()).unwrap();
    assert!(
        rule.contains("width: 1.2em;") && rule.contains("height: 1.2em;"),
        "{rule}"
    );
    let at = css.find(".card-action-btn.card-action-icon {").unwrap();
    let button = css.get(at..).unwrap();
    let button = button.get(..button.find('}').unwrap()).unwrap();
    assert!(
        button.contains("padding: calc(0.2rem - 0.1em) 0.4rem;"),
        "the larger icon takes its room from the padding, so the height stays the text's: {button}"
    );
}

#[test]
fn a_card_s_details_open_its_image_with_the_icon() {
    fn app() -> Element {
        let card: zwipe_core::domain::card::Card =
            serde_json::from_str(include_str!("../../zwipe-ui/src/test_support/card.json"))
                .unwrap();
        rsx! {
            components::CardDetailsPanel {
                card,
                on_image: Some(EventHandler::new(move |()| {})),
                on_printings: Some(EventHandler::new(move |()| {})),
            }
        }
    }
    let html = render(app);
    assert!(html.contains("card-action-icon"), "{html}");
    assert!(!html.contains(">Image</button>"), "{html}");
    assert!(html.contains(">Printings</button>"), "{html}");
}

#[test]
fn flip_and_meld_are_icons_named_for_a_screen_reader_and_never_alike() {
    fn app() -> Element {
        rsx! {
            components::CardIconButton { label: "Flip", path: components::icons::FLIP, action: "flip", on_click: move |()| {} }
            components::CardIconButton { label: "Meld", path: components::MELD, action: "meld", pressed: Some(true), on_click: move |()| {} }
        }
    }
    let html = render(app);
    for part in [
        r#"aria-label="Flip""#,
        r#"title="Flip""#,
        r#"data-action="flip""#,
        components::icons::FLIP,
        r#"class="card-action-btn card-action-icon selected""#,
        r#"aria-label="Meld""#,
        r#"aria-pressed="true""#,
        components::MELD,
    ] {
        assert!(html.contains(part), "{part} in {html}");
    }
    assert_ne!(components::MELD, components::icons::FLIP);
    assert_ne!(
        components::icons::FLIP,
        components::icons::REFRESH,
        "Flip never reads as Refresh"
    );
    assert!(
        !html.contains(">Flip<") && !html.contains(">Meld<"),
        "{html}"
    );
}

#[test]
fn the_printings_sheet_selects_with_a_check_named_select() {
    let src = include_str!("../src/printing_sheet.rs");
    assert!(src.contains("path: CHECK"), "the check icon");
    assert!(src.contains(r#""aria-label": "Select""#), "named Select");
    assert!(
        !src.contains("\"Select\"\n"),
        "no Select word on the button"
    );
}

#[test]
fn the_star_is_one_toggle_outline_off_and_filled_on() {
    fn off() -> Element {
        rsx! { components::StarButton { starred: false, on_toggle: move |()| {} } }
    }
    fn on() -> Element {
        rsx! { components::StarButton { starred: true, on_toggle: move |()| {} } }
    }
    let html = render(off);
    for part in [
        r#"aria-label="Star""#,
        r#"aria-pressed="false""#,
        components::STAR,
    ] {
        assert!(html.contains(part), "{part} in {html}");
    }
    let html = render(on);
    for part in [
        r#"aria-label="Unstar""#,
        r#"aria-pressed="true""#,
        "selected",
    ] {
        assert!(html.contains(part), "{part} in {html}");
    }
    assert!(
        !html.contains(">Unstar<") && !html.contains(">Star<"),
        "{html}"
    );
    assert!(
        components::MAGIC_CSS.contains(".card-action-icon[data-action=\"star\"][aria-pressed=\"true\"] > .action-dock-icon {\n    fill: currentColor;"),
        "the starred star is filled"
    );
    assert!(
        components::MAGIC_CSS.contains(".card-action-btn.card-action-icon[data-action=\"star\"][aria-pressed=\"true\"] {\n    border-color: var(--color-star);\n    color: var(--ink-color-star, var(--color-star));"),
        "in the star color"
    );
}

#[test]
fn every_star_wears_the_star_color() {
    for selector in [".card-row-mvp {", ".stat-chip.deck-head-card-role-mvp {"] {
        let at = components::MAGIC_CSS.find(selector).unwrap();
        let rule = components::MAGIC_CSS.get(at..).unwrap();
        let rule = rule.get(..rule.find('}').unwrap()).unwrap();
        assert!(
            rule.contains("var(--ink-color-star, var(--color-star))"),
            "{rule}"
        );
        assert!(!rule.contains("palette-6"), "{rule}");
    }
}

#[test]
fn the_image_icon_is_lucide_s_image_with_its_license_beside_it() {
    assert_eq!(
        components::IMAGE_ICON,
        "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM9 7a2 2 0 1 0 0 4 2 2 0 1 0 0-4zM21 15l-3.086-3.086a2 2 0 0 0-2.828 0L6 21",
        "Lucide's image: a square frame, a sun and a hill"
    );
    let notice = include_str!("../LICENSE-LUCIDE");
    for part in [
        "ISC License",
        "Lucide Contributors",
        "Cole Bemis",
        "Permission to use, copy, modify",
    ] {
        assert!(notice.contains(part), "{part} in the notice");
    }
    let source = include_str!("../src/card_icon_button.rs");
    for part in ["LICENSE-LUCIDE", "Lucide's `image`", "Lucide's `merge`"] {
        assert!(source.contains(part), "{part} named in card_icon_button.rs");
    }
}
