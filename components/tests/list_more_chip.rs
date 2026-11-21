//! A card list's Show more under its panel, which Suggested and the Maybeboard share on the app and
//! the web: a chip in a bare box while the list offers more and its panel is open.

use components::ListMore;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn it_shows_as_a_chip_in_a_bare_box_while_the_list_offers_more() {
    fn app() -> Element {
        rsx! { ListMore { offers: true, open: true, leaves: true, onclick: |()| true } }
    }
    let html = render(app);
    assert!(
        html.contains(r#"<div class="card-list-more"><div><button type="button" class="chip">Show more</button></div></div>"#),
        "{html}"
    );
}

#[test]
fn it_hides_with_nothing_more_or_a_shut_panel() {
    fn none() -> Element {
        rsx! { ListMore { offers: false, open: true, leaves: true, onclick: |()| true } }
    }
    fn shut() -> Element {
        rsx! { ListMore { offers: true, open: false, leaves: true, onclick: |()| true } }
    }
    assert!(!render(none).contains("card-list-more"));
    assert!(!render(shut).contains("card-list-more"));
}

#[test]
fn its_exit_ends_on_either_leave_and_not_on_another_animation() {
    assert!(components::ends_list_more_leave("card-list-more-leave"));
    assert!(components::ends_list_more_leave("card-list-more-fade"));
    assert!(!components::ends_list_more_leave("row-enter"));
}
