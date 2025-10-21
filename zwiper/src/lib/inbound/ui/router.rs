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

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Router {
    #[route("/")]
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
