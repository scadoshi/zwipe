//! The keyboard shortcuts on screen: the sheet that lists them, the keys that open things on
//! every page (`?` the page's hint, Shift+H the sheet, `t` the theme sheet), and the keyboard
//! button in a hint that opens the sheet where a keyboard is likely (a pointer that hovers and is
//! precise, at any width).
//!
//! Components never know a host's screens. A screen offers its keys as a [`ShortcutsList`], and
//! the sheet lists the newest one's.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::shortcuts::{self, GlobalKey, RowPart, SheetGroup};

use crate::hint::KeyGlyph;
use crate::keys::guard;
use crate::media::use_pointer;
use crate::{DialogAction, HintKey, data_action};

/// The hint's keyboard button's icon: a keyboard, its keys and space bar.
const KEYBOARD: &str = "M3 7h18v10H3zM7 10.5h.01M10.5 10.5h.01M14 10.5h.01M17.5 10.5h.01M8 14h8";

/// The keyboard button's name for a screen reader and its tooltip.
pub const SHORTCUTS_LABEL: &str = "Keyboard shortcuts";

/// Whether the shortcuts sheet is open, shared by the frame that draws it and everything that
/// opens it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShortcutsOpen(pub Signal<bool>);

/// The keys one screen lists in the shortcuts sheet, in groups under small headings, the keys
/// every screen answers among them ([`shortcuts::sheet_groups`]), and the line above them, if
/// any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutsList {
    /// The groups, each a heading over its keys and what they do.
    pub groups: Vec<SheetGroup>,
    /// The line the sheet shows above the keys.
    pub note: Option<&'static str>,
    /// Whether the screen's own keys use `j` and `k`, so they don't scroll the page there.
    pub claims_steps: bool,
}

/// The mounted screens that offer the sheet, newest last, which [`use_offer_shortcuts`] keeps:
/// Shift+H opens the sheet only while one is mounted, and the sheet lists the newest one's keys.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShortcutsOffered(pub Signal<Vec<ShortcutsList>>);

/// Every live [`use_global_keys`] registration in mount order, each with whether its page has a
/// hint. The newest answers `?`, so the sheet lists the hint key only while it has one.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PageHints(Signal<Vec<(Uuid, bool)>>);

/// Whether the newest page's `?` opens a hint.
fn newest_has_hint(hints: Option<PageHints>) -> bool {
    hints.is_some_and(|PageHints(stack)| stack.read().last().is_some_and(|(_, hint)| *hint))
}

/// Provides [`ShortcutsOpen`] and [`ShortcutsOffered`] to the routes under it and answers the
/// keys of a page with no hint of its own. Call once, above every route. Every page offers the
/// keys it answers anywhere ([`shortcuts::everywhere_groups`], the theme sheet's among them), so
/// Shift+H and the hint's keyboard button work on every page, and a screen with keys of its own
/// lists those over them. The page keys scroll `scroller` (a selector), else the document.
pub fn use_shortcuts_root(scroller: Option<&'static str>) -> Signal<bool> {
    let open = use_context_provider(|| ShortcutsOpen(Signal::new(false))).0;
    use_context_provider(|| PageHints(Signal::new(Vec::new())));
    use_context_provider(|| {
        ShortcutsOffered(Signal::new(vec![ShortcutsList {
            groups: shortcuts::everywhere_groups(),
            note: None,
            claims_steps: false,
        }]))
    });
    use_global_keys(None);
    use_page_scroll_keys(scroller);
    open
}

/// orangui's page keys under every route: `g` `g` and Shift+G to the top and bottom on every
/// screen, and `j` and `k` a step where the newest offered screen leaves them to the page
/// ([`ShortcutsList::claims_steps`]), in `scroller` (a selector) or the document. `t` and Shift+H stay [`use_global_keys`]'s, and every modal
/// in [`MODAL_SELECTORS`](crate::keys::MODAL_SELECTORS) keeps the keys from the page.
fn use_page_scroll_keys(scroller: Option<&'static str>) {
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    let steps = use_memo(move || {
        offered.is_some_and(|stack| {
            stack
                .read()
                .last()
                .is_some_and(|screen| !screen.claims_steps)
        })
    });
    orangui::use_page_keys(
        orangui::PageKeys {
            themes: false,
            shortcuts: None,
            blocked_by: &guard::MODAL_SELECTORS,
            scroller,
        },
        steps.into(),
    );
}

/// Offers `screen`'s keys while this screen is mounted, so Shift+H and the hint's keyboard button
/// open the shortcuts sheet with them. Elsewhere the sheet lists only the keys every page
/// answers.
pub fn use_offer_shortcuts(screen: ShortcutsList) {
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    let pushed = screen.clone();
    use_hook(move || {
        if let Some(mut stack) = offered {
            stack.write().push(pushed);
        }
    });
    use_drop(move || {
        if let Some(mut stack) = offered {
            let at = stack.peek().iter().rposition(|s| *s == screen);
            if let Some(at) = at {
                stack.write().remove(at);
            }
        }
    });
}

/// Answers `?` with `on_hint`, the page's own hint as its `?` button opens it, Shift+H with the
/// shortcuts sheet while a screen that offers it is mounted ([`use_offer_shortcuts`]), and `t`
/// with the theme sheet, pressing the button marked [`data_action::THEMES`], while this
/// registration is the newest and the key reaches the screen. A page without a hint passes
/// `None`, and `?` does nothing there.
pub fn use_global_keys(on_hint: Option<Callback<()>>) {
    let open = try_use_context::<ShortcutsOpen>().map(|ShortcutsOpen(open)| open);
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    let hints = try_use_context::<PageHints>().map(|PageHints(stack)| stack);
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        if let Some(mut stack) = hints {
            stack.write().push((id, on_hint.is_some()));
        }
    });
    // The page's current hint, which can change between renders.
    let mut hint = use_signal(|| on_hint);
    use_effect(use_reactive!(|on_hint| {
        hint.set(on_hint);
        if let Some(mut stack) = hints {
            let has = on_hint.is_some();
            let entry = stack
                .peek()
                .iter()
                .position(|(at, had)| *at == id && *had != has);
            if let Some(entry) = entry
                && let Some(slot) = stack.write().get_mut(entry)
            {
                slot.1 = has;
            }
        }
    }));
    use_hook(move || {
        let script = GLOBAL_KEYS_JS
            .replace("__KEY__", guard::PRESSED_KEY)
            .replace("__ID__", &id.to_string())
            .replace("__KEYS__", &format!("{:?}", shortcuts::GLOBAL_KEYS))
            .replace("__HINT_KEY__", &format!("'{}'", shortcuts::HINT_KEY))
            .replace("__HINT_GUARD__", &guard::hint_guard_js())
            .replace("__MODAL_HINT__", &guard::modal_hint_js())
            .replace("__GUARD__", &guard::guard_js());
        spawn(async move {
            let mut keys = document::eval(&script);
            while let Ok(key) = keys.recv::<String>().await {
                match shortcuts::global_key(&key) {
                    Some(GlobalKey::Hint) => {
                        let on_hint = *hint.peek();
                        if let Some(on_hint) = on_hint {
                            on_hint.call(());
                        }
                    }
                    Some(GlobalKey::Shortcuts) => {
                        let offered_here = offered.is_some_and(|stack| !stack.peek().is_empty());
                        if let Some(mut open) = open.filter(|_| offered_here) {
                            open.set(true);
                        }
                    }
                    Some(GlobalKey::Themes) => {
                        document::eval(&guard::press_first_js("body", &[data_action::THEMES]));
                    }
                    None => {}
                }
            }
        });
    });
    use_drop(move || {
        if let Some(mut stack) = hints {
            stack.write().retain(|(at, _)| *at != id);
        }
        document::eval(&GLOBAL_KEYS_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// The shortcuts sheet: every key of the newest screen that offers it with what it does, in its
/// groups, in the hint's key style, on orangui's [`ShortcutsSheet`](orangui::ShortcutsSheet),
/// which sits above every screen, closes from its `Got it`, takes focus as it opens, keeps Tab
/// inside and gives focus back as it closes. The hint key's row shows only while the newest page
/// has a hint for `?` to open.
#[component]
pub fn ShortcutsSheet(open: Signal<bool>) -> Element {
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    let hints = try_use_context::<PageHints>();
    // No screen offering the sheet (the one that did has gone): nothing to list.
    let Some(mut screen) = offered.and_then(|stack| stack.read().last().cloned()) else {
        return rsx! {};
    };
    if !newest_has_hint(hints) {
        screen.groups = shortcuts::without_hint(screen.groups);
    }
    rsx! {
        orangui::ShortcutsSheet { open, note: screen.note.map(str::to_owned),
            for group in screen.groups {
                orangui::ShortcutGroup { key: "{group.title}", title: group.title.to_string(),
                    for (at, (keys, does)) in group.rows.into_iter().enumerate() {
                        orangui::ShortcutRow { key: "{at}", keys: keys.iter().map(ToString::to_string).collect::<Vec<_>>(),
                            // An icon button shows its icon beside its name, so the sheet teaches it.
                            for part in shortcuts::row_parts(does) {
                                match part {
                                    RowPart::Icon(icon) => rsx! { HintKey { KeyGlyph { icon } } },
                                    RowPart::Text(words) => rsx! { "{words}" },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The keyboard button a screen's hint offers beside its guide link, which closes the hint and
/// runs `on_click`: only while a screen with keys is `offered` and a `keyboard` is likely (a
/// pointer that hovers and is precise, at any width; not touch).
#[must_use]
pub fn shortcuts_action(
    offered: bool,
    keyboard: bool,
    on_click: Callback<()>,
) -> Option<DialogAction> {
    (offered && keyboard).then(|| DialogAction::Icon {
        label: SHORTCUTS_LABEL.to_owned(),
        path: KEYBOARD,
        on_click,
    })
}

/// The keyboard button for the hint of the screen on top ([`shortcuts_action`]), opening the
/// shortcuts sheet, kept current as screens mount and the window changes.
pub fn use_shortcuts_action() -> Option<DialogAction> {
    let open = try_use_context::<ShortcutsOpen>().map(|ShortcutsOpen(open)| open);
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    let keyboard = use_pointer();
    let on_click = use_callback(move |()| {
        if let Some(mut open) = open {
            open.set(true);
        }
    });
    let offered_here = offered.is_some_and(|stack| !stack.read().is_empty());
    shortcuts_action(offered_here, keyboard(), on_click)
}

/// Registers one page's listener for the keys that work everywhere. Only the newest
/// registration answers, so a page's `?` reaches its own hint.
const GLOBAL_KEYS_JS: &str = r"
window.__zwipeGlobalKeys = window.__zwipeGlobalKeys || [];
window.__zwipeGlobalKeys.push('__ID__');
const onKey = (e) => {
    const stack = window.__zwipeGlobalKeys;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    if (!__KEYS__.includes(__KEY__)) return;
    // `?` opens the topmost open modal's hint; the page's only with no modal open.
    if (__KEY__ === __HINT_KEY__) {
        if (!__HINT_GUARD__) return;
        e.preventDefault();
        if (__MODAL_HINT__) return;
        dioxus.send(__KEY__);
        return;
    }
    if (!__GUARD__) return;
    e.preventDefault();
    dioxus.send(__KEY__);
};
window['__zwipeGlobalKey__ID__'] = onKey;
document.addEventListener('keydown', onKey);
";

/// Removes one page's listener for the keys that work everywhere.
const GLOBAL_KEYS_DROP_JS: &str = r"
const onKey = window['__zwipeGlobalKey__ID__'];
if (onKey) document.removeEventListener('keydown', onKey);
delete window['__zwipeGlobalKey__ID__'];
window.__zwipeGlobalKeys = (window.__zwipeGlobalKeys || []).filter((id) => id !== '__ID__');
";

#[cfg(test)]
mod tests {
    use super::*;

    /// The action for `offered` and `keyboard`, built inside a runtime as a callback needs.
    fn action(offered: bool, keyboard: bool) -> Option<DialogAction> {
        let dom = VirtualDom::new(|| rsx! {});
        dom.in_runtime(|| {
            dom.in_scope(ScopeId::ROOT, || {
                shortcuts_action(offered, keyboard, Callback::new(|()| {}))
            })
        })
    }

    #[test]
    fn the_hint_offers_the_keyboard_button_where_a_keyboard_is_likely_and_a_screen_has_keys() {
        let Some(DialogAction::Icon { label, path, .. }) = action(true, true) else {
            panic!("the keyboard button");
        };
        assert_eq!((label.as_str(), path), (SHORTCUTS_LABEL, KEYBOARD));
        assert!(action(false, true).is_none(), "a screen with no keys");
        assert!(action(true, false).is_none(), "a phone or a narrow window");
    }

    #[test]
    fn question_mark_goes_to_the_topmost_open_modal_before_the_page() {
        let at = GLOBAL_KEYS_JS
            .find("__KEY__ === __HINT_KEY__")
            .expect("the hint key's branch");
        let branch = GLOBAL_KEYS_JS.get(at..).unwrap();
        let modal = branch.find("__MODAL_HINT__").unwrap();
        let page = branch.find("dioxus.send(__KEY__)").unwrap();
        assert!(modal < page, "a modal answers before the page: {branch}");
        let js = guard::modal_hint_js();
        for modal in guard::MODAL_SELECTORS {
            assert!(js.contains(modal), "{modal} in {js}");
        }
        assert!(js.contains("open[open.length - 1]"), "the topmost: {js}");
        assert!(js.contains(guard::MODAL_HINT_BUTTON), "{js}");
    }

    #[test]
    fn the_global_listener_takes_its_keys_from_global_keys() {
        assert!(
            GLOBAL_KEYS_JS.contains("__KEYS__.includes(__KEY__)"),
            "{GLOBAL_KEYS_JS}"
        );
        for key in shortcuts::GLOBAL_KEYS {
            assert!(
                !GLOBAL_KEYS_JS.contains(&format!("'{key}'")),
                "{key} hard-coded"
            );
        }
    }
}
