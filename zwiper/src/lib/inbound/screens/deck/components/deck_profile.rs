use dioxus::prelude::*;
use zwipe_core::domain::{card::Card, deck::deck_profile::DeckProfile};

#[component]
pub(crate) fn DeckProfileSection(
    deck_profile: DeckProfile,
    commander: Option<Card>,
    /// Total card count (command-zone variants included) for the "Cards" row.
    card_count: usize,
) -> Element {
    let is_oathbreaker = deck_profile
        .format
        .as_ref()
        .is_some_and(|f| f.has_signature_spell());
    let commander_label = if is_oathbreaker {
        "Oathbreaker"
    } else {
        "Commander"
    };

    rsx! {
        div { class: "info-list",
            div { class: "card-header",
                span { class: "card-title", "Profile" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "Name" }
                span { class: "info-row-value", "{deck_profile.name}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "Cards" }
                span { class: "info-row-value", "{card_count}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "Format" }
                span { class: "info-row-value",
                    if let Some(fmt) = deck_profile.format {
                        { fmt.display_name().to_string() }
                    } else {
                        "None"
                    }
                }
            }
            if deck_profile.format.is_some_and(|f| f.has_commander()) {
                div { class: "info-row",
                    span { class: "info-row-label", "{commander_label}" }
                    span { class: "info-row-value",
                        if let Some(cmd) = commander {
                            { cmd.scryfall_data.name }
                        } else if let Some(name) = &deck_profile.commander_name {
                            { name.clone() }
                        } else {
                            "None"
                        }
                    }
                }
            }
            if let Some(name) = &deck_profile.partner_commander_name {
                div { class: "info-row",
                    span { class: "info-row-label", "Partner" }
                    span { class: "info-row-value", { name.clone() } }
                }
            }
            if let Some(name) = &deck_profile.background_name {
                div { class: "info-row",
                    span { class: "info-row-label", "Background" }
                    span { class: "info-row-value", { name.clone() } }
                }
            }
            if let Some(name) = &deck_profile.signature_spell_name {
                div { class: "info-row",
                    span { class: "info-row-label", "Signature spell" }
                    span { class: "info-row-value", { name.clone() } }
                }
            }
            if let Some(pl) = deck_profile.power_level {
                div { class: "info-row",
                    span { class: "info-row-label", "Power level" }
                    span { class: "info-row-value", { pl.display_name().to_string() } }
                }
            }
        }
    }
}
