//! The keyboard shortcuts on screen: the sheet that lists them, the keys that work everywhere
//! (`?` for the page's hint, Shift+H for the sheet), and the swipe screens' dock button that
//! opens the sheet on a computer.

use components::icons::DockIcon;
use components::{BottomSheet, HintKey, SheetFrame, SheetWide};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::inbound::chrome::shortcuts::{self, GlobalKey};

/// The dock button's icon: a keyboard, its keys and space bar.
const KEYBOARD: &str = "M3 7h18v10H3zM7 10.5h.01M10.5 10.5h.01M14 10.5h.01M17.5 10.5h.01M8 14h8";

/// Whether the shortcuts sheet is open, shared by the frame that draws it and everything that
/// opens it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShortcutsOpen(pub Signal<bool>);

/// Provides [`ShortcutsOpen`] to the routes under it and answers the keys that work everywhere
/// with no hint of their own: Shift+H. Call once, in the frame around every route.
pub fn use_shortcuts_root() -> Signal<bool> {
    let open = use_context_provider(|| ShortcutsOpen(Signal::new(false))).0;
    use_global_keys(None);
    open
}

/// Answers `?` with `on_hint`, the page's own hint as its `?` button opens it, and Shift+H with
/// the shortcuts sheet, while this registration is the newest and the key reaches the screen.
/// A page without a hint passes `None`, and `?` does nothing there.
pub fn use_global_keys(on_hint: Option<Callback<()>>) {
    let open = try_use_context::<ShortcutsOpen>().map(|ShortcutsOpen(open)| open);
    // The page's current hint, which can change between renders.
    let mut hint = use_signal(|| on_hint);
    use_effect(use_reactive!(|on_hint| hint.set(on_hint)));
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = GLOBAL_KEYS_JS
            .replace("__ID__", &id.to_string())
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
                        if let Some(mut open) = open {
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

/// The shortcuts sheet: every key with what it does, in the hint's key style.
#[component]
pub fn ShortcutsSheet(open: Signal<bool>) -> Element {
    rsx! {
        SheetFrame { wide: SheetWide::Modal,
            BottomSheet {
                open,
                title: "Keyboard shortcuts".to_string(),
                closable: true,
                ul { class: "web-shortcuts",
                    for (keys, does) in shortcuts::sheet_rows() {
                        li { key: "{does}", class: "web-shortcut",
                            span { class: "web-shortcut-keys",
                                for label in keys.iter().copied() {
                                    HintKey { key: "{label}", "{label}" }
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
    if ((e.key !== '?' && e.key !== 'H') || !__GUARD__) return;
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
