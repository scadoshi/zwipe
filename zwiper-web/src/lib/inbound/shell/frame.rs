//! The frames screens draw in.

use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::router::Route;
use crate::inbound::shell::footer::SiteFooter;
use crate::inbound::shell::top_nav::PublicNav;

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

/// A screen under the top nav: no header bar, so the page starts right
/// under the nav. `title` names the screen for assistive tech and the
/// browser tab. The header's `?` (`on_hint`) moves to the right end of the
/// screen's toolbar, which sits under the nav at every width. The public
/// screens' `!` is the nav's `Help`. The screen
/// scrolls as one column and ends in the footer, which sits at the window's
/// foot on a short screen.
#[component]
pub fn ScreenFrame(
    title: String,
    #[props(default)] on_hint: Option<Callback<()>>,
    children: Element,
) -> Element {
    let signed_in =
        try_use_context::<Signal<Option<Session>>>().is_some_and(|s| s.read().is_some());
    rsx! {
        document::Title { "{title} | Zwipe" }
        div { class: "screen web-screen web-scroll-screen",
            h1 { class: "web-sr-only", "{title}" }
            if let Some(open) = on_hint {
                button {
                    class: "btn-action web-screen-hint",
                    "aria-label": "About this screen",
                    onclick: move |_| open.call(()),
                    "?"
                }
            }
            {children}
            SiteFooter { signed_in }
        }
    }
}
