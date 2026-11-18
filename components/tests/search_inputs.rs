//! A picker's search is the filter's compact input, the same height, padding, text size, radius
//! and border as Name contains.

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

#[test]
fn a_picker_s_search_is_the_filter_s_compact_input() {
    fn picker() -> Element {
        let query = use_signal(String::new);
        rsx! { components::PickerSearch { id: "send-to-search", placeholder: "Search decks", query } }
    }
    let html = render(picker);
    assert!(html.contains(r#"class="input input-compact""#), "{html}");
}
