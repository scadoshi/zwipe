//! The main menu behind the header's hamburger.
//!
//! Signed in, it lists the deck sections, then a footer with the `!` for
//! Help and feedback, the icons for Home, Account, Settings and About, the
//! theme's palette and the mode's sun or moon, and Log out.
//! Signed out, it lists the auth screens. The entry for the screen
//! on view, or the section it sits under, is lit.
//!
//! The open state is app-wide, provided at the root as
//! [`MenuOpen`](crate::inbound::components::navigation::menu::MenuOpen), so a
//! tap on an entry only navigates: the new screen's header mounts with the
//! panel still open and folds it shut over its own content.

/// The menu's sections, the shape shared with zwiper-web; the routes they
/// open are this app's, in [`route_of`] and [`section_of`].
pub use components::MenuSection as Section;
use components::icons::DockIcon;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::account::LOG_OUT;

use crate::inbound::components::auth::signal_logout::SignalLogout;
use crate::inbound::components::logout_dialog::LogoutDialog;
use crate::inbound::components::navigation::menu_theme::MenuTheme;
use crate::inbound::components::support::SupportButton;
use crate::inbound::router::Router;

/// The screen `section` opens.
#[must_use]
pub const fn route_of(section: Section) -> Router {
    match section {
        Section::Home => Router::Home {},
        Section::Decks => Router::DeckList {},
        Section::Playstyle => Router::SuggestedCommanders {},
        Section::Maybeboard => Router::CommanderMaybeboard {},
        Section::Activity => Router::Activity {},
        Section::Account => Router::Account {},
        Section::Config => Router::Config {},
        Section::About => Router::About {},
    }
}

/// The section `route` sits under, lit in the menu; `None` for the auth
/// screens.
#[must_use]
pub const fn section_of(route: &Router) -> Option<Section> {
    match route {
        Router::Home {} => Some(Section::Home),
        Router::DeckList {}
        | Router::CreateDeck
        | Router::EditDeck { .. }
        | Router::ViewDeck { .. }
        | Router::ImportDeck { .. }
        | Router::ExportDeck { .. }
        | Router::AddDeckCard { .. }
        | Router::ViewDeckCard { .. }
        | Router::RemoveDeckCard { .. } => Some(Section::Decks),
        Router::SuggestedCommanders {} => Some(Section::Playstyle),
        Router::CommanderMaybeboard {} => Some(Section::Maybeboard),
        Router::Activity {} => Some(Section::Activity),
        Router::Account {} => Some(Section::Account),
        Router::Config {} => Some(Section::Config),
        Router::About {} | Router::PrivacyPolicy {} => Some(Section::About),
        Router::Login {} | Router::Register {} | Router::ForgotPassword {} => None,
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
    let current = section_of(&route);
    rsx! {
        for section in Section::LISTED {
            li { key: "{section.label()}",
                MenuEntry {
                    label: section.label(),
                    to: route_of(section),
                    current: current == Some(section),
                    open,
                }
            }
        }
    }
}

/// One menu button, lit when it is `current`: the label as text, or with an
/// `icon` the icon alone, the label read out for it. A tap pushes `to`; on
/// the screen it opens, it closes the menu instead.
#[component]
fn MenuEntry(
    label: &'static str,
    to: Router,
    current: bool,
    mut open: Signal<bool>,
    #[props(default)] icon: Option<&'static str>,
) -> Element {
    let route: Router = use_route();
    let nav = use_navigator();
    let class = match (icon.is_some(), current) {
        (false, false) => "nav-link",
        (false, true) => "nav-link active",
        (true, false) => "btn-action nav-icon nav-link",
        (true, true) => "btn-action nav-icon nav-link active",
    };
    rsx! {
        button {
            class,
            "aria-label": icon.map(|_| label),
            onclick: move |_| {
                if to == route {
                    open.set(false);
                } else {
                    nav.push(to.clone());
                }
            },
            match icon {
                Some(path) => rsx! { DockIcon { path } },
                None => rsx! { "{label}" },
            }
        }
    }
}

/// The menu's footer, for the nav's `trailing` slot: the `!` for Help and
/// feedback and, signed in, the icons for Home, Account, Settings and About
/// beside it, then the theme pair, and a small Log out on the right that
/// asks first.
#[component]
pub fn MenuFooter(open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let route: Router = use_route();
    let mut confirm = use_signal(|| false);
    let signed_in = session.read().is_some();
    let current = section_of(&route);
    rsx! {
        div { class: "nav-trailing-row",
            div { class: "nav-trailing-group",
                SupportButton {}
                if signed_in {
                    for section in Section::FOOTER {
                        MenuEntry {
                            key: "{section.label()}",
                            label: section.label(),
                            to: route_of(section),
                            current: current == Some(section),
                            open,
                            icon: section.icon(),
                        }
                    }
                    MenuTheme { open }
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
            assert_eq!(section_of(&route_of(section)), Some(section), "{section:?}");
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
            assert_eq!(section_of(&route), Some(Section::Decks), "{route:?}");
        }
    }

    #[test]
    fn the_auth_screens_light_up_nothing() {
        for route in [
            Router::Login {},
            Router::Register {},
            Router::ForgotPassword {},
        ] {
            assert_eq!(section_of(&route), None, "{route:?}");
        }
    }

    #[test]
    fn the_privacy_policy_sits_under_about() {
        assert_eq!(section_of(&Router::PrivacyPolicy {}), Some(Section::About));
    }
}
