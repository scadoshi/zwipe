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
use zwipe_ui::deck_cards::cursor::MOVE_KEYS;

use crate::inbound::chrome::fresh_build::{NewBuildNotice, use_new_build_watch};
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
/// Keyboard mode: a card cursor key hides the pointer and lets it through the cards until the
/// mouse moves, with its key list and text fields filled in by [`keyboard_pointer_js`].
const KEYBOARD_POINTER_JS: &str = include_str!("../../../assets/keyboard_pointer.js");
/// The keys besides the cursor's moves that start keyboard mode: the first and last card.
const KEYBOARD_POINTER_JUMPS: [&str; 2] = ["Home", "End"];

/// Every element a screen scrolls in: the screen's column and any layer over it (a deck's
/// Cards). The page keys scroll the topmost one shown.
const PAGE_SCROLLERS: &str = ".web-scroll-screen";

/// The app root. Launch it with a [`ZwipeClient`](zwipe_client::ZwipeClient)
/// in the root context.
#[component]
pub fn App() -> Element {
    let upgrade_required = upkeep::use_session_upkeep();
    telemetry::use_telemetry(use_context(), use_context());
    caches::use_root_caches();
    catalog::use_catalog_cache();
    card_stores::use_card_stores();
    // A page that stays put hears of a newer build.
    use_new_build_watch();
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
    let esc_back = use_context_provider(|| EscBack(Signal::new(None)));
    let esc_keeps = use_context_provider(|| EscKeeps(Signal::new(None)));
    use_future(move || async move {
        let mut escapes = document::eval(KEYS_JS);
        while escapes.recv::<bool>().await.is_ok() {
            let closed = closes_on_escape(*esc_keeps.0.peek(), overlay_back_stack.depth())
                && overlay_back_stack.close_top();
            if let Some(back) = back_after_escape(closed, *esc_back.0.peek()) {
                back.call(());
            }
        }
    });
    use_hook(|| {
        document::eval(&hint_focus_js());
        document::eval(DIALOG_FOCUS_JS);
        document::eval(FIND_KEYS_JS);
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
    // The screen scrolls in its own column under the nav, not the document; a layer over it
    // (a deck's Cards) scrolls on its own, and the topmost shown one takes the page keys.
    let shortcuts_open = use_shortcuts_root(Some(PAGE_SCROLLERS));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        document::Link { rel: "manifest", href: MANIFEST }
        document::Script { {components::SCROLL_FADE_JS} }
        document::Script { {HEADER_HIDE_JS} }
        document::Script { {keyboard_pointer_js()} }
        document::Script { {components::KEYBOARD_JS} }
        // Enter runs the open sheet's main action.
        document::Script { {components::keys::SHEET_ENTER_JS} }
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
                    NewBuildNotice {}
                }
            }
        }
    }
}

/// Where focus goes around a hint (a dialog drawing hint lines), installed
/// once. As a hint opens it takes
/// focus on its first button, so Enter answers the hint rather than the `?`
/// under it. As it closes focus returns to where it was opened from: the
/// card the deck cursor held, else the open drawer, else the form or panel it
/// sits in, else the page's main column; never the nav, and never the `?`,
/// which Enter would open again.
/// A sheet that opened as the hint closed keeps focus for itself.
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
            open.querySelector('.alert-dialog-actions > button')?.focus({ preventScroll: true });
        } else if (!open && opener) {
            const from = opener;
            opener = null;
            // A sheet the hint opened (the shortcuts sheet from its keyboard button) takes focus itself.
            const opened = [...document.querySelectorAll('.bottom-sheet.show, .bottom-sheet-premount')].some((s) => !s.contains(from));
            if (opened) return;
            const stop = from.isConnected && from.closest && from.closest('__STOPS__');
            if (stop) { stop.focus({ preventScroll: true }); return; }
            const inNav = from.closest('.web-app-nav, nav');
            const target = from.closest('.bottom-sheet.show')
                || (!inNav && from.closest('form, .web-form-panel, .panel-card, .settings-list'))
                || document.querySelector('.web-main');
            place(target);
        }
    }).observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['data-state'] });
}
"#;

/// [`HINT_FOCUS_JS`] with the deck cursor's stops filled in.
/// [`KEYBOARD_POINTER_JS`] with the card cursor's keys and the tags a key types into.
fn keyboard_pointer_js() -> String {
    let quoted = |items: &[&str]| {
        let list = items
            .iter()
            .map(|item| format!("'{item}'"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{list}]")
    };
    let keys: Vec<&str> = MOVE_KEYS
        .iter()
        .chain(KEYBOARD_POINTER_JUMPS.iter())
        .copied()
        .collect();
    KEYBOARD_POINTER_JS
        .replace("__KEYS__", &quoted(&keys))
        .replace("__TYPING__", &quoted(&components::keys::TYPING_TAGS))
}

fn hint_focus_js() -> String {
    HINT_FOCUS_JS.replace("__STOPS__", components::deck_cursor::STOP_SELECTOR)
}

/// Where focus goes around every other dialog (a hint has its own,
/// [`HINT_FOCUS_JS`], and the leave guard's its own Stay), installed once. As one opens it takes focus on its
/// first field (Clone's name, its text selected), else its primary action (a non-danger
/// confirm, such as Share's Create link), else its first link (Buy's
/// stores), else its Cancel, so a danger dialog never starts on its danger
/// button. As it closes, once it has faded out of the page, focus returns
/// to the element it was opened from, while that is still in the page and
/// nothing else took focus.
const DIALOG_FOCUS_JS: &str = r#"
if (!window.__zwipeDialogFocus) {
    window.__zwipeDialogFocus = true;
    let pressed = null;
    let opener = null;
    document.addEventListener('pointerdown', (e) => { pressed = e.target; }, true);
    const TEXT = 'input:not([type]), input[type="text"], input[type="search"], textarea';
    const FIRST = ['input:not([type="hidden"]):not([disabled]), textarea:not([disabled]), select:not([disabled])', '.alert-dialog-actions .alert-dialog-action:not([disabled])', '.alert-dialog-actions a[href]', '.alert-dialog-actions .alert-dialog-cancel:not([disabled])'];
    new MutationObserver(() => {
        const open = document.querySelector('.alert-dialog[data-state="open"]');
        if (open && !open.matches('.web-leave-dialog') && !open.querySelector('.hint-body, .hint-line, .hint-bullets')) {
            if (opener) return;
            const active = document.activeElement;
            opener = active && active !== document.body ? active : (pressed || document.body);
            if (open.contains(active)) return;
            const target = FIRST.map((sel) => open.querySelector(sel)).find(Boolean);
            if (target) {
                target.focus({ preventScroll: true });
                // A prefilled text field (Clone's name) selects its text, so typing replaces it.
                if (target.matches(TEXT) && target.select) target.select();
            }
        } else if (!open && opener) {
            const from = opener;
            opener = null;
            // The closed dialog stays in the page, hidden, with focus on its button: that, or the
            // page, hands focus back. Another dialog open by now keeps it.
            if (!from.isConnected || from === document.body) return;
            const now = document.activeElement;
            const left = !now || now === document.body || !now.isConnected
                || !!(now.closest && now.closest('.alert-dialog:not([data-state="open"])'));
            if (left) from.focus({ preventScroll: true });
        }
    }).observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['data-state'] });
}
"#;

/// What the screen's Back button does, which Esc does with nothing open: the
/// nav's back arrow sets it on a deck's Add, Remove and Edit, and leaves it
/// empty everywhere else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EscBack(pub Signal<Option<Callback<()>>>);

/// The back-stack layer Esc leaves open, by its z-index while it is open: the
/// deck's Cards on a phone, a screen of its own rather than a sheet over one.
/// The Cards layer sets it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EscKeeps(pub Signal<Option<u32>>);

/// Whether Esc closes the top of a back stack `depth` layers deep: always,
/// unless that top is the layer [`EscKeeps`] holds at z-index `kept`.
fn closes_on_escape(kept: Option<u32>, depth: usize) -> bool {
    let Some(top) = depth.checked_sub(1) else {
        return false;
    };
    kept != Some(components::overlay_z(top))
}

/// The Back Esc runs after closing what it could: `back` when `closed` is
/// false, so Esc closes one layer at a time and goes back only from a bare
/// screen.
fn back_after_escape<T>(closed: bool, back: Option<T>) -> Option<T> {
    if closed { None } else { back }
}

/// The page-wide keys. While a dialog is open the kit's own listener runs
/// Escape and Enter, and the kit's theme sheet handles its arrows and
/// Enter, so this one steps aside. Otherwise Escape in a text field only
/// leaves the field; anywhere else it sends `true` to the root, which closes
/// the top overlay unless it is the deck's Cards layer ([`EscKeeps`]), or,
/// with nothing to close on a deck's Add, Remove or Edit, goes back as the
/// screen's Back button does. Enter with nothing focused submits the page's form when it has exactly
/// one, a form in a panel or the deck form around its panels, and no picker, sheet or modal is
/// open over it, as Enter in one of its fields does.
///
/// Escape is marked taken, so no later listener closes the same overlay
/// again. With a dialog open it is taken too: a help closing over a sheet
/// hands focus back to the sheet within the same key, and the kit's sheet
/// Esc would close the sheet as well. The root's send waits until the key's
/// dispatch is over, so a dialog the close opens (a picker asking before it
/// discards) draws after the kit's listener has passed on this same key and
/// doesn't take it as a dismiss.
const KEYS_JS: &str = r#"
document.addEventListener('keydown', (e) => {
    if (document.querySelector('.alert-dialog[data-state="open"]')) {
        if (e.key === 'Escape') e.preventDefault();
        return;
    }
    const focus = document.activeElement;
    const inField = focus && focus.closest('input, textarea, select, [contenteditable="true"]');
    if (e.key === 'Escape') {
        e.preventDefault();
        if (inField) {
            focus.blur();
            return;
        }
        setTimeout(() => dioxus.send(true), 0);
        return;
    }
    const onControl = focus && (focus.tagName === 'BUTTON' || focus.tagName === 'A');
    if (e.key === 'Enter' && !e.isComposing && !onControl && !inField) {
        const forms = Array.from(document.querySelectorAll('.web-form-panel form, form:has(.web-form-panel)'))
            .filter((form) => form.offsetParent !== null);
        if (forms.length === 1 && !document.querySelector('.alert-dialog[data-state="open"], .bottom-sheet.show, .screen.web-large-modal, .swipe-select-screen.show')) {
            e.preventDefault();
            forms[0].requestSubmit();
        }
    }
}, true);
"#;

/// Holds back the browser's default for its quick find keys, `/` and Firefox's links-only `'`,
/// while focus sits outside a text field and no modifier is held, so the find bar never opens over
/// the app. The key still travels on to the app's own listeners, so `/` opens the filter wherever
/// it does.
const FIND_KEYS_JS: &str = r#"window.addEventListener('keydown', (e) => {
    if (e.key !== '/' && e.key !== "'") return;
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const focus = document.activeElement;
    if (focus && focus.closest && focus.closest('input, textarea, select, [contenteditable]:not([contenteditable="false"])')) return;
    e.preventDefault();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dialog_opens_on_its_first_field_else_its_primary_else_a_link_else_cancel() {
        let order = [
            "input:not([type=\"hidden\"])",
            ".alert-dialog-actions .alert-dialog-action:not([disabled])",
            ".alert-dialog-actions a[href]",
            ".alert-dialog-actions .alert-dialog-cancel:not([disabled])",
        ];
        let at: Vec<usize> = order
            .iter()
            .map(|sel| DIALOG_FOCUS_JS.find(sel).unwrap_or(usize::MAX))
            .collect();
        assert!(at.iter().all(|i| *i != usize::MAX), "{DIALOG_FOCUS_JS}");
        assert!(at.windows(2).all(|w| w[0] < w[1]), "in that order: {at:?}");
        assert!(
            DIALOG_FOCUS_JS
                .contains("!open.querySelector('.hint-body, .hint-line, .hint-bullets')"),
            "a hint keeps its own focus"
        );
        assert!(
            DIALOG_FOCUS_JS.contains("if (!from.isConnected || from === document.body) return;"),
            "closing hands focus back to the opener"
        );
        assert!(
            DIALOG_FOCUS_JS.contains("now.closest('.alert-dialog:not([data-state=\"open\"])')"),
            "from the closed dialog, which stays in the page"
        );
    }

    #[test]
    fn the_page_keys_reach_a_layer_over_the_screen_as_well_as_the_screen() {
        assert_eq!(
            PAGE_SCROLLERS, ".web-scroll-screen",
            "the deck's Cards layer is one too"
        );
    }

    #[test]
    fn escape_is_taken_once_the_root_closes_the_top_overlay_with_it() {
        // Unmarked, the kit's last-chance Esc (a sheet's own close) closes the same overlay again,
        // and on a history-backed layer (the deck's Cards) that is a second step back.
        assert!(
            KEYS_JS.contains("if (e.key === 'Escape') {\n        e.preventDefault();"),
            "{KEYS_JS}"
        );
    }

    #[test]
    fn esc_that_closes_a_help_over_a_sheet_leaves_the_sheet_open() {
        // The help closes and hands focus back to the sheet within the same key, so the kit's
        // sheet Esc, which skips a taken key, would otherwise close the sheet too.
        let guard = KEYS_JS
            .split("if (document.querySelector('.alert-dialog[data-state=\"open\"]')) {")
            .nth(1)
            .and_then(|rest| rest.split("return;").next())
            .unwrap_or_default();
        assert!(
            guard.contains("if (e.key === 'Escape') e.preventDefault();"),
            "with a dialog open, Escape is marked taken before the root steps aside: {KEYS_JS}"
        );
    }

    #[test]
    fn enter_with_nothing_focused_finds_the_deck_form_around_its_panels() {
        let fields = include_str!("screens/deck/form/fields.rs");
        let deck_form = fields
            .split_once("form {\n            class: \"container-sm web-deck-form\",")
            .map_or("", |(_, rest)| rest);
        assert!(
            deck_form.contains("div { class: \"web-form-panel\","),
            "the deck form holds its panels, where every other form sits in one"
        );
        assert!(
            KEYS_JS.contains(
                "document.querySelectorAll('.web-form-panel form, form:has(.web-form-panel)')"
            ),
            "a form in a panel and a form around panels: {KEYS_JS}"
        );
    }

    #[test]
    fn enter_with_nothing_focused_waits_while_any_picker_or_sheet_is_open() {
        // A phone-width picker is a screen over the deck form, which stays laid out behind it.
        let modals = components::keys::MODAL_SELECTORS.join(", ");
        assert!(
            KEYS_JS.contains(&format!(
                "if (forms.length === 1 && !document.querySelector('{modals}')) {{"
            )),
            "{modals}: {KEYS_JS}"
        );
    }

    #[test]
    fn esc_in_quick_add_or_any_field_only_leaves_the_field() {
        let esc = KEYS_JS
            .split("if (e.key === 'Escape') {")
            .nth(1)
            .unwrap_or_default();
        let blur =
            esc.find("if (inField) {\n            focus.blur();\n            return;\n        }");
        let send = esc.find("dioxus.send(true)");
        assert!(
            blur.is_some_and(|b| send.is_some_and(|s| b < s)),
            "a focused field blurs and the key goes no further: {KEYS_JS}"
        );
        assert!(
            KEYS_JS.contains("const inField = focus && focus.closest('input, textarea, select, [contenteditable=\"true\"]');"),
            "Quick add is an input: {KEYS_JS}"
        );
    }

    #[test]
    fn esc_closes_the_menu_and_the_image_preview_but_leaves_the_cards_layer() {
        let cards = Some(components::overlay_z(0));
        assert!(
            !closes_on_escape(cards, 1),
            "the Cards layer alone on the stack stays"
        );
        assert!(
            closes_on_escape(cards, 2),
            "the phone menu or a card's image preview over the Cards layer closes"
        );
        assert!(
            closes_on_escape(None, 1),
            "the phone menu or the image preview with no Cards layer closes"
        );
        assert!(
            closes_on_escape(Some(components::overlay_z(1)), 3),
            "a layer over the Cards layer, which sits second, closes"
        );
        assert!(!closes_on_escape(None, 0), "nothing to close");
    }

    #[test]
    fn esc_goes_back_only_when_nothing_closed() {
        assert_eq!(
            back_after_escape(/* closed */ true, Some(1)),
            None,
            "an overlay closed"
        );
        assert_eq!(
            back_after_escape(false, Some(1)),
            Some(1),
            "nothing was open"
        );
        assert_eq!(
            back_after_escape::<u8>(false, None),
            None,
            "a screen with no Back"
        );
    }

    #[test]
    fn the_browser_s_quick_find_keys_never_open_its_find_bar_outside_a_text_field() {
        assert!(
            FIND_KEYS_JS.starts_with("window.addEventListener('keydown',"),
            "on window, ahead of every document listener: {FIND_KEYS_JS}"
        );
        assert!(
            FIND_KEYS_JS.trim_end().ends_with("}, true);"),
            "in the capture phase: {FIND_KEYS_JS}"
        );
        for key in ["'/'", "\"'\""] {
            assert!(FIND_KEYS_JS.contains(key), "{key}: {FIND_KEYS_JS}");
        }
        assert!(
            FIND_KEYS_JS.contains("e.ctrlKey || e.metaKey || e.altKey"),
            "a combo keeps the browser's default: {FIND_KEYS_JS}"
        );
        assert!(
            FIND_KEYS_JS.contains(
                "input, textarea, select, [contenteditable]:not([contenteditable=\"false\"])"
            ),
            "a text field still types the key: {FIND_KEYS_JS}"
        );
        assert!(
            !FIND_KEYS_JS.contains("stopPropagation")
                && !FIND_KEYS_JS.contains("stopImmediatePropagation"),
            "the app's own listeners still hear the key: {FIND_KEYS_JS}"
        );
    }

    #[test]
    fn a_closed_dialog_hands_focus_back_to_its_opener_though_it_stays_in_the_page() {
        assert!(
            DIALOG_FOCUS_JS.contains("now.closest('.alert-dialog:not([data-state=\"open\"])')"),
            "focus left in the closed dialog goes back to the opener: {DIALOG_FOCUS_JS}"
        );
    }

    #[test]
    fn a_dialog_s_text_field_takes_focus_with_its_text_selected() {
        assert!(
            DIALOG_FOCUS_JS.contains("if (target.matches(TEXT) && target.select) target.select();"),
            "typing replaces the prefilled name rather than adding to it: {DIALOG_FOCUS_JS}"
        );
    }

    #[test]
    fn a_header_pushed_up_keeps_its_closed_sheets_out_of_the_page() {
        // translateY makes the header the containing block of the fixed theme drawer inside it,
        // whose layout then overflows the window: a scrollbar, a narrower page, rewrapped text.
        assert!(
            HEADER_HIDE_JS.contains("nav.style.overflow = pushed ? \"clip\" : \"\";"),
            "{HEADER_HIDE_JS}"
        );
    }

    #[test]
    fn every_card_cursor_key_starts_keyboard_mode_and_only_a_real_mouse_move_ends_it() {
        let script = keyboard_pointer_js();
        for key in MOVE_KEYS.iter().chain(KEYBOARD_POINTER_JUMPS.iter()) {
            assert!(script.contains(&format!("'{key}'")), "{key}: {script}");
        }
        assert!(
            !script.contains("__KEYS__") && !script.contains("__TYPING__"),
            "{script}"
        );
        // The move Chrome sends a still mouse when the page scrolls under it has no distance.
        assert!(
            script.contains("if (e.movementX || e.movementY) mouse();"),
            "{script}"
        );
        assert!(
            script.contains("root.classList.add('keyboard-nav');"),
            "{script}"
        );
    }

    #[test]
    fn keyboard_mode_hides_the_pointer_and_lets_it_through_the_cards() {
        let css = include_str!("../../../assets/web.css");
        assert!(
            css.contains(
                ":root.keyboard-nav,\n:root.keyboard-nav * {\n    cursor: none !important;"
            ),
            "the pointer hides"
        );
        for card in components::deck_cursor::STOP_SELECTOR.split(", ") {
            let class = card.split(['[', ':', ' ']).next().unwrap_or_default();
            assert!(
                css.lines()
                    .any(|line| line.starts_with(":root.keyboard-nav :is(") && line.contains(class)),
                "{class} lets the pointer through"
            );
        }
    }

    #[test]
    fn a_closing_hint_hands_focus_back_to_the_card_the_cursor_held() {
        let script = hint_focus_js();
        assert!(
            script.contains(components::deck_cursor::STOP_SELECTOR),
            "{script}"
        );
        assert!(
            script.contains("const stop = from.isConnected && from.closest && from.closest("),
            "{script}"
        );
        assert!(!script.contains("__STOPS__"), "{script}");
    }

    #[test]
    fn an_opening_hint_takes_focus_without_scrolling_from_its_top() {
        assert!(
            HINT_FOCUS_JS.contains(
                "open.querySelector('.alert-dialog-actions > button')?.focus({ preventScroll: true });"
            ),
            "a hint taller than the window opens at its title: {HINT_FOCUS_JS}"
        );
    }

    #[test]
    fn a_closing_hint_leaves_focus_to_a_sheet_it_opened() {
        assert!(
            HINT_FOCUS_JS.contains(
                "[...document.querySelectorAll('.bottom-sheet.show, .bottom-sheet-premount')].some((s) => !s.contains(from))"
            ),
            "{HINT_FOCUS_JS}"
        );
        assert!(
            HINT_FOCUS_JS.contains("if (opened) return;"),
            "{HINT_FOCUS_JS}"
        );
    }
}
