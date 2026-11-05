//! Every route, at the app's paths, so a link means the same screen in both.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::inbound::screens::about::About;
use crate::inbound::screens::account::Account;
use crate::inbound::screens::activity::Activity;
use crate::inbound::screens::auth::forgot_password::ForgotPassword;
use crate::inbound::screens::auth::login::Login;
use crate::inbound::screens::auth::register::Register;
use crate::inbound::screens::commander_maybeboard::CommanderMaybeboard;
use crate::inbound::screens::config::Config;
use crate::inbound::screens::deck::cards::add::AddDeckCard;
use crate::inbound::screens::deck::cards::remove::RemoveDeckCard;
use crate::inbound::screens::deck::cards::view::ViewDeckCard;
use crate::inbound::screens::deck::create::CreateDeck;
use crate::inbound::screens::deck::edit::EditDeck;
use crate::inbound::screens::deck::export::ExportDeck;
use crate::inbound::screens::deck::import::ImportDeck;
use crate::inbound::screens::deck::layout::DeckLayout;
use crate::inbound::screens::deck::list::DeckList;
use crate::inbound::screens::deck::view::ViewDeck;
use crate::inbound::screens::home::Home;
use crate::inbound::screens::not_found::NotFound;
use crate::inbound::screens::playstyle::Playstyle;
use crate::inbound::screens::privacy_policy::PrivacyPolicy;
use crate::inbound::session::auth_gate::AuthGate;
use crate::inbound::shell::{OpenFrame, PublicFrame, Shell};

/// The app's routes. The public ones sit in [`PublicFrame`], Privacy policy,
/// About and Not found in [`OpenFrame`] (either nav), and the rest need a session ([`AuthGate`])
/// and sit under the top nav [`Shell`]. `/changelog` redirects to About and
/// `/config` to Settings.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
#[expect(missing_docs, reason = "each route field is the URL parameter its path names")]
pub enum Route {
    #[redirect("/changelog", || Route::About {})]
    #[redirect("/config", || Route::Config {})]
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

    // == open: the signed-in nav or the public one ==
    #[layout(OpenFrame)]
        /// The privacy policy, from Account, or from About signed out.
        #[route("/privacy")]
        PrivacyPolicy {},
        /// The version, the links, the credits and the release history.
        #[route("/about")]
        About {},
    #[end_layout]

    // == authed ==
    #[layout(AuthGate)]
    #[layout(Shell)]
        /// The hero, the Decks and Colors panels and the featured flavor card.
        #[route("/")]
        Home {},
        /// Username, email, password, log out, delete account.
        #[route("/user")]
        Account {},
        /// Settings: theme, pips and Universes Beyond. The old `/config`
        /// redirects here.
        #[route("/settings")]
        Config {},
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
        #[layout(DeckLayout)]
            #[route("/deck/get/:deck_id")]
            ViewDeck { deck_id: Uuid },
            /// Cards from a pasted list.
            #[route("/deck/card/import/:deck_id")]
            ImportDeck { deck_id: Uuid },
            /// The deck as a text list.
            #[route("/deck/card/export/:deck_id")]
            ExportDeck { deck_id: Uuid },
        #[end_layout]
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

    // == anything else: zite's dead end, under either nav (the layout runs
    // to the end) ==
    #[layout(OpenFrame)]
        /// Any path no route matches.
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}
