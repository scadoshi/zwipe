//! Every route, at resource paths: a deck lives under `/decks/:deck_id`,
//! and its screens under it.

// Routable's derive builds each `String` field's parse error variant from an
// `Infallible`, and rustc reads that constructor as an unreachable call. The
// derive's items sit beside the enum, so only a module-level expect reaches them.
#![expect(
    unreachable_code,
    reason = "Routable's derive wraps String fields' Infallible parse errors"
)]

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
use crate::inbound::screens::deck::list::DeckList;
use crate::inbound::screens::deck::view::ViewDeck;
use crate::inbound::screens::home::Home;
use crate::inbound::screens::not_found::NotFound;
use crate::inbound::screens::playstyle::Playstyle;
use crate::inbound::screens::privacy_policy::PrivacyPolicy;
use crate::inbound::session::auth_gate::AuthGate;
use crate::inbound::session::signed_out_gate::SignedOutGate;
use crate::inbound::shell::{OpenFrame, PublicFrame};

/// The app's routes. Login, Register and Forgot password sit in
/// [`PublicFrame`]; everything else sits in one [`OpenFrame`], so a
/// signed-in user keeps the one top nav
/// [`Shell`](crate::inbound::shell::Shell) from screen to screen, About and
/// Privacy policy included. Privacy policy, About and Not found are open to
/// everyone, and the rest need a session ([`AuthGate`]). `/changelog`
/// redirects to About and `/config` to Settings.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
#[expect(missing_docs, reason = "each route field is the URL parameter its path names")]
#[expect(clippy::duplicated_attributes, reason = "each nested layout closes with its own #[end_layout]")]
pub enum Route {
    #[redirect("/changelog", || Route::About {})]
    #[redirect("/config", || Route::Config {})]
    // == public: Home instead, for a load with a live session ==
    #[layout(SignedOutGate)]
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
    #[end_layout]

    // == the rest: the signed-in nav or the public one, the layout running
    // to the end ==
    #[layout(OpenFrame)]
        /// The privacy policy, from Account, or from About signed out.
        #[route("/privacy")]
        PrivacyPolicy {},
        /// The version, the links, the credits and the release history.
        #[route("/about")]
        About {},

    // == authed ==
    #[layout(AuthGate)]
        /// The hero, the Decks and Colors panels and the featured flavor card.
        #[route("/")]
        Home {},
        /// Username, email, password, log out, delete account.
        #[route("/account")]
        Account {},
        /// Settings: theme, pips and Universes Beyond. The old `/config`
        /// redirects here.
        #[route("/settings")]
        Config {},
        /// How the user picks, from their swipes.
        #[route("/activity")]
        Activity {},
        /// Every deck the user has.
        #[route("/decks")]
        DeckList {},
        /// Commanders saved for later from commander search.
        #[route("/maybeboard")]
        CommanderMaybeboard {},
        /// What the user builds, with suggested commanders.
        #[route("/playstyle")]
        Playstyle {},
        /// A new deck.
        #[route("/decks/new")]
        CreateDeck {},
        /// A deck's profile.
        #[route("/decks/:deck_id/edit")]
        EditDeck { deck_id: Uuid },
        /// A deck: profile, sections and stats.
        #[route("/decks/:deck_id")]
        ViewDeck { deck_id: Uuid },
        /// Cards from a pasted list.
        #[route("/decks/:deck_id/import")]
        ImportDeck { deck_id: Uuid },
        /// The deck as a text list.
        #[route("/decks/:deck_id/export")]
        ExportDeck { deck_id: Uuid },
        /// Swipe to add cards.
        #[route("/decks/:deck_id/add")]
        AddDeckCard { deck_id: Uuid },
        /// Every card in the deck.
        #[route("/decks/:deck_id/cards")]
        ViewDeckCard { deck_id: Uuid },
        /// Swipe to remove cards.
        #[route("/decks/:deck_id/remove")]
        RemoveDeckCard { deck_id: Uuid },
    #[end_layout]

    // == anything else: zite's dead end ==
        /// Any path no route matches.
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;

    use components::ThemeConfig;
    use dioxus::history::{History, MemoryHistory};
    use reqwest::Url;
    use zwipe_client::ZwipeClient;
    use zwipe_core::domain::auth::models::platform::ClientPlatform;
    use zwipe_core::domain::auth::models::session::Session;
    use zwipe_core::domain::user::models::settings::PipStyle;
    use zwipe_ui::telemetry::SharedUsageBuffer;

    use super::*;
    use crate::config::APP_VERSION;
    use crate::inbound::chrome::hint_topic::HintTopic;
    use crate::inbound::chrome::leave_guard;
    use crate::inbound::shell::{DeckOpener, MenuOpen, use_footer_lift};
    use crate::inbound::{caches, card_stores, catalog};
    use crate::test_fixtures;

    /// The app's root contexts and the router, opening at `/account`, with
    /// `session` as the session.
    #[component]
    fn Root(session: Option<Session>) -> Element {
        let history: Rc<dyn History> = Rc::new(MemoryHistory::with_initial_path("/account"));
        use_hook(|| provide_context(history));
        use_context_provider(|| Signal::new(session));
        use_context_provider(|| {
            Signal::new(ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap()))
        });
        use_context_provider(|| SharedUsageBuffer::new(ClientPlatform::Web, APP_VERSION));
        use_context_provider(|| Signal::new(ThemeConfig::default()));
        use_context_provider(|| Signal::new(PipStyle::default()));
        caches::use_root_caches();
        catalog::use_catalog_cache();
        card_stores::use_card_stores();
        let overlays = components::use_overlay_back_stack();
        use_context_provider(|| overlays);
        let dialogs = components::use_dialog_host();
        use_context_provider(|| dialogs);
        use_context_provider(|| Signal::new(None::<HintTopic>));
        use_context_provider(|| components::SplashGate(Signal::new(true)));
        use_context_provider(|| MenuOpen(Signal::new(false)));
        DeckOpener::use_provider();
        use_footer_lift();
        rsx! {
            components::ToastStack { default_duration: components::TOAST_NORMAL, Router::<Route> {} }
        }
    }

    /// The app at `/account` with `session`, rendered.
    fn app(session: Option<Session>) -> VirtualDom {
        let mut dom = VirtualDom::new_with_props(Root, RootProps { session });
        dom.rebuild_in_place();
        dom
    }

    /// Navigates to `route` and renders what it draws.
    fn go(dom: &mut VirtualDom, route: Route) {
        dom.in_scope(ScopeId::APP, || {
            dioxus::router::root_router().unwrap().push(route);
        });
        // The router repaints once queued events run; this renders the new
        // route now.
        dom.mark_all_dirty();
        dom.render_immediate_to_vec();
    }

    // Off the browser, screens start their timers on tokio.
    #[tokio::test]
    async fn a_signed_in_user_keeps_one_leave_dialog_through_about_and_privacy() {
        let mut dom = app(Some(test_fixtures::session()));
        let dialog = leave_guard::mounted_dialog().unwrap();

        for route in [Route::About {}, Route::PrivacyPolicy {}, Route::Account {}] {
            let path = route.to_string();
            go(&mut dom, route);
            assert_eq!(
                leave_guard::mounted_dialog(),
                Some(dialog),
                "{path} draws in the Shell /account drew in"
            );
        }
    }

    #[tokio::test]
    async fn a_signed_out_visit_to_about_has_no_leave_dialog() {
        let mut dom = app(None);
        go(&mut dom, Route::About {});
        assert_eq!(
            leave_guard::mounted_dialog(),
            None,
            "the public frame has no nav dialog"
        );
    }

    fn id() -> Uuid {
        Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef)
    }

    #[test]
    fn every_route_prints_and_parses_at_its_path() {
        let d = "01234567-89ab-cdef-0123-456789abcdef";
        let deck_id = id();
        let routes = [
            (Route::Login {}, "/login".to_string()),
            (Route::Register {}, "/register".to_string()),
            (Route::ForgotPassword {}, "/forgot-password".to_string()),
            (Route::PrivacyPolicy {}, "/privacy".to_string()),
            (Route::About {}, "/about".to_string()),
            (Route::Home {}, "/".to_string()),
            (Route::Account {}, "/account".to_string()),
            (Route::Config {}, "/settings".to_string()),
            (Route::Activity {}, "/activity".to_string()),
            (Route::DeckList {}, "/decks".to_string()),
            (Route::CommanderMaybeboard {}, "/maybeboard".to_string()),
            (Route::Playstyle {}, "/playstyle".to_string()),
            (Route::CreateDeck {}, "/decks/new".to_string()),
            (Route::ViewDeck { deck_id }, format!("/decks/{d}")),
            (Route::EditDeck { deck_id }, format!("/decks/{d}/edit")),
            (Route::ViewDeckCard { deck_id }, format!("/decks/{d}/cards")),
            (Route::AddDeckCard { deck_id }, format!("/decks/{d}/add")),
            (
                Route::RemoveDeckCard { deck_id },
                format!("/decks/{d}/remove"),
            ),
            (Route::ImportDeck { deck_id }, format!("/decks/{d}/import")),
            (Route::ExportDeck { deck_id }, format!("/decks/{d}/export")),
        ];
        for (route, path) in routes {
            assert_eq!(route.to_string(), path);
            assert_eq!(Route::from_str(&path).ok(), Some(route), "{path}");
        }
    }

    #[test]
    fn the_old_action_paths_are_not_found() {
        for old in [
            "/user",
            "/deck",
            "/deck/create",
            "/deck/suggested-commanders",
        ] {
            assert!(
                matches!(Route::from_str(old), Ok(Route::NotFound { .. })),
                "{old}"
            );
        }
    }
}
