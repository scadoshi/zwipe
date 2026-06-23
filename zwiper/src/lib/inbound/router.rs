//! Application routing configuration.
//!
//! Defines all client-side routes and maps them to their corresponding screen components.

use crate::inbound::screens::{
    auth::{login::Login, register::Register, forgot_password::ForgotPassword},
    deck::{
        card::{
            add::Add as AddDeckCard,
            view::View as ViewDeckCard,
            remove::Remove as RemoveDeckCard,
        },
        create::CreateDeck,
        edit::EditDeck,
        export::ExportDeck,
        import::ImportDeck,
        list::DeckList,
        view::ViewDeck,
    },
    home::Home,
    profile::Profile,
};
use dioxus::prelude::*;
use uuid::Uuid;

/// Application routes mapping URLs to screen components.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
#[allow(missing_docs)]
pub enum Router {
    /// Landing screen — greeting, navigation to login/register/decks.
    #[route("/")]
    Home {},

    /// Email + password login form.
    #[route("/login")]
    Login {},
    /// New account registration form.
    #[route("/register")]
    Register {},
    /// Request a password reset email.
    #[route("/forgot-password")]
    ForgotPassword {},

    /// User profile overview — username/email/password edits, preferences, delete account.
    #[route("/user")]
    Profile {},

    /// List all user's decks with name, format, and card count.
    #[route("/deck")]
    DeckList {},
    /// Create a new deck — name, format, commander selection.
    #[route("/deck/create")]
    CreateDeck,
    /// Edit an existing deck's profile — name, format, commander.
    #[route("/deck/update/:deck_id")]
    EditDeck { deck_id: Uuid },
    /// View deck details — profile, stats, price breakdown, mana curve, type/color distribution.
    #[route("/deck/get/:deck_id")]
    ViewDeck { deck_id: Uuid },

    /// Import cards from a plain-text decklist into the deck.
    #[route("/deck/card/import/:deck_id")]
    ImportDeck { deck_id: Uuid },
    /// Export the deck as a plain-text decklist for sharing.
    #[route("/deck/card/export/:deck_id")]
    ExportDeck { deck_id: Uuid },

    /// Swipe-based card search — swipe right to add cards to the deck.
    #[route("/deck/card/add/:deck_id")]
    AddDeckCard {
        deck_id: Uuid,
    },
    /// Browse all cards in the deck — expandable rows with card details and image preview.
    #[route("/deck/card/view/:deck_id")]
    ViewDeckCard {
        deck_id: Uuid,
    },
    /// Swipe-based card removal — swipe right to remove cards from the deck.
    #[route("/deck/card/remove/:deck_id")]
    RemoveDeckCard {
        deck_id: Uuid,
     },
}

impl Default for Router {
    fn default() -> Self {
        Router::Home {}
    }
}
