use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;
use zwiper::{
    config::Config,
    domain::development::Spoof,
    inbound::ui::Router,
    outbound::{client::auth::AuthClient, session::Persist},
};

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
        // use this later when actual pesrsistence is achieved
        // Session::infallible_load()
        // for now we can just spoof it
        Some(Session::spoof())
    });
    use_context_provider(|| session);

    let auth_client: Signal<AuthClient> = use_signal(|| AuthClient::new());
    use_context_provider(|| auth_client);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<Router> {}
    }
}
