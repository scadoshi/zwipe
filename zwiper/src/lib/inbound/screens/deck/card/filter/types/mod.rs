//! Card type filter component.

pub(crate) mod basic_types;
pub(crate) mod other_types;

use basic_types::BasicTypes;
use dioxus::prelude::*;
use other_types::OtherTypes;

/// Filter component for selecting card types (creature, instant, etc.).
#[component]
pub fn Types() -> Element {
    rsx! {
        div { class: "flex-col gap-half",
            BasicTypes {}
            OtherTypes {}
        }
    }
}
