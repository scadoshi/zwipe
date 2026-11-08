//! A group header's color identity: the mana symbols on screen, and the
//! identity's name ("Mono-red", "Azorius") for a screen reader in their
//! place.

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::{Color, identity_name};

/// The `colors` as mana symbols, hidden from a screen reader, which reads the
/// identity's name instead, followed by a space so the header's count reads
/// as its own word.
#[component]
pub fn IdentityPips(colors: Vec<Color>) -> Element {
    let name = identity_name(&colors);
    rsx! {
        span { class: "identity-pips", "aria-hidden": "true",
            for color in colors.iter() {
                i {
                    key: "{color.to_short_name()}",
                    class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                }
            }
        }
        span { class: "sr-only", "{name} " }
    }
}
