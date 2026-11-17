//! Send to on the web: the sheet above the signed-in screens. It loads the user's decks as it
//! opens, leaves out the deck the page under it is on, and adds the card to the deck clicked,
//! on the board picked, as a write that runs to its end even if the page goes first.

use components::{SendTo, SendToSheet, TOAST_QUICK, spawn_write};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
use zwipe_ui::send_to::{self, Target};
use zwipe_ui::telemetry::vocabulary::component;

use crate::inbound::chrome::authed::use_authed;
use crate::inbound::router::Route;

/// Provides [`SendTo`] to the pages under it, and returns it for [`SendToHost`].
pub fn use_send_to() -> SendTo {
    let send_to = SendTo {
        open: use_signal(|| false),
        card: use_signal(|| None::<Card>),
    };
    use_context_provider(|| send_to)
}

/// The Send to sheet for `send_to`, beside the pages it serves.
#[component]
pub fn SendToHost(send_to: SendTo) -> Element {
    let SendTo { open, card } = send_to;
    let route = use_route::<Route>();
    let current = deck_of(&route);
    let authed = use_authed().in_component(component::SEND_TO_SHEET);
    let toast = use_toast();
    let mut decks = use_signal(|| None::<Vec<Target>>);
    use_effect(move || {
        if !open() {
            return;
        }
        decks.set(None);
        spawn(async move {
            let loaded = authed
                .run("send_to_decks", |c, s| async move {
                    c.fetch_deck_profiles(&s).await
                })
                .await;
            if let Some(loaded) = loaded {
                decks.set(Some(loaded.iter().map(Target::from).collect()));
            }
        });
    });
    rsx! {
        SendToSheet {
            open,
            decks: decks(),
            current,
            on_send: move |(target, board): (Target, Board)| {
                let Some(sent) = card.peek().clone() else {
                    return;
                };
                let mut open = open;
                open.set(false);
                let request = HttpCreateDeckCard::new(
                    &sent.scryfall_data,
                    1,
                    Some(board.display_name().to_owned()),
                );
                spawn_write(async move {
                    let added = authed
                        .run("send_to", |c, s| async move {
                            c.create_deck_card(target.id, &request, &s).await
                        })
                        .await;
                    if added.is_some() {
                        toast.info(
                            send_to::sent(&target.name, board),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    }
                });
            },
        }
    }
}

/// The deck `route` is on, if any.
fn deck_of(route: &Route) -> Option<Uuid> {
    if let Route::ViewDeck { deck_id, .. }
    | Route::EditDeck { deck_id }
    | Route::ImportDeck { deck_id }
    | Route::ExportDeck { deck_id }
    | Route::AddDeckCard { deck_id }
    | Route::RemoveDeckCard { deck_id } = route
    {
        Some(*deck_id)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deck_page_names_its_deck_and_others_none() {
        let id = Uuid::from_u128(7);
        assert_eq!(
            deck_of(&Route::ViewDeck {
                deck_id: id,
                view: None
            }),
            Some(id)
        );
        assert_eq!(deck_of(&Route::AddDeckCard { deck_id: id }), Some(id));
        assert_eq!(deck_of(&Route::CommanderMaybeboard {}), None);
        assert_eq!(deck_of(&Route::Home {}), None);
    }
}
