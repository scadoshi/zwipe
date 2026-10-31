//! The root component: stylesheets, theme, session upkeep, toasts, dialogs
//! and the router.

use components::{
    DialogHostView, ThemeConfig, ToastStack, use_dialog_host, use_overlay_back_stack,
};
use dioxus::prelude::*;
use zwipe_core::domain::user::models::preferences::PipStyle;

use crate::inbound::router::Route;
use crate::inbound::screens::update_required::UpdateRequired;
use crate::inbound::session::upkeep;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const WEB_CSS: Asset = asset!("/assets/web.css");
const FONT_JBM_400: Asset = asset!("/assets/fonts/jetbrains-mono-400.woff2");
const FONT_JBM_500: Asset = asset!("/assets/fonts/jetbrains-mono-500.woff2");
const FONT_JBM_700: Asset = asset!("/assets/fonts/jetbrains-mono-700.woff2");

/// zwiper's own stylesheets, shared as they are. `web.css` loads after them
/// and holds only what a browser window changes.
const ZWIPER_MAIN_CSS: &str = include_str!("../../../../zwiper/assets/main.css");
const ZWIPER_ACCORDION_CSS: &str = include_str!("../../../../zwiper/assets/accordion.css");

/// The app root. Launch it with a [`ZwipeClient`](zwipe_client::ZwipeClient)
/// in the root context.
#[component]
pub fn App() -> Element {
    let upgrade_required = upkeep::use_session_upkeep();
    // Dialogs and sheets register here, so Esc closes the top one.
    let mut overlay_back_stack = use_overlay_back_stack();
    use_context_provider(|| overlay_back_stack);
    use_future(move || async move {
        let mut escapes = document::eval(ESCAPE_JS);
        while escapes.recv::<bool>().await.is_ok() {
            overlay_back_stack.close_top();
        }
    });
    let dialogs = use_dialog_host();
    use_context_provider(|| dialogs);
    // No splash on the web, so the entrances run at once.
    use_context_provider(|| components::SplashGate(Signal::new(true)));

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
        document::Style { {ZWIPER_MAIN_CSS} }
        document::Style { {ZWIPER_ACCORDION_CSS} }
        document::Style { {components::TOAST_CSS} }
        document::Link { rel: "stylesheet", href: WEB_CSS }
        document::Script { {components::SCROLL_FADE_JS} }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }

        ThemeShell {
            if upgrade_required.required() {
                UpdateRequired {}
            } else {
                ToastStack {
                    default_duration: components::TOAST_NORMAL,
                    Router::<Route> {}
                    DialogHostView {}
                }
            }
        }
    }
}

/// Sends `true` to the root each time Escape goes down anywhere on the
/// page.
const ESCAPE_JS: &str = "document.addEventListener('keydown', (e) => { if (e.key === 'Escape') { dioxus.send(true); } });";

/// The themed element every screen sits in. A theme change wipes over it.
#[component]
fn ThemeShell(children: Element) -> Element {
    let follow = components::use_theme_follow(use_context::<Signal<ThemeConfig>>(), ".app-shell");
    use_context_provider(|| follow);
    let class = follow.shown.read().css_class();
    let pips = components::pip_style_class(use_context::<Signal<PipStyle>>()());
    rsx! {
        div { class: "app-shell {class} {pips}",
            components::GridCells { still: true }
            {children}
        }
    }
}
