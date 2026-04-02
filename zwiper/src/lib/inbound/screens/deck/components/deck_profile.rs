use dioxus::prelude::*;
use zwipe::domain::{
    card::models::Card,
    deck::models::deck::{deck_profile::DeckProfile, deck_warning::DeckWarning},
};

#[component]
pub(crate) fn DeckProfileSection(
    deck_profile: DeckProfile,
    commander: Option<Card>,
    warnings: Vec<DeckWarning>,
) -> Element {
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

        if !warnings.is_empty() {
            label { class: "label", "warnings" }
            div { class: "info-list",
                style: "border-color: var(--border-warning);",
                for warning in warnings.iter() {
                    div { class: "info-row",
                        style: "justify-content: flex-start; gap: 0.5rem;",
                        span { class: "text-muted", "{warning.to_lowercase()}" }
                    }
                }
            }
        }
    }
}
