//! Focus for a sheet whose body takes its own keys: the body takes focus as the sheet opens, so
//! its keys work at once, and the element that had focus before gets it back as the sheet closes.

use dioxus::prelude::*;
use uuid::Uuid;

/// How long after opening the body takes focus, in milliseconds: the sheet's first frame has
/// drawn it by then.
const FOCUS_DELAY_MS: u32 = 60;

/// Focuses the element matching `body` (a selector) as `open` turns true, remembering the element
/// that had focus, and hands focus back to that element as `open` turns false or the sheet
/// unmounts open. `body` names the open sheet's own body, such as
/// `.bottom-sheet.show .sheet-body`, so a closed copy of the sheet never matches.
pub fn use_sheet_focus(open: ReadSignal<bool>, body: &'static str) {
    let id = use_hook(Uuid::new_v4);
    let mut was_open = use_signal(|| false);
    use_effect(move || {
        let now = open();
        let before = *was_open.peek();
        if now == before {
            return;
        }
        was_open.set(now);
        let script = if now {
            focus_js(&id, body)
        } else {
            restore_js(&id)
        };
        document::eval(&script);
    });
    use_drop(move || {
        if *was_open.peek() {
            document::eval(&restore_js(&id));
        }
    });
}

/// Remembers the focused element under `id` and focuses `body` once the sheet has drawn.
fn focus_js(id: &Uuid, body: &str) -> String {
    format!(
        "window.__zwipeSheetFocus = window.__zwipeSheetFocus || {{}}; window.__zwipeSheetFocus['{id}'] = document.activeElement; setTimeout(() => {{ const b = document.querySelector('{body}'); if (b) b.focus({{ preventScroll: true }}); }}, {FOCUS_DELAY_MS});"
    )
}

/// Focuses the element remembered under `id` again, while it is still in the page.
fn restore_js(id: &Uuid) -> String {
    format!(
        "{{ const store = window.__zwipeSheetFocus || {{}}; const el = store['{id}']; delete store['{id}']; if (el && el.isConnected && el !== document.body && el.focus) el.focus({{ preventScroll: true }}); }}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_remembers_the_focused_element_and_focuses_the_open_body() {
        let id = Uuid::nil();
        let js = focus_js(&id, ".bottom-sheet.show .body");
        assert!(
            js.contains(&format!("['{id}'] = document.activeElement")),
            "{js}"
        );
        assert!(
            js.contains("document.querySelector('.bottom-sheet.show .body')"),
            "{js}"
        );
        assert!(js.contains("b.focus({ preventScroll: true })"), "{js}");
    }

    #[test]
    fn closing_hands_focus_back_to_the_remembered_element_still_in_the_page() {
        let id = Uuid::nil();
        let js = restore_js(&id);
        assert!(js.contains(&format!("store['{id}']")), "{js}");
        assert!(js.contains(&format!("delete store['{id}']")), "{js}");
        assert!(js.contains("el.isConnected"), "{js}");
        assert!(js.contains("el.focus({ preventScroll: true })"), "{js}");
    }
}
