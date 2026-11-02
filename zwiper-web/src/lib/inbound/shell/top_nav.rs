//! The top nav over every screen, zite's bar: the Z mark home on the
//! left and the screens and `Community` on the right, folding behind a hamburger
//! below the breakpoint.

use components::NavBar;
use dioxus::prelude::*;
use zwipe_core::domain::logo;
use zwipe_core::domain::site::WEB_BASE;

use crate::inbound::router::Route;
use crate::inbound::shell::nav::Section;

/// The nav for a signed-in user over the matched screen: Decks, Activity,
/// Playstyle, Account, Config and `Community`, the current one lit. The mark is
/// the way Home.
#[component]
pub fn Shell() -> Element {
    let route: Route = use_route();
    let current = Section::of(&route);
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
                        CommunityLink { open }
                    }
                },
            }
            main { class: "web-main",
                Outlet::<Route> {}
            }
        }
    }
}

/// The nav over the public screens: the mark, then `Log in`, `Create
/// profile` and `Privacy policy`, the current one lit, and `Community`.
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
                    CommunityLink { open }
                }
            },
        }
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

/// `Community`, a nav pill to zite's Community page (Discord, email
/// support) in a new tab, folding the menu.
#[component]
fn CommunityLink(mut open: Signal<bool>) -> Element {
    rsx! {
        a {
            class: "nav-link",
            href: "{WEB_BASE}/community",
            target: "_blank",
            rel: "noopener",
            onclick: move |_| open.set(false),
            "Community"
            span { class: "web-buy-ext", "\u{2197}" }
        }
    }
}
