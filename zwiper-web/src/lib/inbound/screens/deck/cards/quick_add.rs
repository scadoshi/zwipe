//! The deck cards view's Quick add: type a name, pick a result, and the card
//! goes on the main deck.
//!
//! A search waits for a pause in typing and two characters, offers five
//! names, and surfaces skipped and removed cards too, since a typed name is
//! deliberate. The results sit under the box, pushing the rows below down.
//! ↑ and ↓ move through them and Enter adds the highlighted one; a pick
//! clears the box, which closes them. Esc closes them and leaves the box, so the page's keys
//! answer again. `/` focuses the box.

use components::{ClearWrap, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::deck::DeckEntry;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
use zwipe_ui::generation::Generation;
use zwipe_ui::stores::undo::UndoAction;
use zwipe_ui::timing::{QUICK_ADD_DEBOUNCE, QUICK_ADD_LIMIT, QUICK_ADD_MIN_CHARS};

use crate::inbound::card_stores::CardStores;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::telemetry;
use crate::outbound::timer;

/// The input's id, for `/`.
pub(crate) const QUICK_ADD_ID: &str = "quick-add";

/// Takes focus off the box.
const BLUR_JS: &str = "document.getElementById('__ID__')?.blur();";

/// The Quick add row. A picked card lands in `entries`, and `on_added` lets
/// the view run its pass again.
#[component]
pub(crate) fn QuickAdd(
    deck_id: Uuid,
    mut entries: Signal<Vec<DeckEntry>>,
    on_added: EventHandler<()>,
) -> Element {
    let authed = use_authed();
    let usage = telemetry::use_usage();
    let toast = use_toast();
    let CardStores { mut undo, .. } = use_context();
    let mut query = use_signal(String::new);
    let mut results: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut searching = use_signal(|| false);
    let mut show = use_signal(|| false);
    let mut highlight = use_signal(|| 0usize);
    let mut searches = use_signal(Generation::default);

    use_effect(move || {
        let q = query();
        let ticket = searches.write().next_ticket();
        highlight.set(0);
        let trimmed = q.trim().to_owned();
        if trimmed.len() < QUICK_ADD_MIN_CHARS {
            show.set(false);
            return;
        }
        show.set(true);
        searching.set(true);
        spawn(async move {
            timer::sleep(QUICK_ADD_DEBOUNCE).await;
            if !searches.peek().is_current(ticket) {
                return;
            }
            let mut builder = CardQueryBuilder::with_name_contains(&trimmed);
            builder
                .set_synergy(false)
                .set_include_skipped(true)
                .set_sort(CardSortKey::Name)
                .set_limit(QUICK_ADD_LIMIT);
            let Ok(filter) = builder.build() else {
                show.set(false);
                return;
            };
            usage.peek().record_search();
            let reply = authed
                .run_quiet("quick_add_search", |c, s| async move {
                    c.search_deck_cards(deck_id, &filter, &s).await
                })
                .await;
            if !searches.peek().is_current(ticket) {
                return;
            }
            searching.set(false);
            match reply {
                Some(page) => results.set(page.cards),
                None => show.set(false),
            }
        });
    });

    let mut add = move |card: Card| {
        let id = card.scryfall_data.id;
        query.set(String::new());
        results.set(Vec::new());
        show.set(false);
        let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
        spawn(async move {
            let Some(deck_card) = authed
                .run("quick_add_card", |c, s| async move {
                    c.create_deck_card(deck_id, &request, &s).await
                })
                .await
            else {
                return;
            };
            let card_name = card.scryfall_data.name.clone();
            entries.write().push(DeckEntry { card, deck_card });
            on_added.call(());
            undo.write().push(
                deck_id,
                UndoAction::Added {
                    card_id: id,
                    card_name: card_name.clone(),
                },
            );
            toast.info(
                format!("Added {card_name}"),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        });
    };

    let shown = results();
    let lit = highlight().min(shown.len().saturating_sub(1));
    let keyed = shown.clone();
    rsx! {
        div { class: "quick-add",
            div { class: "chip-row",
                label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: QUICK_ADD_ID, "Quick add" }
                ClearWrap {
                    filled: !query().is_empty(),
                    on_clear: move |()| {
                        query.set(String::new());
                        results.set(Vec::new());
                        show.set(false);
                    },
                    label: "Clear the search".to_string(),
                    input {
                        class: "input input-compact",
                        id: QUICK_ADD_ID,
                        r#type: "text",
                        placeholder: "Search a card to add",
                        value: "{query}",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        onfocus: move |_| {
                            document::eval(
                                "const el = document.querySelector('.web-deck-cards-controls'); if (el) el.scrollTo({ top: 0, behavior: 'smooth' });",
                            );
                        },
                        oninput: move |e| query.set(e.value()),
                        onkeydown: move |e: Event<KeyboardData>| match e.key() {
                            Key::ArrowDown => {
                                e.prevent_default();
                                highlight.set(lit.saturating_add(1).min(keyed.len().saturating_sub(1)));
                            }
                            Key::ArrowUp => {
                                e.prevent_default();
                                highlight.set(lit.saturating_sub(1));
                            }
                            Key::Enter => {
                                if let Some(card) = keyed.get(lit).cloned()
                                    && !searching()
                                {
                                    e.prevent_default();
                                    add(card);
                                }
                            }
                            Key::Escape => {
                                results.set(Vec::new());
                                show.set(false);
                                document::eval(&BLUR_JS.replace("__ID__", QUICK_ADD_ID));
                            }
                            _ => {}
                        },
                    }
                }
            }
            if show() {
                div { class: "card-filter-results quick-add-results",
                    if searching() {
                        div { class: "chip-unselected", "Searching…" }
                    } else if shown.is_empty() {
                        div { class: "chip-unselected", "No results" }
                    } else {
                        for (i, card) in shown.into_iter().enumerate() {
                            button {
                                key: "{card.scryfall_data.id}",
                                r#type: "button",
                                class: if i == lit { "chip-unselected card-filter-result card-filter-highlight" } else { "chip-unselected card-filter-result" },
                                style: "animation-delay: {i * 40}ms;",
                                onmousedown: move |e| e.prevent_default(),
                                onclick: move |_| add(card.clone()),
                                "{card.scryfall_data.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
