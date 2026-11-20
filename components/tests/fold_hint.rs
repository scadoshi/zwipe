//! A card list panel's `?` (`CardListFold`'s `hint`) follows its title, and the panel's own
//! actions (Swipe) keep the header's right end.

use components::CardListFold;
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

const SCREENS_B: &str = include_str!("../assets/screens/screens-b.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

fn rule(selector: &str) -> &'static str {
    SCREENS_B
        .split_once(&format!("\n{selector} {{"))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn the_hint_sits_after_the_title_and_before_the_actions() {
    let html = render(|| {
        rsx! {
            CardListFold {
                title: "Suggested",
                hint: rsx! { button { class: "info-button", "?" } },
                actions: rsx! { button { "Swipe" } },
                p { "row" }
            }
        }
    });
    let title = html.find("<span>Suggested</span></span>").unwrap();
    let hint = html.find(r#"<span class="card-list-fold-hint">"#).unwrap();
    let actions = html.find(r#"class="card-group-header-actions""#).unwrap();
    assert!(title < hint && hint < actions, "{html}");
    let hint_html = &html[hint..actions];
    assert!(hint_html.contains("info-button"), "{hint_html}");
}

#[test]
fn a_fold_with_no_hint_draws_no_hint_box() {
    let html = render(|| rsx! { CardListFold { title: "Maybeboard", p { "row" } } });
    assert!(!html.contains("card-list-fold-hint"), "{html}");
}

#[test]
fn the_hint_takes_the_free_room_after_it_so_the_actions_hold_the_right_end() {
    let hint = rule(".card-group-header-with-hint .card-list-fold-hint");
    assert!(hint.contains("margin-right: auto;"), "{hint}");
}
