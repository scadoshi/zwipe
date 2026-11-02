//! The frames screens draw in.

use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::router::Route;
use crate::inbound::shell::top_nav::{PublicNav, Shell};

/// The layout over the public routes: the public nav over the screen.
#[component]
pub fn PublicFrame() -> Element {
    rsx! {
        div { class: "web-shell web-public",
            PublicNav {}
            main { class: "web-main",
                Outlet::<Route> {}
            }
        }
    }
}

/// The layout over a route open to everyone (Privacy policy): the
/// signed-in nav with a live session, the public nav without one.
#[component]
pub fn OpenFrame() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let signed_in = session.read().as_ref().is_some_and(|s| !s.is_expired());
    if signed_in {
        rsx! { Shell {} }
    } else {
        rsx! { PublicFrame {} }
    }
}

/// A screen under the top nav: no header bar, so the page starts right
/// under the nav. `title` names the screen for assistive tech and the
/// browser tab. The screen scrolls as one column, and its action bar, the
/// last thing in it, sticks to the window's bottom edge at every width. The
/// header's `?` (`on_hint`) rides at the bar's right end. Help is the
/// nav's `Community` link. There is no site footer.
#[component]
pub fn ScreenFrame(
    title: String,
    #[props(default)] on_hint: Option<Callback<()>>,
    children: Element,
) -> Element {
    rsx! {
        document::Title { "{title} | Zwipe" }
        div { class: "screen web-screen web-scroll-screen",
            h1 { class: "web-sr-only", "{title}" }
            {children}
            if let Some(open) = on_hint {
                button {
                    class: "btn-action web-screen-hint",
                    "aria-label": "About this screen",
                    onclick: move |_| open.call(()),
                    "?"
                }
            }
        }
    }
}
