//! The frames screens draw in.

use components::PageHeader;
use dioxus::prelude::*;

use crate::inbound::router::Route;
use crate::inbound::shell::HelpButton;
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

/// A screen: the app's header with `title`, then `children`. Under the top
/// nav the header's `!` is the nav's `Help`; on a public screen (`help`) the
/// header carries the `!` as in the app.
#[component]
pub fn ScreenFrame(
    title: String,
    #[props(default)] help: bool,
    #[props(default)] on_hint: Option<Callback<()>>,
    children: Element,
) -> Element {
    let leading = if help {
        rsx! { HelpButton {} }
    } else {
        rsx! { span { class: "page-header-corner" } }
    };
    rsx! {
        div { class: "screen web-screen",
            PageHeader { title, leading, on_hint }
            {children}
        }
    }
}
