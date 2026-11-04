//! The deck profile section on the view deck screen.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_ui::deck::command_zone::Slot;

#[component]
pub(crate) fn DeckProfileSection(
    deck_profile: DeckProfile,
    commander: Option<Card>,
    /// Total card count (command-zone variants included) for the "Cards" row.
    card_count: usize,
    /// The buttons on the deck as a whole, at the end of the header row.
    #[props(default)]
    actions: Option<Element>,
) -> Element {
    let format = deck_profile.format;
    let commander_label = Slot::Commander.label(format);

    rsx! {
        div { class: "settings-list",
            div { class: "settings-list-header",
                span { class: "settings-list-title", "Profile" }
                if let Some(actions) = actions {
                    div { class: "deck-profile-actions", {actions} }
                }
            }
            div { class: "settings-row",
                span { class: "settings-row-label", "Name" }
                span { class: "info-row-value", "{deck_profile.name}" }
            }
            div { class: "settings-row",
                span { class: "settings-row-label", "Cards" }
                span { class: "info-row-value", "{card_count}" }
            }
            div { class: "settings-row",
                span { class: "settings-row-label", "Format" }
                span { class: "info-row-value",
                    if let Some(fmt) = deck_profile.format {
                        { fmt.display_name().to_string() }
                    } else {
                        "None"
                    }
                }
            }
            if deck_profile.format.is_some_and(|f| f.has_commander()) {
                div { class: "settings-row",
                    span { class: "settings-row-label", "{commander_label}" }
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
                div { class: "settings-row",
                    span { class: "settings-row-label", {Slot::Partner.label(format)} }
                    span { class: "info-row-value", { name.clone() } }
                }
            }
            if let Some(name) = &deck_profile.background_name {
                div { class: "settings-row",
                    span { class: "settings-row-label", {Slot::Background.label(format)} }
                    span { class: "info-row-value", { name.clone() } }
                }
            }
            if let Some(name) = &deck_profile.signature_spell_name {
                div { class: "settings-row",
                    span { class: "settings-row-label", {Slot::SignatureSpell.label(format)} }
                    span { class: "info-row-value", { name.clone() } }
                }
            }
            if let Some(pl) = deck_profile.power_level {
                div { class: "settings-row",
                    span { class: "settings-row-label", "Power level" }
                    span { class: "info-row-value", { pl.display_name().to_string() } }
                }
            }
        }
    }
}
