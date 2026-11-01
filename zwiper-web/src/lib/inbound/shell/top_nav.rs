//! The top nav over every screen, zite's bar: the Z mark home on the
//! left and the screens and `Help` on the right, folding behind a hamburger
//! below the breakpoint.

use components::NavBar;
use dioxus::prelude::*;
use zwipe_core::domain::logo;

use crate::inbound::router::Route;
use crate::inbound::shell::HelpDialog;
use crate::inbound::shell::nav::Section;

/// The nav for a signed-in user over the matched screen: Decks, Activity,
/// Playstyle, Account, Config and `Help`, the current one lit. The mark is
/// the way Home.
#[component]
pub fn Shell() -> Element {
    let route: Route = use_route();
    let current = Section::of(&route);
    let help = use_signal(|| false);
    let mut open = use_signal(|| false);

    // A new route closes the folded menu.
    use_effect(use_reactive!(|route| {
        let _ = route;
        open.set(false);
    }));

    rsx! {
        div { class: "web-shell",
            NavBar {
                open,
                brand: rsx! { NavMark { open } },
                links: rsx! {
                    for section in Section::ALL.into_iter().filter(|s| *s != Section::Home) {
                        li { key: "{section.label()}",
                            Link {
                                to: section.route(),
                                class: if current == Some(section) { "active" } else { "" },
                                onclick: move |_| open.set(false),
                                "{section.label()}"
                            }
                        }
                    }
                    li {
                        HelpLink { open, help }
                    }
                },
            }
            main { class: "web-main",
                Outlet::<Route> {}
            }
        }
        HelpDialog { open: help }
    }
}

/// The nav over the public screens: the mark, then `Log in`, `Create
/// profile` and `Privacy policy`, the current one lit, and `Help`.
#[component]
pub fn PublicNav() -> Element {
    let route: Route = use_route();
    let mut open = use_signal(|| false);
    let help = use_signal(|| false);
    use_effect(use_reactive!(|route| {
        let _ = route;
        open.set(false);
    }));
    let links = [
        (Route::Login {}, "Log in"),
        (Route::Register {}, "Create profile"),
        (Route::PrivacyPolicy {}, "Privacy policy"),
    ];
    rsx! {
        NavBar {
            open,
            brand: rsx! { NavMark { open } },
            links: rsx! {
                for (to, label) in links {
                    li { key: "{label}",
                        Link {
                            class: if to == route { "active" } else { "" },
                            to,
                            onclick: move |_| open.set(false),
                            "{label}"
                        }
                    }
                }
                li {
                    HelpLink { open, help }
                }
            },
        }
        HelpDialog { open: help }
    }
}

/// The Z mark, the app icon's art, in the logo color, linking Home (Login
/// when signed out, through the auth gate).
#[component]
fn NavMark(open: Signal<bool>) -> Element {
    rsx! {
        Link {
            to: Route::Home {},
            class: "nav-brand",
            "aria-label": "Home",
            onclick: move |_| open.set(false),
            span { class: "nav-logo web-nav-logo", "{logo::Z}" }
        }
    }
}

/// `Help`, a nav pill that folds the menu and opens `Help & feedback`.
#[component]
fn HelpLink(mut open: Signal<bool>, mut help: Signal<bool>) -> Element {
    rsx! {
        button {
            class: "nav-link",
            onclick: move |_| {
                open.set(false);
                help.set(true);
            },
            "Help"
        }
    }
}
