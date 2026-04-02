use crate::{
    inbound::components::fields::text_input::TextInput,
    outbound::client::{ZwipeClient, card::search_cards::ClientSearchCards},
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use zwipe::domain::{
    auth::models::session::Session,
    card::models::{
        Card,
        search_card::card_filter::{builder::CardFilterBuilder, error::InvalidCardFilter},
    },
    deck::models::deck::format::Format,
};

/// Format chip selector and commander search input with debounced dropdown.
///
/// Reads `Signal<Option<Session>>` and `Signal<ZwipeClient>` from context.
#[component]
pub(crate) fn DeckFields(
    mut deck_name: Signal<String>,
    mut selected_format: Signal<Option<Format>>,
    mut commander: Signal<Option<Card>>,
    mut commander_display: Signal<String>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // commander search state
    let mut search_query = use_signal(String::new);
    let mut search_results = use_signal(Vec::<Card>::new);
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

    let commander_enabled = use_memo(move || selected_format().is_some_and(|f| f.has_commander()));

    // debounced commander search effect
    use_effect(move || {
        let query = search_query();

        if query.len() < 3 {
            show_dropdown.set(false);
            is_searching.set(false);
            return;
        }

        is_searching.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let Ok(card_filter) = CardFilterBuilder::with_name_contains(&query)
                    .set_limit(5)
                    .build()
                else {
                    tracing::error!("{}", InvalidCardFilter::Empty.to_string());
                    return;
                };
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        search_results.set(cards);
                        is_searching.set(false);
                        show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        is_searching.set(false);
                        show_dropdown.set(false);
                    }
                }
            }
        });
    });

    rsx! {
        // name
        TextInput {
            label: "deck name",
            value: deck_name,
            id: "deck_name",
            placeholder: "deck name",
        }


        // commander selector
        div { class: "",
            div { class: "label-row",
                label {
                    class: if commander_enabled() { "label" } else { "label text-muted" },
                    "commander"
                }
                if commander().is_some() && commander_enabled() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            commander.set(None);
                            commander_display.set(String::new());
                            search_query.set(String::new());
                            show_dropdown.set(false);
                        },
                        "\u{00d7}"
                    }
                }
            }
            input { class: if commander_enabled() { "input" } else { "input disabled" },
                id: "commander",
                r#type: "text",
                placeholder: if commander_enabled() { "commander" } else { "select a commander format" },
                value: "{commander_display}",
                autocapitalize: "none",
                spellcheck: "false",
                disabled: !commander_enabled(),
                onclick: move |_| {
                    if commander_enabled() {
                        search_query.set(String::new());
                        commander_display.set(String::new());
                        commander.set(None);
                    }
                },
                oninput: move |event| {
                    if commander_enabled() {
                        search_query.set(event.value());
                        commander_display.set(event.value());
                    }
                }
            }

            if show_dropdown() && commander_enabled() {
                div { class: "dropdown",
                    if is_searching() {
                        div { "searching..." }
                    } else if search_results().is_empty() {
                        div { "no results" }
                    } else {
                        for card in search_results().iter().cloned() {
                            div { class: "dropdown-item",
                                onclick: move |_| {
                                    commander.set(Some(card.clone()));
                                    commander_display.set(card.scryfall_data.name.to_lowercase());
                                    show_dropdown.set(false);
                                },
                                { card.scryfall_data.name.to_lowercase() }
                            }
                        }
                    }
                }
            }
        }

        // format selector (chips)
        div { class: "mb-4",
            div { class: "label-row",
                label { class: "label", "format" }
                if selected_format().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            selected_format.set(None);
                            commander.set(None);
                            commander_display.set(String::new());
                        },
                        "\u{00d7}"
                    }
                }
            }
            div { class: "flex flex-wrap gap-1 flex-center",
                for fmt in Format::all().iter().copied() {
                    div {
                        class: if selected_format() == Some(fmt) { "chip selected" } else { "chip" },
                        onclick: move |_| {
                            if selected_format() == Some(fmt) {
                                selected_format.set(None);
                                commander.set(None);
                                commander_display.set(String::new());
                            } else {
                                selected_format.set(Some(fmt));
                                if !fmt.has_commander() {
                                    commander.set(None);
                                    commander_display.set(String::new());
                                }
                            }
                        },
                        { fmt.display_name().to_lowercase() }
                    }
                }
            }
        }
    }
}
