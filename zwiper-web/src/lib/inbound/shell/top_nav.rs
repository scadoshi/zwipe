//! The top nav over every screen, the kit's `NavBar`. On a wide window it is
//! zite's bar: the Z mark home on the left and the screens and `Community`
//! on the right. Below the breakpoint it is the app's header: a back arrow
//! on the left on every screen but Home and the auth screens, the screen's
//! title and its `?` in the middle, and the hamburger on the right, with the
//! menu dropping from it. The header follows the scroll there
//! (`assets/header_hide.js`).

use std::cell::RefCell;
use std::rc::Rc;

use components::{InfoButton, NavBar, use_overlay_back};
use dioxus::prelude::*;
use zwipe_core::domain::logo;

use crate::inbound::chrome::back_arrow::BackArrow;
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

/// While `open`, an overlay screen that covers the page under the nav
/// takes the nav's title and `?` (`title`, `on_hint`), and its back arrow
/// calls `on_back`. Closing it gives the page's own back.
pub fn use_overlay_nav(
    open: Signal<bool>,
    title: String,
    on_hint: Callback<()>,
    on_back: Callback<()>,
) {
    let page = try_use_context::<Signal<Option<PageTitle>>>();
    // The page's own title, held while the overlay has the nav.
    let held = use_hook(|| Rc::new(RefCell::new(None::<Option<PageTitle>>)));
    let on_drop = Rc::clone(&held);
    let mine = title.clone();
    use_effect(move || {
        let Some(mut page) = page else {
            return;
        };
        if open() {
            if held.borrow().is_none() {
                *held.borrow_mut() = Some(page.peek().clone());
                page.set(Some(PageTitle {
                    title: title.clone(),
                    on_hint: Some(on_hint),
                    on_back: Some(on_back),
                }));
            }
        } else if let Some(previous) = held.borrow_mut().take()
            && shows_overlay(page.peek().as_ref(), &title)
        {
            page.set(previous);
        }
    });
    use_drop(move || {
        if let (Some(mut page), Some(previous)) = (page, on_drop.borrow_mut().take())
            && shows_overlay(page.peek().as_ref(), &mine)
        {
            page.set(previous);
        }
    });
}

/// Whether the nav still shows the overlay titled `title`, so closing it
/// may put the page's own title back. When the page itself is going (a
/// browser back from under the overlay) its frame has already cleared or
/// replaced the title, and the held one carries that page's callbacks,
/// which are dropping with it: putting it back would hand the nav dead
/// callbacks.
fn shows_overlay(current: Option<&PageTitle>, title: &str) -> bool {
    is_overlay_title(
        current.map(|p| (p.title.as_str(), p.on_back.is_some())),
        title,
    )
}

/// [`shows_overlay`] on the nav's `(title, has a back override)`: an
/// overlay's title is the one with a back override.
fn is_overlay_title(current: Option<(&str, bool)>, title: &str) -> bool {
    current.is_some_and(|(shown, overlay)| overlay && shown == title)
}

/// The `?` a wide window's nav carries for the screen on view, if any.
fn nav_hint(page: Signal<Option<PageTitle>>) -> Option<Callback<()>> {
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
    let page = use_context_provider(|| Signal::new(None::<PageTitle>));
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

/// The nav over the public screens: the mark, then the app's signed-out
/// menu, `Log in`, `Create account` and `Forgot password`, the current one
/// lit, and the footer's `!`, About icon and theme pair, in the bar on a
/// wide window and under the folded list below the breakpoint.
#[component]
pub fn PublicNav() -> Element {
    let route: Route = use_route();
    let open = use_menu_open(&route);
    let page: Signal<Option<PageTitle>> = use_context();
    let links = [
        (Route::Login {}, "Log in"),
        (Route::Register {}, "Create account"),
        (Route::ForgotPassword {}, "Forgot password"),
    ];
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
fn NavTitle(page: Signal<Option<PageTitle>>) -> Element {
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
    page: Signal<Option<PageTitle>>,
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
    fn closing_an_overlay_restores_the_page_title_only_while_the_nav_shows_it() {
        assert!(is_overlay_title(
            Some(("Swipe select", true)),
            "Swipe select"
        ));
    }

    #[test]
    fn a_page_going_from_under_the_overlay_keeps_its_title_out() {
        // The page's frame cleared the title as it went.
        assert!(!is_overlay_title(None, "Swipe select"));
        // Or the next page's frame already put its own up.
        assert!(!is_overlay_title(
            Some(("Goblin Horde", false)),
            "Swipe select"
        ));
        // Another overlay with another title has the nav.
        assert!(!is_overlay_title(
            Some(("Suggested commanders", true)),
            "Swipe select"
        ));
    }
}
