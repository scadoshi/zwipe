//! Combat stats filter component (power/toughness).

pub(crate) mod power;
pub(crate) mod toughness;

use dioxus::prelude::*;

use power::PowerFilter;
use toughness::ToughnessFilter;

/// Filter component for card power and toughness values.
#[component]
pub fn Combat() -> Element {
    rsx! {
        div { class: "flex-col gap-half",
            PowerFilter {}
            ToughnessFilter {}
        }
    }
}
