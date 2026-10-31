//! The sidebar shell over every authed route.

use dioxus::prelude::*;
use zwipe_core::domain::logo;

use crate::inbound::router::Route;
use crate::inbound::shell::HelpDialog;
use crate::inbound::shell::nav::Section;

/// The sidebar beside the matched screen. Below the narrow breakpoint the
/// sidebar folds into a top bar whose `Menu` button opens it over the
/// screen.
#[component]
pub fn Shell() -> Element {
    let route: Route = use_route();
    let current = Section::of(&route);
    let mut help = use_signal(|| false);
    let mut menu_open = use_signal(|| false);

    // A new route closes the folded menu.
    use_effect(use_reactive!(|route| {
        let _ = route;
        menu_open.set(false);
    }));

    rsx! {
        div { class: "web-shell",
            div { class: "web-topbar",
                button {
                    class: "btn-action",
                    "aria-expanded": "{menu_open}",
                    onclick: move |_| menu_open.toggle(),
                    "Menu"
                }
                span { class: "web-topbar-mark", "Zwipe" }
            }
            nav {
                class: if menu_open() { "web-sidebar open" } else { "web-sidebar" },
                "aria-label": "Main",
                div { class: "web-sidebar-mark logo", "{logo::Z}" }
                ul {
                    for section in Section::ALL {
                        li { key: "{section.label()}",
                            Link {
                                to: section.route(),
                                class: if current == Some(section) { "web-nav-link active" } else { "web-nav-link" },
                                "{section.label()}"
                            }
                        }
                    }
                }
                button {
                    class: "web-nav-link web-nav-help",
                    onclick: move |_| help.set(true),
                    "Help"
                }
            }
            main { class: "web-main",
                Outlet::<Route> {}
            }
        }
        HelpDialog { open: help }
    }
}
