use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, signal_logout::SignalLogout},
        router::Router,
    },
    outbound::client::ZwipeClient,
};
use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();

    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();

    let mut show_logout_dialog = use_signal(|| false);

    let logo = logo::ZWIPE;

    rsx! {
        Bouncer {
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "logo", "{logo}" }
                div { class : "container-sm text-center flex-col",
                }

                if let Some(sesh) = session() {
                    div { class : "welcome-message", { format!("hello, {}!", sesh.user.username) } }
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
