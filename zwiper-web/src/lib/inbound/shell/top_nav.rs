//! The top nav over every screen, zite's bar: the ZWIPE mark home on the
//! left and the screens on the right, folding behind a hamburger below the
//! breakpoint.

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
    let mut help = use_signal(|| false);
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
                        button {
                            class: "nav-link",
                            onclick: move |_| {
                                open.set(false);
                                help.set(true);
                            },
                            "Help"
                        }
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

/// The nav over the public screens: the mark, then `Log in` and `Create
/// profile`, the current one lit.
#[component]
pub fn PublicNav() -> Element {
    let route: Route = use_route();
    let mut open = use_signal(|| false);
    use_effect(use_reactive!(|route| {
        let _ = route;
        open.set(false);
    }));
    let links = [
        (Route::Login {}, "Log in"),
        (Route::Register {}, "Create profile"),
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
            },
        }
    }
}

/// The ZWIPE mark in the logo color, linking Home (Login when signed out,
/// through the auth gate).
#[component]
fn NavMark(open: Signal<bool>) -> Element {
    rsx! {
        Link {
            to: Route::Home {},
            class: "nav-brand",
            "aria-label": "Home",
            onclick: move |_| open.set(false),
            span { class: "nav-logo web-nav-logo", "{logo::ZWIPE}" }
        }
    }
}
