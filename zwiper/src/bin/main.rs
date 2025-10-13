use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;
use zwiper::{config::Config, screens::Screen, session::Persist};

const FAVICON: Asset = asset!("/assets/favicon/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    let config = Config::from_env();
    tracing_subscriber::fmt()
        .with_max_level(config.rust_log)
        .init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let session: Signal<Option<Session>> = use_signal(|| Session::infallible_load());
    use_context_provider(|| session);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Screen> {}
    }
}
