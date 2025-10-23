use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn UpdateDeck(deck_id: Uuid) -> Element {
    rsx! {
        p { "{deck_id" }
    }
}
