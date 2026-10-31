//! The deck view's cards column on a wide window: the command zone on top,
//! then the mainboard by type with the lands last, as compact rows that
//! expand on click. It is for reading; `Cards` opens the editor.

use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck_cards::{self, Zone};

use crate::inbound::cards::card_row::CardRow;

/// The command zone's cards in their slots, matched by printing id.
fn zone(deck: &Deck) -> Zone {
    let profile = &deck.deck_profile;
    let find = |id: Option<Uuid>| -> Option<Card> {
        let id = id?;
        deck.command_zone_cards
            .iter()
            .find(|c| c.scryfall_data.id == id)
            .cloned()
    };
    Zone {
        commander: find(profile.commander_id),
        partner: find(profile.partner_commander_id),
        background: find(profile.background_id),
        signature: find(profile.signature_spell_id),
    }
}

/// A command-zone slot's header, as the deck cards view names it.
fn slot_label(slot: Slot, profile: &DeckProfile) -> &'static str {
    match slot {
        Slot::Commander if profile.format.is_some_and(|f| f.has_signature_spell()) => "Oathbreaker",
        Slot::Commander => "Commander",
        Slot::Partner => "Partner",
        Slot::Background => "Background",
        Slot::SignatureSpell => "Signature spell",
    }
}

/// The column for `deck`.
#[component]
pub fn CardsColumn(deck: Deck) -> Element {
    let expanded: Signal<Option<Uuid>> = use_signal(|| None);
    let held = zone(&deck);
    let mut entries = deck.entries.clone();
    held.strip(&mut entries);
    let qty: HashMap<Uuid, i32> = entries
        .iter()
        .filter(|e| e.deck_card.board.is_active())
        .map(|e| (e.card.scryfall_data.id, *e.deck_card.quantity))
        .collect();
    let shown = deck_cards::pass(
        &entries,
        &held,
        &CardQueryBuilder::new(),
        GroupByOption::CardType,
    );
    let count = |cards: &[Card]| -> i32 {
        cards
            .iter()
            .map(|c| qty.get(&c.scryfall_data.id).copied().unwrap_or(1))
            .fold(0, i32::saturating_add)
    };
    let empty = shown.zone.is_empty() && shown.groups.is_empty() && shown.lands.is_empty();
    let lands_count = count(&shown.lands);
    rsx! {
        div { class: "web-deck-cards",
            for (slot, card) in shown.zone {
                div { key: "{slot:?}", class: "card-group row-enter",
                    div { class: "card-group-header", "{slot_label(slot, &deck.deck_profile)}" }
                    CardRow { card, qty: 1, expanded_card: expanded }
                }
            }
            for group in shown.groups {
                div { key: "{group.key}", class: "card-group row-enter",
                    div { class: "card-group-header", "{group.label} ({count(&group.cards)})" }
                    for card in group.cards {
                        CardRow {
                            key: "{card.scryfall_data.id}",
                            qty: qty.get(&card.scryfall_data.id).copied().unwrap_or(1),
                            card,
                            expanded_card: expanded,
                        }
                    }
                }
            }
            if !shown.lands.is_empty() {
                div { class: "card-group row-enter",
                    div { class: "card-group-header", "Lands ({lands_count})" }
                    for card in shown.lands {
                        CardRow {
                            key: "{card.scryfall_data.id}",
                            qty: qty.get(&card.scryfall_data.id).copied().unwrap_or(1),
                            card,
                            expanded_card: expanded,
                        }
                    }
                }
            }
            if empty {
                p { class: "text-muted", "No cards" }
            }
        }
    }
}
