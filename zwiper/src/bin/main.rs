use dioxus::prelude::*;
use zwipe::domain::logo;
use zwiper::{
    config::Config,
    inbound::{components::auth::session_upkeep::spawn_upkeeper, router::Router},
};

const FAVICON: Asset = asset!("/assets/favicon/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

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
        Router::<Router> {}
    }
}
