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

    let mut confirm_logout = use_signal(|| false);

    let logo = logo::ZWIPE;

    rsx! {
        Bouncer {
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "logo", "{logo}" }
                div { class : "container-sm text-center flex-col",
                    button { class : "btn",
                        onclick : move |_| {
                            navigator.push(Router::Profile {} );
                        }, "profile"
                    }
                    button { class : "btn",
                        onclick : move |_| {
                            navigator.push(Router::DeckList {} );
                        }, "decks"
                    }

                    if !confirm_logout() {
                        button { class : "btn",
                            onclick : move |_| confirm_logout.set(true),
                            "logout"
                        }
                    }

                    if confirm_logout() {
                        label { class: "label", r#for : "confirmation-prompt", "are you sure?" }
                        div { class : "flex flex-between gap-2",
                            id : "confirmation-prompt",
                            button { class : "btn btn-half",
                                onclick: move |_| session.logout(client),
                                "yes"
                            }
                            button { class : "btn btn-half",
                                onclick: move |_| confirm_logout.set(false),
                                "no"
                            }
                        }
                    }
                }
            }
        }
    }
}
