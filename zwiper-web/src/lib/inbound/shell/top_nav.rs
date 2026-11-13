//! The top nav over every screen, the kit's `NavBar`. On a wide window it is
//! zite's bar: the Z mark home on the left and the screens and `Community`
//! on the right. Below the breakpoint it is the app's header: a back arrow
//! on the left on every screen but Home and the auth screens, the screen's
//! title and its `?` in the middle, and the hamburger on the right, with the
//! menu dropping from it. The header follows the scroll there
//! (`assets/header_hide.js`).

use components::{BackArrow, InfoButton, NavBar, use_overlay_back};
use dioxus::prelude::*;
use zwipe_core::domain::logo;

use crate::inbound::chrome::leave_guard::LeaveDialog;
use crate::inbound::router::Route;
use crate::inbound::shell::deck_opener::DeckOpener;
use crate::inbound::shell::menu_footer::MenuFooter;
use crate::inbound::shell::nav::{self, Back, MenuOpen, Section, route_of, section_of};

/// The screen on view's title and its `?`, which the folded nav shows
/// between the corners and a wide window's nav shows in its icon row. The
/// screen's frame sets it.
#[derive(Clone, PartialEq)]
pub(super) struct PageTitle {
    /// The screen's title.
    pub(super) title: String,
    /// Opens the screen's hint, when it has one.
    pub(super) on_hint: Option<Callback<()>>,
    /// What the back arrow does in place of leaving the screen: an overlay
    /// screen under the nav closes itself.
    pub(super) on_back: Option<Callback<()>>,
}

/// Keeps `--web-nav-h` on the root at the nav's height, rounded to whole
/// pixels and grown when its row wraps, for screens that sit under the nav
/// while covering the page.
const NAV_HEIGHT_JS: &str = r"
const nav = document.querySelector('.web-shell > .web-app-nav');
if (window.__zwipeNavHeight) window.__zwipeNavHeight.disconnect();
if (nav) {
    const set = () => document.documentElement.style.setProperty('--web-nav-h', Math.round(nav.getBoundingClientRect().height) + 'px');
    window.__zwipeNavHeight = new ResizeObserver(set);
    window.__zwipeNavHeight.observe(nav);
    set();
}
";

/// The titles of the overlay screens that hold the nav, oldest first: each covers the page under
/// the nav, and the newest shows in place of the page's own ([`shown_title`]). Kept apart from the
/// page's title, so a page that sets its title late (a deck's name arriving) never takes the nav
/// from an overlay open over it.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct OverlayTitles(pub(super) Signal<Vec<PageTitle>>);

/// While `open`, an overlay screen that covers the page under the nav
/// takes the nav's title and `?` (`title`, `on_hint`), and its back arrow
/// calls `on_back`. Closing it gives the page's own back.
pub fn use_overlay_nav(
    open: Signal<bool>,
    title: String,
    on_hint: Callback<()>,
    on_back: Callback<()>,
) {
    let titles = try_use_context::<OverlayTitles>();
    let mine = title.clone();
    use_effect(move || {
        let Some(OverlayTitles(mut titles)) = titles else {
            return;
        };
        let held = titles.peek().iter().any(|t| t.title == title);
        if open() && !held {
            titles.write().push(PageTitle {
                title: title.clone(),
                on_hint: Some(on_hint),
                on_back: Some(on_back),
            });
        } else if !open() && held {
            titles.write().retain(|t| t.title != title);
        }
    });
    use_drop(move || {
        if let Some(OverlayTitles(mut titles)) = titles {
            titles.write().retain(|t| t.title != mine);
        }
    });
}

/// What the nav shows: the newest overlay's title, else the page's own.
fn shown_title<'a, T>(overlays: &'a [T], page: Option<&'a T>) -> Option<&'a T> {
    overlays.last().or(page)
}

/// The nav's title, `?` and back override: [`shown_title`] of the overlays over the page.
fn use_shown_title(page: Signal<Option<PageTitle>>) -> Memo<Option<PageTitle>> {
    let overlays = try_use_context::<OverlayTitles>();
    use_memo(move || {
        let held = overlays.map_or_default(|OverlayTitles(t)| t.read().clone());
        let own = page.read().clone();
        shown_title(&held, own.as_ref()).cloned()
    })
}

/// The `?` a wide window's nav carries for the screen on view, if any.
fn nav_hint(page: Memo<Option<PageTitle>>) -> Option<Callback<()>> {
    page.read().as_ref().and_then(|p| p.on_hint)
}

/// The nav for a signed-in user over the matched screen: the app's menu at
/// every width. A wide window shows Decks, Playstyle, Activity and
/// Commander maybeboard as pills, the current one lit, then the footer's
/// icons, the screen's `?` and Log out in the bar, with the mark the way
/// Home. Below the
/// breakpoint the same list folds behind the hamburger over the footer row.
#[component]
pub fn Shell() -> Element {
    let route: Route = use_route();
    let current = section_of(&route);
    let open = use_menu_open(&route);
    let own = use_context_provider(|| Signal::new(None::<PageTitle>));
    use_context_provider(|| OverlayTitles(Signal::new(Vec::new())));
    let page = use_shown_title(own);
    use_effect(|| {
        document::eval(NAV_HEIGHT_JS);
    });

    rsx! {
        div { class: "web-shell",
            LeaveDialog {}
            div { class: "web-app-nav",
                NavBar {
                    open,
                    brand: rsx! {
                        NavMark { open }
                        BackButton { back: Back::of(&route), page, open }
                    },
                    persistent: rsx! {
                        NavTitle { page }
                    },
                    links: rsx! {
                        for section in Section::LISTED {
                            li { key: "{section.label()}",
                                NavLink { to: route_of(section), label: section.label(), current: current == Some(section), open }
                            }
                        }
                    },
                    trailing: rsx! {
                        MenuFooter { open, signed_in: true, hint: nav_hint(page) }
                    },
                }
            }
            main { class: "web-main",
                Outlet::<Route> {}
            }
        }
    }
}

/// The signed-out menu: `Log in` and `Create account`. Forgot password is a
/// link on the Log in screen, not a place in the nav.
fn public_links() -> [(Route, &'static str); 2] {
    [
        (Route::Login {}, "Log in"),
        (Route::Register {}, "Create account"),
    ]
}

/// The nav over the public screens: the mark, then the signed-out menu
/// ([`public_links`]), the current one lit, and the footer's `!`, About icon
/// and theme pair, in the bar on a wide window and under the folded list
/// below the breakpoint.
#[component]
pub fn PublicNav() -> Element {
    let route: Route = use_route();
    let open = use_menu_open(&route);
    let own: Signal<Option<PageTitle>> = use_context();
    let page = use_shown_title(own);
    let links = public_links();
    rsx! {
        div { class: "web-app-nav",
            NavBar {
                open,
                brand: rsx! {
                    NavMark { open }
                    BackButton { back: Back::of(&route), page, open }
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
                },
                trailing: rsx! {
                    MenuFooter { open, signed_in: false, hint: nav_hint(page) }
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
fn NavTitle(page: Memo<Option<PageTitle>>) -> Element {
    let Some(PageTitle { title, on_hint, .. }) = page() else {
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
            span { class: "nav-logo", "{logo::Z}" }
        }
    }
}

/// The back arrow in the left corner below the breakpoint, boxed like the
/// hamburger, on every screen but Home and the auth screens, which hold its
/// slot empty. It goes to the screen's fixed parent, or the screen before
/// this one, or Home when the tab has no history.
///
/// While an overlay screen sits under the nav (see [`use_overlay_nav`]),
/// the arrow closes it instead.
#[component]
fn BackButton(
    back: Option<Back>,
    page: Memo<Option<PageTitle>>,
    mut open: Signal<bool>,
) -> Element {
    let navigator = use_navigator();
    let opener: DeckOpener = use_context();
    if let Some(close) = page.read().as_ref().and_then(|p| p.on_back) {
        return rsx! {
            BackArrow {
                onclick: move |()| {
                    open.set(false);
                    close.call(());
                },
            }
        };
    }
    // A screen with no back keeps the box's slot, so the bar holds still
    // between Home and the screens below it.
    let Some(back) = back else {
        return rsx! {
            span { class: "nav-brand app-nav-back nav-back-held", "aria-hidden": "true" }
        };
    };
    rsx! {
        BackArrow {
            onclick: move |()| {
                open.set(false);
                let parent = match &back {
                    Back::To(route) => route.clone(),
                    Back::Opener(deck_id) => opener.back_from(*deck_id),
                    Back::Previous => Route::Home {},
                };
                nav::back_or(navigator, parent);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_signed_out_menu_is_log_in_and_create_account_only() {
        let labels: Vec<&str> = public_links().iter().map(|(_, label)| *label).collect();
        assert_eq!(labels, ["Log in", "Create account"]);
        assert!(
            !public_links()
                .iter()
                .any(|(to, _)| *to == Route::ForgotPassword {}),
            "Forgot password lives on the Log in screen"
        );
    }

    #[test]
    fn an_overlay_s_title_shows_over_the_page_s_whenever_the_page_sets_its_own() {
        let page = "Goblin Horde";
        assert_eq!(shown_title(&["Cards"], Some(&page)), Some(&"Cards"));
        assert_eq!(
            shown_title(&["Cards", "Swipe select"], Some(&page)),
            Some(&"Swipe select"),
            "the newest overlay"
        );
        assert_eq!(shown_title::<&str>(&[], Some(&page)), Some(&"Goblin Horde"));
        assert_eq!(shown_title::<&str>(&[], None), None);
    }
}
