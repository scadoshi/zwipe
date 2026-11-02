//! Edit deck screen.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::screens::deck_form::Open;

use super::components::deck_form::DeckForm;

/// Screen for editing a deck with name and settings.
#[component]
pub fn EditDeck(deck_id: Uuid) -> Element {
    rsx! {
        DeckForm { open: Open::Edit(deck_id) }
    }
}
