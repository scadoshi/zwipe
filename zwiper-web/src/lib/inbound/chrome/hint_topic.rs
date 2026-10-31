//! The `?` hints: the one root host that draws the open topic's dialog,
//! and the `?` button that posts one.
//!
//! A `?` posts its [`HintTopic`] into a root `Signal<Option<HintTopic>>`, and
//! [`HintHost`] draws it as a dialog with one `Got it` button. One hint shows
//! at a time; none is recorded, so a `?` opens its hint every time. The
//! topics and their copy are zwipe-ui's.

use components::{AlertDialog, DialogAction};
use dioxus::prelude::*;
use zwipe_ui::hint::GOT_IT;
pub use zwipe_ui::hint::HintTopic;

use crate::inbound::chrome::hint_body::HintBody;

/// Draws the posted topic as a dialog and clears the channel when it
/// closes. Mount once at the root, beside the router, under the context that
/// holds the `Signal<Option<HintTopic>>`.
#[component]
pub fn HintHost() -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    let mut open = use_signal(|| false);
    use_effect(move || {
        if hint().is_some() {
            open.set(true);
        }
    });
    use_effect(move || {
        if !open() {
            hint.set(None);
        }
    });
    let topic = hint();
    rsx! {
        AlertDialog {
            open,
            title: topic.map(HintTopic::title).unwrap_or_default().to_string(),
            actions: vec![DialogAction::Dismiss(GOT_IT.into())],
            if let Some(topic) = topic {
                HintBody { blocks: topic.blocks() }
            }
        }
    }
}

/// The inline `?` beside a field or row label. It posts `topic` to the root
/// host and stops the click there.
#[component]
pub fn InfoButton(topic: HintTopic) -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    rsx! {
        components::InfoButton { onclick: move |_| hint.set(Some(topic)) }
    }
}

/// A click handler that posts `topic`, for the `on_info` `?` of a kit field
/// such as [`TextInput`](components::TextInput).
pub fn open_topic(topic: HintTopic) -> EventHandler<MouseEvent> {
    let mut hint: Signal<Option<HintTopic>> = consume_context();
    EventHandler::new(move |_| hint.set(Some(topic)))
}
