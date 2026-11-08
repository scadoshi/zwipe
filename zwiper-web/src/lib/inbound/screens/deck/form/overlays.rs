//! What the form opens over itself: the three pickers and a Swipe select for
//! each command-zone slot. Render beside the screen's frame.

use dioxus::prelude::*;
use dioxus_primitives::toast::use_toast;
use zwipe_core::domain::card::Card;
use zwipe_ui::deck::command_zone;
use zwipe_ui::deck::form::Slot;

use super::DeckForm;
use super::command_zone::autofill_named_partner;
use super::pickers::{DeckTagPicker, FormatPicker, OracleTagPicker};
use crate::inbound::cards::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::use_authed;

/// The pickers and the Swipe selects. The commander's Swipe select is keyed
/// by the format and the signature spell's by the commander, so a change
/// deals a fresh pile. The commander's waits for a format, and the
/// signature spell's deals nothing without a commander, as in the app.
#[component]
pub fn DeckFormOverlays(form: DeckForm) -> Element {
    let authed = use_authed();
    let toast = use_toast();
    let format = (form.format)();
    let commander = form.commander.card.read().clone();
    let commander_key = commander
        .as_ref()
        .map(|c| c.scryfall_data.id.to_string())
        .unwrap_or_default();
    let pick = move |slot: Slot, card: Card| {
        let state = form.slot(slot);
        if slot == Slot::Commander {
            autofill_named_partner(form, &card, authed, toast);
        }
        state.pick(card);
        let mut open = state.swipe_open;
        open.set(false);
    };
    // Every slot's pile leaves out the cards already in the command zone.
    let zone_ids = use_memo(move || {
        command_zone::zone_oracle_ids([
            form.commander.card.read().as_ref(),
            form.partner.card.read().as_ref(),
            form.background.card.read().as_ref(),
            form.signature_spell.card.read().as_ref(),
        ])
    });
    let close = move |slot: Slot| {
        let mut open = form.slot(slot).swipe_open;
        open.set(false);
    };
    rsx! {
        FormatPicker { form }
        DeckTagPicker { form }
        OracleTagPicker { form }
        if let Some(format) = format {
            SwipeSelect {
                key: "{format.to_legality_key()}",
                open: form.commander.swipe_open,
                mode: SwipeMode::Commander(format),
                exclude_oracle_ids: Some(zone_ids.into()),
                on_select: move |card: Card| pick(Slot::Commander, card),
                on_close: move |()| close(Slot::Commander),
            }
        }
        SwipeSelect {
            open: form.partner.swipe_open,
            mode: SwipeMode::Partner,
            exclude_oracle_ids: Some(zone_ids.into()),
            on_select: move |card: Card| pick(Slot::Partner, card),
            on_close: move |()| close(Slot::Partner),
        }
        SwipeSelect {
            open: form.background.swipe_open,
            mode: SwipeMode::Background,
            exclude_oracle_ids: Some(zone_ids.into()),
            on_select: move |card: Card| pick(Slot::Background, card),
            on_close: move |()| close(Slot::Background),
        }
        for key in [commander_key] {
            SwipeSelect {
                key: "{key}",
                open: form.signature_spell.swipe_open,
                mode: SwipeMode::SignatureSpell,
                exclude_oracle_ids: Some(zone_ids.into()),
                commander: commander.clone(),
                on_select: move |card: Card| pick(Slot::SignatureSpell, card),
                on_close: move |()| close(Slot::SignatureSpell),
            }
        }
    }
}
