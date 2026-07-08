//! Shared screen header.
//!
//! Every screen opens with the same top bar: a centered title and, on screens
//! that have a hint dialog, a faded top-right "?" trigger. This component is the
//! single source for that markup. The screen still owns its `HintDialog` (with
//! screen-specific content); passing its `open` signal as `hint` just wires up
//! the trigger button.

use crate::inbound::components::support::SupportButton;
use dioxus::prelude::*;
use zwipe_components::{Button, ButtonVariant};

/// Centered page title flanked by the always-present "!" help button (left) and
/// an optional "?" hint trigger (right).
///
/// Omit `hint` for screens without a hint dialog (the button isn't rendered).
#[component]
pub fn ScreenHeader(title: String, hint: Option<Signal<bool>>) -> Element {
    rsx! {
        div { class: "page-header",
            SupportButton {}
            h2 { "{title}" }
            if let Some(mut hint) = hint {
                Button {
                    variant: ButtonVariant::Util,
                    class: "page-header-corner",
                    onclick: move |_| hint.set(true),
                    "?"
                }
            }
        }
    }
}
