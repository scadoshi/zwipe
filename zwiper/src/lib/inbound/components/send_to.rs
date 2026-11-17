//! Send to in the app: the sheet above the signed-in screens. It loads the user's decks as it
//! opens, leaves out the deck the screen under it is on, and adds the card to the deck tapped,
//! on the board picked, as a task of the sheet's host, so the write runs to its end even if the
//! screen goes first.

use components::{SendTo, SendToSheet, TOAST_QUICK, ToastOptions, use_toast};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
use zwipe_ui::send_to::{self, Target};

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::telemetry::vocabulary::{
    AuthScreen, DeckScreen, ProfileScreen, Screen, component,
};
use crate::inbound::router::Router;

/// Provides [`SendTo`] to the screens under it, and returns it for [`SendToHost`].
pub(crate) fn use_send_to() -> SendTo {
    let send_to = SendTo {
        open: use_signal(|| false),
        card: use_signal(|| None::<Card>),
    };
    use_context_provider(|| send_to)
}

/// The Send to sheet for `send_to`, beside the screens it serves.
#[component]
pub(crate) fn SendToHost(send_to: SendTo) -> Element {
    let SendTo { open, card } = send_to;
    let route = use_route::<Router>();
    let current = deck_of(&route);
    let screen = host_screen(&route);
    let authed = use_authed(screen);
    let toast = use_toast();
    let mut decks = use_signal(|| None::<Vec<Target>>);
    use_effect(move || {
        if !open() {
            return;
        }
        decks.set(None);
        spawn(async move {
            let loaded = authed
                .run_at(
                    component::SEND_TO_SHEET,
                    "send_to_decks",
                    |c, s| async move { c.fetch_deck_profiles(&s).await },
                )
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
                // A task of this host, which sits above the screens, so the write runs to its
                // end when the screen it opened over goes.
                spawn(async move {
                    let added = authed
                        .run_at(component::SEND_TO_SHEET, "send_to", |c, s| async move {
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
fn deck_of(route: &Router) -> Option<Uuid> {
    if let Router::ViewDeck { deck_id, .. }
    | Router::EditDeck { deck_id }
    | Router::ImportDeck { deck_id }
    | Router::ExportDeck { deck_id }
    | Router::AddDeckCard { deck_id }
    | Router::RemoveDeckCard { deck_id } = route
    {
        Some(*deck_id)
    } else {
        None
    }
}

/// The screen a Send to opened over, for its error reports.
const fn host_screen(route: &Router) -> Screen {
    match route {
        Router::Login {} => Screen::Auth(AuthScreen::Login),
        Router::Register {} => Screen::Auth(AuthScreen::Register),
        Router::ForgotPassword {} => Screen::Auth(AuthScreen::ForgotPassword),
        Router::About {} => Screen::Changelog,
        Router::Home {} | Router::PrivacyPolicy {} => Screen::Home,
        Router::Account {} => Screen::Profile(ProfileScreen::Main),
        Router::Config {} => Screen::Profile(ProfileScreen::Settings),
        Router::Activity {} => Screen::Activity,
        Router::DeckList {} => Screen::Deck(DeckScreen::List),
        Router::CommanderMaybeboard {} => Screen::CommanderMaybeboard,
        Router::SuggestedCommanders {} => Screen::SuggestedCommanders,
        Router::CreateDeck => Screen::Deck(DeckScreen::Create),
        Router::EditDeck { .. } => Screen::Deck(DeckScreen::Edit),
        Router::ViewDeck { .. } => Screen::Deck(DeckScreen::View),
        Router::ImportDeck { .. } => Screen::Deck(DeckScreen::Import),
        Router::ExportDeck { .. } => Screen::Deck(DeckScreen::Export),
        Router::AddDeckCard { .. } => Screen::Deck(DeckScreen::CardAdd),
        Router::RemoveDeckCard { .. } => Screen::Deck(DeckScreen::CardRemove),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deck_screen_names_its_deck_and_others_none() {
        let id = Uuid::from_u128(7);
        assert_eq!(
            deck_of(&Router::ViewDeck {
                deck_id: id,
                view: None
            }),
            Some(id)
        );
        assert_eq!(deck_of(&Router::AddDeckCard { deck_id: id }), Some(id));
        assert_eq!(deck_of(&Router::RemoveDeckCard { deck_id: id }), Some(id));
        assert_eq!(deck_of(&Router::CommanderMaybeboard {}), None);
        assert_eq!(deck_of(&Router::Home {}), None);
    }
}
