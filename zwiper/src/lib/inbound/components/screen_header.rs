//! Shared screen header.
//!
//! Every screen opens with the same top bar: a centered title and, on screens
//! that have a hint dialog, a faded top-right "?" trigger. This wraps the
//! shared [`PageHeader`] with the app's help button on the left. The screen
//! still owns its `HintDialog` (with screen-specific content); passing its
//! `open` signal as `hint` just wires up the trigger button.

use crate::inbound::components::support::SupportButton;
use dioxus::prelude::*;
use zwipe_components::PageHeader;

/// Centered page title flanked by the always-present "!" help button (left) and
/// an optional "?" hint trigger (right).
///
/// Omit `hint` for screens without a hint dialog (the button isn't rendered).
#[component]
pub fn ScreenHeader(title: String, hint: Option<Signal<bool>>) -> Element {
    rsx! {
        PageHeader {
            title,
            leading: rsx! { SupportButton {} },
            on_hint: hint.map(|mut h| Callback::new(move |()| h.set(true))),
        }
    }
}
