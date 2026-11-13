//! The shortcuts sheet lists the keys of the newest screen that offers it, and the keys every page
//! answers while none does.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed runtime build fails the test"
)]

use components::keys::{ShortcutsList, ShortcutsSheet, use_offer_shortcuts, use_shortcuts_root};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

/// Renders `app` once its effects have run, inside the runtime the kit's sheets time their
/// entrance on.
fn render(app: fn() -> Element) -> String {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let _runtime = runtime.enter();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// A screen whose keys are `p` and `d`.
#[component]
fn Screen() -> Element {
    use_offer_shortcuts(ShortcutsList {
        rows: vec![(&["p"], "Printings"), (&["d"], "Card details")],
        note: Some("Hover a card"),
        claims_steps: false,
    });
    rsx! {}
}

#[test]
fn the_sheet_lists_the_offering_screen_s_keys_and_note() {
    fn app() -> Element {
        let open = use_shortcuts_root(None);
        rsx! {
            Screen {}
            ShortcutsSheet { open }
        }
    }
    let html = render(app);
    for text in [
        "Printings",
        "Card details",
        "Hover a card",
        "Keyboard shortcuts",
    ] {
        assert!(html.contains(text), "{text}: {html}");
    }
}

#[test]
fn with_no_screen_offering_keys_the_sheet_lists_the_keys_every_page_answers() {
    fn app() -> Element {
        let open = use_shortcuts_root(None);
        rsx! {
            ShortcutsSheet { open }
        }
    }
    let html = render(app);
    for text in [
        "Keyboard shortcuts",
        "These shortcuts",
        "Move between themes in Themes",
        "Pick the theme in Themes",
    ] {
        assert!(html.contains(text), "{text}: {html}");
    }
    assert!(!html.contains("Printings"), "{html}");
}
