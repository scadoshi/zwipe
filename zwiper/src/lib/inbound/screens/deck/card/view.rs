use super::components::card_row::CardRow;
use super::components::image_preview::ImagePreview;
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        screens::deck::card::filter::{card_filter_sheet::CardFilterSheet, deck_cards::DeckCards},
    },
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{
            get_deck::ClientGetDeck, get_deck_profile::ClientGetDeckProfile,
            get_deck_tokens::ClientGetDeckTokens,
        },
        deck_card::{
            delete_deck_card::ClientDeleteDeckCard, update_deck_card::ClientUpdateDeckCard,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe::inbound::http::ApiError;
use zwipe_core::http::contracts::deck_card::HttpUpdateDeckCard;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::{
    Card,
    search_card::{
        card_filter::builder::CardFilterBuilder,
        filter_cards::{FilterCards, SortCards},
        group_cards::{CardGroup, GroupByOption, GroupCards},
    },
};
use zwipe_core::domain::deck::{DeckEntry, quantity::Quantity};

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

    // Source of truth — all non-commander entries (active + maybeboard)
    let mut deck_entries: Signal<Vec<DeckEntry>> = use_signal(Vec::new);

    // Provide Card list context for filter sheet (derives from all entries)
    let mut deck_cards_for_filter: Signal<Vec<Card>> = use_signal(Vec::new);
    use_context_provider(|| DeckCards(deck_cards_for_filter));

    // Guards filter effect from running before the deck has loaded
    let mut deck_loaded: Signal<bool> = use_signal(|| false);
    // What the UI renders — grouped card lists (active cards only)
    let mut displayed_groups: Signal<Vec<CardGroup>> = use_signal(Vec::new);
    // Current grouping mode
    let mut group_by_option: Signal<GroupByOption> = use_signal(|| GroupByOption::CardType);
    // Which card row is expanded (None = all collapsed)
    let mut expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    // Commander is always pinned — never part of groupable entries
    let mut commander_card: Signal<Option<Card>> = use_signal(|| None);
    // Commander filtered by the active filter (None = filtered out or no commander)
    let mut displayed_commander: Signal<Option<Card>> = use_signal(|| None);
    // Toggle to show/hide land cards (default: hidden)
    let mut show_lands: Signal<bool> = use_signal(|| false);
    // Toggle to show/hide tokens at the top of the list
    let mut show_tokens: Signal<bool> = use_signal(|| false);
    // Toggle to show/hide maybeboard section
    let mut show_maybeboard: Signal<bool> = use_signal(|| false);

    // Card image preview — stores the image URL of the card to preview (None = closed)
    let preview_image_url: Signal<Option<String>> = use_signal(|| None);
    // Controls the dismiss animation before clearing the URL
    let preview_dismissing: Signal<bool> = use_signal(|| false);

    // Helper: look up quantity by card ID
    let qty_for = move |card_id: Uuid| -> i32 {
        deck_entries
            .peek()
            .iter()
            .find(|e| e.card.scryfall_data.id == card_id)
            .map(|e| *e.deck_card.quantity)
            .unwrap_or(1)
    };

    // Effect 1 — mount load (reads `session` reactively)
    // Fetches deck entries, separates the commander into its own pinned slot.
    use_effect(move || {
        session.upkeep(client);
        let Some(session) = session() else {
            return;
        };

        spawn(async move {
            let mut entries = match client().get_deck(deck_id, &session).await {
                Ok(deck) => deck.entries,
                Err(e) => {
                    toast.error(
                        e.to_string(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };

            // Resolve commander: pull from entries if present, otherwise fetch separately.
            // Either way remove it from the groupable list.
            if let Ok(profile) = client().get_deck_profile(deck_id, &session).await
                && let Some(commander_id) = profile.commander_id
            {
                let cmd = if let Some(idx) = entries
                    .iter()
                    .position(|e| e.card.scryfall_data.id == commander_id)
                {
                    Some(entries.remove(idx).card)
                } else {
                    client().get_card(commander_id, &session).await.ok()
                };
                commander_card.set(cmd);
            }

            // Update DeckCards context for filter sheet (all cards including maybeboard)
            let all_cards: Vec<Card> = entries.iter().map(|e| e.card.clone()).collect();
            deck_cards_for_filter.set(all_cards);

            deck_entries.set(entries);
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

    let tokens_resource: Resource<Result<Vec<Card>, ApiError>> = use_resource(move || async move {
        session.upkeep(client);
        let Some(session) = session() else {
            return Ok(Vec::new());
        };
        client().get_deck_tokens(deck_id, &session).await
    });

    // Effect 2 — filter + group (reads `filter_reset_counter`, `group_by_option`, `show_lands` reactively)
    // Operates on active (non-maybeboard) cards only.
    use_effect(move || {
        let _ = filter_reset_counter();
        let _ = group_by_option();
        let _ = show_lands();

        if !*deck_loaded.peek() {
            return;
        }

        // Extract active (non-maybeboard) cards for filtering and grouping
        let active_cards: Vec<Card> = deck_entries
            .peek()
            .iter()
            .filter(|e| !e.deck_card.maybeboard)
            .map(|e| e.card.clone())
            .collect();

        // Also update the DeckCards context for the filter sheet (all cards)
        // We can't write to a Memo, so we just use the context signal
        // (The filter sheet reads DeckCards context for extracting selectable values)

        let builder = filter_builder.peek().clone();
        let group_option = *group_by_option.peek();
        let lands_visible = *show_lands.peek();
        let cmd = commander_card.peek().clone();

        let (mut filtered, new_displayed_commander) = if builder.is_empty() {
            (active_cards, cmd)
        } else {
            let mut b = builder.clone();
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => {
                    let filtered = active_cards.filter_by(&filter);
                    let cmd_visible =
                        cmd.filter(|c| !vec![c.clone()].filter_by(&filter).is_empty());
                    (filtered, cmd_visible)
                }
                Err(_) => {
                    let fallback: Vec<Card> = deck_entries
                        .peek()
                        .iter()
                        .filter(|e| !e.deck_card.maybeboard)
                        .map(|e| e.card.clone())
                        .collect();
                    (fallback, cmd)
                }
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

    let mut change_quantity = move |card_id: Uuid, delta: i32, _is_basic_land: bool| {
        let current_qty = qty_for(card_id);

        // - at 1 → delete
        let should_delete = current_qty + delta < 1;

        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        if should_delete {
            // Optimistic: remove from entries
            deck_entries.write().retain(|e| e.card.scryfall_data.id != card_id);
            // Trigger re-filter
            let current = *filter_reset_counter.peek();
            filter_reset_counter.set(current + 1);

            toast.info(
                "card removed".to_string(),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );

            spawn(async move {
                if let Err(e) = client().delete_deck_card(deck_id, card_id, &session).await {
                    toast.error(e.to_string(), ToastOptions::default());
                }
            });
        } else {
            // Optimistic: update quantity in entries
            if let Some(entry) = deck_entries
                .write()
                .iter_mut()
                .find(|e| e.card.scryfall_data.id == card_id)
                && let Ok(new_qty) = Quantity::new(current_qty + delta)
            {
                entry.deck_card.quantity = new_qty;
            }

            let request = HttpUpdateDeckCard::new(Some(delta), None);
            spawn(async move {
                if let Err(e) = client()
                    .update_deck_card(deck_id, card_id, &request, &session)
                    .await
                {
                    toast.error(e.to_string(), ToastOptions::default());
                    // Rollback optimistic update
                    if let Some(entry) = deck_entries
                        .write()
                        .iter_mut()
                        .find(|e| e.card.scryfall_data.id == card_id)
                        && let Ok(reverted) = Quantity::new(current_qty)
                    {
                        entry.deck_card.quantity = reverted;
                    }
                }
            });
        }
    };

    let mut move_to_maybeboard = move |card_id: Uuid| {
        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        // Optimistic: flip the flag
        if let Some(entry) = deck_entries
            .write()
            .iter_mut()
            .find(|e| e.card.scryfall_data.id == card_id)
        {
            entry.deck_card.maybeboard = true;
        }
        let current = *filter_reset_counter.peek();
        filter_reset_counter.set(current + 1);

        let request = HttpUpdateDeckCard::new(None, Some(true));
        spawn(async move {
            if let Err(e) = client()
                .update_deck_card(deck_id, card_id, &request, &session)
                .await
            {
                toast.error(e.to_string(), ToastOptions::default());
                // Rollback
                if let Some(entry) = deck_entries
                    .write()
                    .iter_mut()
                    .find(|e| e.card.scryfall_data.id == card_id)
                {
                    entry.deck_card.maybeboard = false;
                }
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
            }
        });

        toast.info(
            "moved to maybeboard".to_string(),
            ToastOptions::default().duration(Duration::from_millis(1500)),
        );
    };

    let mut move_to_deck = move |card_id: Uuid| {
        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        // Optimistic: flip the flag
        if let Some(entry) = deck_entries
            .write()
            .iter_mut()
            .find(|e| e.card.scryfall_data.id == card_id)
        {
            entry.deck_card.maybeboard = false;
        }
        let current = *filter_reset_counter.peek();
        filter_reset_counter.set(current + 1);

        let request = HttpUpdateDeckCard::new(None, Some(false));
        spawn(async move {
            if let Err(e) = client()
                .update_deck_card(deck_id, card_id, &request, &session)
                .await
            {
                toast.error(e.to_string(), ToastOptions::default());
                // Rollback
                if let Some(entry) = deck_entries
                    .write()
                    .iter_mut()
                    .find(|e| e.card.scryfall_data.id == card_id)
                {
                    entry.deck_card.maybeboard = true;
                }
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
            }
        });

        toast.info(
            "added to deck".to_string(),
            ToastOptions::default().duration(Duration::from_millis(1500)),
        );
    };

    let maybeboard_entries: Vec<DeckEntry> = deck_entries()
        .into_iter()
        .filter(|e| e.deck_card.maybeboard)
        .collect();
    let mb_count = maybeboard_entries.len();

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "deck cards" }
                }

                div { class: "screen-content",

                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    // Group-by row
                    div { class: "chip-row",
                        span { class: "chip-row-label", "group by:" }
                        for option in GroupByOption::all() {
                            button {
                                key: "{option}",
                                class: if group_by_option() == option { "chip selected" } else { "chip" },
                                onclick: move |_| group_by_option.set(option),
                                "{option}"
                            }
                        }
                    }

                    // Show toggles row
                    div { class: "chip-row",
                        span { class: "chip-row-label", "show:" }
                        button {
                            class: if show_lands() { "chip selected" } else { "chip" },
                            onclick: move |_| show_lands.set(!show_lands()),
                            "lands"
                        }
                        button {
                            class: if show_tokens() { "chip selected" } else { "chip" },
                            onclick: move |_| show_tokens.set(!show_tokens()),
                            "tokens"
                        }
                        if mb_count > 0 {
                            button {
                                class: if show_maybeboard() { "chip selected" } else { "chip" },
                                onclick: move |_| show_maybeboard.set(!show_maybeboard()),
                                "maybeboard ({mb_count})"
                            }
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

                    // Token list (above everything when toggled)
                    if show_tokens() {
                        if let Some(Ok(tokens)) = tokens_resource() {
                            if !tokens.is_empty() {
                                div { class: "card-group row-enter",
                                    div { class: "card-group-header", "tokens ({tokens.len()})" }
                                    for token in tokens.iter() {
                                        CardRow {
                                            card: token.clone(),
                                            qty: 1,
                                            expanded_card,
                                            preview_image_url,
                                            preview_dismissing,
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Maybeboard section (between tokens and commander)
                    if show_maybeboard() && !maybeboard_entries.is_empty() {
                        div { class: "card-group row-enter",
                            div { class: "card-group-header", "maybeboard ({mb_count})" }
                            for entry in maybeboard_entries.iter() {
                                {
                                    let card_id = entry.card.scryfall_data.id;
                                    let is_basic_land = entry.card.scryfall_data.is_basic_land();
                                    let qty = *entry.deck_card.quantity;
                                    rsx! {
                                        CardRow {
                                            card: entry.card.clone(),
                                            qty,
                                            expanded_card,
                                            preview_image_url,
                                            preview_dismissing,
                                            on_qty_change: move |delta: i32| change_quantity(card_id, delta, is_basic_land),
                                            on_maybeboard_toggle: move |_| move_to_deck(card_id),
                                            maybeboard_label: "to deck".to_string(),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Pinned commander group
                    if let Some(cmd) = displayed_commander() {
                        div { class: "card-group row-enter",
                            div { class: "card-group-header", "commander" }
                            CardRow {
                                card: cmd,
                                qty: 1,
                                expanded_card,
                                preview_image_url,
                                preview_dismissing,
                            }
                        }
                    }

                    // Card groups (active deck cards only)
                    for group in displayed_groups() {
                        {
                            let qty_count: i32 = group.cards.iter()
                                .map(|c| qty_for(c.scryfall_data.id))
                                .sum();
                            rsx! {
                        div { class: "card-group row-enter",
                            div { class: "card-group-header",
                                "{group.label} ({qty_count})"
                            }
                            for card in group.cards.iter() {
                                {
                                    let card_id = card.scryfall_data.id;
                                    let is_basic_land = card.scryfall_data.is_basic_land();
                                    let qty = qty_for(card_id);
                                    rsx! {
                                        CardRow {
                                            card: card.clone(),
                                            qty,
                                            expanded_card,
                                            preview_image_url,
                                            preview_dismissing,
                                            on_qty_change: move |delta: i32| change_quantity(card_id, delta, is_basic_land),
                                            on_maybeboard_toggle: move |_| move_to_maybeboard(card_id),
                                            maybeboard_label: "to maybe".to_string(),
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
                    onclick: move |_| {
                        navigator.push(crate::inbound::router::Router::AddDeckCard { deck_id });
                    },
                    "add"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(crate::inbound::router::Router::RemoveDeckCard { deck_id });
                    },
                    "remove"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| filters_overlay_open.set(true),
                    "filter"
                    if !filter_builder.read().is_empty() {
                        span { class: "filter-dot" }
                    }
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
                        "clear filter"
                    }
                }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
            }

            ImagePreview { url: preview_image_url, dismissing: preview_dismissing }
            }
        }
    }
}
