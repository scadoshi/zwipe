//! A load that failed, with the way to try it again.

use dioxus::prelude::*;
use orangui::{Button, ButtonVariant};

/// The Retry button's label.
pub const RETRY: &str = "Retry";

/// `message` as a centered gray chip with a [`RETRY`] button under it, which
/// calls `on_retry`. `class` joins the block's own classes, for a host's
/// spacing.
#[component]
pub fn LoadFailed(
    message: &'static str,
    on_retry: EventHandler<()>,
    #[props(default)] class: &'static str,
) -> Element {
    rsx! {
        div { class: "message-empty load-failed {class}",
            span { class: "chip-note", {message} }
            Button {
                variant: ButtonVariant::Action,
                class: "load-failed-retry",
                onclick: move |_| on_retry.call(()),
                {RETRY}
            }
        }
    }
}
