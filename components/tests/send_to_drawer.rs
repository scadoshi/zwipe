//! Send to is a small choice, so at phone width it opens as the side drawer the card details
//! use there, along the right edge under the header, not as a screen over the page. The other
//! pickers keep their screen. Off the browser the width reads narrow, so these renders are the
//! phone's.

use components::{PickerSheet, SendToSheet};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

/// Renders `app` once its effects have run.
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
fn send_to_opens_as_the_side_drawer_at_phone_width() {
    let html = render(|| {
        let open = use_signal(|| true);
        rsx! {
            SendToSheet { open, decks: None, current: None, on_send: move |_| {} }
        }
    });
    assert!(
        html.contains("bottom-sheet show sheet-pulls drawer-every-width"),
        "Send to is the every-width drawer: {html}"
    );
    assert!(
        html.contains("sheet-form-drawer"),
        "inside the drawer's frame, which places it under the header: {html}"
    );
    assert!(
        !html.contains("picker-screen"),
        "no screen over the page: {html}"
    );
}

#[test]
fn a_picker_that_does_not_ask_keeps_its_screen_at_phone_width() {
    let html = render(|| {
        let open = use_signal(|| true);
        rsx! {
            PickerSheet { open, title: "Format", search_id: "format-search", on_back: move |()| {}, "choices" }
        }
    });
    assert!(
        html.contains("screen swipe-select-screen picker-screen show"),
        "{html}"
    );
    assert!(!html.contains("drawer-every-width"), "{html}");
}
