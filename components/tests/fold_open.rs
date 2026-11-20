//! A card list panel (`CardListFold`) can take its open state from its parent, so what sits
//! outside the panel (a Show more under it) can follow the fold.

use components::CardListFold;
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

#[test]
fn a_panel_handed_a_shut_state_draws_shut() {
    let html = render(|| {
        let open = use_signal(|| false);
        rsx! { CardListFold { title: "Suggested", open, p { "row" } } }
    });
    assert!(
        html.contains(r#"class="maybeboard-list card-group row-enter collapsed""#),
        "{html}"
    );
    assert!(html.contains(r#"aria-expanded="false""#), "{html}");
}

#[test]
fn a_panel_left_to_itself_starts_open() {
    let html = render(|| rsx! { CardListFold { title: "Suggested", p { "row" } } });
    assert!(html.contains(r#"aria-expanded="true""#), "{html}");
}
