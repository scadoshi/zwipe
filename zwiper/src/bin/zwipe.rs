use components::{DialogHostView, ThemeConfig, ToastStack, use_dialog_host};
use dioxus::prelude::*;
use tracing_subscriber::EnvFilter;
use zwipe_core::domain::logo;
use zwiper::{
    config::Config,
    inbound::{
        components::{
            auth::session_upkeep::spawn_upkeeper, hint_host::HintHost,
            update_required::UpdateRequired,
        },
        router::Router,
    },
};

const FAVICON: Asset = asset!("/assets/favicon/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const ACCORDION_CSS: Asset = asset!("/assets/accordion.css");

// Self-hosted JetBrains Mono. Registered via asset!() (not a CSS url(), which
// manganis does not bundle) so the woff2 files ship in the app bundle. The full
// font includes U+2580–U+259F block elements at the monospace advance width,
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
    // Crash capture: a panic writes its report to disk; the next launch posts
    // it (session_upkeep) and clears the file on a 2xx. Native-only.
    #[cfg(not(target_arch = "wasm32"))]
    zwiper::outbound::crash_store::install_panic_hook();
    launch_app();
}

// Native shells (iOS/Android/desktop) render into a WebView that paints dark
// and keeps #main hidden until the styles are in (App mounts BootReveal last
// among its head elements); web has no such config, so it falls back to the
// plain launch. dioxus::mobile and dioxus::desktop are both re-exports of the same
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
                .with_background_color(components::BOOT_BG)
                .with_custom_head(components::BOOT_HEAD.to_string()),
        )
        .launch(App);
}

#[cfg(not(any(feature = "desktop", feature = "mobile")))]
fn launch_app() {
    dioxus::launch(App);
}

#[component]
fn ThemeWrapper(children: Element) -> Element {
    // Whatever sets the theme (the Themes sheet, the dark-mode toggle, a
    // session's preferences), the shell wipes over to it.
    let follow = components::use_theme_follow(use_context::<Signal<ThemeConfig>>(), ".app-shell");
    // The Themes sheet reads it to leave inside the wipe that restores a
    // discarded pick.
    use_context_provider(|| follow);
    let class = follow.shown.read().css_class();
    rsx! {
        div { class: "app-shell {class}",
            // In place from the start; the splash is what rains.
            components::GridCells { still: true }
            // The opening: the field rains in under the mark, then fades to
            // the first screen while it mounts underneath.
            components::Splash {
                div { class: "logo", components::Decode { text: logo::ZWIPE } }
            }
            {children}
        }
    }
}

#[component]
fn App() -> Element {
    // Not in main(): UIApplication does not exist until the app has launched,
    // so calling this earlier would silently do nothing.
    use_hook(zwiper::outbound::shake_to_edit::disable);

    let upgrade_required = spawn_upkeeper();
    let dialogs = use_dialog_host();
    use_context_provider(|| dialogs);
    // Closed until the splash starts to fade; the entrances wait on it.
    use_context_provider(|| components::SplashGate(Signal::new(false)));
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Style {
            {format!(
                "@font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:400;font-display:swap;src:url({FONT_JBM_400}) format('woff2');}}\
                 @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:500;font-display:swap;src:url({FONT_JBM_500}) format('woff2');}}\
                 @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:700;font-display:swap;src:url({FONT_JBM_700}) format('woff2');}}"
            )}
        }
        document::Style { {components::THEMES_CSS} }
        document::Style { {components::COMPONENTS_CSS} }
        document::Style { {components::MAGIC_CSS} }
        document::Style { {components::APP_CSS} }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: ACCORDION_CSS }
        document::Style { {components::TOAST_CSS} }
        // Drives the edge fades on sideways scrollers where CSS scroll
        // timelines are missing (the dictionary's letter rail).
        document::Script { {components::SCROLL_FADE_JS} }
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover"
        }
        // Last of the head elements: shows #main, which the launch config
        // hides, once every stylesheet above is in.
        components::BootReveal {}

        ThemeWrapper {
            // Min-version gate: a build below the server minimum gets the
            // blocking update screen instead of the app. No dismiss.
            if upgrade_required.required() {
                UpdateRequired {}
            } else {
                ToastStack {
                    default_duration: components::TOAST_NORMAL,
                    Router::<Router> {}
                    // App-root receiver for on-demand "?" hints.
                    HintHost {}
                    // Every dialog draws here, beside the router, so its
                    // overlay escapes each screen's containing block.
                    DialogHostView {}
                }
            }
        }
    }
}
