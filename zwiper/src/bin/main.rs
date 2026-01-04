use dioxus::prelude::*;
use dioxus_primitives::toast::ToastProvider;
use zwipe::domain::logo;
use zwiper::{
    config::Config,
    inbound::{components::auth::session_upkeep::spawn_upkeeper, router::Router},
};

const FAVICON: Asset = asset!("/assets/favicon/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const ACCORDION_CSS: Asset = asset!("/assets/accordion.css");
const ALERT_DIALOG_CSS: Asset = asset!("/assets/alert-dialog.css");
const TOAST_CSS: Asset = asset!("/assets/toast.css");

fn main() {
    logo::Zwiper::print();
    let config = Config::from_env();
    tracing_subscriber::fmt()
        .with_max_level(config.rust_log)
        .init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    spawn_upkeeper();
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: ACCORDION_CSS }
        document::Link { rel: "stylesheet", href: ALERT_DIALOG_CSS }
        document::Link { rel: "stylesheet", href: TOAST_CSS }
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover"
        }

        ToastProvider {
            max_toasts: 3_usize,
            Router::<Router> {}
        }
    }
}
