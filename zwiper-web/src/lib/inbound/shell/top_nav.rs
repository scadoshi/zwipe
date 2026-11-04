//! The top nav over every screen, the kit's `NavBar`. On a wide window it is
//! zite's bar: the Z mark home on the left and the screens and `Community`
//! on the right. Below the breakpoint it is the app's header: a back arrow
//! on the left on every screen but Home and the auth screens, the screen's
//! title and its `?` in the middle, and the hamburger on the right, with the
//! menu dropping from it. The header follows the scroll there
//! (`assets/header_hide.js`).

use components::{InfoButton, NavBar, use_overlay_back};
use dioxus::prelude::*;
use zwipe_core::domain::logo;
use zwipe_core::domain::site::WEB_BASE;

use crate::inbound::chrome::back_arrow::BackArrow;
use crate::inbound::router::Route;
use crate::inbound::shell::menu_footer::MenuFooter;
use crate::inbound::shell::menu_theme::MenuTheme;
use crate::inbound::shell::nav::{Back, MenuOpen, Section, route_of, section_of};

/// The screen on view's title and its `?`, which the folded nav shows
/// between the corners. A screen with a page head sets it.
#[derive(Clone, PartialEq)]
pub(super) struct PageTitle {
    /// The screen's title.
    pub(super) title: String,
    /// Opens the screen's hint, when it has one.
    pub(super) on_hint: Option<Callback<()>>,
}

/// The nav for a signed-in user over the matched screen. A wide window
/// shows Decks, Activity, Playstyle, Account, Settings and `Community` as
/// pills, the current one lit, with the mark the way Home. Below the
/// breakpoint the menu is the app's: Decks, Playstyle, Activity and
/// Commander maybeboard, over the footer row of icons.
#[component]
pub fn Shell() -> Element {
    let route: Route = use_route();
    let current = section_of(&route);
    let open = use_menu_open(&route);
    let page = use_context_provider(|| Signal::new(None::<PageTitle>));

    rsx! {
        div { class: "web-shell",
            div { class: "web-app-nav",
                NavBar {
                    open,
                    brand: rsx! {
                        NavMark { open }
                        BackButton { back: Back::of(&route), open }
                    },
                    persistent: rsx! {
                        NavTitle { page }
                    },
                    links: rsx! {
                        // The app's list; a wide window hides the maybeboard
                        // and orders Activity before Playstyle (web.css).
                        for section in Section::LISTED {
                            li {
                                key: "{section.label()}",
                                class: if section == Section::Maybeboard { "web-nav-narrow" } else { "" },
                                "data-section": section.label(),
                                NavLink { to: route_of(section), label: section.label(), current: current == Some(section), open }
                            }
                        }
                        // Account and Config stay pills on a wide window; the
                        // folded menu's footer holds them as icons.
                        for section in [Section::Account, Section::Config] {
                            li { key: "{section.label()}", class: "web-nav-wide",
                                NavLink { to: route_of(section), label: section.label(), current: current == Some(section), open }
                            }
                        }
                        li { class: "web-nav-wide",
                            CommunityLink { open }
                        }
                    },
                    trailing: rsx! {
                        MenuFooter { open, signed_in: true }
                    },
                }
            }
            main { class: "web-main",
                Outlet::<Route> {}
            }
        }
    }
}

/// The nav over the public screens: the mark, then the app's signed-out
/// menu, `Log in`, `Create profile`, `Forgot password` and `About`, the
/// current one lit. A wide window adds `Community` and the theme pair as
/// pills; below the breakpoint the footer holds the `!`, the About icon and
/// the theme pair.
#[component]
pub fn PublicNav() -> Element {
    let route: Route = use_route();
    let open = use_menu_open(&route);
    let page: Signal<Option<PageTitle>> = use_context();
    let links = [
        (Route::Login {}, "Log in"),
        (Route::Register {}, "Create profile"),
        (Route::ForgotPassword {}, "Forgot password"),
        (Route::About {}, Section::About.label()),
    ];
    rsx! {
        div { class: "web-app-nav",
            NavBar {
                open,
                brand: rsx! {
                    NavMark { open }
                    BackButton { back: Back::of(&route), open }
                },
                persistent: rsx! {
                    NavTitle { page }
                },
                links: rsx! {
                    for (to, label) in links {
                        li { key: "{label}",
                            NavLink { current: to == route, to, label, open }
                        }
                    }
                    li { class: "web-nav-wide",
                        CommunityLink { open }
                    }
                    // No Settings signed out, so the theme pair sits in the
                    // bar; the pick stays on this device until a profile's
                    // theme takes over.
                    li { class: "web-nav-wide web-nav-theme",
                        MenuTheme { open }
                    }
                },
                trailing: rsx! {
                    MenuFooter { open, signed_in: false }
                },
            }
        }
    }
}

/// The root's menu state. A new route closes the menu: the state outlives
/// the screen, so the kit's close transition plays over the new one. While
/// the menu is open, Escape and the browser's back close it instead of
/// leaving the screen.
fn use_menu_open(route: &Route) -> Signal<bool> {
    let route = route.clone();
    let mut open = use_context::<MenuOpen>().0;
    use_overlay_back(open);
    use_effect(use_reactive!(|route| {
        let _ = route;
        open.set(false);
    }));
    open
}

/// One nav entry, lit when it is `current`. A tap navigates and leaves the
/// menu to the next screen to close; on the screen it opens, it closes the
/// menu instead.
#[component]
fn NavLink(to: Route, label: &'static str, current: bool, mut open: Signal<bool>) -> Element {
    rsx! {
        Link {
            to,
            class: if current { "active" } else { "" },
            onclick: move |_| {
                if current {
                    open.set(false);
                }
            },
            "{label}"
        }
    }
}

/// The screen's title and `?`, in the middle of the folded nav.
#[component]
fn NavTitle(page: Signal<Option<PageTitle>>) -> Element {
    let Some(PageTitle { title, on_hint }) = page() else {
        return rsx! {};
    };
    rsx! {
        div { class: "app-nav-title",
            h2 { class: "app-nav-title-text", "{title}" }
            if let Some(open) = on_hint {
                InfoButton { onclick: move |_| open.call(()) }
            }
        }
    }
}

/// The Z mark, the app icon's art, in the logo color, linking Home (Login
/// when signed out, through the auth gate). A wide window's brand, with
/// the back arrow beside it; below the breakpoint the corner holds the
/// arrow alone.
#[component]
fn NavMark(open: Signal<bool>) -> Element {
    rsx! {
        Link {
            to: Route::Home {},
            class: "nav-brand web-nav-mark web-nav-wide",
            "aria-label": "Home",
            onclick: move |_| open.set(false),
            span { class: "nav-logo web-nav-logo", "{logo::Z}" }
        }
    }
}

/// The back arrow in the left corner below the breakpoint, boxed like the
/// hamburger, on every screen but Home and the auth screens. It goes to
/// the screen's fixed parent, or the screen before this one, or Home when
/// the tab has no history.
#[component]
fn BackButton(back: Option<Back>, mut open: Signal<bool>) -> Element {
    let navigator = use_navigator();
    let Some(back) = back else {
        return rsx! {};
    };
    rsx! {
        BackArrow {
            onclick: move |()| {
                open.set(false);
                match &back {
                    Back::To(route) => {
                        navigator.push(route.clone());
                    }
                    Back::Previous if navigator.can_go_back() => navigator.go_back(),
                    Back::Previous => {
                        navigator.push(Route::Home {});
                    }
                }
            },
        }
    }
}

/// `Community`, a nav pill to zite's Community page (Discord, email
/// support), folding the menu.
#[component]
fn CommunityLink(mut open: Signal<bool>) -> Element {
    rsx! {
        a {
            class: "nav-link",
            href: "{WEB_BASE}/community",
            onclick: move |_| open.set(false),
            "Community"
        }
    }
}
