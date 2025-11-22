use crate::inbound::ui::components::screens::{
    app::{
        deck::{
            card::{
                add::Add as AddDeckCard,
                filter::{
                    combat::Combat, mana::Mana, printing::Printing, text::Text, types::Types,
                    Filter,
                },
                remove::Remove as RemoveDeckCard,
            },
            create::CreateDeck,
            edit::EditDeck,
            list::DeckList,
            view::ViewDeck,
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
    #[route("/user/change-username")]
    ChangeUsername {},
    #[route("/user/change-email")]
    ChangeEmail {},
    #[route("/user/change-password")]
    ChangePassword {},

    // deck
    #[route("/deck")]
    DeckList {},
    #[route("/deck/create")]
    CreateDeck,
    #[route("/deck/update/:deck_id")]
    EditDeck { deck_id: Uuid },
    #[route("/deck/get/:deck_id")]
    ViewDeck { deck_id: Uuid },

    //deck card
    #[route("/deck/card/add/:deck_id")]
    AddDeckCard { 
        deck_id: Uuid,
    },
    #[route("/deck/card/remove/:deck_id")]
    RemoveDeckCard { 
        deck_id: Uuid,
     },

     // filtering
    #[route("/deck/card/filter")]
    Filter {},
    #[route("/deck/card/filter/types")]
    Types {},
    #[route("/deck/card/filter/text")]
    Text {},
    #[route("/deck/card/filter/combat")]
    Combat {},
    #[route("/deck/card/filter/printing")]
    Printing {},
    #[route("/deck/card/filter/mana")]
    Mana {},
}

impl Default for Router {
    fn default() -> Self {
        Router::Home {}
    }
}
