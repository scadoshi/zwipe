use crate::{
    inbound::{
        components::{
            accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        },
        screens::deck::card::filter::{
            combat::Combat, config::Config, mana::Mana, rarity::Rarity, set::Set, sort::Sort,
            text::Text, types::Types,
        },
    },
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{get_deck::ClientGetDeck, get_deck_profile::ClientGetDeckProfile},
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe::domain::{
    auth::models::session::Session,
    card::models::{
        Card,
        search_card::{
            card_filter::builder::CardFilterBuilder,
            filter_cards::FilterCards,
            group_cards::{CardGroup, GroupByOption, GroupCards},
        },
    },
};

#[component]
pub fn View(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut browse_error = use_signal(|| None::<String>);

    // Filter overlay state
    let mut filters_overlay_open = use_signal(|| false);

    // Incrementing this re-runs the filter + group effect
    let mut filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    // Source of truth — all cards in the deck
    let mut deck_cards: Signal<Vec<Card>> = use_signal(Vec::new);
    // Guards filter effect from running before the deck has loaded
    let mut deck_loaded: Signal<bool> = use_signal(|| false);
    // What the UI renders — grouped card lists
    let mut displayed_groups: Signal<Vec<CardGroup>> = use_signal(Vec::new);
    // Current grouping mode
    let mut group_by_option: Signal<GroupByOption> = use_signal(|| GroupByOption::CardType);
    // Which card row is expanded (None = all collapsed)
    let mut expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    // Commander is always pinned — never part of groupable deck_cards
    let mut commander_card: Signal<Option<Card>> = use_signal(|| None);

    // Effect 1 — mount load (reads `session` reactively)
    // Fetches deck cards, separates the commander into its own pinned slot.
    use_effect(move || {
        session.upkeep(client);
        let Some(session) = session() else {
            return;
        };

        spawn(async move {
            let mut cards = match client().get_deck(deck_id, &session).await {
                Ok(deck) => deck.cards,
                Err(e) => {
                    browse_error.set(Some(e.to_string()));
                    return;
                }
            };

            // Resolve commander: pull from deck cards if present, otherwise fetch separately.
            // Either way remove it from the groupable list.
            if let Ok(profile) = client().get_deck_profile(deck_id, &session).await
                && let Some(commander_id) = profile.commander_id
            {
                let cmd = if let Some(idx) = cards.iter().position(|c| c.scryfall_data.id == commander_id) {
                    Some(cards.remove(idx))
                } else {
                    client().get_card(commander_id, &session).await.ok()
                };
                commander_card.set(cmd);
            }

            deck_cards.set(cards);
            deck_loaded.set(true);

            if !filter_builder.peek().is_empty() {
                toast.warning(
                    "filter is active".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(2000)),
                );
            }

            let current = *filter_reset_counter.peek();
            filter_reset_counter.set(current + 1);
        });
    });

    // Effect 2 — filter + group (reads `filter_reset_counter` and `group_by_option` reactively)
    use_effect(move || {
        let _ = filter_reset_counter();
        let _ = group_by_option();

        if !*deck_loaded.peek() {
            return;
        }

        let all_cards = deck_cards.peek().clone();
        let builder = filter_builder.peek().clone();
        let group_option = *group_by_option.peek();

        let filtered = if builder.is_empty() {
            all_cards
        } else {
            let mut b = builder;
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => all_cards.filter_by(&filter),
                Err(_) => deck_cards.peek().clone(),
            }
        };

        let groups = filtered.group_by(group_option);
        displayed_groups.set(groups);
        expanded_card.set(None);
    });

    let mut clear_filters = move || {
        let opts = ToastOptions::default().duration(Duration::from_millis(1500));
        if filter_builder.read().is_empty() {
            toast.warning("filter already cleared".to_string(), opts);
        } else {
            filter_builder.write().clear();
            toast.info("filter cleared".to_string(), opts);
        }
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "deck cards" }
                }

                div { class: "screen-content",

                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    // Group-by chips
                    div { class: "chip-row",
                        for option in GroupByOption::all() {
                            button {
                                key: "{option}",
                                class: if group_by_option() == option { "chip selected" } else { "chip" },
                                onclick: move |_| group_by_option.set(option),
                                "{option}"
                            }
                        }
                    }

                    // Column headers
                    div { class: "card-row-compact card-row-header",
                        span { class: "card-row-name", "name" }
                        span { class: "card-row-cmc", "cmc" }
                        span { class: "card-row-pt", "p/t" }
                        span { class: "card-row-colors", "colors" }
                    }

                    // Pinned commander group
                    if let Some(cmd) = commander_card() {
                        div { class: "card-group",
                            div { class: "card-group-header", "commander" }
                            {
                                let card_id = cmd.scryfall_data.id;
                                let is_expanded = expanded_card() == Some(card_id);
                                let sd = &cmd.scryfall_data;
                                let name = sd.name.clone();
                                let cmc_display = sd.cmc
                                    .map(|c| {
                                        let floored = c.floor() as i64;
                                        if c == c.floor() { format!("{floored}") } else { format!("{c}") }
                                    })
                                    .unwrap_or_default();
                                let pt_display = match (&sd.power, &sd.toughness) {
                                    (Some(p), Some(t)) => format!("{p}/{t}"),
                                    _ => String::new(),
                                };
                                let color_display = sd.color_identity
                                    .iter()
                                    .map(|c| format!("{{{}}}", c.to_short_name()))
                                    .collect::<Vec<_>>()
                                    .join("");
                                let oracle_text = sd.oracle_text.clone().unwrap_or_default().to_lowercase();
                                let type_line = sd.type_line.clone().unwrap_or_default().to_lowercase();
                                let rarity_name = sd.rarity.to_long_name().to_lowercase();
                                let set_name = sd.set_name.clone().to_lowercase();
                                rsx! {
                                    div {
                                        key: "{card_id}",
                                        class: if is_expanded { "card-row expanded" } else { "card-row" },
                                        onclick: move |_| {
                                            if expanded_card() == Some(card_id) {
                                                expanded_card.set(None);
                                            } else {
                                                expanded_card.set(Some(card_id));
                                            }
                                        },
                                        div { class: "card-row-compact",
                                            span { class: "card-row-name", "{name}" }
                                            span { class: "card-row-cmc", "{cmc_display}" }
                                            span { class: "card-row-pt", "{pt_display}" }
                                            span { class: "card-row-colors", "{color_display}" }
                                        }
                                        if is_expanded {
                                            div { class: "card-row-detail",
                                                if !type_line.is_empty() {
                                                    span { class: "opacity-50", style: "display:block;margin-bottom:0.5rem;", "{type_line}" }
                                                }
                                                if !oracle_text.is_empty() {
                                                    p { class: "card-detail-oracle", "{oracle_text}" }
                                                }
                                                div { class: "card-detail-meta",
                                                    span { "{rarity_name} | {set_name}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Card groups
                    for group in displayed_groups() {
                        div { class: "card-group",
                            // Group header
                            div { class: "card-group-header",
                                "{group.label} ({group.cards.len()})"
                            }

                            // Card rows
                            for card in group.cards.iter() {
                                {
                                    let card_id = card.scryfall_data.id;
                                    let is_expanded = expanded_card() == Some(card_id);
                                    let sd = &card.scryfall_data;

                                    let name = sd.name.clone();
                                    let cmc_display = sd.cmc
                                        .map(|c| {
                                            let floored = c.floor() as i64;
                                            if c == c.floor() {
                                                format!("{floored}")
                                            } else {
                                                format!("{c}")
                                            }
                                        })
                                        .unwrap_or_default();

                                    let pt_display = match (&sd.power, &sd.toughness) {
                                        (Some(p), Some(t)) => format!("{p}/{t}"),
                                        _ => String::new(),
                                    };

                                    let color_display = sd.color_identity
                                        .iter()
                                        .map(|c| format!("{{{}}}", c.to_short_name()))
                                        .collect::<Vec<_>>()
                                        .join("");

                                    // Expanded details
                                    let oracle_text = sd.oracle_text.clone().unwrap_or_default().to_lowercase();
                                    let type_line = sd.type_line.clone().unwrap_or_default().to_lowercase();
                                    let rarity_name = sd.rarity.to_long_name().to_lowercase();
                                    let set_name = sd.set_name.clone().to_lowercase();

                                    rsx! {
                                        div {
                                            key: "{card_id}",
                                            class: if is_expanded { "card-row expanded" } else { "card-row" },
                                            onclick: move |_| {
                                                if expanded_card() == Some(card_id) {
                                                    expanded_card.set(None);
                                                } else {
                                                    expanded_card.set(Some(card_id));
                                                }
                                            },

                                            // Compact row
                                            div { class: "card-row-compact",
                                                span { class: "card-row-name", "{name}" }
                                                span { class: "card-row-cmc", "{cmc_display}" }
                                                span { class: "card-row-pt", "{pt_display}" }
                                                span { class: "card-row-colors", "{color_display}" }
                                            }

                                            // Expanded detail
                                            if is_expanded {
                                                div { class: "card-row-detail",
                                                    if !type_line.is_empty() {
                                                        span { class: "opacity-50", style: "display:block;margin-bottom:0.5rem;", "{type_line}" }
                                                    }
                                                    if !oracle_text.is_empty() {
                                                        p { class: "card-detail-oracle", "{oracle_text}" }
                                                    }
                                                    div { class: "card-detail-meta",
                                                        span { "{rarity_name} | {set_name}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if displayed_groups().is_empty() && *deck_loaded.peek() {
                        p { class: "text-muted", "no cards" }
                    }

                    if let Some(err) = browse_error() {
                        div { class: "message-error", "{err}" }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| filters_overlay_open.set(true),
                    "filter"
                }
            }

            // Modal backdrop
            div {
                class: if filters_overlay_open() { "modal-backdrop show" } else { "modal-backdrop" },
                onclick: move |_| filters_overlay_open.set(false),
            }

            // Bottom sheet
            div {
                class: if filters_overlay_open() { "bottom-sheet show" } else { "bottom-sheet" },

                div { class: "modal-header",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| {
                            filter_reset_counter.set(filter_reset_counter() + 1);
                            filters_overlay_open.set(false);
                        },
                        "apply"
                    }
                }

                div { class: "modal-content",
                    Accordion {
                        key: "{filter_reset_counter()}",
                        id: "filter-accordion",
                        allow_multiple_open: false,
                        collapsible: true,

                        AccordionItem { index: 1,
                            AccordionTrigger { "text" }
                            AccordionContent { Text {} }
                        }
                        AccordionItem { index: 2,
                            AccordionTrigger { "types" }
                            AccordionContent { Types {} }
                        }
                        AccordionItem { index: 3,
                            AccordionTrigger { "mana" }
                            AccordionContent { Mana {} }
                        }
                        AccordionItem { index: 4,
                            AccordionTrigger { "combat" }
                            AccordionContent { Combat {} }
                        }
                        AccordionItem { index: 5,
                            AccordionTrigger { "rarity" }
                            AccordionContent { Rarity {} }
                        }
                        AccordionItem { index: 6,
                            AccordionTrigger { "set" }
                            AccordionContent { Set {} }
                        }
                        AccordionItem { index: 7,
                            AccordionTrigger { "sort" }
                            AccordionContent { Sort {} }
                        }
                        AccordionItem { index: 8,
                            AccordionTrigger { "config" }
                            AccordionContent { Config {} }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| clear_filters(),
                        "clear"
                    }
                }
            }
            }
        }
    }
}
