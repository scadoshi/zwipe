use crate::screens::Screen;
use dioxus::prelude::*;

#[component]
pub fn Layout() -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "main-layout",
            header {
                onclick : move |_| {
                    navigator.push(Screen::Home {});
                },
                class: "main-header",
            }
            main { class: "main-content",
                Outlet::<Screen> {}
            }
        }
    }
}
