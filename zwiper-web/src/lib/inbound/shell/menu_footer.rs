//! The folded menu's footer below the breakpoint, the app's: the `!` for
//! Help and feedback and, signed in, the icons for Home, Account, Settings
//! and About, the theme pair, and a small Log out that asks first.

use components::{AlertDialog, DialogAction, DockIcon};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::site::DISCORD_URL;
use zwipe_ui::screens::account::LOG_OUT;
use zwipe_ui::support;

use crate::config::APP_VERSION;
use crate::inbound::chrome::logout_dialog::LogoutDialog;
use crate::inbound::router::Route;
use crate::inbound::session::logout;
use crate::inbound::shell::menu_theme::MenuTheme;
use crate::inbound::shell::nav::{Section, route_of, section_of};

/// The platform the problem report email names.
const PLATFORM: &str = "Web";

/// The footer, for the nav's `trailing` slot. Signed out it holds only the
/// `!`. A wide window hides it: the bar's pills carry the same screens.
#[component]
pub fn MenuFooter(open: Signal<bool>, signed_in: bool) -> Element {
    let route: Route = use_route();
    let current = section_of(&route);
    let mut confirm = use_signal(|| false);
    rsx! {
        div { class: "nav-trailing-row web-nav-narrow",
            div { class: "nav-trailing-group",
                SupportButton {}
                if signed_in {
                    for section in Section::FOOTER {
                        MenuIcon {
                            key: "{section.label()}",
                            section,
                            current: current == Some(section),
                            open,
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
        if signed_in {
            LogoutConfirm { open, confirm }
        }
    }
}

/// The Log out dialog, which ends the session once confirmed and folds the
/// menu as Login takes over.
#[component]
fn LogoutConfirm(mut open: Signal<bool>, confirm: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    rsx! {
        LogoutDialog {
            open: confirm,
            on_confirm: move |()| {
                open.set(false);
                logout::logout(session, client);
            },
        }
    }
}

/// One footer icon, lit on the screen it opens. A tap navigates and leaves
/// the menu to the next screen to close; on its own screen it closes the
/// menu instead.
#[component]
fn MenuIcon(section: Section, current: bool, mut open: Signal<bool>) -> Element {
    let route: Route = use_route();
    let navigator = use_navigator();
    let Some(path) = section.icon() else {
        return rsx! {};
    };
    let to = route_of(section);
    rsx! {
        button {
            class: if current { "btn-action nav-icon nav-link active" } else { "btn-action nav-icon nav-link" },
            "aria-label": section.label(),
            onclick: move |_| {
                if to == route {
                    open.set(false);
                } else {
                    navigator.push(to.clone());
                }
            },
            DockIcon { path }
        }
    }
}

/// The `!` and its Help and feedback dialog: a problem report email, filled
/// with the version and the platform, or the Discord.
#[component]
fn SupportButton() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        button {
            class: "btn-action app-nav-support",
            "aria-label": support::BUTTON_LABEL,
            onclick: move |_| open.set(true),
            {support::GLYPH}
        }
        AlertDialog {
            open,
            title: support::TITLE,
            actions: vec![
                DialogAction::Dismiss(support::CLOSE.into()),
                DialogAction::Link {
                    label: support::REPORT.into(),
                    href: support::report_mailto(APP_VERSION, PLATFORM),
                },
                DialogAction::Link {
                    label: support::DISCORD.into(),
                    href: DISCORD_URL.into(),
                },
            ],
            {support::BODY}
        }
    }
}
