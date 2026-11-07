//! The zwiper app: logging, crash capture, then the Dioxus launch.

use components::{DialogHostView, ThemeConfig, ToastStack, use_dialog_host};
use dioxus::prelude::*;
use tracing_subscriber::EnvFilter;
use zwipe_core::domain::logo;
use zwipe_core::domain::user::settings::PipStyle;
use zwiper::config::Config;
use zwiper::inbound::components::auth::session_upkeep;
use zwiper::inbound::components::hint_host::HintHost;
use zwiper::inbound::components::navigation::leave_guard::GuardedHistory;
use zwiper::inbound::components::navigation::menu::MenuOpen;
use zwiper::inbound::components::update_required::UpdateRequired;
use zwiper::inbound::router::Router;

const FAVICON: Asset = asset!("/assets/favicon/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
/// Slides each screen's header away as its content scrolls down and back as it scrolls up.
const HEADER_HIDE_JS: &str = include_str!("../../assets/header_hide.js");

// Self-hosted JetBrains Mono. Registered via asset!() (not a CSS url(), which
// manganis does not bundle) so the woff2 files ship in the app bundle. The full
// font includes U+2580–U+259F block elements at the monospace advance width,
// fontsource's subsets drop them, so the ASCII logo's block glyphs otherwise
// fall back to a misaligned symbol font on Android WebView.
const FONT_JBM_400: Asset = asset!("/assets/fonts/jetbrains-mono-400.woff2");
const FONT_JBM_500: Asset = asset!("/assets/fonts/jetbrains-mono-500.woff2");
const FONT_JBM_700: Asset = asset!("/assets/fonts/jetbrains-mono-700.woff2");

fn main() -> anyhow::Result<()> {
    logo::Zwiper::print();
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&config.rust_log))
        .init();
    tracing::info!("zwiper v{} starting", env!("CARGO_PKG_VERSION"));
    // Crash capture: a panic writes its report to disk; the next launch posts
    // it (session_upkeep) and clears the file on a 2xx. Native-only.
    #[cfg(not(target_arch = "wasm32"))]
    zwiper::outbound::crash_store::install_panic_hook();
    launch_app(config);
    Ok(())
}

// Native shells (iOS/Android/desktop) render into a WebView that paints dark
// and keeps #main hidden until the styles are in (App mounts BootReveal last
// among its head elements); web has no such config, so it falls back to the
// plain launch. dioxus::mobile and dioxus::desktop are both re-exports of the same
// dioxus-desktop crate, feature-gated by which platform this build targets.
// Either way the config rides in as root context for `spawn_upkeeper`.
#[cfg(any(feature = "desktop", feature = "mobile"))]
fn launch_app(config: Config) {
    dioxus::LaunchBuilder::new()
        .with_context(config)
        .with_cfg(webview_config())
        .launch(App);
}

/// The webview's launch config: dark before any HTML paints, `#main` hidden
/// until the styles are in. A rebuilt window takes the same.
#[cfg(feature = "desktop")]
fn webview_config() -> dioxus::desktop::Config {
    dioxus::desktop::Config::new()
        .with_background_color(components::BOOT_BG)
        .with_custom_head(components::BOOT_HEAD.to_string())
}

/// The webview's launch config: dark before any HTML paints, `#main` hidden
/// until the styles are in. A rebuilt window takes the same.
#[cfg(all(feature = "mobile", not(feature = "desktop")))]
fn webview_config() -> dioxus::mobile::Config {
    dioxus::mobile::Config::new()
        .with_background_color(components::BOOT_BG)
        .with_custom_head(components::BOOT_HEAD.to_string())
}

/// Rebuilds the app in a new window when iOS killed its page, then closes the
/// dead one. The new window comes first, since closing the last window ends
/// the app.
#[cfg(any(feature = "desktop", feature = "mobile"))]
fn use_rebuild() -> Callback<()> {
    #[cfg(feature = "desktop")]
    use dioxus::desktop::window;
    #[cfg(all(feature = "mobile", not(feature = "desktop")))]
    use dioxus::mobile::window;

    let config: Config = use_context();
    use_callback(move |()| {
        tracing::warn!("the page was killed; rebuilding the app in a new window");
        let dead = window();
        let dom = VirtualDom::new(App).with_root_context(config.clone());
        let _ = dead.new_window(dom, webview_config());
        spawn(async move {
            tokio::time::sleep(REBUILD_HANDOVER).await;
            dead.close();
        });
    })
}

/// No windows to rebuild in a browser.
#[cfg(not(any(feature = "desktop", feature = "mobile")))]
fn use_rebuild() -> Callback<()> {
    use_callback(|()| {})
}

/// How long the dead window stays up behind the new one before it closes.
#[cfg(any(feature = "desktop", feature = "mobile"))]
const REBUILD_HANDOVER: std::time::Duration = std::time::Duration::from_secs(2);

#[cfg(not(any(feature = "desktop", feature = "mobile")))]
fn launch_app(config: Config) {
    dioxus::LaunchBuilder::new()
        .with_context(config)
        .launch(App);
}

#[component]
fn ThemeWrapper(children: Element) -> Element {
    // Whatever sets the theme (the Themes sheet, the dark-mode toggle, a
    // session's settings), the shell wipes over to it.
    let follow = components::use_theme_follow(use_context::<Signal<ThemeConfig>>(), ".app-shell");
    // The Themes sheet reads it to leave inside the wipe that restores a
    // discarded pick.
    use_context_provider(|| follow);
    let class = follow.shown.read().css_class();
    // The page under the shell carries the theme class too, so the strip the
    // WebView shows past the shell (a rubber band, the gap over the keyboard)
    // is the theme's page color and not the launch color.
    let shown = follow.shown;
    use_effect(move || {
        let class = shown.read().css_class();
        document::eval(&format!("document.documentElement.className = {class:?};"));
    });
    let pips = components::pip_style_class(use_context::<Signal<PipStyle>>()());
    rsx! {
        div { class: "app-shell {class} {pips}",
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
    // Unclaimed link clicks open in the in-app browser, not Safari.
    components::use_in_app_links();

    // The router's history, with the guard that asks before a screen with
    // unsaved changes is left.
    use_hook(GuardedHistory::provide);

    // A page iOS killed comes back in a rebuilt window.
    components::use_page_kill_recovery(
        "Zwipe",
        zwiper::inbound::components::navigation::back_handler::webview,
        use_rebuild(),
    );
    // Before the upkeep, which reacts to each return to the foreground.
    zwiper::inbound::components::lifecycle::use_lifecycle_root();
    let upgrade_required = session_upkeep::spawn_upkeeper();
    let dialogs = use_dialog_host();
    use_context_provider(|| dialogs);
    // Closed until the splash starts to fade; the entrances wait on it.
    use_context_provider(|| components::SplashGate(Signal::new(false)));
    // The main menu's open state, above the router so it carries across
    // screens.
    use_context_provider(|| MenuOpen(Signal::new(false)));
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
        document::Style { {components::SCREENS_CSS} }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Style { {components::FILTER_CSS} }
        document::Style { {components::TOAST_CSS} }
        // The drawer the card details open in.
        document::Style { {components::FILTER_CSS} }
        // Drives the edge fades on sideways scrollers where CSS scroll
        // timelines are missing (the dictionary's letter rail).
        document::Script { {components::SCROLL_FADE_JS} }
        document::Script { {HEADER_HIDE_JS} }
        document::Script { {components::KEYBOARD_INSET_JS} }
        document::Script { {components::KEYBOARD_JS} }
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
