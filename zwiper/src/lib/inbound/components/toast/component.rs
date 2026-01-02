use dioxus::prelude::*;
use dioxus_primitives::toast::{self, ToastProviderProps};

#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
    rsx! {
        // Context: the AlertDialog element could not load its .css with this method for some
        // reason.. After some time spent debugging, I decided to just load in
        // zwiper/src/bin/main.rs for now and attend to later. All other elements with their own
        // style sheets will get the same treatment to keep consistency for now..
        //
        //document::Link { rel: "stylesheet", href: asset!("/assets/toast.css") }
        toast::ToastProvider {
            default_duration: props.default_duration,
            max_toasts: props.max_toasts,
            render_toast: props.render_toast,
            {props.children}
        }
    }
}
