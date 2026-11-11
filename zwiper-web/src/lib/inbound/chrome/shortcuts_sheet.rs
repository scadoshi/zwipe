//! The keyboard shortcuts on screen: the sheet that lists them, the keys that open things (`?`
//! the page's hint everywhere, Shift+H the sheet on the swipe screens and the deck cards screen),
//! and the swipe screens' dock button that opens the sheet on a computer.

use components::icons::DockIcon;
use components::{BottomSheet, HintKey, SheetFrame, SheetWide};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::inbound::chrome::shortcuts::{self, GlobalKey, ShortcutsScreen};

/// The dock button's icon: a keyboard, its keys and space bar.
const KEYBOARD: &str = "M3 7h18v10H3zM7 10.5h.01M10.5 10.5h.01M14 10.5h.01M17.5 10.5h.01M8 14h8";

/// Whether the shortcuts sheet is open, shared by the frame that draws it and everything that
/// opens it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShortcutsOpen(pub Signal<bool>);

/// The mounted screens that offer the sheet, newest last, which [`use_offer_shortcuts`] keeps:
/// Shift+H opens the sheet only while one is mounted, and the sheet lists the newest one's keys.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShortcutsOffered(pub Signal<Vec<ShortcutsScreen>>);

/// Provides [`ShortcutsOpen`] and [`ShortcutsOffered`] to the routes under it and answers the
/// keys of a page with no hint of its own. Call once, in the frame around every route.
pub fn use_shortcuts_root() -> Signal<bool> {
    let open = use_context_provider(|| ShortcutsOpen(Signal::new(false))).0;
    use_context_provider(|| ShortcutsOffered(Signal::new(Vec::new())));
    use_global_keys(None);
    open
}

/// Marks this screen as `screen` while it is mounted, so Shift+H opens the shortcuts sheet with
/// that screen's keys, a swipe screen's swipes named as its dock names them. Elsewhere Shift+H
/// does nothing.
pub fn use_offer_shortcuts(screen: ShortcutsScreen) {
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    use_hook(move || {
        if let Some(mut stack) = offered {
            stack.write().push(screen);
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

/// Answers `?` with `on_hint`, the page's own hint as its `?` button opens it, and Shift+H with
/// the shortcuts sheet while a screen that offers it is mounted ([`use_offer_shortcuts`]), while this
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
            .replace("__ID__", &id.to_string())
            .replace("__KEYS__", &format!("{:?}", shortcuts::GLOBAL_KEYS))
            .replace("__GUARD__", &shortcuts::guard_js());
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
                    None => {}
                }
            }
        });
    });
    use_drop(move || {
        document::eval(&GLOBAL_KEYS_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// The shortcuts sheet: every key of the screen on top with what it does, in the hint's key
/// style, a swipe screen's swipes named as it names them. It sits above every screen, the
/// full-window Commander swipe included.
#[component]
pub fn ShortcutsSheet(open: Signal<bool>) -> Element {
    let offered = try_use_context::<ShortcutsOffered>().map(|ShortcutsOffered(stack)| stack);
    // No screen offering the sheet (the one that did has gone): nothing to list.
    let Some(screen) = offered.and_then(|stack| stack.read().last().copied()) else {
        return rsx! {};
    };
    rsx! {
        div { class: "web-shortcuts-layer",
        SheetFrame { wide: SheetWide::Modal,
            BottomSheet {
                open,
                title: "Keyboard shortcuts".to_string(),
                closable: true,
                if let Some(note) = shortcuts::sheet_note(&screen) {
                    p { class: "web-shortcuts-note", "{note}" }
                }
                ul { class: "web-shortcuts",
                    for (keys, does) in shortcuts::sheet_rows(&screen) {
                        li { key: "{does}", class: "web-shortcut",
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
                            span { class: "web-shortcut-does", "{does}" }
                        }
                    }
                }
            }
        }
    }
    }
}

/// The swipe screens' dock button that opens the shortcuts sheet, shown only on a computer
/// (a mouse or trackpad that hovers).
#[component]
pub fn DockShortcuts() -> Element {
    let open = try_use_context::<ShortcutsOpen>().map(|ShortcutsOpen(open)| open);
    rsx! {
        button {
            class: "action-dock-btn web-dock-shortcuts",
            "aria-label": "Keyboard shortcuts",
            title: "Keyboard shortcuts",
            onclick: move |_| {
                if let Some(mut open) = open {
                    open.set(true);
                }
            },
            DockIcon { path: KEYBOARD }
        }
    }
}

/// Registers one page's listener for the keys that work everywhere. Only the newest
/// registration answers, so a page's `?` reaches its own hint.
const GLOBAL_KEYS_JS: &str = r"
window.__zwipeGlobalKeys = window.__zwipeGlobalKeys || [];
window.__zwipeGlobalKeys.push('__ID__');
const onKey = (e) => {
    const stack = window.__zwipeGlobalKeys;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    if (!__KEYS__.includes(e.key) || !__GUARD__) return;
    e.preventDefault();
    dioxus.send(e.key);
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

    #[test]
    fn the_global_listener_takes_its_keys_from_global_keys() {
        assert!(
            GLOBAL_KEYS_JS.contains("__KEYS__.includes(e.key)"),
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
