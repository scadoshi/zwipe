//! The root component: stylesheets, theme, session upkeep, toasts, dialogs
//! and the router.

use components::{
    DialogHostView, ThemeConfig, ToastStack, use_dialog_host, use_overlay_back_stack,
};
use dioxus::prelude::*;
use zwipe_core::domain::logo;
use zwipe_core::domain::user::models::preferences::PipColor;

use crate::inbound::chrome::hint_topic::{HintHost, HintTopic};
use crate::inbound::router::Route;
use crate::inbound::screens::auth::login::LoginPrefill;
use crate::inbound::screens::update_required::UpdateRequired;
use crate::inbound::session::upkeep;
use crate::inbound::shell::MenuOpen;
use crate::inbound::{caches, card_stores, catalog};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const WEB_CSS: Asset = asset!("/assets/web.css");
/// The header's scroll follow at phone width: the web's own copy of the
/// app's `header_hide.js`, keyed to the shell's header and scroller.
const HEADER_HIDE_JS: &str = include_str!("../../../assets/header_hide.js");
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
    caches::use_root_caches();
    catalog::use_catalog_cache();
    card_stores::use_card_stores();
    // Dialogs and sheets register here, so Esc closes the top one.
    let mut overlay_back_stack = use_overlay_back_stack();
    use_context_provider(|| overlay_back_stack);
    use_future(move || async move {
        let mut escapes = document::eval(KEYS_JS);
        while escapes.recv::<bool>().await.is_ok() {
            overlay_back_stack.close_top();
        }
    });
    let dialogs = use_dialog_host();
    use_context_provider(|| dialogs);
    use_context_provider(|| LoginPrefill(Signal::new(None)));
    // The `?` buttons post here and the HintHost draws the topic.
    use_context_provider(|| Signal::new(None::<HintTopic>));
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
        document::Style { {ZWIPER_MAIN_CSS} }
        document::Style { {ZWIPER_ACCORDION_CSS} }
        document::Style { {components::TOAST_CSS} }
        document::Link { rel: "stylesheet", href: WEB_CSS }
        document::Script { {components::SCROLL_FADE_JS} }
        document::Script { {HEADER_HIDE_JS} }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }

        ThemeShell {
            if upgrade_required.required() {
                UpdateRequired {}
            } else {
                ToastStack {
                    default_duration: components::TOAST_NORMAL,
                    Router::<Route> {}
                    HintHost {}
                    DialogHostView {}
                }
            }
        }
    }
}

/// The page-wide keys. Escape closes an open dialog the way a click on its
/// overlay does, so a dialog always goes before the sheet under it however
/// they registered; with no dialog it sends `true` to the root, which closes
/// the top overlay. Enter with a dialog open runs its non-danger primary action, if
/// it has one, and never reaches the page under the dialog; a focused button
/// or link in the dialog answers Enter itself. Enter with nothing focused
/// submits the page's form panel when it has exactly one, as Enter in one
/// of its fields does. With the Themes panel open,
/// the up and down arrows preview the row above or below the selected one,
/// and Enter is `Save` unless a button in the panel has focus.
const KEYS_JS: &str = r#"
document.addEventListener('keydown', (e) => {
    const dialog = document.querySelector('.alert-dialog[data-state="open"]');
    if (e.key === 'Escape') {
        const overlay = document.querySelector('.alert-dialog-overlay[data-state="open"]');
        if (overlay) { overlay.click(); } else { dioxus.send(true); }
        return;
    }
    const focus = document.activeElement;
    const onControl = focus && (focus.tagName === 'BUTTON' || focus.tagName === 'A');
    if (dialog) {
        if (e.key !== 'Enter' || e.isComposing) return;
        if (onControl && dialog.contains(focus)) return;
        e.preventDefault();
        e.stopPropagation();
        const primary = dialog.querySelector('.alert-dialog-actions .alert-dialog-action:not([disabled])');
        if (primary) primary.click();
        return;
    }
    const inField = focus && focus.closest('input, textarea, select, [contenteditable="true"]');
    if (e.key === 'Enter' && !e.isComposing && !onControl && !inField) {
        const forms = Array.from(document.querySelectorAll('.web-form-panel form'))
            .filter((form) => form.offsetParent !== null);
        if (forms.length === 1 && !document.querySelector('.bottom-sheet.show')) {
            e.preventDefault();
            forms[0].requestSubmit();
            return;
        }
    }
    const themes = document.querySelector('.web-theme-sheet .bottom-sheet.show');
    if (!themes) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        const rows = Array.from(themes.querySelectorAll('.pref-row'));
        const at = rows.findIndex((row) => row.classList.contains('selected'));
        const next = rows[e.key === 'ArrowDown' ? Math.min(at + 1, rows.length - 1) : Math.max(at - 1, 0)];
        e.preventDefault();
        if (next && next !== rows[at]) {
            next.click();
            next.scrollIntoView({ block: 'nearest' });
        }
    } else if (e.key === 'Enter' && !(onControl && themes.contains(focus))) {
        const buttons = themes.querySelectorAll('.action-bar button');
        const save = buttons[buttons.length - 1];
        e.preventDefault();
        if (save && !save.disabled) save.click();
    }
}, true);
"#;

/// The themed element every screen sits in. A theme change wipes over it.
#[component]
fn ThemeShell(children: Element) -> Element {
    let follow = components::use_theme_follow(use_context::<Signal<ThemeConfig>>(), ".app-shell");
    use_context_provider(|| follow);
    let class = follow.shown.read().css_class();
    let pips = components::pip_color_class(use_context::<Signal<PipColor>>()());
    rsx! {
        div { class: "app-shell {class} {pips}",
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
