//! One command-zone field: its label row with `Filter`, `×` and `Swipe`, the
//! floating results, and the input with its own `×`.
//!
//! Typing searches after zwipe-ui's debounce once the slot's minimum is
//! typed; ↑ and ↓ move through the results and Enter picks the highlighted
//! one. A picked card clears when typing starts, never on a click, a kept
//! deviation from the app.

use components::{ClearWrap, TOAST_NORMAL};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::commander_eligibility::{self, PartnerKind};
use zwipe_ui::deck::form::{self, Slot};
use zwipe_ui::generation::Generation;
use zwipe_ui::timing::COMMAND_ZONE_SEARCH_DEBOUNCE;

use super::DeckForm;
use crate::inbound::chrome::authed::{Authed, use_authed};
use crate::inbound::chrome::hint_topic::InfoButton;
use crate::outbound::timer;

/// The input's id, for the label.
fn input_id(slot: Slot) -> &'static str {
    match slot {
        Slot::Commander => "commander",
        Slot::Partner => "partner_commander",
        Slot::Background => "background",
        Slot::SignatureSpell => "signature_spell",
    }
}

/// After an explicit commander pick, fills the partner slot when the
/// commander names its partner: the exact name, else the front face. It
/// lands only while the commander is still that card and no partner is
/// set, with a toast. Generic partners pair freely and are left alone.
pub fn autofill_named_partner(form: DeckForm, selected: &Card, authed: Authed, toast: Toasts) {
    let Some(PartnerKind::Named(mate)) = commander_eligibility::partner_kind(selected) else {
        return;
    };
    let selected_id = selected.scryfall_data.id;
    spawn(async move {
        let mut builder = CardQueryBuilder::with_name_contains(&mate);
        builder.set_limit(Slot::Commander.limit());
        let Ok(query) = builder.build() else {
            return;
        };
        let Some(cards) = authed
            .run_quiet("partner_autofill", |c, s| async move {
                c.search_cards(&query, &s).await
            })
            .await
        else {
            return;
        };
        let Some(found) = form::named_partner(&cards, &mate).cloned() else {
            tracing::warn!("partner autofill: no exact match for {mate}");
            return;
        };
        let unchanged = form
            .commander
            .card
            .peek()
            .as_ref()
            .is_some_and(|c| c.scryfall_data.id == selected_id);
        if !unchanged || form.partner.card.peek().is_some() {
            return;
        }
        let name = found.scryfall_data.name.clone();
        form.partner.pick(found);
        toast.info(
            format!("Partner found and selected: {name}"),
            ToastOptions::default().duration(TOAST_NORMAL),
        );
    });
}

/// The field for `slot`, labeled `label`.
#[component]
pub fn CommandZoneField(form: DeckForm, slot: Slot, label: String) -> Element {
    let state = form.slot(slot);
    let authed = use_authed();
    let toast = use_toast();
    let mut results: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut searching = use_signal(|| false);
    let mut dropdown = use_signal(|| false);
    let mut highlight: Signal<Option<usize>> = use_signal(|| None);
    let mut searches = use_signal(Generation::default);
    let mut filter_on = state.filter_on;
    let mut swipe_open = state.swipe_open;
    let mut query = state.query;
    let mut text = state.text;
    let card = state.card;

    use_effect(move || {
        let typed = query();
        let ticket = searches.write().next_ticket();
        highlight.set(None);
        if typed.len() < slot.min_chars() {
            dropdown.set(false);
            searching.set(false);
            return;
        }
        searching.set(true);
        dropdown.set(true);
        spawn(async move {
            timer::sleep(COMMAND_ZONE_SEARCH_DEBOUNCE).await;
            if !searches.peek().is_current(ticket) {
                return;
            }
            let commander = form.commander.card.peek().clone();
            let Ok(filter) = form::search_query(
                slot,
                &typed,
                *filter_on.peek(),
                *form.format.peek(),
                commander.as_ref(),
            ) else {
                searching.set(false);
                dropdown.set(false);
                return;
            };
            let reply = authed
                .run_quiet("command_zone_search", |c, s| async move {
                    c.search_cards(&filter, &s).await
                })
                .await;
            if !searches.peek().is_current(ticket) {
                return;
            }
            searching.set(false);
            if let Some(cards) = reply {
                results.set(cards);
                dropdown.set(true);
            } else {
                dropdown.set(false);
            }
        });
    });

    let mut pick = move |picked: Card| {
        dropdown.set(false);
        highlight.set(None);
        if slot == Slot::Commander {
            autofill_named_partner(form, &picked, authed, toast);
        }
        state.pick(picked);
    };

    let found = results();
    let lit = highlight();
    rsx! {
        div {
            div { class: "label-row",
                label { class: "label", r#for: input_id(slot), "{label}" }
                InfoButton { topic: slot.hint((form.format)()) }
                button {
                    r#type: "button",
                    class: if filter_on() { "chip-xs selected" } else { "chip-xs" },
                    onclick: move |_| filter_on.toggle(),
                    "Filter"
                }
                button {
                    r#type: "button",
                    class: "chip-xs chip-primary",
                    onclick: move |_| swipe_open.set(true),
                    "Swipe"
                }
            }
            if dropdown() {
                div { class: "search-float-anchor",
                    div { class: "search-float-results",
                        if searching() {
                            div { class: "chip-unselected search-float-chip", "Searching…" }
                        } else if found.is_empty() {
                            div { class: "chip-unselected search-float-chip", "No results" }
                        } else {
                            for (i, hit) in found.into_iter().enumerate() {
                                div {
                                    key: "{hit.scryfall_data.id}",
                                    class: if lit == Some(i) { "chip-unselected search-float-chip web-float-highlight" } else { "chip-unselected search-float-chip" },
                                    style: "animation-delay: {i.saturating_mul(40)}ms, {i.saturating_mul(40).saturating_add(250)}ms;",
                                    onclick: move |_| pick(hit.clone()),
                                    "{hit.scryfall_data.name}"
                                }
                            }
                        }
                    }
                }
            }
            // The × in the box clears what it shows: the search typed into
            // it, or the card picked.
            ClearWrap {
                filled: !query().is_empty() || card.read().is_some(),
                on_clear: move |()| {
                    if query.peek().is_empty() {
                        state.clear_all();
                    } else {
                        query.set(String::new());
                        text.set(String::new());
                    }
                    dropdown.set(false);
                },
                input {
                    class: "input",
                    id: input_id(slot),
                    r#type: "text",
                    placeholder: "Not set",
                    value: "{text}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |event| {
                        if card.peek().is_some() {
                            let mut card = card;
                            card.set(None);
                        }
                        query.set(event.value());
                        text.set(event.value());
                    },
                    onkeydown: move |event: Event<KeyboardData>| {
                        let count = if dropdown() && !searching() { results.peek().len() } else { 0 };
                        match event.key() {
                            Key::ArrowDown if count > 0 => {
                                event.prevent_default();
                                let next = highlight().map_or(0, |at| at.saturating_add(1).min(count.saturating_sub(1)));
                                highlight.set(Some(next));
                            }
                            Key::ArrowUp if count > 0 => {
                                event.prevent_default();
                                highlight.set(highlight().and_then(|at| at.checked_sub(1)));
                            }
                            Key::Enter => {
                                event.prevent_default();
                                event.stop_propagation();
                                let hit = highlight().and_then(|at| results.peek().get(at).cloned());
                                if let Some(hit) = hit {
                                    pick(hit);
                                }
                            }
                            _ => {}
                        }
                    },
                }
            }
        }
    }
}
