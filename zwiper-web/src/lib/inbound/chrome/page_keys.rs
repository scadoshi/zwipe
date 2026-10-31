//! Single keys a screen answers while no text field has focus, such as `/`
//! to focus its search.
//!
//! Screens and overlays stack: the newest one registered takes the keys
//! until it unmounts. No key reaches a screen while a dialog or a sheet is
//! open, or with Ctrl, Alt or Meta held.

use dioxus::prelude::*;
use uuid::Uuid;

/// Calls `on_key` with each key (`KeyboardEvent.key`, such as `"/"` or
/// `"a"`) pressed while this registration is the newest and no text field,
/// dialog or sheet has focus. The browser's default for the key is kept
/// unless `on_key` is registered for `/`, which never types into the page.
pub fn use_page_keys(on_key: Callback<String>) {
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = PAGE_KEYS_JS.replace("__ID__", &id.to_string());
        spawn(async move {
            let mut keys = document::eval(&script);
            while let Ok(key) = keys.recv::<String>().await {
                on_key.call(key);
            }
        });
    });
    use_drop(move || {
        document::eval(&PAGE_KEYS_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// Focuses the element with `id` and selects its text, for a `/` key.
pub fn focus_by_id(id: &str) {
    document::eval(&format!(
        "const el = document.getElementById('{id}'); if (el) {{ el.focus(); if (el.select) el.select(); }}"
    ));
}

/// Registers one screen's key listener.
const PAGE_KEYS_JS: &str = r#"
window.__zwipePageKeys = window.__zwipePageKeys || [];
window.__zwipePageKeys.push('__ID__');
const onKey = (e) => {
    const stack = window.__zwipePageKeys;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    if (e.altKey || e.ctrlKey || e.metaKey || e.isComposing || e.key.length !== 1) return;
    const focus = document.activeElement;
    if (focus && (focus.tagName === 'INPUT' || focus.tagName === 'TEXTAREA' || focus.isContentEditable)) return;
    if (document.querySelector('.alert-dialog[data-state="open"]')) return;
    if (document.querySelector('.bottom-sheet.show')) return;
    if (e.key === '/') e.preventDefault();
    dioxus.send(e.key);
};
window['__zwipePageKey__ID__'] = onKey;
document.addEventListener('keydown', onKey);
"#;

/// Removes one screen's key listener.
const PAGE_KEYS_DROP_JS: &str = r"
const onKey = window['__zwipePageKey__ID__'];
if (onKey) document.removeEventListener('keydown', onKey);
delete window['__zwipePageKey__ID__'];
window.__zwipePageKeys = (window.__zwipePageKeys || []).filter((id) => id !== '__ID__');
";
