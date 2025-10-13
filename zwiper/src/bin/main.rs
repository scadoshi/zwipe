use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;
use zwiper::{config::Config, helpers::development::Spoof, screens::Screen};

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
    let session: Signal<Option<Session>> = use_signal(|| {
        // use this later when actual persistence is achieved
        // Session::infallible_load()
        // for now we can just spoof it
        Some(Session::spoof())
    });
    use_context_provider(|| session);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Screen> {}
    }
}
