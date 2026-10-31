//! The frames screens draw in.

use components::PageHeader;
use dioxus::prelude::*;

use crate::inbound::router::Route;
use crate::inbound::shell::HelpButton;
use crate::inbound::shell::sidebar::ShellMenu;

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
/// shell the header's `!` is the sidebar's `Help`, and below the narrow
/// breakpoint its left corner holds the `Menu` button that opens the folded
/// sidebar. On a public screen (`help`) the header carries the `!` as in the
/// app.
#[component]
pub fn ScreenFrame(
    title: String,
    #[props(default)] help: bool,
    #[props(default)] on_hint: Option<Callback<()>>,
    children: Element,
) -> Element {
    let menu = try_use_context::<ShellMenu>();
    let leading = match (help, menu) {
        (true, _) => rsx! { HelpButton {} },
        (false, Some(ShellMenu(mut open))) => rsx! {
            button {
                class: "btn-action page-header-support web-header-menu",
                "aria-expanded": "{open}",
                onclick: move |_| open.toggle(),
                "Menu"
            }
        },
        (false, None) => rsx! { span { class: "page-header-corner" } },
    };
    rsx! {
        div { class: "screen web-screen",
            PageHeader { title, leading, on_hint }
            {children}
        }
    }
}
