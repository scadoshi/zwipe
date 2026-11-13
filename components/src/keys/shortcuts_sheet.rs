//! The keyboard shortcuts on screen: the sheet that lists them, the keys that open things on
//! every page (`?` the page's hint, Shift+H the sheet, `t` the theme sheet), and the keyboard
//! button in a hint that opens the sheet on a computer.
//!
//! Components never know a host's screens. A screen offers its keys as a [`ShortcutsList`], and
//! the sheet lists the newest one's.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::shortcuts::{self, GlobalKey, RowPart, SheetRow};

use crate::hint::KeyGlyph;
use crate::keys::guard;
use crate::media::use_computer;
use crate::{BottomSheet, DialogAction, HintKey, SheetFrame, SheetWide, data_action};

/// The hint's keyboard button's icon: a keyboard, its keys and space bar.
const KEYBOARD: &str = "M3 7h18v10H3zM7 10.5h.01M10.5 10.5h.01M14 10.5h.01M17.5 10.5h.01M8 14h8";

/// The keyboard button's name for a screen reader and its tooltip.
pub const SHORTCUTS_LABEL: &str = "Keyboard shortcuts";

/// Whether the shortcuts sheet is open, shared by the frame that draws it and everything that
/// opens it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShortcutsOpen(pub Signal<bool>);

/// The keys one screen lists in the shortcuts sheet: each row's keys and what they do, the
/// screen's own first and then the keys every screen answers, and the line above them, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutsList {
    /// Each key, with its other names, and what it does.
    pub rows: Vec<SheetRow>,
    /// The line the sheet shows above the keys.
    pub note: Option<&'static str>,
}

/// The mounted screens that offer the sheet, newest last, which [`use_offer_shortcuts`] keeps:
/// Shift+H opens the sheet only while one is mounted, and the sheet lists the newest one's keys.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShortcutsOffered(pub Signal<Vec<ShortcutsList>>);

/// Provides [`ShortcutsOpen`] and [`ShortcutsOffered`] to the routes under it and answers the
/// keys of a page with no hint of its own. Call once, above every route. Every page offers the
/// keys it answers anywhere ([`shortcuts::EVERYWHERE_ROWS`], the theme sheet's among them), so
/// Shift+H and the hint's keyboard button work on every page, and a screen with keys of its own
/// lists those over them.
pub fn use_shortcuts_root() -> Signal<bool> {
    let open = use_context_provider(|| ShortcutsOpen(Signal::new(false))).0;
    use_context_provider(|| {
        ShortcutsOffered(Signal::new(vec![ShortcutsList {
            rows: shortcuts::EVERYWHERE_ROWS.to_vec(),
            note: None,
        }]))
    });
    use_global_keys(None);
    open
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
    // The page's current hint, which can change between renders.
    let mut hint = use_signal(|| on_hint);
    use_effect(use_reactive!(|on_hint| hint.set(on_hint)));
    let id = use_hook(Uuid::new_v4);
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
        document::eval(&GLOBAL_KEYS_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// The shortcuts sheet: every key of the newest screen that offers it with what it does, in the
/// hint's key style. It sits above every screen, a full-window one included.
#[component]
pub fn ShortcutsSheet(open: Signal<bool>) -> Element {
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    // No screen offering the sheet (the one that did has gone): nothing to list.
    let Some(screen) = offered.and_then(|stack| stack.read().last().cloned()) else {
        return rsx! {};
    };
    rsx! {
        div { class: "web-shortcuts-layer",
        SheetFrame { wide: SheetWide::Modal,
            BottomSheet {
                open,
                title: SHORTCUTS_LABEL.to_string(),
                closable: true,
                if let Some(note) = screen.note {
                    p { class: "web-shortcuts-note", "{note}" }
                }
                ul { class: "web-shortcuts",
                    for (at, (keys, does)) in screen.rows.into_iter().enumerate() {
                        li { key: "{at}", class: "web-shortcut",
                            span { class: "web-shortcut-keys",
                                for (i, label) in keys.iter().copied().enumerate() {
                                    span { key: "{label}",
                                        if i > 0 {
                                            " "
                                        }
                                        HintKey { "{label}" }
                                    }
                                }
                            }
                            // An icon button shows its icon beside its name, so the sheet teaches it.
                            span { class: "web-shortcut-does",
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
    }
}

/// The keyboard button a screen's hint offers beside its guide link, which closes the hint and
/// runs `on_click`: only while a screen with keys is `offered` and on a `computer`'s wide layout.
#[must_use]
pub fn shortcuts_action(
    offered: bool,
    computer: bool,
    on_click: Callback<()>,
) -> Option<DialogAction> {
    (offered && computer).then(|| DialogAction::Icon {
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
    let computer = use_computer();
    let on_click = use_callback(move |()| {
        if let Some(mut open) = open {
            open.set(true);
        }
    });
    let offered_here = offered.is_some_and(|stack| !stack.read().is_empty());
    shortcuts_action(offered_here, computer(), on_click)
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

    /// The action for `offered` and `computer`, built inside a runtime as a callback needs.
    fn action(offered: bool, computer: bool) -> Option<DialogAction> {
        let dom = VirtualDom::new(|| rsx! {});
        dom.in_runtime(|| {
            dom.in_scope(ScopeId::ROOT, || {
                shortcuts_action(offered, computer, Callback::new(|()| {}))
            })
        })
    }

    #[test]
    fn the_hint_offers_the_keyboard_button_on_a_computer_where_a_screen_has_keys() {
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
