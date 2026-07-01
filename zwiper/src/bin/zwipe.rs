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

// Anti-FOUC boot styling for the native (iOS/Android) WebView. Our stylesheets
// are injected at render time (the document::Link tags in App), so without this
// the WebView paints a white background, then unstyled HTML, before the CSS
// lands. BOOT_BG is the default theme's --bg-primary (gruvbox-dark #282828); it
// is set on the WebView itself, before any HTML paints. BOOT_HEAD is injected
// into the static <head> before first render: it repeats the dark background as
// a fallback and hides #main until main.css applies (main.css flips it back to
// opacity:1), so the app never paints unstyled.
const BOOT_BG: (u8, u8, u8, u8) = (0x28, 0x28, 0x28, 0xff);
const BOOT_HEAD: &str = "<style>html,body{background-color:#282828;}#main{opacity:0;}</style>";

fn main() {
    logo::Zwiper::print();
    let config = Config::from_env();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&config.rust_log))
        .init();
    tracing::info!("zwiper v{} starting", env!("CARGO_PKG_VERSION"));
    launch_app();
}

// Native shells (iOS/Android/desktop) render into a WebView we can configure to
// kill the load flash; web has no such config, so it falls back to the plain
// launch. dioxus::mobile and dioxus::desktop are both re-exports of the same
// dioxus-desktop crate, feature-gated by which platform this build targets.
#[cfg(any(feature = "desktop", feature = "mobile"))]
fn launch_app() {
    #[cfg(feature = "desktop")]
    use dioxus::desktop::Config as WebviewConfig;
    #[cfg(all(feature = "mobile", not(feature = "desktop")))]
    use dioxus::mobile::Config as WebviewConfig;

    dioxus::LaunchBuilder::new()
        .with_cfg(
            WebviewConfig::new()
                .with_background_color(BOOT_BG)
                .with_custom_head(BOOT_HEAD.to_string()),
        )
        .launch(App);
}

#[cfg(not(any(feature = "desktop", feature = "mobile")))]
fn launch_app() {
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
            if upgrade_required.required() {
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
