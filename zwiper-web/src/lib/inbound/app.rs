//! The root component: icons, scripts, theme, session upkeep, toasts,
//! dialogs and the router. The stylesheets are linked in `index.html`'s head
//! at build ([`sheets`](crate::inbound::sheets)).

use std::time::Duration;

use components::{
    DialogHostView, ThemeConfig, ToastStack, use_dialog_host, use_overlay_back_stack,
};
use dioxus::prelude::*;
use zwipe_core::domain::logo;
use zwipe_core::domain::user::models::settings::PipStyle;

use crate::inbound::chrome::hint_topic::{HintHost, HintTopic, InfoButton};
use crate::inbound::chrome::leave_guard::{self, GuardedHistory, RouteOverlays};
use crate::inbound::chrome::overlay_steps::StepMove;
use crate::inbound::chrome::shortcuts_sheet::{ShortcutsSheet, use_shortcuts_root};
use crate::inbound::chrome::splash;
use crate::inbound::chrome::trail::TrailHistory;
use crate::inbound::router::Route;
use crate::inbound::screens::auth::login::LoginPrefill;
use crate::inbound::screens::update_required::UpdateRequired;
use crate::inbound::session::upkeep;
use crate::inbound::shell::{DeckOpener, MenuOpen, use_footer_lift};
use crate::inbound::{caches, card_stores, catalog, telemetry};
use crate::outbound::details_store::DetailsStore;
use crate::outbound::{browser, timer};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const FAVICON_16: Asset = asset!("/assets/favicon-16x16.png");
const FAVICON_32: Asset = asset!("/assets/favicon-32x32.png");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/icon-180.png");
/// How long the overlay stack holds still before the browser's history follows it.
const OVERLAY_SETTLE: Duration = Duration::from_millis(50);
/// The install manifest; its 192 and 512 icons sit unhashed in `public/icons`.
const MANIFEST: Asset = asset!("/assets/site.webmanifest");
/// The header's scroll follow at phone width: the web's own copy of the
/// app's `header_hide.js`, keyed to the shell's header and scroller.
const HEADER_HIDE_JS: &str = include_str!("../../../assets/header_hide.js");

/// The app root. Launch it with a [`ZwipeClient`](zwipe_client::ZwipeClient)
/// in the root context.
#[component]
pub fn App() -> Element {
    let upgrade_required = upkeep::use_session_upkeep();
    telemetry::use_telemetry(use_context(), use_context());
    caches::use_root_caches();
    catalog::use_catalog_cache();
    card_stores::use_card_stores();
    // Dialogs and sheets register here, so Esc closes the top one.
    let mut overlay_back_stack = use_overlay_back_stack();
    use_context_provider(|| overlay_back_stack);
    leave_guard::watch_overlays(overlay_back_stack);
    // Each open overlay owns a browser history entry, so the browser's back closes it, but for
    // one its route already stands for. The history follows the stack once it settles, so an
    // overlay that leaves and comes straight back, as one does while it draws again, moves
    // nothing.
    let route_overlays = use_context_provider(|| RouteOverlays(Signal::new(0)));
    use_effect(move || {
        let _ = overlay_back_stack.depth();
        let _ = route_overlays.0();
        spawn(async move {
            timer::sleep(OVERLAY_SETTLE).await;
            let owning = overlay_back_stack
                .depth()
                .saturating_sub(*route_overlays.0.peek());
            match leave_guard::follow_overlays(owning) {
                Some(StepMove::Close(n)) => {
                    for _ in 0..n {
                        overlay_back_stack.close_top();
                    }
                }
                Some(step) => {
                    if let Some(script) = step.script() {
                        document::eval(&script);
                    }
                }
                None => {}
            }
        });
    });
    use_future(move || async move {
        let mut escapes = document::eval(KEYS_JS);
        while escapes.recv::<bool>().await.is_ok() {
            overlay_back_stack.close_top();
        }
    });
    use_hook(|| {
        document::eval(HINT_FOCUS_JS);
    });
    // A swipe screen's card slides clear of its docked details panel.
    components::use_details_clear();
    // The swipe surfaces' eye keeps its last choice in this browser's storage.
    use_context_provider(|| components::DetailsEyeHost {
        window_size: Callback::new(|()| browser::window_size()),
        load: Callback::new(|()| DetailsStore::browser().load()),
        save: Callback::new(|shown: bool| DetailsStore::browser().save(shown)),
    });
    // The deck page's stat panels open the web's hints from their `?`.
    use_context_provider(|| {
        components::StatHints(Callback::new(|topic: components::StatTopic| {
            rsx! { InfoButton { topic: stat_hint_topic(topic) } }
        }))
    });
    let dialogs = use_dialog_host();
    use_context_provider(|| dialogs);
    use_context_provider(|| LoginPrefill(Signal::new(None)));
    // The `?` buttons post here and the HintHost draws the topic.
    use_context_provider(|| Signal::new(None::<HintTopic>));
    // The splash plays once a tab session. While it plays the gate is
    // closed until it starts to fade, and the entrances wait on it; a load
    // that skips it opens the gate at once.
    let splash = use_hook(splash::splash_plays_this_load);
    use_context_provider(|| components::SplashGate(Signal::new(!splash)));
    // The main menu's open state, above the router so it carries across
    // screens.
    use_context_provider(|| MenuOpen(Signal::new(false)));
    // Which screen opened the deck on screen, for the deck's back arrow.
    DeckOpener::use_provider();
    use_footer_lift();
    // The router's history, with the leave guard in front of it and the
    // app's trail over both.
    use_hook(GuardedHistory::provide);
    use_hook(TrailHistory::provide);
    // A card drawn without its image shrinks its text to fit its box.
    components::use_card_proxy_fit();
    // Lists and groups marked to settle ease into place instead of snapping.
    components::use_settle();
    // The keyboard shortcuts sheet, over every route, the auth screens included, opened by
    // Shift+H, a hint's keyboard button or a swipe screen's dock button.
    let shortcuts_open = use_shortcuts_root();

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        document::Link { rel: "manifest", href: MANIFEST }
        document::Script { {components::SCROLL_FADE_JS} }
        document::Script { {HEADER_HIDE_JS} }
        document::Script { {components::KEYBOARD_JS} }
        // A guide's framed screenshot fades in once it can paint.
        document::Script { {components::MEDIA_FRAME_JS} }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }

        ThemeShell { splash,
            if upgrade_required.required() {
                UpdateRequired {}
            } else {
                ToastStack {
                    default_duration: components::TOAST_NORMAL,
                    Router::<Route> {}
                    ShortcutsSheet { open: shortcuts_open }
                    HintHost {}
                    DialogHostView {}
                }
            }
        }
    }
}

/// Where focus goes around a hint (a dialog drawing hint lines), installed
/// once. As a hint opens it takes
/// focus on its first button, so Enter answers the hint rather than the `?`
/// under it. As it closes focus returns to where it was opened from: the
/// open drawer, else the form or panel it sits in, else the page's main
/// column; never the nav, and never the `?`, which Enter would open again.
/// The opener is the focused element, or the last thing pressed where a
/// click does not focus a button (Safari).
const HINT_FOCUS_JS: &str = r#"
if (!window.__zwipeHintFocus) {
    window.__zwipeHintFocus = true;
    let pressed = null;
    let opener = null;
    document.addEventListener('pointerdown', (e) => { pressed = e.target; }, true);
    const place = (el) => {
        if (!el) return;
        if (!el.hasAttribute('tabindex')) el.setAttribute('tabindex', '-1');
        el.focus({ preventScroll: true });
    };
    new MutationObserver(() => {
        const open = document.querySelector('.alert-dialog[data-state="open"]');
        if (open && open.querySelector('.hint-body, .hint-line, .hint-bullets')) {
            if (opener) return;
            const active = document.activeElement;
            opener = active && active !== document.body ? active : (pressed || document.body);
            open.querySelector('.alert-dialog-actions > button')?.focus();
        } else if (!open && opener) {
            const from = opener;
            opener = null;
            const inNav = from.closest('.web-app-nav, nav');
            const target = from.closest('.bottom-sheet.show')
                || (!inNav && from.closest('form, .web-form-panel, .panel-card, .settings-list'))
                || document.querySelector('.web-main');
            place(target);
        }
    }).observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['data-state'] });
}
"#;

/// The page-wide keys. While a dialog is open the kit's own listener runs
/// Escape and Enter, and the kit's theme sheet handles its arrows and
/// Enter, so this one steps aside. Otherwise Escape sends `true` to the
/// root, which closes the top overlay, and Enter with nothing focused
/// submits the page's form panel when it has exactly one, as Enter in one
/// of its fields does.
///
/// Escape sends once the key's dispatch is over, so a dialog the close
/// opens (a picker asking before it discards) draws after the kit's
/// listener has passed on this same key and doesn't take it as a dismiss.
const KEYS_JS: &str = r#"
document.addEventListener('keydown', (e) => {
    if (document.querySelector('.alert-dialog[data-state="open"]')) return;
    if (e.key === 'Escape') {
        setTimeout(() => dioxus.send(true), 0);
        return;
    }
    const focus = document.activeElement;
    const onControl = focus && (focus.tagName === 'BUTTON' || focus.tagName === 'A');
    const inField = focus && focus.closest('input, textarea, select, [contenteditable="true"]');
    if (e.key === 'Enter' && !e.isComposing && !onControl && !inField) {
        const forms = Array.from(document.querySelectorAll('.web-form-panel form'))
            .filter((form) => form.offsetParent !== null);
        if (forms.length === 1 && !document.querySelector('.bottom-sheet.show')) {
            e.preventDefault();
            forms[0].requestSubmit();
        }
    }
}, true);
"#;

/// Copies the shell's page color and sink onto html as `--page-edge` and
/// `--page-sink`, which html and body paint: html sits above the themed
/// shell, and themes.css's `:root` defaults outweigh a theme class there, so
/// it reads the theme's colors from the shell.
const ROOT_THEME_JS: &str = "(() => { const shell = document.querySelector('.app-shell'); if (!shell) return; const css = getComputedStyle(shell); const root = document.documentElement.style; root.setProperty('--page-edge', css.getPropertyValue('--bg-primary').trim()); root.setProperty('--page-sink', css.getPropertyValue('--bg-sink').trim()); })()";

/// The themed element every screen sits in. A theme change wipes over it.
/// `splash` opens it on the splash.
#[component]
fn ThemeShell(splash: bool, children: Element) -> Element {
    let follow = components::use_theme_follow(use_context::<Signal<ThemeConfig>>(), ".app-shell");
    use_context_provider(|| follow);
    let class = follow.shown.read().css_class();
    use_effect(move || {
        let _shown = follow.shown.read();
        document::eval(ROOT_THEME_JS);
    });
    let pips = components::pip_style_class(use_context::<Signal<PipStyle>>()());
    rsx! {
        div { class: "app-shell {class} {pips}",
            components::GridCells { still: true }
            // The opening: the field rains in under the mark, then fades to
            // the first screen while it mounts underneath.
            if splash {
                components::Splash {
                    div { class: "logo", components::Decode { text: logo::ZWIPE } }
                }
            }
            {children}
        }
    }
}

/// The web's hint for a stat panel's `?`.
fn stat_hint_topic(topic: components::StatTopic) -> HintTopic {
    match topic {
        components::StatTopic::DeckTags => HintTopic::DeckViewDeckTags,
        components::StatTopic::OracleTags => HintTopic::DeckViewOracleTags,
        components::StatTopic::TypeChart => HintTopic::TypeChart,
        components::StatTopic::RoleChart => HintTopic::RoleChart,
        components::StatTopic::OracleTagChart => HintTopic::OracleTagChart,
        components::StatTopic::ManaCurve => HintTopic::ManaCurve,
        components::StatTopic::ColorChart => HintTopic::ColorChart,
        components::StatTopic::ManaFulfillment => HintTopic::ManaFulfillment,
        components::StatTopic::DrawOdds => HintTopic::DrawOdds,
    }
}
