use dioxus::prelude::*;
use zwipe::domain::card::models::Card;
use zwipe_core::domain::deck::deck_profile::DeckProfile;

#[component]
pub(crate) fn DeckProfileSection(deck_profile: DeckProfile, commander: Option<Card>) -> Element {
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
                    span { class: "info-row-label", "commander" }
                    span { class: "info-row-value",
                        if let Some(cmd) = commander {
                            { cmd.scryfall_data.name.to_lowercase() }
                        } else {
                            "none"
                        }
                    }
                }
            }
        }
    }
}
