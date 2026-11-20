//! Shared screen header.
//!
//! Every screen opens with the same bar, the kit's `NavBar` folded behind
//! its hamburger: a back arrow on the left on every screen but Home and the
//! auth screens, then the card search's chip while signed in, the screen's
//! title and,
//! on screens with a hint dialog, its `?` sit in the middle, and the hamburger
//! on the right opens the [main menu](crate::inbound::components::navigation::menu).
//! The screen still owns its `HintDialog` (with screen-specific content);
//! passing its `open` signal as `hint` wires up the `?`.

use components::{InfoButton, NavBar, use_overlay_back};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::components::card_search::HeaderSearch;
use crate::inbound::components::navigation::menu::{MenuFooter, MenuLinks, MenuOpen};
use crate::inbound::router::Router;

/// The header bar with the main menu, titled `title`.
///
/// Omit `hint` for screens without a hint dialog (the `?` isn't rendered).
/// Pass `back` on a screen below a section: the left corner then shows a back
/// arrow that calls it; without it the corner is empty. `search: false` leaves
/// the card search's chip out, for a picker that opens as a screen over
/// another, whose own header stays mounted under it.
#[component]
pub fn ScreenHeader(
    title: String,
    hint: Option<Signal<bool>>,
    back: Option<Callback<()>>,
    #[props(default = true)] search: bool,
) -> Element {
    let route: Router = use_route();
    let mut open = use_context::<MenuOpen>().0;
    let session: Signal<Option<Session>> = use_context();
    let signed_in = session.read().is_some();
    // While the menu is open, Android back and the iOS edge swipe close it
    // instead of leaving the screen.
    use_overlay_back(open);

    // A new route closes the menu. The menu's open state is app-wide, so
    // this header mounts with the panel open and the kit's close transition
    // plays over the new screen.
    use_effect(use_reactive!(|route| {
        let _ = route;
        open.set(false);
    }));

    rsx! {
        div { class: "app-nav",
            NavBar {
                open,
                brand: rsx! {
                    if let Some(back) = back {
                        button {
                            class: "nav-brand app-nav-back",
                            "aria-label": "Back",
                            onclick: move |_| {
                                open.set(false);
                                back.call(());
                            },
                            BackArrow {}
                        }
                    }
                    if signed_in && search {
                        HeaderSearch {}
                    }
                },
                persistent: rsx! {
                    div { class: "app-nav-title",
                        h1 { class: "app-nav-title-text", "{title}" }
                        if let Some(mut hint) = hint {
                            InfoButton { onclick: move |_| hint.set(true) }
                        }
                    }
                },
                links: rsx! {
                    MenuLinks { open, route }
                },
                trailing: rsx! {
                    MenuFooter { open }
                },
            }
        }
    }
}

/// The back arrow: a left chevron drawn in the text color.
#[component]
fn BackArrow() -> Element {
    rsx! {
        svg {
            class: "app-nav-back-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: "M15 5 8 12l7 7" }
        }
    }
}
