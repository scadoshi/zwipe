//! Send to on the web: the sheet above the signed-in screens. It loads the user's decks as it
//! opens, leaves out the deck the page under it is on, and adds the card to the deck clicked,
//! on the board picked, as a write that runs to its end even if the page goes first. A send from
//! a deck's page goes on that deck's undo.

use components::{SendTo, SendToSheet, TOAST_NORMAL, TOAST_QUICK, spawn_write};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_ui::send_to::{self, SendUndo, Target};
use zwipe_ui::telemetry::vocabulary::component;
use zwipe_ui::write_repeat::{Verdict, WriteKey};

use crate::inbound::card_stores::CardStores;
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
    let CardStores { mut undo, .. } = use_context();
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
                let oracle_id = sent.scryfall_data.oracle_id;
                spawn_write(async move {
                    let added = authed
                        .try_run_quiet("send_to", |c, s| async move {
                            c.create_deck_card(target.id, &request, &s).await
                        })
                        .await;
                    let key = WriteKey::add(target.id, sent.scryfall_data.id);
                    // A lost earlier try that landed reads as sent.
                    let added = match authed.settle(key, &added) {
                        Verdict::LandedEarlier => Ok(()),
                        Verdict::Done | Verdict::Failed => added.map(|_| ()),
                    };
                    match added {
                        Ok(()) => {
                            if let Some((from, entry)) = send_to::undo_entry(
                                current,
                                &target,
                                board,
                                sent.scryfall_data.id,
                                &sent.scryfall_data.name,
                            ) {
                                undo.write().push(from, entry);
                            }
                            toast.info(
                                send_to::sent(&target.name, board),
                                ToastOptions::default().duration(TOAST_QUICK),
                            );
                        }
                        Err(e) => {
                            // A duplicate names the board that holds the card, read from the deck.
                            let holding = match (&e, oracle_id) {
                                (ClientError::Conflict(_), Some(oracle_id)) => authed
                                    .try_run_quiet("send_to_board", |c, s| async move {
                                        c.fetch_deck(target.id, &s).await
                                    })
                                    .await
                                    .ok()
                                    .and_then(|deck| send_to::holding_board(&deck.entries, oracle_id)),
                                _ => None,
                            };
                            toast.error(
                                failure_message(&e, &target.name, holding),
                                ToastOptions::default().duration(TOAST_NORMAL),
                            );
                        }
                    }
                });
            },
        }
    }
}

/// Takes back the `copies` a Send to put on `board` in the deck `deck_id`: reads the card's row
/// there, then lowers it by that many or deletes it ([`send_to::send_undo`]). `Ok(false)` when
/// the row moved boards or left, which it leaves alone.
///
/// # Errors
///
/// Returns the [`ClientError`] of the read, the lowering or the delete.
pub(crate) async fn take_back(
    client: &ZwipeClient,
    session: &Session,
    deck_id: Uuid,
    card_id: Uuid,
    board: Board,
    copies: i32,
) -> Result<bool, ClientError> {
    let deck = client.fetch_deck(deck_id, session).await?;
    let row = deck
        .entries
        .iter()
        .find(|e| e.deck_card.scryfall_data_id == card_id)
        .map(|e| &e.deck_card);
    match send_to::send_undo(row, board, copies) {
        None => Ok(false),
        Some(SendUndo::Remove) => client
            .delete_undone_deck_card(deck_id, card_id, session)
            .await
            .map(|()| true),
        Some(SendUndo::Lower(quantity)) => {
            let request = HttpPatchDeckCard::new(Some(*quantity), None);
            client
                .update_deck_card(deck_id, card_id, &request, session)
                .await
                .map(|_| true)
        }
    }
}

/// What a failed send toasts: the deck by name, and the board when it is known, when it already
/// holds the card, else the error's own plain words.
fn failure_message(error: &ClientError, deck: &str, board: Option<Board>) -> String {
    if matches!(error, ClientError::Conflict(_)) {
        send_to::already_in(deck, board)
    } else {
        error.to_user_message()
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
    fn a_duplicate_names_the_deck_it_went_to_and_other_errors_keep_their_words() {
        let duplicate = ClientError::Conflict("That card is already in this deck".to_owned());
        assert_eq!(
            failure_message(&duplicate, "Krenko", Some(Board::Maybeboard)),
            "Already in Krenko's maybeboard"
        );
        assert_eq!(
            failure_message(&duplicate, "Krenko", None),
            "Already in Krenko",
            "the board unknown"
        );
        let limit = ClientError::UnprocessableEntity("Card limit reached".to_owned());
        assert_eq!(
            failure_message(&limit, "Krenko", None),
            limit.to_user_message()
        );
    }

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
