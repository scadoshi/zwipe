use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let mut username_or_email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    rsx! {
        div {
            "Login"
        }
    }
}
