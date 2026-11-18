//! A card's quantity stepper wraps as one: when its row runs out of room, `-`, the count and `+`
//! move to the next line together, never split across two.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed render or a missing rule fails the test"
)]

use components::CardEditButtons;
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

/// The declarations of the first rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let (_, rest) = css.split_once(&format!("\n{selector} {{")).unwrap();
    rest.split_once('}').unwrap().0
}

#[test]
fn the_stepper_s_three_parts_sit_in_one_box() {
    let html = render(|| {
        rsx! {
            CardEditButtons { qty: 2, on_qty_change: Some(EventHandler::new(|_: i32| {})) }
        }
    });
    let (_, stepper) = html
        .split_once(r#"<span class="card-action-stepper">"#)
        .unwrap_or_else(|| panic!("a stepper box: {html}"));
    // The box closes after its `+` button.
    let stepper = stepper.split_once("</button></span>").unwrap().0;
    for part in [">-</button>", r#"class="card-action-count">2"#, ">+"] {
        assert!(stepper.contains(part), "{part} in the box: {stepper}");
    }
}

#[test]
fn the_box_wraps_whole_and_keeps_the_row_s_gap() {
    let box_rule = rule(components::MAGIC_CSS, ".card-action-stepper");
    for want in ["display: inline-flex;", "flex: none;", "gap: inherit;"] {
        assert!(box_rule.contains(want), "{want} in {box_rule}");
    }
}
