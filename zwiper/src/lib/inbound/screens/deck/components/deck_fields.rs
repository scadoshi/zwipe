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
    let mut commander_filter_on = use_signal(|| true);

    let commander_enabled = use_memo(move || selected_format().is_some_and(|f| f.has_commander()));

    // Reset toggle when format changes
    use_effect(move || {
        let _ = selected_format();
        commander_filter_on.set(true);
    });

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
                let mut builder = CardFilterBuilder::with_name_contains(&query);
                if commander_filter_on()
                    && let Some(fmt) = selected_format()
                {
                    builder.set_is_commander_in_format(fmt);
                }
                builder.set_limit(5);
                let Ok(card_filter) = builder.build()
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

        // commander selector (only visible for commander formats)
        if commander_enabled() {
            div {
                div { class: "label-row",
                    label { class: "label", "commander" }
                    div {
                        class: if commander_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            commander_filter_on.set(!commander_filter_on());
                        },
                        "filter"
                    }
                    if commander().is_some() {
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

                // search result chips (above input, like artist/set pattern)
                if show_dropdown() {
                    if is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "searching..." }
                        }
                    } else if search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "no results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in search_results().iter().cloned() {
                                div { class: "chip-unselected",
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

                input { class: "input",
                    id: "commander",
                    r#type: "text",
                    placeholder: "commander",
                    value: "{commander_display}",
                    autocapitalize: "none",
                    spellcheck: "false",
                    onclick: move |_| {
                        search_query.set(String::new());
                        commander_display.set(String::new());
                        commander.set(None);
                    },
                    oninput: move |event| {
                        search_query.set(event.value());
                        commander_display.set(event.value());
                    }
                }

            }
        }
    }
}
