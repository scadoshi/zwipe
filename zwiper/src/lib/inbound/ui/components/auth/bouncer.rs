use crate::inbound::ui::router::Router;
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Bouncer(children: Element) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();

    use_effect(move || {
        if session.read().is_none() {
            navigator.push(Router::Login {});
        }
    });

    rsx! { { children } }
}
