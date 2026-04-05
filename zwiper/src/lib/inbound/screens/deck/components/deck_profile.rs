use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::deck_profile::DeckProfile;

#[component]
pub(crate) fn DeckProfileSection(deck_profile: DeckProfile, commander: Option<Card>) -> Element {
    let is_oathbreaker = deck_profile
        .format
        .as_ref()
        .is_some_and(|f| f.has_signature_spell());
    let commander_label = if is_oathbreaker {
        "oathbreaker"
    } else {
        "commander"
    };

    rsx! {
        label { class: "label", "profile" }
        div { class: "info-list",
            div { class: "info-row",
                span { class: "info-row-label", "name" }
                span { class: "info-row-value", "{deck_profile.name}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "format" }
                span { class: "info-row-value",
                    if let Some(fmt) = deck_profile.format {
                        { fmt.display_name().to_lowercase() }
                    } else {
                        "none"
                    }
                }
            }
            if deck_profile.format.is_some_and(|f| f.has_commander()) {
                div { class: "info-row",
                    span { class: "info-row-label", "{commander_label}" }
                    span { class: "info-row-value",
                        if let Some(cmd) = commander {
                            { cmd.scryfall_data.name.to_lowercase() }
                        } else if let Some(name) = &deck_profile.commander_name {
                            { name.to_lowercase() }
                        } else {
                            "none"
                        }
                    }
                }
            }
            if let Some(name) = &deck_profile.partner_commander_name {
                div { class: "info-row",
                    span { class: "info-row-label", "partner" }
                    span { class: "info-row-value", { name.to_lowercase() } }
                }
            }
            if let Some(name) = &deck_profile.background_name {
                div { class: "info-row",
                    span { class: "info-row-label", "background" }
                    span { class: "info-row-value", { name.to_lowercase() } }
                }
            }
            if let Some(name) = &deck_profile.signature_spell_name {
                div { class: "info-row",
                    span { class: "info-row-label", "signature spell" }
                    span { class: "info-row-value", { name.to_lowercase() } }
                }
            }
        }
    }
}
