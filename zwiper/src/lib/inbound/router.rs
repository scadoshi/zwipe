//! Application routing configuration.
//!
//! Defines all client-side routes and maps them to their corresponding screen components.

use crate::inbound::{
    components::{auth::auth_gate::AuthGate, navigation::back_handler::BackHandlerLayout},
    screens::{
        auth::{forgot_password::ForgotPassword, login::Login, register::Register},
        changelog::Changelog,
        deck::{
            card::{
                add::Add as AddDeckCard, remove::Remove as RemoveDeckCard,
                view::View as ViewDeckCard,
            },
            create::CreateDeck,
            edit::EditDeck,
            export::ExportDeck,
            import::ImportDeck,
            list::DeckList,
            view::ViewDeck,
        },
        home::Home,
        legal::privacy_policy::PrivacyPolicy,
        profile::Profile,
    },
};
use dioxus::prelude::*;
use uuid::Uuid;

/// Application routes mapping URLs to screen components.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
#[allow(missing_docs)]
pub enum Router {
    // Wrap every route so the OS back-intent bridge (edge swipe / hardware
    // back) mounts once inside the router context and persists across routes.
    #[layout(BackHandlerLayout)]

    // ── public (pre-auth) ── the only screens reachable without a session.
    /// Email + password login form.
    #[route("/login")]
    Login {},
    /// New account registration form.
    #[route("/register")]
    Register {},
    /// Request a password reset email.
    #[route("/forgot-password")]
    ForgotPassword {},

    // ── authed ── everything below requires a valid session (AuthGate
    // redirects to /login otherwise), so screens no longer self-gate.
    #[layout(AuthGate)]
    /// Landing screen — greeting, navigation to decks/profile.
    #[route("/")]
    Home {},

    /// User profile overview — username/email/password edits, preferences, delete account.
    #[route("/user")]
    Profile {},

    /// Privacy policy — shared legal copy, reached from Profile.
    #[route("/privacy")]
    PrivacyPolicy {},

    /// Release history — shared changelog, reached from Profile.
    #[route("/changelog")]
    Changelog {},

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
