use crate::routing::Route;
use dioxus::prelude::*;
use zwipe::domain::ascii_logo;

#[component]
pub fn Layout() -> Element {
    let navigator = use_navigator();
    let ascii_logo = ascii_logo::logo();

    rsx! {
        div { class: "app-layout",
            header {
                onclick : move |_| {
                    navigator.push(Route::Home {});
                },
                class: "app-header",
                pre { class: "ascii-logo", "{ascii_logo}" }
            }
            main { class: "app-content",
                Outlet::<Route> {}
            }
        }
    }
}
