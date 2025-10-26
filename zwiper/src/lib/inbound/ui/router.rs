use crate::inbound::ui::components::screens::{
    app::{
        deck::{
            create_deck::CreateDeck, deck_list::DeckList, get_deck::GetDeck,
            update_deck::UpdateDeck,
        },
        home::Home,
        profile::{
            change_email::ChangeEmail, change_password::ChangePassword,
            change_username::ChangeUsername, Profile,
        },
    },
    auth::{login::Login, register::Register},
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::domain::deck::models::deck::deck_profile::DeckProfile;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Router {
    // home
    #[route("/")]
    Home {},

    // auth
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    
    // user
    #[route("/user")]
    Profile {},
    #[child]
    #[route("/change-username")]
    ChangeUsername {},
    #[child]
    #[route("/change-email")]
    ChangeEmail {},
    #[child]
    #[route("/change-password")]
    ChangePassword {},

    // deck
    #[route("/deck")]
    DeckList {},
    #[route("/deck/create")]
    CreateDeck,
    #[route("/deck/update/:deck_id")]
    UpdateDeck { deck_id: Uuid },
    #[route("/deck/get/:deck_id")]
    GetDeck { deck_id: Uuid },
}
