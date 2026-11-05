//! zwiper-web: logging, the API client, then the Dioxus launch.

use reqwest::Url;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::logo;
use zwiper_web::config::{APP_VERSION, BACKEND_URL};
use zwiper_web::inbound::app::App;

fn main() {
    #[cfg(target_arch = "wasm32")]
    // A second init fails, and the first one stands.
    let _ = dioxus::logger::init(tracing::Level::INFO);
    // The app's mark, through the logger: on the web stdout goes nowhere and
    // the console, and the dev server's log pane, carry the logger.
    tracing::info!("\n{}", logo::ZWIPER);
    tracing::info!("zwiper-web v{APP_VERSION} starting against {BACKEND_URL}");
    let url: Url = match BACKEND_URL.parse() {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("BACKEND_URL {BACKEND_URL:?} does not parse: {e}");
            return;
        }
    };
    dioxus::LaunchBuilder::new()
        .with_context(ZwipeClient::new(url))
        .launch(App);
}
