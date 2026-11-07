//! The card screens: Add cards and Remove cards, the swipe surfaces, and
//! the deck cards view, the editor. They share zwipe-ui's stores through
//! [`CardStores`](crate::inbound::card_stores::CardStores).

pub mod add;
pub(crate) mod editor;
mod featured;
pub(crate) mod quick_add;
pub mod remove;
pub mod view;

use components::{CardDetailsPanel, CardDetailsPanelSkeleton, TOAST_NORMAL};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts};
use zwipe_core::domain::card::Card;
use zwipe_ui::deck_signals::Level;

use crate::inbound::cards::otag::use_describe_tag;

/// Raises a land or budget signal from zwipe-ui as the app's toast: info or
/// warning, up for the normal band.
fn signal_toast(toast: Toasts, signal: Option<(Level, String)>) {
    let Some((level, message)) = signal else {
        return;
    };
    let options = ToastOptions::default().duration(TOAST_NORMAL);
    match level {
        Level::Info => toast.info(message, options),
        Level::Warning => toast.warning(message, options),
    }
}

/// Presses the flip control of the top swipe card, if it has one.
fn flip_top_card() {
    document::eval(
        r#"const flip = document.querySelector('.swipe-stack-card.top [aria-label="Flip card"]'); if (flip) flip.click();"#,
    );
}

/// The top card's details beside it on a wide window, what the phone keeps
/// behind the eye button, or their ghost while the cards load.
#[component]
fn SwipeDetails(card: Option<Card>, #[props(default)] loading: bool) -> Element {
    let describe_tag = use_describe_tag();
    rsx! {
        div { class: "web-swipe-details",
            if let Some(card) = card {
                CardDetailsPanel {
                    key: "{card.scryfall_data.id}",
                    card,
                    describe_tag: Some(describe_tag),
                }
            } else if loading {
                CardDetailsPanelSkeleton {}
            }
        }
    }
}
