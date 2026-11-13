//! A swipe surface with a control of its own beside the swipes, such as the Oracle tag examples'
//! details eye, draws them as one dock: the swipes, a rule, then the control.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed runtime build fails the test"
)]

use components::swipe_stack::{Swipe, SwipeDock, use_swipe_control};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

/// Renders `app` with its effects run.
fn render(app: fn() -> Element) -> String {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let _runtime = runtime.enter();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

fn with_tail() -> Element {
    let control = use_swipe_control();
    rsx! {
        SwipeDock {
            control,
            buttons: vec![(Swipe::Left, "Next"), (Swipe::Down, "Back one")],
            card_on_top: true,
            on_undo: |()| {},
            tail: rsx! {
                button { class: "action-dock-btn eye", "eye" }
            },
        }
    }
}

#[test]
fn a_tail_joins_the_swipes_dock_after_a_rule() {
    let html = render(with_tail);
    assert_eq!(
        html.matches("class=\"action-dock web-swipe-dock\"").count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"action-dock ").count(),
        1,
        "one dock: {html}"
    );
    let next = html.find("Back one").unwrap();
    let rule = html.find("action-dock-divider").unwrap();
    let eye = html.find("action-dock-btn eye").unwrap();
    assert!(next < rule && rule < eye, "swipes, rule, eye: {html}");
}
