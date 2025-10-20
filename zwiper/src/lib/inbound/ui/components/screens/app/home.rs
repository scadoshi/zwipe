use crate::inbound::ui::router::Router;
use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo::logo};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let mut session: Signal<Option<Session>> = use_context();
    let logo = logo();

    rsx! {
        div { class : "nicely-centered",
            div { class : "logo", "{logo}" }
            div { class : "form-container",
                button {
                    onclick : move |_| {
                        navigator.push(Router::Profile {} );
                    }, "profile"
                }
                button {
                    onclick : move |_| {
                        navigator.push(Router::Decks {} );
                    }, "decks"
                }
                button {
                    onclick : move |_| {
                        // logout
                        session.set(None);
                        navigator.push(Router::Login {});
                    }, "logout"
                }
            }
        }
    }
}
