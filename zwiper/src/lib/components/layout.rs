use crate::routing::Route;
use dioxus::prelude::*;

#[component]
pub fn Layout() -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "main-layout",
            header {
                onclick : move |_| {
                    navigator.push(Route::Home {});
                },
                class: "main-header",
            }
            main { class: "main-content",
                Outlet::<Route> {}
            }
        }
    }
}
