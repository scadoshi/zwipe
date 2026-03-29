use crate::{
    inbound::{
        components::{
            accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        },
        screens::deck::card::filter::{
            artist::Artist, combat::Combat, config::Config, deck_cards::DeckCards,
            flavor_text::FlavorText, mana::Mana, name::Name, oracle_text::OracleText,
            rarity::Rarity, set::Set, sort::Sort, types::Types,
        },
    },
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{get_deck::ClientGetDeck, get_deck_profile::ClientGetDeckProfile},
        deck_card::{
            delete_deck_card::ClientDeleteDeckCard, update_deck_card::ClientUpdateDeckCard,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use zwipe::domain::{
    auth::models::session::Session,
    card::models::{
        Card,
        scryfall_data::image_uris::ImageUris,
        search_card::{
            card_filter::builder::CardFilterBuilder,
            filter_cards::{FilterCards, SortCards},
            group_cards::{CardGroup, GroupByOption, GroupCards},
        },
    },
    deck::models::deck::copy_max::CopyMax,
};
use zwipe::inbound::http::handlers::deck_card::update_deck_card::HttpUpdateDeckCard;

#[component]
pub fn View(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

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
    use_context_provider(|| DeckCards(deck_cards));
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
    // Commander filtered by the active filter (None = filtered out or no commander)
    let mut displayed_commander: Signal<Option<Card>> = use_signal(|| None);
    // Quantity per card (scryfall_data_id → quantity as i32)
    let mut quantity_map: Signal<HashMap<Uuid, i32>> = use_signal(HashMap::new);
    // Copy limit for this deck (None = unlimited, Some(1) = singleton, Some(4) = standard)
    let mut deck_copy_max: Signal<Option<CopyMax>> = use_signal(|| None);
    // Toggle to show/hide land cards (default: hidden)
    let mut show_lands: Signal<bool> = use_signal(|| false);

    // Card image preview — stores the image URL of the card to preview (None = closed)
    let mut preview_image_url: Signal<Option<String>> = use_signal(|| None);
    // Controls the dismiss animation before clearing the URL
    let mut preview_dismissing: Signal<bool> = use_signal(|| false);

    // Effect 1 — mount load (reads `session` reactively)
    // Fetches deck cards, separates the commander into its own pinned slot.
    use_effect(move || {
        session.upkeep(client);
        let Some(session) = session() else {
            return;
        };

        spawn(async move {
            let (mut cards, qty_map): (Vec<Card>, HashMap<Uuid, i32>) =
                match client().get_deck(deck_id, &session).await {
                    Ok(deck) => {
                        let cards = deck.entries.iter().map(|e| e.card.clone()).collect();
                        let qty = deck
                            .entries
                            .iter()
                            .map(|e| (e.card.scryfall_data.id, *e.deck_card.quantity))
                            .collect();
                        (cards, qty)
                    }
                    Err(e) => {
                        toast.error(
                            e.to_string(),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        );
                        return;
                    }
                };

            quantity_map.set(qty_map);

            // Resolve commander: pull from deck cards if present, otherwise fetch separately.
            // Either way remove it from the groupable list.
            if let Ok(profile) = client().get_deck_profile(deck_id, &session).await {
                deck_copy_max.set(profile.copy_max);
                if let Some(commander_id) = profile.commander_id {
                    let cmd = if let Some(idx) = cards
                        .iter()
                        .position(|c| c.scryfall_data.id == commander_id)
                    {
                        Some(cards.remove(idx))
                    } else {
                        client().get_card(commander_id, &session).await.ok()
                    };
                    commander_card.set(cmd);
                }
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

    // Effect 2 — filter + group (reads `filter_reset_counter`, `group_by_option`, `show_lands` reactively)
    use_effect(move || {
        let _ = filter_reset_counter();
        let _ = group_by_option();
        let _ = show_lands();

        if !*deck_loaded.peek() {
            return;
        }

        let all_cards = deck_cards.peek().clone();
        let builder = filter_builder.peek().clone();
        let group_option = *group_by_option.peek();
        let lands_visible = *show_lands.peek();
        let cmd = commander_card.peek().clone();

        let (mut filtered, new_displayed_commander) = if builder.is_empty() {
            (all_cards, cmd)
        } else {
            let mut b = builder.clone();
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => {
                    let filtered = all_cards.filter_by(&filter);
                    let cmd_visible =
                        cmd.filter(|c| !vec![c.clone()].filter_by(&filter).is_empty());
                    (filtered, cmd_visible)
                }
                Err(_) => (deck_cards.peek().clone(), cmd),
            }
        };

        if builder.is_empty() {
            filtered.sort_by_filter(&builder);
        }

        if !lands_visible {
            filtered.retain(|c| !c.scryfall_data.is_land());
        }

        let groups = filtered.group_by(group_option);
        displayed_groups.set(groups);
        displayed_commander.set(new_displayed_commander);
        expanded_card.set(None);
    });

    let mut change_quantity = move |card_id: Uuid, delta: i32, is_basic_land: bool| {
        let current_qty = quantity_map.peek().get(&card_id).copied().unwrap_or(1);
        let copy_max = *deck_copy_max.peek();
        let is_singleton = copy_max == Some(CopyMax::singleton());

        // + at max → toast error, no-op (basic lands are exempt)
        if delta > 0
            && !is_basic_land
            && let Some(max) = copy_max
            && current_qty >= *max
        {
            toast.warning(
                "max copies reached".to_string(),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );
            return;
        }

        // - at 1 or any - on singleton (non-basic-land) → delete
        let should_delete =
            current_qty + delta < 1 || (is_singleton && delta < 0 && !is_basic_land);

        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        if should_delete {
            // Optimistic: remove from local state
            quantity_map.write().remove(&card_id);
            deck_cards.write().retain(|c| c.scryfall_data.id != card_id);
            // Trigger re-filter
            let current = *filter_reset_counter.peek();
            filter_reset_counter.set(current + 1);

            spawn(async move {
                if let Err(e) = client().delete_deck_card(deck_id, card_id, &session).await {
                    toast.error(e.to_string(), ToastOptions::default());
                }
            });
        } else {
            // Optimistic: update local quantity
            quantity_map
                .write()
                .entry(card_id)
                .and_modify(|q| *q += delta);

            let request = HttpUpdateDeckCard::new(delta);
            spawn(async move {
                if let Err(e) = client()
                    .update_deck_card(deck_id, card_id, &request, &session)
                    .await
                {
                    toast.error(e.to_string(), ToastOptions::default());
                    // Rollback optimistic update
                    quantity_map
                        .write()
                        .entry(card_id)
                        .and_modify(|q| *q -= delta);
                }
            });
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
                    // Group-by chips + show lands toggle
                    div { class: "chip-row",
                        for option in GroupByOption::all() {
                            button {
                                key: "{option}",
                                class: if group_by_option() == option { "chip selected" } else { "chip" },
                                onclick: move |_| group_by_option.set(option),
                                "{option}"
                            }
                        }
                        div { style: "flex:1;" }
                        button {
                            class: if show_lands() { "chip selected" } else { "chip" },
                            onclick: move |_| show_lands.set(!show_lands()),
                            "show lands"
                        }
                    }

                    // Column headers
                    div { class: "card-row-compact card-row-header",
                        span { class: "card-row-qty", "qty" }
                        span { class: "card-row-name", "name" }
                        span { class: "card-row-cmc", "cmc" }
                        span { class: "card-row-pt", "p/t" }
                        span { class: "card-row-colors", "colors" }
                    }

                    // Pinned commander group
                    if let Some(cmd) = displayed_commander() {
                        div { class: "card-group row-enter",
                            div { class: "card-group-header", "commander" }
                            {
                                let card_id = cmd.scryfall_data.id;
                                let is_expanded = expanded_card() == Some(card_id);
                                let sd = &cmd.scryfall_data;
                                let name = sd.name.to_lowercase();
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
                                            span { class: "card-row-qty", "1" }
                                            span { class: "card-row-name", "{name}" }
                                            span { class: "card-row-cmc", "{cmc_display}" }
                                            span { class: "card-row-pt", "{pt_display}" }
                                            span { class: "card-row-colors", "{color_display}" }
                                        }
                                        if is_expanded {
                                            div { class: "card-row-detail",
                                                p { style: "margin-bottom:0.35rem;word-break:break-word;white-space:normal;", "{name}" }
                                                if !type_line.is_empty() {
                                                    span { class: "opacity-50", style: "display:block;margin-bottom:0.5rem;", "{type_line}" }
                                                }
                                                if !oracle_text.is_empty() {
                                                    p { class: "card-detail-oracle", "{oracle_text}" }
                                                }
                                                div { class: "card-detail-meta",
                                                    span { "{rarity_name} | {set_name}" }
                                                }
                                                if let Some(ImageUris { large: Some(url), .. }) = &sd.image_uris {
                                                    {
                                                        let url = url.clone();
                                                        rsx! {
                                                            div { class: "qty-row",
                                                                button {
                                                                    class: "qty-btn-remove",
                                                                    onclick: move |evt| {
                                                                        evt.stop_propagation();
                                                                        preview_image_url.set(Some(url.clone()));
                                                                        preview_dismissing.set(false);
                                                                    },
                                                                    "image"
                                                                }
                                                            }
                                                        }
                                                    }
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
                        {
                            let qty_count: i32 = {
                                let qm = quantity_map();
                                group.cards.iter().map(|c| qm.get(&c.scryfall_data.id).copied().unwrap_or(1)).sum()
                            };
                            rsx! {
                        div { class: "card-group row-enter",
                            // Group header
                            div { class: "card-group-header",
                                "{group.label} ({qty_count})"
                            }

                            // Card rows
                            for card in group.cards.iter() {
                                {
                                    let card_id = card.scryfall_data.id;
                                    let is_expanded = expanded_card() == Some(card_id);
                                    let sd = &card.scryfall_data;

                                    let name = sd.name.to_lowercase();
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
                                    let is_basic_land = sd.is_basic_land();
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
                                                span { class: "card-row-qty",
                                                    "{quantity_map().get(&card_id).copied().unwrap_or(1)}"
                                                }
                                                span { class: "card-row-name", "{name}" }
                                                span { class: "card-row-cmc", "{cmc_display}" }
                                                span { class: "card-row-pt", "{pt_display}" }
                                                span { class: "card-row-colors", "{color_display}" }
                                            }

                                            // Expanded detail
                                            if is_expanded {
                                                div { class: "card-row-detail",
                                                    p { style: "margin-bottom:0.35rem;word-break:break-word;white-space:normal;", "{name}" }
                                                    if !type_line.is_empty() {
                                                        span { class: "opacity-50", style: "display:block;margin-bottom:0.5rem;", "{type_line}" }
                                                    }
                                                    if !oracle_text.is_empty() {
                                                        p { class: "card-detail-oracle", "{oracle_text}" }
                                                    }
                                                    div { class: "card-detail-meta",
                                                        span { "{rarity_name} | {set_name}" }
                                                    }
                                                    // Quantity controls
                                                    {
                                                        let qty = quantity_map().get(&card_id).copied().unwrap_or(1);
                                                        let copy_max = deck_copy_max();
                                                        let is_singleton = copy_max == Some(CopyMax::singleton());
                                                        let singleton_fixed = is_singleton && !is_basic_land;
                                                        let image_url: Option<String> = sd.image_uris.as_ref().and_then(|iu| iu.large.clone());
                                                        rsx! {
                                                            if singleton_fixed {
                                                                div { class: "qty-row",
                                                                    if let Some(url) = image_url.clone() {
                                                                        button {
                                                                            class: "qty-btn-remove",
                                                                            onclick: move |evt| {
                                                                                evt.stop_propagation();
                                                                                preview_image_url.set(Some(url.clone()));
                                                                                preview_dismissing.set(false);
                                                                            },
                                                                            "image"
                                                                        }
                                                                    }
                                                                    button {
                                                                        class: "qty-btn-remove",
                                                                        onclick: move |evt| {
                                                                            evt.stop_propagation();
                                                                            change_quantity(card_id, -1, false);
                                                                        },
                                                                        "remove"
                                                                    }
                                                                }
                                                            } else {
                                                                div { class: "qty-row",
                                                                    if let Some(url) = image_url {
                                                                        button {
                                                                            class: "qty-btn",
                                                                            onclick: move |evt| {
                                                                                evt.stop_propagation();
                                                                                preview_image_url.set(Some(url.clone()));
                                                                                preview_dismissing.set(false);
                                                                            },
                                                                            "image"
                                                                        }
                                                                    }
                                                                    button {
                                                                        class: "qty-btn",
                                                                        onclick: move |evt| {
                                                                            evt.stop_propagation();
                                                                            change_quantity(card_id, -1, is_basic_land);
                                                                        },
                                                                        "-"
                                                                    }
                                                                    span { class: "qty-label", "{qty}" }
                                                                    button {
                                                                        class: "qty-btn",
                                                                        onclick: move |evt| {
                                                                            evt.stop_propagation();
                                                                            change_quantity(card_id, 1, is_basic_land);
                                                                        },
                                                                        "+"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
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
                if !filter_builder.read().is_empty() {
                    button {
                        class: "util-btn util-btn-clear",
                        onclick: move |_| {
                            filter_builder.write().clear();
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                            toast.info(
                                "filter cleared".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        },
                        "clear"
                    }
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
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(1)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "name" }
                            AccordionContent { Name {} }
                        }
                        AccordionItem { index: 2,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(2)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "oracle text" }
                            AccordionContent { OracleText {} }
                        }
                        AccordionItem { index: 3,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(3)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "types" }
                            AccordionContent { Types {} }
                        }
                        AccordionItem { index: 4,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(4)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "mana" }
                            AccordionContent { Mana {} }
                        }
                        AccordionItem { index: 5,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(5)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "combat" }
                            AccordionContent { Combat {} }
                        }
                        AccordionItem { index: 6,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(6)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "flavor text" }
                            AccordionContent { FlavorText {} }
                        }
                        AccordionItem { index: 7,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(7)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "artist" }
                            AccordionContent { Artist {} }
                        }
                        AccordionItem { index: 8,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(8)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "rarity" }
                            AccordionContent { Rarity {} }
                        }
                        AccordionItem { index: 9,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(9)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "set" }
                            AccordionContent { Set {} }
                        }
                        AccordionItem { index: 10,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(10)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "sort" }
                            AccordionContent { Sort {} }
                        }
                        AccordionItem { index: 11,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(11)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "config" }
                            AccordionContent { Config {} }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| {
                            filter_builder.write().clear();
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                            toast.info(
                                "filter cleared".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        },
                        "clear"
                    }
                }
            }

            // Image preview overlay
            if preview_image_url().is_some() || preview_dismissing() {
                div {
                    class: "modal-backdrop show",
                }
                div {
                    class: if preview_dismissing() {
                        "image-preview-container show dismissing"
                    } else {
                        "image-preview-container show"
                    },
                    onclick: move |_| {
                        preview_dismissing.set(true);
                        spawn(async move {
                            sleep(Duration::from_millis(200)).await;
                            preview_image_url.set(None);
                            preview_dismissing.set(false);
                        });
                    },
                    if let Some(url) = preview_image_url() {
                        img {
                            src: "{url}",
                            alt: "card preview",
                            class: "card-image",
                        }
                    }
                }
            }
            }
        }
    }
}
