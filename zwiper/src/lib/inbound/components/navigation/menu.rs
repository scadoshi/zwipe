//! The main menu behind the header's hamburger.
//!
//! Signed in, it lists the sections, then a footer with the `!` for Help and
//! feedback and Log out. Signed out, it lists the auth screens. The entry
//! for the screen on view, or the section it sits under, is lit.

use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::account::LOG_OUT;

use crate::inbound::components::auth::signal_logout::SignalLogout;
use crate::inbound::components::logout_dialog::LogoutDialog;
use crate::inbound::components::support::SupportButton;
use crate::inbound::router::Router;

/// A screen the signed-in menu opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Home.
    Home,
    /// The deck list, and every screen inside a deck.
    Decks,
    /// Playstyle, with its suggested commanders.
    Playstyle,
    /// The commander maybeboard.
    Maybeboard,
    /// Activity.
    Activity,
    /// Account.
    Account,
    /// Settings (the `config` screen).
    Config,
    /// About, and the privacy policy it links to.
    About,
}

impl Section {
    /// Every section, in menu order: Home, the deck screens, then the user's own.
    pub const ALL: [Self; 8] = [
        Self::Home,
        Self::Decks,
        Self::Playstyle,
        Self::Maybeboard,
        Self::Activity,
        Self::Account,
        Self::Config,
        Self::About,
    ];

    /// The entry's label, the title of the screen it opens.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Decks => "Decks",
            Self::Playstyle => "Playstyle",
            Self::Maybeboard => "Commander maybeboard",
            Self::Activity => "Activity",
            Self::Account => "Account",
            Self::Config => "Settings",
            Self::About => "About",
        }
    }

    /// The screen the entry opens.
    #[must_use]
    pub const fn route(self) -> Router {
        match self {
            Self::Home => Router::Home {},
            Self::Decks => Router::DeckList {},
            Self::Playstyle => Router::SuggestedCommanders {},
            Self::Maybeboard => Router::CommanderMaybeboard {},
            Self::Activity => Router::Activity {},
            Self::Account => Router::Account {},
            Self::Config => Router::Config {},
            Self::About => Router::About {},
        }
    }

    /// The section a route sits under, lit in the menu; `None` for the auth
    /// screens.
    #[must_use]
    pub const fn of(route: &Router) -> Option<Self> {
        match route {
            Router::Home {} => Some(Self::Home),
            Router::DeckList {}
            | Router::CreateDeck
            | Router::EditDeck { .. }
            | Router::ViewDeck { .. }
            | Router::ImportDeck { .. }
            | Router::ExportDeck { .. }
            | Router::AddDeckCard { .. }
            | Router::ViewDeckCard { .. }
            | Router::RemoveDeckCard { .. } => Some(Self::Decks),
            Router::SuggestedCommanders {} => Some(Self::Playstyle),
            Router::CommanderMaybeboard {} => Some(Self::Maybeboard),
            Router::Activity {} => Some(Self::Activity),
            Router::Account {} => Some(Self::Account),
            Router::Config {} => Some(Self::Config),
            Router::About {} | Router::PrivacyPolicy {} => Some(Self::About),
            Router::Login {} | Router::Register {} | Router::ForgotPassword {} => None,
        }
    }
}

/// The auth screens the signed-out menu lists, with their labels.
const SIGNED_OUT: [(&str, Router); 3] = [
    ("Log in", Router::Login {}),
    ("Create profile", Router::Register {}),
    ("Forgot password", Router::ForgotPassword {}),
];

/// The menu's link rows, for the nav's `links` slot. Each closes the menu as
/// it navigates.
#[component]
pub fn MenuLinks(open: Signal<bool>, route: Router) -> Element {
    let session: Signal<Option<Session>> = use_context();
    if session.read().is_none() {
        return rsx! {
            for (label, to) in SIGNED_OUT {
                li { key: "{label}",
                    Link {
                        class: if to == route { "nav-link active" } else { "nav-link" },
                        to,
                        onclick: move |_| open.set(false),
                        "{label}"
                    }
                }
            }
        };
    }
    let current = Section::of(&route);
    rsx! {
        for section in Section::ALL {
            SectionLink { key: "{section.label()}", section, current, open }
        }
    }
}

/// One section's row, lit when it is `current`.
#[component]
fn SectionLink(section: Section, current: Option<Section>, mut open: Signal<bool>) -> Element {
    rsx! {
        li {
            Link {
                class: if current == Some(section) { "nav-link active" } else { "nav-link" },
                to: section.route(),
                onclick: move |_| open.set(false),
                "{section.label()}"
            }
        }
    }
}

/// The menu's footer, for the nav's `trailing` slot: the `!` for Help and
/// feedback and, signed in, a small Log out that asks first.
#[component]
pub fn MenuFooter(open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut confirm = use_signal(|| false);
    let signed_in = session.read().is_some();
    rsx! {
        div { class: "app-nav-footer",
            SupportButton {}
            if signed_in {
                button {
                    class: "btn-action app-nav-logout",
                    onclick: move |_| confirm.set(true),
                    {LOG_OUT}
                }
            }
        }
        LogoutDialog {
            open: confirm,
            on_confirm: move |()| {
                let mut open = open;
                open.set(false);
                session.logout(client);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn every_section_lights_up_on_its_own_screen() {
        for section in Section::ALL {
            assert_eq!(Section::of(&section.route()), Some(section), "{section:?}");
        }
    }

    #[test]
    fn the_screens_inside_a_deck_light_up_decks() {
        let deck_id = Uuid::nil();
        for route in [
            Router::CreateDeck,
            Router::EditDeck { deck_id },
            Router::ViewDeck { deck_id },
            Router::ImportDeck { deck_id },
            Router::ExportDeck { deck_id },
            Router::AddDeckCard { deck_id },
            Router::ViewDeckCard { deck_id },
            Router::RemoveDeckCard { deck_id },
        ] {
            assert_eq!(Section::of(&route), Some(Section::Decks), "{route:?}");
        }
    }

    #[test]
    fn the_auth_screens_light_up_nothing() {
        for route in [
            Router::Login {},
            Router::Register {},
            Router::ForgotPassword {},
        ] {
            assert_eq!(Section::of(&route), None, "{route:?}");
        }
    }

    #[test]
    fn the_privacy_policy_sits_under_about() {
        assert_eq!(Section::of(&Router::PrivacyPolicy {}), Some(Section::About));
    }
}
