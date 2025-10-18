pub mod components;

use crate::inbound::ui::components::screens::{
    app::home::Home as AppHome, auth::home::Home as AuthHome,
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Router {
    #[route("/")]
    HomeGuard {},
}

#[component]
fn HomeGuard() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let has_session = use_memo(move || session.read().is_some());

    if *has_session.read() {
        rsx! { AppHome {} }
    } else {
        rsx! { AuthHome {} }
    }
}
