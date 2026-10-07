//! Persistent, on-demand "?" help button.
//!
//! A pure sender: the shared inline "?" posts a [`HintTopic`] to the app-root
//! hint channel, and the single [`HintHost`](super::hint_host::HintHost)
//! renders the dialog. It deliberately renders no dialog of its own, so it can
//! never be trapped by an ancestor's containing block (the reason inline
//! dialogs clipped to the content column). Not one-time; no session
//! dependency.

use crate::inbound::components::hint_host::HintTopic;
use dioxus::prelude::*;

/// Inline "?" glyph that opens the given concept's hint via the app-root host.
#[component]
pub fn InfoButton(topic: HintTopic) -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    rsx! {
        zwipe_components::InfoButton { onclick: move |_| hint.set(Some(topic)) }
    }
}
