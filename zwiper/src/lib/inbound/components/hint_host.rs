//! App-root hint host: a sender/receiver for on-demand "?" help.
//!
//! An [`InfoButton`](crate::inbound::components::info_button::InfoButton)
//! anywhere in the tree posts a [`HintTopic`](zwipe_ui::hint::HintTopic) to
//! a context signal; this single
//! [`HintHost`](crate::inbound::components::hint_host::HintHost), mounted at
//! the app root, hands the topic's dialog to orangui's dialog host. A single
//! `Option` means only one hint shows at a time. The copy is zwipe-ui's
//! [`HintTopic::blocks`](zwipe_ui::hint::HintTopic::blocks), drawn by
//! [`HintBody`](components::hint::HintBody) for every topic.

use components::hint::HintBody;
use components::{AlertDialog, DialogAction};
use dioxus::prelude::*;
pub use zwipe_ui::hint::HintTopic;

/// The dialog body for `topic`: the model's copy.
fn body(topic: HintTopic) -> Element {
    rsx! { HintBody { blocks: topic.blocks() } }
}

/// The receiver: reads the hint channel and describes the current topic's dialog
/// to the app root's dialog host. Mount once at the app root, beside the router.
/// `spawn_upkeeper` provides the `Signal<Option<HintTopic>>` this reads.
#[component]
pub fn HintHost() -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    let mut open = use_signal(|| false);
    // A posted topic opens the dialog; closing it clears the channel.
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
            title: topic.map_or_default(HintTopic::title).to_string(),
            actions: vec![DialogAction::Dismiss("Got it".into())],
            if let Some(topic) = topic {
                {body(topic)}
            }
        }
    }
}
