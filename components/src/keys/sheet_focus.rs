//! Focus for a sheet whose body takes its own keys: the body takes focus as the sheet opens, so
//! its keys work at once, and the element that had focus before gets it back as the sheet closes.
//! An opener a save draws again, such as a card row's Printings button once the row shows the
//! new printing, is found again by its action and its place among the buttons with that action.

use dioxus::prelude::*;
use uuid::Uuid;

/// How long after opening the body takes focus, in milliseconds: the sheet's first frame has
/// drawn it by then.
const FOCUS_DELAY_MS: u32 = 60;

/// How long the body keeps trying to take focus, in milliseconds: a sheet is hidden, and refuses
/// focus, until it has started to slide in, and the tap that opened it can hand focus back to the
/// page or the opener meanwhile.
const FOCUS_TRY_MS: u32 = 1000;

/// How often the body tries to take focus while it has not, in milliseconds.
const FOCUS_RETRY_MS: u32 = 50;

/// How long after closing the opener is found again if it is drawn anew, in milliseconds: long
/// enough for a save to come back from the server.
const RESTORE_WATCH_MS: u32 = 3000;

/// Focuses the element matching `body` (a selector) as `open` turns true, remembering the element
/// that had focus, and hands focus back to that element as `open` turns false or the sheet
/// unmounts open. `body` names the open sheet's own body, such as
/// `.bottom-sheet.show .sheet-body`, so a closed copy of the sheet never matches.
pub fn use_sheet_focus(open: ReadSignal<bool>, body: &'static str) {
    let _ = use_sheet_focus_handing_back(open, body);
}

/// The hand back of a sheet's focus, from [`use_sheet_focus_handing_back`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SheetFocus {
    /// The elements focus goes back to in place of the opener, as selectors, the first that is in
    /// the page and not inert taking it.
    hand_back: Signal<Option<Vec<String>>>,
}

impl SheetFocus {
    /// Hands focus back to the first element matching one of `selectors`, in order, once it is in
    /// the page and not inert, in place of the opener: for a sheet whose action draws its opener
    /// anew, such as Printings, whose save draws the row again for the new printing.
    pub fn hand_back_to(mut self, selectors: Vec<String>) {
        self.hand_back.set(Some(selectors));
    }
}

/// [`use_sheet_focus`], with a [`SheetFocus`] that names where focus goes back to when the
/// opener will not be there.
#[must_use]
pub fn use_sheet_focus_handing_back(open: ReadSignal<bool>, body: &'static str) -> SheetFocus {
    let id = use_hook(Uuid::new_v4);
    let mut was_open = use_signal(|| false);
    let mut hand_back = use_signal(|| None::<Vec<String>>);
    use_effect(move || {
        let now = open();
        let before = *was_open.peek();
        if now == before {
            return;
        }
        was_open.set(now);
        let script = if now {
            hand_back.set(None);
            focus_js(&id, body)
        } else {
            restore_js(&id, hand_back.peek().as_deref())
        };
        document::eval(&script);
    });
    use_drop(move || {
        if *was_open.peek() {
            document::eval(&restore_js(&id, hand_back.peek().as_deref()));
        }
    });
    SheetFocus { hand_back }
}

/// Remembers the focused element under `id`, with its action and its place among the elements
/// with that action, and focuses `body` once the sheet has drawn, trying again until it takes
/// focus while focus is on the page, the opener or the sheet's × the kit focused as it opened.
fn focus_js(id: &Uuid, body: &str) -> String {
    format!(
        "{{ const store = window.__zwipeSheetFocus = window.__zwipeSheetFocus || {{}}; const el = document.activeElement; const action = el && el.getAttribute && el.getAttribute('data-action'); const sel = action ? '[data-action=\"' + action + '\"]' : null; store['{id}'] = {{ el, sel, at: sel ? [...document.querySelectorAll(sel)].indexOf(el) : -1 }}; const until = Date.now() + {FOCUS_TRY_MS}; const take = () => {{ const b = document.querySelector('{body}'); const a = document.activeElement; if (b && b.contains(a)) return; if (b && (!a || a === document.body || a === el || a === window.__oranguiSheetEntry)) b.focus({{ preventScroll: true }}); if ((!b || !b.contains(document.activeElement)) && Date.now() < until && (!document.activeElement || document.activeElement === document.body || document.activeElement === el || document.activeElement === window.__oranguiSheetEntry)) setTimeout(take, {FOCUS_RETRY_MS}); }}; setTimeout(take, {FOCUS_DELAY_MS}); }}"
    )
}

/// Focuses the element remembered under `id` again, the `hand_back` element in its place once
/// that is in the page, or, once a save draws the opener anew, the element with its action in its
/// place, skipping an inert one. It keeps at it while focus falls to the page, until focus is on a
/// new element or the user moved it on.
fn restore_js(id: &Uuid, hand_back: Option<&[String]>) -> String {
    let hand_back =
        hand_back.map_or_else(|| "null".to_owned(), |selectors| format!("{selectors:?}"));
    format!(
        "{{ const store = window.__zwipeSheetFocus || {{}}; const kept = store['{id}']; delete store['{id}']; const handBack = {hand_back}; if (kept && kept.el !== document.body) {{ const live = (t) => t && t.isConnected && !t.closest('[inert]') && t.focus ? t : null; const find = () => (handBack && handBack.map((sel) => [...document.querySelectorAll(sel)].map(live).find(Boolean)).find(Boolean)) || live(kept.el) || live(kept.sel && kept.at >= 0 ? document.querySelectorAll(kept.sel)[kept.at] : null); const give = () => {{ const t = find(); if (t && document.activeElement !== t) t.focus({{ preventScroll: true }}); }}; give(); const until = Date.now() + {RESTORE_WATCH_MS}; const watch = setInterval(() => {{ const a = document.activeElement; const t = find(); if (Date.now() > until || (a && a !== document.body && a !== kept.el && a !== t) || (t && a === t && t !== kept.el)) {{ clearInterval(watch); return; }} if (!a || a === document.body) give(); }}, 100); }} }}"
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
            js.contains(&format!("setTimeout(take, {FOCUS_RETRY_MS})")),
            "tries again while the sheet still refuses focus: {js}"
        );
        assert!(
            js.contains(&format!("Date.now() + {FOCUS_TRY_MS}")),
            "for a second at most: {js}"
        );
        assert!(
            js.contains("a === document.body || a === el"),
            "only while focus is on the page or the opener: {js}"
        );
    }

    #[test]
    fn closing_hands_focus_back_to_the_opener_or_the_one_drawn_in_its_place() {
        let id = Uuid::nil();
        let js = restore_js(&id, None);
        assert!(js.contains(&format!("delete store['{id}']")), "{js}");
        assert!(js.contains("live(kept.el)"), "{js}");
        assert!(
            js.contains("document.querySelectorAll(kept.sel)[kept.at]"),
            "{js}"
        );
        assert!(
            js.contains(&format!("Date.now() + {RESTORE_WATCH_MS}")),
            "{js}"
        );
        assert!(
            js.contains("(a && a !== document.body && a !== kept.el && a !== t)"),
            "stops once the user moves focus on: {js}"
        );
    }

    #[test]
    fn a_named_hand_back_comes_first_and_an_inert_element_never_takes_focus() {
        let id = Uuid::nil();
        let selectors = vec![
            "[data-row=\"x\"] [data-action=\"printings\"]".to_owned(),
            ".card".to_owned(),
        ];
        let js = restore_js(&id, Some(&selectors));
        assert!(
            js.contains(
                r#"const handBack = ["[data-row=\"x\"] [data-action=\"printings\"]", ".card"];"#
            ),
            "the selectors in order: {js}"
        );
        assert!(
            js.contains("(handBack && handBack.map((sel) => [...document.querySelectorAll(sel)].map(live).find(Boolean)).find(Boolean)) || live(kept.el)"),
            "{js}"
        );
        assert!(js.contains("!t.closest('[inert]')"), "{js}");
        assert!(restore_js(&id, None).contains("const handBack = null;"));
    }
}
