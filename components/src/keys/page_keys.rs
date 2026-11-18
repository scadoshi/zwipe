//! Single keys a screen answers while no text field has focus, such as `/`
//! to focus its search.
//!
//! Screens and overlays stack: the newest one registered takes the keys
//! until it unmounts. No key reaches a screen while a dialog or a sheet is
//! open, or with Ctrl, Alt or Meta held.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::shortcuts;

use crate::keys::guard;

/// Calls `on_key` with each key (`KeyboardEvent.key`, such as `"/"` or
/// `"a"`) pressed while this registration is the newest and no text field,
/// dialog or sheet has focus. The browser's default for the key is kept,
/// except for the keys that open Quick add and the filter's name box, which
/// never type themselves into the field they open.
pub fn use_page_keys(on_key: Callback<String>) {
    use_keys_under_guard(guard::guard_js(), on_key);
}

/// [`use_page_keys`], except that while the open sheet is the one with `sheet_class`, `keys`
/// still reach `on_key` ([`guard::guard_js_through`]). Any other sheet or a dialog keeps
/// every key.
pub fn use_page_keys_through(
    sheet_class: &'static str,
    keys: &'static [&'static str],
    on_key: Callback<String>,
) {
    use_keys_under_guard(guard::guard_js_through(sheet_class, keys), on_key);
}

/// Registers `on_key` for the keys `guard` (a browser expression) lets reach the screen.
fn use_keys_under_guard(guard: String, on_key: Callback<String>) {
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = page_keys_js(&guard, id);
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

/// The keys that put focus in a text field (Quick add, the filter's name box). Their own input is
/// held back, so the key never types itself into the field it opens.
const FIELD_KEYS: [&str; 2] = [shortcuts::QUICK_ADD_KEY, shortcuts::FILTER_NAME_KEY];

/// Registers one screen's key listener, registration `id`, which answers while it is the newest
/// and `guard` (a browser expression) holds.
fn page_keys_js(guard: &str, id: Uuid) -> String {
    let key = guard::PRESSED_KEY;
    let field_keys = format!("{FIELD_KEYS:?}");
    format!(
        r"
window.__zwipePageKeys = window.__zwipePageKeys || [];
window.__zwipePageKeys.push('{id}');
const onKey = (e) => {{
    const stack = window.__zwipePageKeys;
    if (!stack.length || stack[stack.length - 1] !== '{id}') return;
    if ({key}.length !== 1 || !{guard}) return;
    if ({field_keys}.includes({key})) e.preventDefault();
    dioxus.send({key});
}};
window['__zwipePageKey{id}'] = onKey;
document.addEventListener('keydown', onKey);
"
    )
}

/// Calls `on_undo` for Ctrl+Z (Cmd+Z on a Mac) while this registration is
/// the newest and no text field, dialog or sheet has focus, so a text box
/// keeps its own undo.
pub fn use_undo_key(on_undo: Callback<()>) {
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = UNDO_KEY_JS
            .replace("__ID__", &id.to_string())
            .replace("__GUARD__", &guard::guard_js_with_modifiers());
        spawn(async move {
            let mut presses = document::eval(&script);
            while presses.recv::<bool>().await.is_ok() {
                on_undo.call(());
            }
        });
    });
    use_drop(move || {
        document::eval(&UNDO_KEY_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// Calls `on_key` with each of `keys` (`KeyboardEvent.key` names, such as
/// `"ArrowLeft"`) pressed while focus sits inside `scope` (a CSS selector)
/// or on nothing, and no text field, dialog or sheet has focus. The
/// browser's default for the key, a scroll for an arrow, is held back.
pub fn use_scoped_keys(
    scope: &'static str,
    keys: &'static [&'static str],
    on_key: Callback<String>,
) {
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let names = keys
            .iter()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let script = SCOPED_KEYS_JS
            .replace("__KEY__", guard::PRESSED_KEY)
            .replace("__ID__", &id.to_string())
            .replace("__KEYS__", &names)
            .replace("__SCOPE__", scope)
            .replace("__GUARD__", &guard::guard_js());
        spawn(async move {
            let mut presses = document::eval(&script);
            while let Ok(key) = presses.recv::<String>().await {
                on_key.call(key);
            }
        });
    });
    use_drop(move || {
        document::eval(&SCOPED_KEYS_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// Focuses the element with `id` and selects its text, for a `/` key.
pub fn focus_by_id(id: &str) {
    document::eval(&format!(
        "const el = document.getElementById('{id}'); if (el) {{ el.focus(); if (el.select) el.select(); }}"
    ));
}

/// Registers one scoped key listener.
const SCOPED_KEYS_JS: &str = r"
const onKey = (e) => {
    if (![__KEYS__].includes(__KEY__) || !__GUARD__) return;
    const at = document.activeElement;
    if (at && at !== document.body && !at.closest('__SCOPE__')) return;
    e.preventDefault();
    dioxus.send(__KEY__);
};
window['__zwipeScopedKey__ID__'] = onKey;
document.addEventListener('keydown', onKey);
";

/// Removes one scoped key listener.
const SCOPED_KEYS_DROP_JS: &str = r"
const onKey = window['__zwipeScopedKey__ID__'];
if (onKey) document.removeEventListener('keydown', onKey);
delete window['__zwipeScopedKey__ID__'];
";

/// Removes one screen's key listener.
const PAGE_KEYS_DROP_JS: &str = r"
const onKey = window['__zwipePageKey__ID__'];
if (onKey) document.removeEventListener('keydown', onKey);
delete window['__zwipePageKey__ID__'];
window.__zwipePageKeys = (window.__zwipePageKeys || []).filter((id) => id !== '__ID__');
";

/// Registers one screen's Ctrl+Z listener.
const UNDO_KEY_JS: &str = r"
window.__zwipeUndoKeys = window.__zwipeUndoKeys || [];
window.__zwipeUndoKeys.push('__ID__');
const onKey = (e) => {
    const stack = window.__zwipeUndoKeys;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    if (!(e.ctrlKey || e.metaKey) || e.altKey || e.shiftKey || e.key.toLowerCase() !== 'z') return;
    if (!__GUARD__) return;
    e.preventDefault();
    dioxus.send(true);
};
window['__zwipeUndoKey__ID__'] = onKey;
document.addEventListener('keydown', onKey);
";

/// Removes one screen's Ctrl+Z listener.
const UNDO_KEY_DROP_JS: &str = r"
const onKey = window['__zwipeUndoKey__ID__'];
if (onKey) document.removeEventListener('keydown', onKey);
delete window['__zwipeUndoKey__ID__'];
window.__zwipeUndoKeys = (window.__zwipeUndoKeys || []).filter((id) => id !== '__ID__');
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_add_and_filter_keys_never_type_into_the_field_they_open() {
        let js = page_keys_js("true", Uuid::nil());
        assert!(
            js.contains(&format!(
                "if ([\"{}\", \"{}\"].includes(",
                shortcuts::QUICK_ADD_KEY,
                shortcuts::FILTER_NAME_KEY
            )),
            "{js}"
        );
        assert_eq!(
            js.matches("e.preventDefault()").count(),
            1,
            "every other key keeps the browser's default: {js}"
        );
    }
}
