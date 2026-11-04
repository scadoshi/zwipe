//! The main menu behind the header's hamburger.
//!
//! Signed in, it lists the deck sections, then a footer with the `!` for
//! Help and feedback, the icons for Home, Account, Settings and About, and
//! Log out. Signed out, it lists the auth screens. The entry for the screen
//! on view, or the section it sits under, is lit.
//!
//! The open state is app-wide, provided at the root as
//! [`MenuOpen`](crate::inbound::components::navigation::menu::MenuOpen), so a
//! tap on an entry only navigates: the new screen's header mounts with the
//! panel still open and folds it shut over its own content.

use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::account::LOG_OUT;

use crate::inbound::components::action_dock::{COG, DockIcon, HOUSE, INFO, PERSON};
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
    /// Every section, in menu order: the listed ones, then the footer's.
    pub const ALL: [Self; 8] = [
        Self::Decks,
        Self::Playstyle,
        Self::Activity,
        Self::Maybeboard,
        Self::Home,
        Self::Account,
        Self::Config,
        Self::About,
    ];

    /// The deck screens, the menu's list, in order.
    pub const LISTED: [Self; 4] = [
        Self::Decks,
        Self::Playstyle,
        Self::Activity,
        Self::Maybeboard,
    ];

    /// The sections the footer holds as icons, left to right.
    pub const FOOTER: [Self; 4] = [Self::Home, Self::Account, Self::Config, Self::About];

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

    /// The footer icon's path, for the sections in [`Self::FOOTER`].
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Home => HOUSE,
            Self::Account => PERSON,
            Self::Config => COG,
            Self::About => INFO,
            Self::Decks | Self::Playstyle | Self::Maybeboard | Self::Activity => "",
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

/// Whether the main menu is open, shared by every screen's header. Provided
/// once at the app root so the panel outlives the screen it was opened on
/// and its close transition plays over the next one.
#[derive(Debug, Clone, Copy)]
pub struct MenuOpen(pub Signal<bool>);

/// The menu's link rows, for the nav's `links` slot. Each navigates and
/// leaves the menu to the next screen's header to close.
#[component]
pub fn MenuLinks(open: Signal<bool>, route: Router) -> Element {
    let session: Signal<Option<Session>> = use_context();
    if session.read().is_none() {
        return rsx! {
            for (label, to) in SIGNED_OUT {
                li { key: "{label}",
                    MenuEntry { label, current: to == route, to, open }
                }
            }
        };
    }
    let current = Section::of(&route);
    rsx! {
        for section in Section::LISTED {
            li { key: "{section.label()}",
                MenuEntry {
                    label: section.label(),
                    to: section.route(),
                    current: current == Some(section),
                    open,
                }
            }
        }
    }
}

/// One entry's button, lit when it is `current`. A tap pushes `to`; on the
/// screen it opens, it closes the menu instead.
#[component]
fn MenuEntry(label: &'static str, to: Router, current: bool, mut open: Signal<bool>) -> Element {
    let route: Router = use_route();
    let nav = use_navigator();
    rsx! {
        button {
            class: if current { "nav-link active" } else { "nav-link" },
            onclick: move |_| {
                if to == route {
                    open.set(false);
                } else {
                    nav.push(to.clone());
                }
            },
            "{label}"
        }
    }
}

/// One footer section's icon button, lit when it is `current`. A tap pushes
/// the section's route; on the screen it opens, it closes the menu instead.
#[component]
fn MenuIcon(section: Section, current: bool, mut open: Signal<bool>) -> Element {
    let route: Router = use_route();
    let nav = use_navigator();
    rsx! {
        button {
            class: if current { "btn-action app-nav-icon nav-link active" } else { "btn-action app-nav-icon nav-link" },
            "aria-label": section.label(),
            onclick: move |_| {
                if section.route() == route {
                    open.set(false);
                } else {
                    nav.push(section.route());
                }
            },
            DockIcon { path: section.icon() }
        }
    }
}

/// The menu's footer, for the nav's `trailing` slot: the `!` for Help and
/// feedback and, signed in, the icons for Home, Account, Settings and About
/// beside it and a small Log out on the right that asks first.
#[component]
pub fn MenuFooter(open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let route: Router = use_route();
    let mut confirm = use_signal(|| false);
    let signed_in = session.read().is_some();
    let current = Section::of(&route);
    rsx! {
        div { class: "app-nav-footer",
            div { class: "app-nav-footer-group",
                SupportButton {}
                if signed_in {
                    for section in Section::FOOTER {
                        MenuIcon { key: "{section.label()}", section, current: current == Some(section), open }
                    }
                }
            }
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
    fn the_listed_and_footer_sections_partition_all() {
        let mut split: Vec<Section> = Section::LISTED.into_iter().chain(Section::FOOTER).collect();
        let mut all: Vec<Section> = Section::ALL.to_vec();
        split.sort_by_key(|section| section.label());
        all.sort_by_key(|section| section.label());
        assert_eq!(split, all);
        for section in Section::FOOTER {
            assert!(!section.icon().is_empty(), "{section:?} has no icon");
        }
    }

    #[test]
    fn the_privacy_policy_sits_under_about() {
        assert_eq!(Section::of(&Router::PrivacyPolicy {}), Some(Section::About));
    }
}
