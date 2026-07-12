//! Persistent, on-demand "?" help button.
//!
//! A pure sender: the inline "?" posts a [`HintTopic`] to the app-root hint
//! channel, and the single [`HintHost`](super::hint_host::HintHost) renders the
//! dialog. It deliberately renders no dialog of its own, so it can never be
//! trapped by an ancestor's containing block (the reason inline dialogs clipped
//! to the content column). Not one-time; no session dependency.

use crate::inbound::components::hint_host::HintTopic;
use dioxus::prelude::*;

/// Inline "?" glyph that opens the given concept's hint via the app-root host.
#[component]
pub fn InfoButton(topic: HintTopic) -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    rsx! {
        button {
            class: "info-button",
            r#type: "button",
            // stop_propagation so tapping "?" inside e.g. a filter accordion
            // header doesn't also toggle the accordion.
            onclick: move |evt| {
                evt.stop_propagation();
                hint.set(Some(topic));
            },
            "?"
        }
    }
}
