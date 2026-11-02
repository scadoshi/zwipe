//! Create new deck screen.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_ui::screens::deck_form::Open;

use super::components::deck_form::DeckForm;

/// Context: one-shot commander seed from the commander maybeboard's "Create
/// deck" action. Parked above the router (`session_upkeep`) so it survives the
/// navigation; taken (and cleared) by [`CreateDeck`] at mount.
#[derive(Clone, Copy)]
pub(crate) struct CreateDeckCommanderSeed(pub(crate) Signal<Option<Card>>);

/// Screen for creating a new deck with name and settings.
#[component]
pub fn CreateDeck() -> Element {
    // The maybeboard's seed is taken once at mount; the model sets the format
    // to Commander directly, so no knock-on clear wipes the seed.
    let seed: CreateDeckCommanderSeed = use_context();
    let open = use_hook(move || {
        let mut slot = seed.0;
        let seed = slot.write().take().map(Box::new);
        Open::Create { seed }
    });
    rsx! {
        DeckForm { open }
    }
}
