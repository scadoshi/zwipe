use crate::{
    inbound::components::fields::text_input::TextInput,
    outbound::client::{ZwipeClient, card::search_cards::ClientSearchCards},
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use zwipe_core::domain::card::{
    Card,
    search_card::{
        card_filter::{builder::CardFilterBuilder, error::InvalidCardFilter},
        commander_eligibility::{has_choose_a_background, partner_kind},
    },
};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::format::Format;

/// Format chip selector and card search inputs with debounced dropdowns.
///
/// Shows/hides commander, partner, background, and signature spell fields
/// based on the selected format and commander's abilities.
///
/// Reads `Signal<Option<Session>>` and `Signal<ZwipeClient>` from context.
#[component]
pub(crate) fn DeckFields(
    mut deck_name: Signal<String>,
    mut selected_format: Signal<Option<Format>>,
    mut commander: Signal<Option<Card>>,
    mut commander_display: Signal<String>,
    mut partner_commander: Signal<Option<Card>>,
    mut partner_commander_display: Signal<String>,
    mut background: Signal<Option<Card>>,
    mut background_display: Signal<String>,
    mut signature_spell: Signal<Option<Card>>,
    mut signature_spell_display: Signal<String>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // ========================================
    // Format search state
    // ========================================
    let mut format_query = use_signal(String::new);

    // ========================================
    // Commander search state
    // ========================================
    let mut cmd_search_query = use_signal(String::new);
    let mut cmd_search_results = use_signal(Vec::<Card>::new);
    let mut cmd_is_searching = use_signal(|| false);
    let mut cmd_show_dropdown = use_signal(|| false);
    let mut cmd_filter_on = use_signal(|| true);

    // ========================================
    // Partner search state
    // ========================================
    let mut partner_search_query = use_signal(String::new);
    let mut partner_search_results = use_signal(Vec::<Card>::new);
    let mut partner_is_searching = use_signal(|| false);
    let mut partner_show_dropdown = use_signal(|| false);
    let mut partner_filter_on = use_signal(|| true);

    // ========================================
    // Background search state
    // ========================================
    let mut bg_search_query = use_signal(String::new);
    let mut bg_search_results = use_signal(Vec::<Card>::new);
    let mut bg_is_searching = use_signal(|| false);
    let mut bg_show_dropdown = use_signal(|| false);
    let mut bg_filter_on = use_signal(|| true);

    // ========================================
    // Signature spell search state
    // ========================================
    let mut spell_search_query = use_signal(String::new);
    let mut spell_search_results = use_signal(Vec::<Card>::new);
    let mut spell_is_searching = use_signal(|| false);
    let mut spell_show_dropdown = use_signal(|| false);
    let mut spell_filter_on = use_signal(|| true);

    // ========================================
    // Visibility memos
    // ========================================
    let show_commander = use_memo(move || {
        selected_format().is_some_and(|f| f.has_commander())
    });

    let show_partner = use_memo(move || {
        show_commander()
            && commander().is_some_and(|c| partner_kind(&c).is_some())
    });

    let show_background = use_memo(move || {
        show_commander()
            && commander().is_some_and(|c| has_choose_a_background(&c))
    });

    let show_signature_spell = use_memo(move || {
        selected_format().is_some_and(|f| f.has_signature_spell())
    });

    let is_oathbreaker = use_memo(move || {
        selected_format().is_some_and(|f| f.has_signature_spell())
    });

    let commander_label = use_memo(move || {
        if is_oathbreaker() { "oathbreaker" } else { "commander" }
    });

    // ========================================
    // Cascading clear effects
    // ========================================

    // Format change → sync display text + reset all filter toggles
    use_effect(move || {
        let fmt = selected_format();
        format_query.set(
            fmt.map(|f| f.display_name().to_lowercase())
                .unwrap_or_default(),
        );
        cmd_filter_on.set(true);
        partner_filter_on.set(true);
        bg_filter_on.set(true);
        spell_filter_on.set(true);
    });

    // Commander change → clear partner and background (they depend on commander's abilities)
    use_effect(move || {
        let _ = commander();
        partner_commander.set(None);
        partner_commander_display.set(String::new());
        partner_search_query.set(String::new());
        partner_show_dropdown.set(false);
        background.set(None);
        background_display.set(String::new());
        bg_search_query.set(String::new());
        bg_show_dropdown.set(false);
    });

    // ========================================
    // Commander debounced search
    // ========================================
    use_effect(move || {
        let query = cmd_search_query();

        if query.len() < 3 {
            cmd_show_dropdown.set(false);
            cmd_is_searching.set(false);
            return;
        }

        cmd_is_searching.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if cmd_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardFilterBuilder::with_name_contains(&query);
                if cmd_filter_on()
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
                        cmd_search_results.set(cards);
                        cmd_is_searching.set(false);
                        cmd_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        cmd_is_searching.set(false);
                        cmd_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    // ========================================
    // Partner debounced search (1 char minimum — small card pool)
    // ========================================
    use_effect(move || {
        let query = partner_search_query();

        if query.is_empty() {
            partner_show_dropdown.set(false);
            partner_is_searching.set(false);
            return;
        }

        partner_is_searching.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if partner_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardFilterBuilder::new();
                if !query.is_empty() {
                    builder.set_name_contains(&query);
                }
                if partner_filter_on() {
                    builder.set_is_partner(true);
                }
                builder.set_limit(10);
                let Ok(card_filter) = builder.build() else {
                    return;
                };
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        partner_search_results.set(cards);
                        partner_is_searching.set(false);
                        partner_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        partner_is_searching.set(false);
                        partner_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    // ========================================
    // Background debounced search (1 char minimum — small card pool)
    // ========================================
    use_effect(move || {
        let query = bg_search_query();

        if query.is_empty() {
            bg_show_dropdown.set(false);
            bg_is_searching.set(false);
            return;
        }

        bg_is_searching.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if bg_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardFilterBuilder::new();
                if !query.is_empty() {
                    builder.set_name_contains(&query);
                }
                if bg_filter_on() {
                    builder.set_is_background(true);
                }
                builder.set_limit(10);
                let Ok(card_filter) = builder.build() else {
                    return;
                };
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        bg_search_results.set(cards);
                        bg_is_searching.set(false);
                        bg_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        bg_is_searching.set(false);
                        bg_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    // ========================================
    // Signature spell debounced search
    // ========================================
    use_effect(move || {
        let query = spell_search_query();

        if query.len() < 3 {
            spell_show_dropdown.set(false);
            spell_is_searching.set(false);
            return;
        }

        spell_is_searching.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if spell_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardFilterBuilder::with_name_contains(&query);
                if spell_filter_on() {
                    builder.set_is_signature_spell(true);
                    // Also restrict to oathbreaker's color identity
                    if let Some(ob) = commander() {
                        builder.set_color_identity_within(ob.scryfall_data.color_identity);
                    }
                }
                builder.set_limit(5);
                let Ok(card_filter) = builder.build() else {
                    return;
                };
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        spell_search_results.set(cards);
                        spell_is_searching.set(false);
                        spell_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        spell_is_searching.set(false);
                        spell_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    rsx! {
        // ========================================
        // Deck name
        // ========================================
        TextInput {
            label: "deck name",
            value: deck_name,
            id: "deck_name",
            placeholder: "deck name",
        }

        // ========================================
        // Format selector (typeahead)
        // ========================================
        div {
            div { class: "label-row",
                label { class: "label", "format" }
                if selected_format().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            selected_format.set(None);
                            format_query.set(String::new());
                            commander.set(None);
                            commander_display.set(String::new());
                            signature_spell.set(None);
                            signature_spell_display.set(String::new());
                        },
                        "\u{00d7}"
                    }
                }
            }

            // Filtered results (chips above input, only when typing with no selection)
            if selected_format().is_none() && !format_query().is_empty() {
                {
                    let query = format_query().to_lowercase();
                    let results: Vec<Format> = Format::all()
                        .iter()
                        .copied()
                        .filter(|f| f.display_name().to_lowercase().contains(&query))
                        .take(5)
                        .collect();

                    if !results.is_empty() {
                        rsx! {
                            div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                                for fmt in results {
                                    div { class: "chip-unselected",
                                        onclick: move |_| {
                                            selected_format.set(Some(fmt));
                                            format_query.set(fmt.display_name().to_lowercase());
                                            commander.set(None);
                                            commander_display.set(String::new());
                                            if !fmt.has_signature_spell() {
                                                signature_spell.set(None);
                                                signature_spell_display.set(String::new());
                                            }
                                        },
                                        { fmt.display_name().to_lowercase() }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                                div { class: "chip-unselected", "no results" }
                            }
                        }
                    }
                }
            }

            // Search input (shows selected format name when selected)
            input { class: "input",
                id: "format-search",
                r#type: "text",
                placeholder: "format",
                value: "{format_query()}",
                autocapitalize: "none",
                spellcheck: "false",
                onclick: move |_| {
                    if selected_format().is_some() {
                        selected_format.set(None);
                        format_query.set(String::new());
                        commander.set(None);
                        commander_display.set(String::new());
                        signature_spell.set(None);
                        signature_spell_display.set(String::new());
                    }
                },
                oninput: move |event| {
                    format_query.set(event.value());
                }
            }
        }

        // ========================================
        // Commander selector
        // ========================================
        if show_commander() {
            div {
                div { class: "label-row",
                    label { class: "label", "{commander_label}" }
                    div {
                        class: if cmd_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            cmd_filter_on.set(!cmd_filter_on());
                        },
                        "filter"
                    }
                    if commander().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                commander.set(None);
                                commander_display.set(String::new());
                                cmd_search_query.set(String::new());
                                cmd_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                }

                if cmd_show_dropdown() {
                    if cmd_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "searching..." }
                        }
                    } else if cmd_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "no results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in cmd_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        commander.set(Some(card.clone()));
                                        commander_display.set(card.scryfall_data.name.to_lowercase());
                                        cmd_show_dropdown.set(false);
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
                    placeholder: "{commander_label}",
                    value: "{commander_display}",
                    autocapitalize: "none",
                    spellcheck: "false",
                    onclick: move |_| {
                        cmd_search_query.set(String::new());
                        commander_display.set(String::new());
                        commander.set(None);
                    },
                    oninput: move |event| {
                        cmd_search_query.set(event.value());
                        commander_display.set(event.value());
                    }
                }
            }
        }

        // ========================================
        // Partner commander selector
        // ========================================
        if show_partner() {
            div {
                div { class: "label-row",
                    label { class: "label", "partner" }
                    div {
                        class: if partner_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            partner_filter_on.set(!partner_filter_on());
                        },
                        "filter"
                    }
                    if partner_commander().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                partner_commander.set(None);
                                partner_commander_display.set(String::new());
                                partner_search_query.set(String::new());
                                partner_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                }

                if partner_show_dropdown() {
                    if partner_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "searching..." }
                        }
                    } else if partner_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "no results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in partner_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        partner_commander.set(Some(card.clone()));
                                        partner_commander_display.set(card.scryfall_data.name.to_lowercase());
                                        partner_show_dropdown.set(false);
                                    },
                                    { card.scryfall_data.name.to_lowercase() }
                                }
                            }
                        }
                    }
                }

                input { class: "input",
                    id: "partner_commander",
                    r#type: "text",
                    placeholder: "partner commander",
                    value: "{partner_commander_display}",
                    autocapitalize: "none",
                    spellcheck: "false",
                    onclick: move |_| {
                        partner_search_query.set(String::new());
                        partner_commander_display.set(String::new());
                        partner_commander.set(None);
                    },
                    oninput: move |event| {
                        partner_search_query.set(event.value());
                        partner_commander_display.set(event.value());
                    }
                }
            }
        }

        // ========================================
        // Background selector
        // ========================================
        if show_background() {
            div {
                div { class: "label-row",
                    label { class: "label", "background" }
                    div {
                        class: if bg_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            bg_filter_on.set(!bg_filter_on());
                        },
                        "filter"
                    }
                    if background().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                background.set(None);
                                background_display.set(String::new());
                                bg_search_query.set(String::new());
                                bg_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                }

                if bg_show_dropdown() {
                    if bg_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "searching..." }
                        }
                    } else if bg_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "no results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in bg_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        background.set(Some(card.clone()));
                                        background_display.set(card.scryfall_data.name.to_lowercase());
                                        bg_show_dropdown.set(false);
                                    },
                                    { card.scryfall_data.name.to_lowercase() }
                                }
                            }
                        }
                    }
                }

                input { class: "input",
                    id: "background",
                    r#type: "text",
                    placeholder: "background",
                    value: "{background_display}",
                    autocapitalize: "none",
                    spellcheck: "false",
                    onclick: move |_| {
                        bg_search_query.set(String::new());
                        background_display.set(String::new());
                        background.set(None);
                    },
                    oninput: move |event| {
                        bg_search_query.set(event.value());
                        background_display.set(event.value());
                    }
                }
            }
        }

        // ========================================
        // Signature spell selector
        // ========================================
        if show_signature_spell() {
            div {
                div { class: "label-row",
                    label { class: "label", "signature spell" }
                    div {
                        class: if spell_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            spell_filter_on.set(!spell_filter_on());
                        },
                        "filter"
                    }
                    if signature_spell().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                signature_spell.set(None);
                                signature_spell_display.set(String::new());
                                spell_search_query.set(String::new());
                                spell_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                }

                if spell_show_dropdown() {
                    if spell_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "searching..." }
                        }
                    } else if spell_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "no results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in spell_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        signature_spell.set(Some(card.clone()));
                                        signature_spell_display.set(card.scryfall_data.name.to_lowercase());
                                        spell_show_dropdown.set(false);
                                    },
                                    { card.scryfall_data.name.to_lowercase() }
                                }
                            }
                        }
                    }
                }

                input { class: "input",
                    id: "signature_spell",
                    r#type: "text",
                    placeholder: "signature spell",
                    value: "{signature_spell_display}",
                    autocapitalize: "none",
                    spellcheck: "false",
                    onclick: move |_| {
                        spell_search_query.set(String::new());
                        signature_spell_display.set(String::new());
                        signature_spell.set(None);
                    },
                    oninput: move |event| {
                        spell_search_query.set(event.value());
                        signature_spell_display.set(event.value());
                    }
                }
            }
        }
    }
}
