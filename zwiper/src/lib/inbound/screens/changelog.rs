//! Changelog screen.
//!
//! Wraps the shared [`zwipe_components::Changelog`] (the same release history
//! shown on the website) in the app's screen chrome. Reached from Profile.

use crate::inbound::components::screen_header::ScreenHeader;
use dioxus::prelude::*;
use zwipe_components::{ActionBar, Button, ButtonVariant, Changelog as ChangelogContent};

/// Full release history, reachable from the Profile screen.
#[component]
pub fn Changelog() -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Changelog" }

            div { class: "screen-content content-enter",
                div { class: "changelog-screen",
                    ChangelogContent {}
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
            }
        }
    }
}
