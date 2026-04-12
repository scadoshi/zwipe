use super::components::card_row::CardRow;
use super::components::image_preview::ImagePreview;
use super::components::printing_sheet::PrintingSheet;
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
            update_deck_profile::ClientUpdateDeckProfile,
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
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::contracts::deck_card::HttpUpdateDeckCard;
use zwipe_core::http::helpers::Opdate;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::{
    Card,
    search_card::{
        card_filter::builder::CardFilterBuilder,
        filter_cards::{FilterCards, SortCards},
        group_cards::{CardGroup, GroupByOption, GroupCards},
    },
};
use zwipe_core::domain::deck::{Board, DeckEntry, quantity::Quantity};

/// Identifies which command zone slot a card occupies for printing updates.
#[derive(Clone, Copy)]
enum CommandZoneSlot {
    Commander,
    Partner,
    Background,
    SignatureSpell,
}

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
    // Signature spell pinned alongside commander (Oathbreaker only)
    let mut signature_spell_card: Signal<Option<Card>> = use_signal(|| None);
    // Whether the format is Oathbreaker (for label swapping)
    let mut is_oathbreaker: Signal<bool> = use_signal(|| false);
    // Commander filtered by the active filter (None = filtered out or no commander)
    let mut displayed_commander: Signal<Option<Card>> = use_signal(|| None);
    // Signature spell filtered by the active filter
    let mut displayed_signature_spell: Signal<Option<Card>> = use_signal(|| None);
    // Partner commander pinned alongside commander
    let mut partner_card: Signal<Option<Card>> = use_signal(|| None);
    let mut displayed_partner: Signal<Option<Card>> = use_signal(|| None);
    // Background enchantment pinned in its own group
    let mut background_card: Signal<Option<Card>> = use_signal(|| None);
    let mut displayed_background: Signal<Option<Card>> = use_signal(|| None);
    // Toggle to show/hide land cards (default: hidden)
    let mut show_lands: Signal<bool> = use_signal(|| false);
    // Toggle to show/hide tokens at the top of the list
    let mut show_tokens: Signal<bool> = use_signal(|| false);
    // Toggle to show/hide command zone pinned sections (default: shown)
    let mut show_command_zone: Signal<bool> = use_signal(|| true);
    // Board filter — multi-select toggles (deck is always on)
    let mut show_deck: Signal<bool> = use_signal(|| true);
    let mut show_maybe: Signal<bool> = use_signal(|| false);
    let mut show_side: Signal<bool> = use_signal(|| false);

    // Card image preview — stores the image URL of the card to preview (None = closed)
    let preview_image_url: Signal<Option<String>> = use_signal(|| None);
    // Printing sheet state
    let mut printing_sheet_open: Signal<bool> = use_signal(|| false);
    let mut printing_sheet_card: Signal<Option<Card>> = use_signal(|| None);
    let mut command_zone_slot: Signal<Option<CommandZoneSlot>> = use_signal(|| None);
    // Controls the dismiss animation before clearing the URL
    let preview_dismissing: Signal<bool> = use_signal(|| false);
    // Whether the next filter effect run should collapse the expanded card
    // (true for structural changes like group-by/filter/board; false for qty bumps)
    let mut should_collapse_expanded: Signal<bool> = use_signal(|| false);
    use_context_provider(|| should_collapse_expanded);

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

            // Resolve commander and signature spell from profile.
            // Pull from entries if present, otherwise fetch separately.
            if let Ok(profile) = client().get_deck_profile(deck_id, &session).await {
                is_oathbreaker.set(profile.format.as_ref().is_some_and(|f| f.has_signature_spell()));

                // Resolve command zone cards by oracle_id (not printing-specific scryfall_data_id).
                // Fetch the card first to get its oracle_id, then remove from entries by oracle_id.
                if let Some(commander_id) = profile.commander_id {
                    let fetched = client().get_card(commander_id, &session).await.ok();
                    if let Some(ref card) = fetched && let Some(oid) = card.scryfall_data.oracle_id {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    commander_card.set(fetched);
                }

                if let Some(spell_id) = profile.signature_spell_id {
                    let fetched = client().get_card(spell_id, &session).await.ok();
                    if let Some(ref card) = fetched && let Some(oid) = card.scryfall_data.oracle_id {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    signature_spell_card.set(fetched);
                }

                if let Some(partner_id) = profile.partner_commander_id {
                    let fetched = client().get_card(partner_id, &session).await.ok();
                    if let Some(ref card) = fetched && let Some(oid) = card.scryfall_data.oracle_id {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    partner_card.set(fetched);
                }

                if let Some(bg_id) = profile.background_id {
                    let fetched = client().get_card(bg_id, &session).await.ok();
                    if let Some(ref card) = fetched && let Some(oid) = card.scryfall_data.oracle_id {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    background_card.set(fetched);
                }
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

    // Effect 2 — filter + group (reads `filter_reset_counter`, `group_by_option`, `show_lands`, `board_filter` reactively)
    use_effect(move || {
        let _ = filter_reset_counter();
        let _ = group_by_option();
        let _ = show_lands();

        if !*deck_loaded.peek() {
            return;
        }

        // Only active deck cards go through the group-by pipeline.
        // Maybeboard and sideboard are rendered as their own sections.
        let active_cards: Vec<Card> = deck_entries
            .peek()
            .iter()
            .filter(|e| e.deck_card.board.is_active())
            .map(|e| e.card.clone())
            .collect();

        let builder = filter_builder.peek().clone();
        let group_option = *group_by_option.peek();
        let lands_visible = *show_lands.peek();
        let cmd = commander_card.peek().clone();
        let spell = signature_spell_card.peek().clone();
        let partner = partner_card.peek().clone();
        let bg = background_card.peek().clone();

        // Apply filter to pinned cards helper
        let filter_pinned = |card: Option<Card>, filter: &zwipe_core::domain::card::search_card::card_filter::CardFilter| -> Option<Card> {
            card.filter(|c| !vec![c.clone()].filter_by(filter).is_empty())
        };

        let (mut filtered, new_cmd, new_spell, new_partner, new_bg) = if builder.is_empty() {
            (active_cards, cmd, spell, partner, bg)
        } else {
            let mut b = builder.clone();
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => {
                    let filtered = active_cards.filter_by(&filter);
                    (
                        filtered,
                        filter_pinned(cmd, &filter),
                        filter_pinned(spell, &filter),
                        filter_pinned(partner, &filter),
                        filter_pinned(bg, &filter),
                    )
                }
                Err(_) => {
                    (active_cards, cmd, spell, partner, bg)
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
        displayed_commander.set(new_cmd);
        displayed_signature_spell.set(new_spell);
        displayed_partner.set(new_partner);
        displayed_background.set(new_bg);
        if *should_collapse_expanded.peek() {
            expanded_card.set(None);
            should_collapse_expanded.set(false);
        }
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
            // Trigger re-render so qty display updates
            let current = *filter_reset_counter.peek();
            filter_reset_counter.set(current + 1);

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
                    let current = *filter_reset_counter.peek();
                    filter_reset_counter.set(current + 1);
                }
            });
        }
    };

    let mut move_to_board = move |card_id: Uuid, target: Board| {
        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        // Save old board for rollback
        let old_board = deck_entries
            .peek()
            .iter()
            .find(|e| e.card.scryfall_data.id == card_id)
            .map(|e| e.deck_card.board)
            .unwrap_or(Board::Deck);

        // Optimistic: set the board
        if let Some(entry) = deck_entries
            .write()
            .iter_mut()
            .find(|e| e.card.scryfall_data.id == card_id)
        {
            entry.deck_card.board = target;
        }
        let current = *filter_reset_counter.peek();
        filter_reset_counter.set(current + 1);

        let request = HttpUpdateDeckCard::new(None, Some(target.display_name().to_string()));
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
                    entry.deck_card.board = old_board;
                }
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
            }
        });

        toast.info(
            format!("moved to {}", target.display_name()),
            ToastOptions::default().duration(Duration::from_millis(1500)),
        );
    };

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
                                onclick: move |_| {
                                    should_collapse_expanded.set(true);
                                    group_by_option.set(option);
                                },
                                "{option}"
                            }
                        }
                    }

                    // Board filter row (multi-select, at least one must be on)
                    div { class: "chip-row",
                        span { class: "chip-row-label", "boards:" }
                        button {
                            class: if show_deck() { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                let new_val = !show_deck();
                                show_deck.set(new_val);
                                // If nothing is on, snap deck back on
                                if !new_val && !show_maybe() && !show_side() {
                                    show_deck.set(true);
                                }
                                should_collapse_expanded.set(true);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "deck"
                        }
                        button {
                            class: if show_maybe() { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                let new_val = !show_maybe();
                                show_maybe.set(new_val);
                                if !show_deck() && !new_val && !show_side() {
                                    show_deck.set(true);
                                }
                                should_collapse_expanded.set(true);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "maybe"
                        }
                        button {
                            class: if show_side() { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                let new_val = !show_side();
                                show_side.set(new_val);
                                if !show_deck() && !show_maybe() && !new_val {
                                    show_deck.set(true);
                                }
                                should_collapse_expanded.set(true);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "side"
                        }
                    }

                    // Show toggles row
                    div { class: "chip-row",
                        span { class: "chip-row-label", "show:" }
                        button {
                            class: if show_lands() { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                should_collapse_expanded.set(true);
                                show_lands.set(!show_lands());
                            },
                            "lands"
                        }
                        button {
                            class: if show_tokens() { "chip selected" } else { "chip" },
                            onclick: move |_| show_tokens.set(!show_tokens()),
                            "tokens"
                        }
                        button {
                            class: if show_command_zone() { "chip selected" } else { "chip" },
                            onclick: move |_| show_command_zone.set(!show_command_zone()),
                            "command zone"
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

                    // Token list
                    if show_tokens() {
                        {
                            let sorted_tokens: Option<Vec<Card>> = tokens_resource().and_then(|r| r.ok()).map(|mut t| {
                                t.sort_by_filter(&filter_builder.peek());
                                t
                            });
                            rsx! {
                                if let Some(tokens) = sorted_tokens {
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
                        }
                    }

                    // Maybeboard section
                    {
                        let mut mb_entries: Vec<DeckEntry> = deck_entries()
                            .into_iter()
                            .filter(|e| e.deck_card.board.is_maybeboard())
                            .collect();
                        mb_entries.sort_by_filter(&filter_builder.peek());
                        let mb_count = mb_entries.len();
                        rsx! {
                            if show_maybe() && !mb_entries.is_empty() {
                                div { class: "card-group row-enter",
                                    div { class: "card-group-header", "maybeboard ({mb_count})" }
                                    for entry in mb_entries.iter() {
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
                                                    on_move_to: move |target: Board| move_to_board(card_id, target),
                                                    current_board: Some(Board::Maybeboard),
                                                    on_printing: move |card: Card| {
                                                        command_zone_slot.set(None);
                                                        printing_sheet_card.set(Some(card));
                                                        printing_sheet_open.set(true);
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Sideboard section
                    {
                        let mut sb_entries: Vec<DeckEntry> = deck_entries()
                            .into_iter()
                            .filter(|e| e.deck_card.board.is_sideboard())
                            .collect();
                        sb_entries.sort_by_filter(&filter_builder.peek());
                        let sb_count = sb_entries.len();
                        rsx! {
                            if show_side() && !sb_entries.is_empty() {
                                div { class: "card-group row-enter",
                                    div { class: "card-group-header", "sideboard ({sb_count})" }
                                    for entry in sb_entries.iter() {
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
                                                    on_move_to: move |target: Board| move_to_board(card_id, target),
                                                    current_board: Some(Board::Sideboard),
                                                    on_printing: move |card: Card| {
                                                        command_zone_slot.set(None);
                                                        printing_sheet_card.set(Some(card));
                                                        printing_sheet_open.set(true);
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if show_command_zone() {
                        // Pinned commander/oathbreaker group (includes partner if present)
                        if displayed_commander().is_some() || displayed_partner().is_some() {
                            div { class: "card-group row-enter",
                                div { class: "card-group-header",
                                    if is_oathbreaker() {
                                        "oathbreaker"
                                    } else if displayed_partner().is_some() {
                                        "commanders"
                                    } else {
                                        "commander"
                                    }
                                }
                                if let Some(cmd) = displayed_commander() {
                                    CardRow {
                                        card: cmd,
                                        qty: 1,
                                        expanded_card,
                                        preview_image_url,
                                        preview_dismissing,
                                        on_printing: move |card: Card| {
                                            command_zone_slot.set(Some(CommandZoneSlot::Commander));
                                            printing_sheet_card.set(Some(card));
                                            printing_sheet_open.set(true);
                                        },
                                    }
                                }
                                if let Some(partner) = displayed_partner() {
                                    CardRow {
                                        card: partner,
                                        qty: 1,
                                        expanded_card,
                                        preview_image_url,
                                        preview_dismissing,
                                        on_printing: move |card: Card| {
                                            command_zone_slot.set(Some(CommandZoneSlot::Partner));
                                            printing_sheet_card.set(Some(card));
                                            printing_sheet_open.set(true);
                                        },
                                    }
                                }
                            }
                        }

                        // Pinned background group
                        if let Some(bg) = displayed_background() {
                            div { class: "card-group row-enter",
                                div { class: "card-group-header", "background" }
                                CardRow {
                                    card: bg,
                                    qty: 1,
                                    expanded_card,
                                    preview_image_url,
                                    preview_dismissing,
                                    on_printing: move |card: Card| {
                                        command_zone_slot.set(Some(CommandZoneSlot::Background));
                                        printing_sheet_card.set(Some(card));
                                        printing_sheet_open.set(true);
                                    },
                                }
                            }
                        }

                        // Pinned signature spell (Oathbreaker only)
                        if let Some(spell) = displayed_signature_spell() {
                            div { class: "card-group row-enter",
                                div { class: "card-group-header", "signature spell" }
                                CardRow {
                                    card: spell,
                                    qty: 1,
                                    expanded_card,
                                    preview_image_url,
                                    preview_dismissing,
                                    on_printing: move |card: Card| {
                                        command_zone_slot.set(Some(CommandZoneSlot::SignatureSpell));
                                        printing_sheet_card.set(Some(card));
                                        printing_sheet_open.set(true);
                                    },
                                }
                            }
                        }
                    }

                    // Card groups (active deck cards only)
                    if show_deck() { for group in displayed_groups() {
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
                                    let card_board = deck_entries.peek().iter()
                                        .find(|e| e.card.scryfall_data.id == card_id)
                                        .map(|e| e.deck_card.board);
                                    rsx! {
                                        CardRow {
                                            card: card.clone(),
                                            qty,
                                            expanded_card,
                                            preview_image_url,
                                            preview_dismissing,
                                            on_qty_change: move |delta: i32| change_quantity(card_id, delta, is_basic_land),
                                            on_move_to: move |target: Board| move_to_board(card_id, target),
                                            current_board: card_board,
                                            on_printing: move |card: Card| {
                                                command_zone_slot.set(None);
                                                printing_sheet_card.set(Some(card));
                                                printing_sheet_open.set(true);
                                            },
                                        }
                                    }
                                }
                            }
                        }
                            }
                        }
                    }
                    }

                    if show_deck() && displayed_groups().is_empty() && *deck_loaded.peek() {
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
                            should_collapse_expanded.set(true);
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

            if let Some(card) = printing_sheet_card() {
                PrintingSheet {
                    card: card.clone(),
                    open: printing_sheet_open,
                    on_save: move |new_card: Card| {
                        let old_id = card.scryfall_data.id;
                        let new_id = new_card.scryfall_data.id;

                        session.upkeep(client);
                        let Some(session_val) = session() else { return; };

                        match command_zone_slot() {
                            Some(slot) => {
                                // Command zone card — update deck profile
                                let id = Opdate::Set(Some(new_id));
                                let request = match slot {
                                    CommandZoneSlot::Commander => HttpUpdateDeckProfile::builder().commander_id(id).build(),
                                    CommandZoneSlot::Partner => HttpUpdateDeckProfile::builder().partner_commander_id(id).build(),
                                    CommandZoneSlot::Background => HttpUpdateDeckProfile::builder().background_id(id).build(),
                                    CommandZoneSlot::SignatureSpell => HttpUpdateDeckProfile::builder().signature_spell_id(id).build(),
                                };
                                spawn(async move {
                                    if client().update_deck_profile(deck_id, &request, &session_val).await.is_ok() {
                                        match slot {
                                            CommandZoneSlot::Commander => commander_card.set(Some(new_card.clone())),
                                            CommandZoneSlot::Partner => partner_card.set(Some(new_card.clone())),
                                            CommandZoneSlot::Background => background_card.set(Some(new_card.clone())),
                                            CommandZoneSlot::SignatureSpell => signature_spell_card.set(Some(new_card.clone())),
                                        }
                                        printing_sheet_card.set(Some(new_card));
                                        let next = filter_reset_counter() + 1;
                                        filter_reset_counter.set(next);
                                    }
                                });
                            }
                            None => {
                                // Regular deck card — update deck card
                                let request = HttpUpdateDeckCard::with_printing(&new_id.to_string());
                                spawn(async move {
                                    if client().update_deck_card(deck_id, old_id, &request, &session_val).await.is_ok() {
                                        if let Some(entry) = deck_entries
                                            .write()
                                            .iter_mut()
                                            .find(|e| e.card.scryfall_data.id == old_id)
                                        {
                                            entry.card = new_card.clone();
                                            entry.deck_card.scryfall_data_id = new_id;
                                        }
                                        printing_sheet_card.set(Some(new_card));
                                        let next = filter_reset_counter() + 1;
                                        filter_reset_counter.set(next);
                                    }
                                });
                            }
                        }
                    },
                }
            }
            }
        }
    }
}
