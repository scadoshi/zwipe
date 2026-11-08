//! Playstyle's identity chips: an identity drawn as its name and pips, and
//! the Show row that narrows the list and the Swipe pile to one identity.

use components::Chip;
use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::{Color, identity_name};
use zwipe_ui::playstyle::identity_label;

/// A chip's stable key: the identity's codes, `C` for colorless.
pub(crate) fn identity_key(identity: &[Color]) -> String {
    if identity.is_empty() {
        "C".to_string()
    } else {
        identity.iter().map(Color::to_short_name).collect()
    }
}

/// An identity's name, then its pips in the deck list's group-header
/// grammar, then `count` when given. A single color is its pip alone;
/// colorless is the name alone. The pips are hidden from a screen reader,
/// which reads the name in their place when it isn't shown.
#[component]
pub fn IdentityPips(
    identity: Vec<Color>,
    #[props(default)] count: Option<String>,
    #[props(default = true)] show_name: bool,
) -> Element {
    let name = identity_label(&identity).filter(|_| show_name);
    let spoken = identity_name(&identity);
    rsx! {
        span { class: "identity-pips",
            if let Some(name) = name {
                span { "{name}" }
            } else if !identity.is_empty() {
                span { class: "sr-only", "{spoken}" }
            }
            for color in identity.iter() {
                i {
                    key: "{color.to_short_name()}",
                    class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                    "aria-hidden": "true",
                }
            }
            if let Some(count) = count {
                span { "{count}" }
            }
        }
    }
}

/// The Show chip row: the label, `All`, one chip per identity in
/// `identities`, then `children` (the screen's `Art` toggle). Single select
/// over `active_identity`, `None` being All; the selected chip tapped again
/// is All. The screen and Swipe select's filter render it over the same
/// signal, so a tap in either filters both.
#[component]
pub fn ShowIdentityChips(
    mut active_identity: Signal<Option<Vec<Color>>>,
    identities: Vec<Vec<Color>>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
            Chip {
                selected: active_identity().is_none(),
                onclick: move |_| active_identity.set(None),
                "All"
            }
            for identity in identities {
                Chip {
                    key: "{identity_key(&identity)}",
                    selected: active_identity().as_deref() == Some(identity.as_slice()),
                    onclick: {
                        let identity = identity.clone();
                        move |_| {
                            if active_identity().as_deref() == Some(identity.as_slice()) {
                                active_identity.set(None);
                            } else {
                                active_identity.set(Some(identity.clone()));
                            }
                        }
                    },
                    IdentityPips { identity: identity.clone() }
                }
            }
            {children}
        }
    }
}
