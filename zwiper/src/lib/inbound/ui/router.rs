use crate::inbound::ui::components::screens::{
    app::{
        decks::Decks,
        home::Home,
        profile::{
            change_email::ChangeEmail, change_password::ChangePassword,
            change_username::ChangeUsername, Profile,
        },
    },
    auth::{login::Login, register::Register},
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Router {
    #[route("/")]
    Guard {},
    #[route("/home")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/user/profile")]
    Profile {},
    #[route("/decks")]
    Decks {},
    #[route("/user/change-username")]
    ChangeUsername {},
    #[route("/user/change-email")]
    ChangeEmail {},
    #[route("/user/change-password")]
    ChangePassword {},
}

#[component]
fn Guard() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();

    if session.read().is_some() {
        navigator.push(Router::Home {});
    } else {
        navigator.push(Router::Login {});
    }

    rsx! {}
}
