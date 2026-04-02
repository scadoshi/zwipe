//! Mana cost and color filter component.

pub(crate) mod cmc;
pub(crate) mod color_identity;
pub(crate) mod produced_mana;

use dioxus::prelude::*;

use cmc::CmcFilter;
use color_identity::ColorIdentityFilter;
use produced_mana::ProducedManaFilter;

/// Filter component for mana cost and color identity.
#[component]
pub fn Mana() -> Element {
    rsx! {
        div { class: "flex-col gap-half",
            CmcFilter {}
            ColorIdentityFilter {}
            ProducedManaFilter {}
        }
    }
}
