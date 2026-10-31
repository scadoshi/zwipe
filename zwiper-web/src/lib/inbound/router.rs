//! Every route, at the app's paths, so a link means the same screen in both.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::inbound::screens::auth::login::Login;
use crate::inbound::screens::placeholders::{
    Account, Activity, AddDeckCard, Changelog, CommanderMaybeboard, Config, CreateDeck, DeckList,
    EditDeck, ExportDeck, ForgotPassword, Home, ImportDeck, NotFound, Playstyle, PrivacyPolicy,
    Register, RemoveDeckCard, ViewDeck, ViewDeckCard,
};
use crate::inbound::session::auth_gate::AuthGate;
use crate::inbound::shell::{PublicFrame, Shell};

/// The app's routes. The public ones sit in [`PublicFrame`]; the rest need a
/// session ([`AuthGate`]) and sit in the sidebar [`Shell`].
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
#[expect(missing_docs, reason = "each route field is the URL parameter its path names")]
pub enum Route {
    // == public ==
    #[layout(PublicFrame)]
        /// Username or email and password.
        #[route("/login")]
        Login {},
        /// A new account.
        #[route("/register")]
        Register {},
        /// Asks for a password reset email.
        #[route("/forgot-password")]
        ForgotPassword {},
    #[end_layout]

    // == authed ==
    #[layout(AuthGate)]
    #[layout(Shell)]
        /// The counters, the color map and the featured flavor card.
        #[route("/")]
        Home {},
        /// Username, email, password, About, log out, delete account.
        #[route("/user")]
        Account {},
        /// Theme, pips and Universes Beyond.
        #[route("/config")]
        Config {},
        /// The privacy policy, from Account.
        #[route("/privacy")]
        PrivacyPolicy {},
        /// Release history, from Account.
        #[route("/changelog")]
        Changelog {},
        /// How the user picks, from their swipes.
        #[route("/activity")]
        Activity {},
        /// Every deck the user has.
        #[route("/deck")]
        DeckList {},
        /// Commanders saved for later from commander search.
        #[route("/deck/commander-maybeboard")]
        CommanderMaybeboard {},
        /// What the user builds, with suggested commanders. The path is the
        /// app's.
        #[route("/deck/suggested-commanders")]
        Playstyle {},
        /// A new deck.
        #[route("/deck/create")]
        CreateDeck {},
        /// A deck's profile.
        #[route("/deck/update/:deck_id")]
        EditDeck { deck_id: Uuid },
        /// A deck: profile, sections and stats.
        #[route("/deck/get/:deck_id")]
        ViewDeck { deck_id: Uuid },
        /// Cards from a pasted list.
        #[route("/deck/card/import/:deck_id")]
        ImportDeck { deck_id: Uuid },
        /// The deck as a text list.
        #[route("/deck/card/export/:deck_id")]
        ExportDeck { deck_id: Uuid },
        /// Swipe to add cards.
        #[route("/deck/card/add/:deck_id")]
        AddDeckCard { deck_id: Uuid },
        /// Every card in the deck.
        #[route("/deck/card/view/:deck_id")]
        ViewDeckCard { deck_id: Uuid },
        /// Swipe to remove cards.
        #[route("/deck/card/remove/:deck_id")]
        RemoveDeckCard { deck_id: Uuid },
    #[end_layout]
    #[end_layout]

    /// Any path no route matches.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
