//! Persistent, on-demand "?" help button.
//!
//! A pure sender: the shared inline "?" posts a [`HintTopic`](crate::inbound::components::hint_host::HintTopic) to the app-root
//! hint channel, and the single [`HintHost`](crate::inbound::components::hint_host::HintHost)
//! renders the dialog. It deliberately renders no dialog of its own, so it can
//! never be trapped by an ancestor's containing block (the reason inline
//! dialogs clipped to the content column). Not one-time; no session
//! dependency.

use dioxus::prelude::*;

use crate::inbound::components::hint_host::HintTopic;

/// Inline "?" glyph that opens the given concept's hint via the app-root host.
#[component]
pub fn InfoButton(topic: HintTopic) -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    rsx! {
        components::InfoButton { onclick: move |_| hint.set(Some(topic)) }
    }
}

/// A click handler that opens the given concept's hint via the app-root host,
/// for the `on_info` "?" of a kit field such as
/// [`TextInput`](components::TextInput).
pub fn open_topic(topic: HintTopic) -> EventHandler<MouseEvent> {
    let mut hint: Signal<Option<HintTopic>> = consume_context();
    EventHandler::new(move |_| hint.set(Some(topic)))
}
