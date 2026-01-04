use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::inbound::router::Router;
use crate::{
    inbound::components::auth::{bouncer::Bouncer, signal_logout::SignalLogout},
    outbound::client::{ZwipeClient, card::search_cards::ClientSearchCards},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe::domain::{
    auth::models::session::Session,
    card::models::search_card::card_filter::{OrderByOptions, builder::CardFilterBuilder},
    logo,
};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();

    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let toast = use_toast();

    let logo = logo::ZWIPE;

    // Show welcome toast on mount
    use_effect(move || {
        if let Some(sesh) = session() {
            toast.info(
                format!("hello, {}!", sesh.user.username),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );
        }
    });

    // Fetch a random card with flavor text
    let random_flavor_card = use_resource(move || async move {
        let sesh = session()?;

        let mut builder = CardFilterBuilder::new();
        builder
            .set_has_flavor_text(true)
            .set_order_by(OrderByOptions::Random)
            .set_limit(1);

        let Ok(filter) = builder.build() else {
            return None;
        };

        match client().search_cards(&filter, &sesh).await {
            Ok(cards) => cards.into_iter().next(),
            Err(_) => None,
        }
    });

    rsx! {
        Bouncer {
            div { class: "sticky top-0 left-0 h-screen flex flex-col items-center overflow-hidden",
                style: "width: 100vw; justify-content: center;",
                div { class : "logo", "{logo}" }

                // Display random flavor text
                div { class: "container-sm text-center flex-col",
                    match &*random_flavor_card.read() {
                        Some(Some(card)) => {
                            if let Some(flavor_text) = card.scryfall_data.flavor_text.as_ref() {
                                rsx! {
                                    div { class: "flavor-quote",
                                        "{flavor_text} "
                                        span { class: "flavor-source", "[{card.scryfall_data.name}]" }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                        Some(None) => rsx! {},
                        None => rsx! {},
                    }
                }
            }
            div { class: "util-bar",
                button { class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::Profile {} );
                    }, "profile"
                }
                button { class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::DeckList {} );
                    }, "decks"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| show_logout_dialog.set(true),
                    "logout"
                }
            }

            AlertDialogRoot {
                open: show_logout_dialog(),
                on_open_change: move |open| show_logout_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "logout" }
                    AlertDialogDescription { "are you sure you want to logout?" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_logout_dialog.set(false),
                            "cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| session.logout(client),
                            "logout"
                        }
                    }
                }
            }
        }
    }
}
