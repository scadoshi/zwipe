//! The deck page's hover keys mount on an editable deck and on a read-only one: a shared deck
//! passes no edit actions and answers only the keys that read.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed runtime build fails the test"
)]

use std::collections::HashMap;

use components::{DeckEditActions, DeckHoverKeys, use_deck_hover_keys};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_core::domain::deck::Board;

/// Mounts `app` and runs its effects.
fn mount(app: fn() -> Element) {
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
}

/// The hover keys of a deck with no rows, editable when `actions` is given.
fn keys(actions: Option<DeckEditActions>) -> DeckHoverKeys {
    DeckHoverKeys {
        row_of: Callback::new(|_| None),
        faces: Signal::new(HashMap::new()),
        details: None,
        actions,
        on_printing: Callback::new(|_| {}),
        on_image: Callback::new(|_| {}),
        other_keys: None,
    }
}

#[test]
fn the_hover_keys_mount_on_an_editable_deck() {
    mount(|| {
        use_deck_hover_keys(keys(Some(DeckEditActions {
            change_quantity: Callback::new(|_| {}),
            move_to_board: Callback::new(|_: (_, Board, bool)| {}),
            toggle_mvp: Callback::new(|_| {}),
        })));
        rsx! {}
    });
}

#[test]
fn the_hover_keys_mount_on_a_read_only_deck() {
    mount(|| {
        use_deck_hover_keys(keys(None));
        rsx! {}
    });
}
