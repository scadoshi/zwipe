//! Quick-add search bar for the deck cards screen.
//!
//! Type a card name, get a debounced, deck-aware search (cards already in the
//! deck are excluded server-side), and add a result straight to the mainboard
//! with one tap. Synergy is deliberately off: `set_synergy(false)` drops the
//! synergy-pool membership, and an explicit name sort keeps the server from
//! applying synergy *ordering* (which it does whenever no sort is set). The
//! result is a plain, alphabetically-ordered name match.

use crate::{
    inbound::components::{
        auth::ensure_session::EnsureFresh,
        telemetry::{
            usage_buffer::UsageBuffer,
            vocabulary::{component, screen},
        },
    },
    outbound::client::{
        ZwipeClient, deck::search_deck_cards::ClientSearchDeckCards,
        deck_card::create_deck_card::ClientCreateDeckCard,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{
            Card,
            search_card::card_filter::{builder::CardQueryBuilder, card_sort_key::CardSortKey},
        },
        deck::DeckEntry,
    },
    http::contracts::deck_card::HttpCreateDeckCard,
};

/// Minimum characters before a search fires — keeps single-letter typing from
/// hammering the server and returning a near-full pool.
const MIN_QUERY_LEN: usize = 2;
/// Debounce so results follow typing without a request per keystroke.
const DEBOUNCE_MS: u64 = 300;
/// Result cap — matches the commander picker's 5, keeping the float to about
/// two chip rows so the header above comfortably covers it.
const RESULT_LIMIT: u32 = 5;

/// Grows the float anchor by exactly the height the results overflow past the
/// scrollport top (back toward 0 when they fit with slack), so results overlay
/// without reflow in most cases and spend real space only when they'd run
/// under the screen header. Self-correcting: each run adds the current
/// signed deficit to the anchor's current height, so it converges as results
/// change. Runs on the next frame (post-layout) and once more after the
/// focus smooth-scroll has settled.
const QUICK_ADD_FIT_JS: &str = r#"
const fit = () => {
    const anchor = document.getElementById('quick-add-anchor');
    const results = document.getElementById('quick-add-results');
    const content = document.querySelector('.screen-content');
    if (!anchor || !results || !content) return;
    // Rendered height (not the inline target) so a mid-transition remeasure
    // still lands on the right total instead of compounding. The 8px gap
    // keeps the top chip row from sitting flush against the header's border.
    const gap = 8;
    const current = anchor.getBoundingClientRect().height;
    const deficit = content.getBoundingClientRect().top + gap - results.getBoundingClientRect().top;
    anchor.style.height = Math.max(0, current + deficit) + 'px';
};
requestAnimationFrame(fit);
setTimeout(fit, 400);
"#;

/// Live card-name search that adds the picked card straight to the mainboard.
///
/// `deck_entries` is the view's source of truth; a successful add pushes the new
/// entry in and bumps `filter_reset_counter` (read from context) so the deck
/// list re-groups and shows the card immediately.
#[component]
pub fn QuickAdd(deck_id: Uuid, deck_entries: Signal<Vec<DeckEntry>>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let mut filter_reset_counter: Signal<u32> = use_context();
    let toast = use_toast();

    let mut query = use_signal(String::new);
    let mut results: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

    // Debounced deck-aware search. Mirrors the commander picker in deck_fields:
    // reveal the dropdown at once (so "Searching..." shows during the wait),
    // sleep, then bail if the query moved on before the timer elapsed.
    use_effect(move || {
        let q = query();

        if q.trim().len() < MIN_QUERY_LEN {
            show_dropdown.set(false);
            is_searching.set(false);
            return;
        }

        is_searching.set(true);
        show_dropdown.set(true);

        spawn(async move {
            sleep(Duration::from_millis(DEBOUNCE_MS)).await;

            if query() != q {
                return;
            }

            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(_) => {
                    is_searching.set(false);
                    show_dropdown.set(false);
                    return;
                }
            };

            let mut builder = CardQueryBuilder::with_name_contains(q.trim());
            // Synergy off: no membership pool, and an explicit sort so the server
            // doesn't fall back to synergy ordering.
            builder.set_synergy(false);
            builder.set_sort(CardSortKey::Name);
            builder.set_limit(RESULT_LIMIT);
            let Ok(card_filter) = builder.build() else {
                is_searching.set(false);
                show_dropdown.set(false);
                return;
            };

            usage_buffer().record_search();
            match client()
                .search_deck_cards(deck_id, &card_filter, &session)
                .await
            {
                Ok((cards, _synergy_warming)) => {
                    results.set(cards);
                    is_searching.set(false);
                    show_dropdown.set(true);
                }
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::QUICK_ADD,
                        "quick_add_search",
                        &e,
                    );
                    is_searching.set(false);
                    show_dropdown.set(false);
                }
            }
        });
    });

    // Add a result to the mainboard. Drops it from the dropdown at once (so it
    // can't be double-added and the tap feels instant), then pushes the real
    // entry once the server returns it and re-runs the deck's filter/group pass.
    let mut add_card = move |card: Card| {
        let card_id = card.scryfall_data.id;
        let card_name = card.scryfall_data.name.clone();
        results.write().retain(|c| c.scryfall_data.id != card_id);

        let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::QUICK_ADD,
                        "quick_add_card",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            match client().create_deck_card(deck_id, &request, &session).await {
                Ok(deck_card) => {
                    deck_entries.write().push(DeckEntry { card, deck_card });
                    let current = *filter_reset_counter.peek();
                    filter_reset_counter.set(current + 1);
                    toast.info(
                        format!("Added {card_name}"),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::QUICK_ADD,
                        "quick_add_card",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

    // Measured overlay: results float over the content above (no reflow) in
    // the common case; when they'd run past the scrollport top — under the
    // screen header — the fit effect below grows the anchor by exactly the
    // overflow, spending real space only in those edge cases.
    use_effect(move || {
        let open = show_dropdown();
        let _ = is_searching();
        let _ = results();
        if open {
            let _ = document::eval(QUICK_ADD_FIT_JS);
        }
    });

    rsx! {
        div {
            // Results float in a layer hung above the bar from the (measured,
            // normally zero-height) anchor. Tap a chip to add it to the
            // mainboard.
            if show_dropdown() {
                div { class: "search-float-anchor", id: "quick-add-anchor",
                    if is_searching() {
                        div { class: "search-float-results", id: "quick-add-results",
                            div { class: "chip-unselected search-float-chip", "Searching..." }
                        }
                    } else if results().is_empty() {
                        div { class: "search-float-results", id: "quick-add-results",
                            div { class: "chip-unselected search-float-chip", "No results" }
                        }
                    } else {
                        div { class: "search-float-results", id: "quick-add-results",
                            for (i, card) in results().iter().cloned().enumerate() {
                                div {
                                    class: "chip-unselected search-float-chip",
                                    style: "animation-delay: {i * 40}ms, {250 + i * 40}ms;",
                                    onclick: move |_| add_card(card.clone()),
                                    "{card.scryfall_data.name}"
                                }
                            }
                        }
                    }
                }
            }

            // Label sits inline-left like the Group by / Boards / Show rows,
            // in the same accent-tertiary `chip-row-label`; the input fills the rest.
            div { class: "chip-row",
                span { class: "chip-row-label", "Quick add:" }
                input {
                    class: "input input-compact",
                    style: "flex: 1; margin-bottom: 0;",
                    id: "quick-add",
                    r#type: "text",
                    placeholder: "Search a card to add",
                    value: "{query}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    // Scroll back to the top on focus so the header above the
                    // bar — the float's canvas — is in view before results
                    // appear, instead of them tucking under the screen header.
                    onfocus: move |_| {
                        let _ = document::eval(
                            "const el = document.querySelector('.screen-content'); if (el) el.scrollTo({ top: 0, behavior: 'smooth' });",
                        );
                    },
                    oninput: move |event| query.set(event.value()),
                }
                if !query().is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            query.set(String::new());
                            results.set(Vec::new());
                            show_dropdown.set(false);
                        },
                        "\u{00d7}"
                    }
                }
            }
        }
    }
}
