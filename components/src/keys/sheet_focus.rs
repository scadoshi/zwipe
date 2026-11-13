//! Focus for a sheet whose body takes its own keys: the body takes focus as the sheet opens, so
//! its keys work at once, and the element that had focus before gets it back as the sheet closes.
//! An opener a save draws again, such as a card row's Printings button once the row shows the
//! new printing, is found again by its action and its place among the buttons with that action.

use dioxus::prelude::*;
use uuid::Uuid;

/// How long after opening the body takes focus, in milliseconds: the sheet's first frame has
/// drawn it by then.
const FOCUS_DELAY_MS: u32 = 60;

/// When the body takes focus again, in milliseconds, if the tap that opened the sheet handed it
/// back to the page or the opener meanwhile: the sheet has slid in by then.
const REFOCUS_MS: u32 = 350;

/// How long after closing the opener is found again if it is drawn anew, in milliseconds: long
/// enough for a save to come back from the server.
const RESTORE_WATCH_MS: u32 = 3000;

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

/// Remembers the focused element under `id`, with its action and its place among the elements
/// with that action, and focuses `body` once the sheet has drawn, and again once it has slid in
/// if focus went back to the page or the opener.
fn focus_js(id: &Uuid, body: &str) -> String {
    format!(
        "{{ const store = window.__zwipeSheetFocus = window.__zwipeSheetFocus || {{}}; const el = document.activeElement; const action = el && el.getAttribute && el.getAttribute('data-action'); const sel = action ? '[data-action=\"' + action + '\"]' : null; store['{id}'] = {{ el, sel, at: sel ? [...document.querySelectorAll(sel)].indexOf(el) : -1 }}; const take = () => {{ const b = document.querySelector('{body}'); if (b) b.focus({{ preventScroll: true }}); }}; setTimeout(take, {FOCUS_DELAY_MS}); setTimeout(() => {{ const b = document.querySelector('{body}'); const a = document.activeElement; if (b && !b.contains(a) && (!a || a === document.body || a === el)) take(); }}, {REFOCUS_MS}); }}"
    )
}

/// Focuses the element remembered under `id` again, or, once a save draws it anew, the element
/// with its action in its place, while focus is on nothing else.
fn restore_js(id: &Uuid) -> String {
    format!(
        "{{ const store = window.__zwipeSheetFocus || {{}}; const kept = store['{id}']; delete store['{id}']; if (kept && kept.el !== document.body) {{ const find = () => kept.el && kept.el.isConnected ? kept.el : (kept.sel && kept.at >= 0 ? document.querySelectorAll(kept.sel)[kept.at] : null); const give = () => {{ const t = find(); if (t && t.focus) t.focus({{ preventScroll: true }}); }}; give(); const until = Date.now() + {RESTORE_WATCH_MS}; const watch = setInterval(() => {{ const a = document.activeElement; if (Date.now() > until || (a && a !== document.body)) {{ clearInterval(watch); return; }} give(); }}, 100); }} }}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_remembers_the_opener_by_element_and_action_and_focuses_the_open_body() {
        let id = Uuid::nil();
        let js = focus_js(&id, ".bottom-sheet.show .body");
        assert!(
            js.contains(&format!("store['{id}'] = {{ el, sel, at:")),
            "{js}"
        );
        assert!(js.contains("getAttribute('data-action')"), "{js}");
        assert!(
            js.contains("document.querySelector('.bottom-sheet.show .body')"),
            "{js}"
        );
        assert!(js.contains("b.focus({ preventScroll: true })"), "{js}");
        assert!(
            js.contains(&format!("}}, {REFOCUS_MS});")),
            "a second focus once the sheet has slid in: {js}"
        );
        assert!(
            js.contains("a === document.body || a === el"),
            "only when the page or the opener took it back: {js}"
        );
    }

    #[test]
    fn closing_hands_focus_back_to_the_opener_or_the_one_drawn_in_its_place() {
        let id = Uuid::nil();
        let js = restore_js(&id);
        assert!(js.contains(&format!("delete store['{id}']")), "{js}");
        assert!(js.contains("kept.el.isConnected ? kept.el"), "{js}");
        assert!(
            js.contains("document.querySelectorAll(kept.sel)[kept.at]"),
            "{js}"
        );
        assert!(
            js.contains(&format!("Date.now() + {RESTORE_WATCH_MS}")),
            "{js}"
        );
        assert!(
            js.contains("(a && a !== document.body)"),
            "stops once focus is somewhere else: {js}"
        );
    }
}
