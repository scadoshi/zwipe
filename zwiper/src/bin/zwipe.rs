use dioxus::prelude::*;
use dioxus_primitives::toast::ToastProvider;
use tracing_subscriber::EnvFilter;
use zwipe_core::domain::logo;
use zwipe_core::domain::user::models::theme::ThemeConfig;
use zwiper::{
    config::Config,
    inbound::{
        components::{auth::session_upkeep::spawn_upkeeper, update_required::UpdateRequired},
        router::Router,
    },
};

const FAVICON: Asset = asset!("/assets/favicon/favicon.ico");
const THEMES_CSS: Asset = asset!("/assets/themes.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const ACCORDION_CSS: Asset = asset!("/assets/accordion.css");
const ALERT_DIALOG_CSS: Asset = asset!("/assets/alert-dialog.css");
const TOAST_CSS: Asset = asset!("/assets/toast.css");

// Self-hosted JetBrains Mono. Registered via asset!() (not a CSS url(), which
// manganis does not bundle) so the woff2 files ship in the app bundle. The full
// font includes U+2580–U+259F block elements at the monospace advance width —
// fontsource's subsets drop them, so the ASCII logo's block glyphs otherwise
// fall back to a misaligned symbol font on Android WebView.
const FONT_JBM_400: Asset = asset!("/assets/fonts/jetbrains-mono-400.woff2");
const FONT_JBM_500: Asset = asset!("/assets/fonts/jetbrains-mono-500.woff2");
const FONT_JBM_700: Asset = asset!("/assets/fonts/jetbrains-mono-700.woff2");

fn main() {
    logo::Zwiper::print();
    let config = Config::from_env();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&config.rust_log))
        .init();
    tracing::info!("zwiper v{} starting", env!("CARGO_PKG_VERSION"));
    dioxus::launch(App);
}

#[component]
fn ThemeWrapper(children: Element) -> Element {
    let theme: Signal<ThemeConfig> = use_context();
    let class = theme.read().css_class();
    rsx! {
        div { class: "app-shell {class}",
            {children}
        }
    }
}

#[component]
fn App() -> Element {
    let upgrade_required = spawn_upkeeper();
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Style {
            {format!(
                "@font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:400;font-display:swap;src:url({FONT_JBM_400}) format('woff2');}}\
                 @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:500;font-display:swap;src:url({FONT_JBM_500}) format('woff2');}}\
                 @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:700;font-display:swap;src:url({FONT_JBM_700}) format('woff2');}}"
            )}
        }
        document::Link { rel: "stylesheet", href: THEMES_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: ACCORDION_CSS }
        document::Link { rel: "stylesheet", href: ALERT_DIALOG_CSS }
        document::Link { rel: "stylesheet", href: TOAST_CSS }
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover"
        }

        ThemeWrapper {
            // Min-version gate: a build below the server minimum gets the
            // blocking update screen instead of the app. No dismiss.
            // TEMP (revert): `true ||` forces the update screen for visual review.
            if true || upgrade_required.required() {
                UpdateRequired {}
            } else {
                ToastProvider {
                    max_toasts: 3_usize,
                    class: "toast-container",
                    Router::<Router> {}
                }
            }
        }
    }
}
