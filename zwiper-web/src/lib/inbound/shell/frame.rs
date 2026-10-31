//! The frames screens draw in.

use components::PageHeader;
use dioxus::prelude::*;

use crate::inbound::router::Route;
use crate::inbound::shell::HelpButton;

/// The layout over the public routes: no sidebar, the screen centered.
#[component]
pub fn PublicFrame() -> Element {
    rsx! {
        div { class: "web-public",
            Outlet::<Route> {}
        }
    }
}

/// A screen: the app's header with `title`, then `children`. In the sidebar
/// shell the header's `!` is the sidebar's `Help`; on a public screen
/// (`help`) the header carries it as in the app.
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
