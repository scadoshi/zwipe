//! The deck cards page's keyboard cursor (`context/plans/deck_keyboard_nav.md`): the arrow keys
//! and `h` `j` `k` `l` move focus between the cards, List rows and stack cards alike, and Home and
//! End go to the first and last card while the cursor shows. The cursor is the browser's focus on
//! a card's own focusable line or button, so Enter opens the card as it always did, a stack card
//! reveals as it does on hover, and the focus ring and a screen reader follow it.
//!
//! The browser gathers the cards as drawn, columns left to right and each card top to bottom, and
//! [`zwipe_ui::deck_cards::cursor`] decides where a move lands. The keys pass through the details
//! drawer, which follows the cursor to each new card, and never through any other sheet, dialog,
//! large modal or the image preview. Only the computer layout takes them (a mouse with hover, from
//! 900px), so a phone or a touch screen never sees the cursor.
//!
//! Which card the card keys act on reads the cursor too ([`KEYED_JS`]): the card in focus when the
//! last input was a key, else the card under the mouse. A mouse move that scrolls nothing counts as
//! mouse input; a page scrolling under a still mouse does not.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::deck_cards::cursor::{MOVE_KEYS, Move, Snapshot, Stop, move_for};

use crate::deck_cards_view::StackDetails;
use crate::keys::{PRESSED_KEY, STACK_DETAILS_CLASS, cursor_guard_js};
use crate::media::COMPUTER_QUERY;

/// The cards the cursor stops on: List rows and stack cards. A featured card in the deck head is
/// not one.
pub(crate) const STOP_SELECTOR: &str = ".web-deck-row[data-row], .card-stack-slot[data-row]";

/// Installs, once a tab, the page-wide trackers the cursor reads: the last input (`key` or
/// `mouse`), the card in focus, `__zwipeDeckFocus`, which focuses a card by id and keeps it in
/// view, and `__zwipeDeckHandOn`, which focuses the cursor's card again, or, once it has left the
/// page, the next card after it in the last reading order, else the one before. While a key moved
/// last, a page change that drops focus to the page (a redraw, an Undo, a removed card) hands it
/// back that way.
const TRACK_JS: &str = r#"
if (!window.__zwipeDeckTrack) {
    window.__zwipeDeckTrack = true;
    window.__zwipeDeckInput = 'mouse';
    const stops = '__STOPS__';
    document.addEventListener('mousemove', (e) => { if (e.movementX || e.movementY) window.__zwipeDeckInput = 'mouse'; }, true);
    document.addEventListener('pointerdown', () => { window.__zwipeDeckInput = 'mouse'; }, true);
    document.addEventListener('focusin', (e) => {
        const stop = e.target.closest && e.target.closest(stops);
        if (stop) window.__zwipeDeckCursor = stop.getAttribute('data-row');
    }, true);
    const find = (id) => document.querySelector('.web-deck-row[data-row="' + id + '"], .card-stack-slot[data-row="' + id + '"]');
    const live = (el) => el && el.offsetParent !== null && !el.closest('[inert]') ? el : null;
    window.__zwipeDeckFocus = (id) => {
        const stop = live(find(id));
        if (!stop) return null;
        const target = stop.querySelector('.card-row-compact, .card-stack-card') || stop;
        target.focus({ preventScroll: true });
        const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
        target.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: still ? 'auto' : 'smooth' });
        window.__zwipeDeckCursor = id;
        return stop;
    };
    window.__zwipeDeckHandOn = () => {
        const id = window.__zwipeDeckCursor;
        if (!id) return null;
        if (live(find(id))) return window.__zwipeDeckFocus(id);
        const order = window.__zwipeDeckOrder || [];
        const at = order.indexOf(id);
        if (at < 0) return null;
        const next = order.slice(at + 1).find((other) => live(find(other))) || order.slice(0, at).reverse().find((other) => live(find(other)));
        return next ? window.__zwipeDeckFocus(next) : null;
    };
    let pending = false;
    new MutationObserver(() => {
        if (pending || window.__zwipeDeckInput !== 'key' || !window.__zwipeDeckCursor) return;
        const at = document.activeElement;
        if (at && at !== document.body) return;
        pending = true;
        setTimeout(() => {
            pending = false;
            const now = document.activeElement;
            if (window.__zwipeDeckInput === 'key' && (!now || now === document.body)) window.__zwipeDeckHandOn();
        }, 150);
    }).observe(document.body, { childList: true, subtree: true });
}
"#;

/// Registers one page's cursor keys: a move key under the guard, on the computer layout, while
/// the page draws cards, gathers the cards as drawn and sends `[key, from, columns]`, `from` the
/// card the move starts at (the hovered card when the mouse moved last, else the card in focus,
/// else the last cursor) and each column `[[id, center_y, in_view], ...]`. Home and End only
/// answer while a card has focus from a key.
const NAV_JS: &str = r"
window.__zwipeDeckNav = window.__zwipeDeckNav || [];
window.__zwipeDeckNav.push('__ID__');
const stops = '__STOPS__';
const onKey = (e) => {
    const stack = window.__zwipeDeckNav;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    const key = __KEY__;
    if (!__MOVES__.includes(key) || e.shiftKey && key.startsWith('Arrow')) return;
    if (!matchMedia('__POINTER__').matches || !__GUARD__) return;
    const all = [...document.querySelectorAll(stops)].filter((s) => s.offsetParent !== null && !s.closest('[inert]'));
    if (!all.length) return;
    const focused = document.activeElement && document.activeElement.closest && document.activeElement.closest(stops);
    const shows = window.__zwipeDeckInput === 'key' && focused;
    if ((key === 'Home' || key === 'End') && !shows) return;
    const hovered = all.filter((s) => s.matches(':hover')).pop();
    const fromEl = window.__zwipeDeckInput === 'mouse' && hovered ? hovered : focused;
    const from = fromEl ? fromEl.getAttribute('data-row') : (window.__zwipeDeckCursor || null);
    e.preventDefault();
    window.__zwipeDeckInput = 'key';
    const stacked = [...document.querySelectorAll('.deck-stacks-column')];
    const listed = document.querySelector('.deck-list-columns');
    const paired = listed && getComputedStyle(listed).display === 'grid';
    const holders = stacked.length ? stacked : paired ? [...listed.querySelectorAll(':scope > .deck-list-column')] : [];
    const height = window.innerHeight;
    const as = (s) => { const r = s.getBoundingClientRect(); return [s.getAttribute('data-row'), r.top + r.height / 2, r.bottom > 0 && r.top < height]; };
    const columns = holders.length ? holders.map((h) => all.filter((s) => h.contains(s)).map(as)) : [all.map(as)];
    window.__zwipeDeckOrder = columns.flat().map((s) => s[0]);
    dioxus.send([key, from, columns]);
};
window['__zwipeDeckNav__ID__'] = onKey;
document.addEventListener('keydown', onKey);
";

/// Removes one page's cursor keys.
const NAV_DROP_JS: &str = r"
const onKey = window['__zwipeDeckNav__ID__'];
if (onKey) document.removeEventListener('keydown', onKey);
delete window['__zwipeDeckNav__ID__'];
window.__zwipeDeckNav = (window.__zwipeDeckNav || []).filter((id) => id !== '__ID__');
";

/// A script expression for the card the card keys act on from the cursor: when the last input
/// was a key, the card in focus, else the card the cursor moves on to from one that left; `null`
/// when the mouse moved last.
pub(crate) const KEYED_JS: &str = "(window.__zwipeDeckInput === 'key' ? ((document.activeElement && document.activeElement.closest && document.activeElement.closest('__STOPS__')) || (window.__zwipeDeckHandOn && window.__zwipeDeckHandOn())) : null)";

/// [`KEYED_JS`] with its selector filled in.
pub(crate) fn keyed_js() -> String {
    KEYED_JS.replace("__STOPS__", STOP_SELECTOR)
}

/// One card as the browser sends it: its id, vertical center and whether it is in view.
type SentStop = (String, f64, bool);

/// Answers the deck page's cursor keys, as the module describes. `details` is the page's Stacks
/// reveal and drawer, which follows the cursor while it is open.
pub(crate) fn use_deck_cursor(details: Option<StackDetails>) {
    let id = use_hook(Uuid::new_v4);
    let mut order = use_signal(Vec::<Uuid>::new);
    use_hook(move || {
        document::eval(&TRACK_JS.replace("__STOPS__", STOP_SELECTOR));
        let script = NAV_JS
            .replace("__ID__", &id.to_string())
            .replace("__STOPS__", STOP_SELECTOR)
            .replace("__KEY__", PRESSED_KEY)
            .replace("__MOVES__", &format!("{MOVE_KEYS:?}"))
            .replace("__POINTER__", COMPUTER_QUERY)
            .replace("__GUARD__", &cursor_guard_js(STACK_DETAILS_CLASS));
        spawn(async move {
            let mut keys = document::eval(&script);
            while let Ok((key, from, columns)) = keys
                .recv::<(String, Option<String>, Vec<Vec<SentStop>>)>()
                .await
            {
                let Some(step) = move_for(&key) else {
                    continue;
                };
                let snapshot = snapshot_of(&columns);
                let from = from.and_then(|from| Uuid::try_parse(&from).ok());
                let Some(target) = land(&snapshot, from, step, &order.peek()) else {
                    continue;
                };
                order.set(snapshot.order());
                document::eval(&format!("window.__zwipeDeckFocus('{target}');"));
                if let Some(details) = details
                    && *details.details_open.peek()
                    && details.reveal.peek().open() != Some(target)
                {
                    details.click(target);
                }
            }
        });
    });
    use_drop(move || {
        document::eval(&NAV_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// The browser's columns as a [`Snapshot`], dropping any card whose id doesn't parse.
fn snapshot_of(columns: &[Vec<SentStop>]) -> Snapshot {
    Snapshot {
        columns: columns
            .iter()
            .map(|column| {
                column
                    .iter()
                    .filter_map(|(id, center_y, in_view)| {
                        Uuid::try_parse(id).ok().map(|id| Stop {
                            id,
                            center_y: *center_y,
                            in_view: *in_view,
                        })
                    })
                    .collect()
            })
            .collect(),
    }
}

/// Where `step` from `from` lands: from a card no longer drawn, the card after it in `order` (the
/// last reading order) that still is, else the one before; else as [`Snapshot::step`] moves.
fn land(snapshot: &Snapshot, from: Option<Uuid>, step: Move, order: &[Uuid]) -> Option<Uuid> {
    if let Some(gone) = from.filter(|id| !snapshot.order().contains(id))
        && let Some(next) = snapshot.hand_on(gone, order)
    {
        return Some(next);
    }
    snapshot.step(from, step)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn sent(n: u128, y: f64) -> SentStop {
        (id(n).to_string(), y, true)
    }

    #[test]
    fn the_sent_columns_become_the_snapshot_and_a_bad_id_drops_out() {
        let columns = vec![
            vec![sent(1, 10.0), ("nope".to_owned(), 20.0, true)],
            vec![sent(2, 30.0)],
        ];
        let snapshot = snapshot_of(&columns);
        assert_eq!(snapshot.order(), vec![id(1), id(2)]);
    }

    #[test]
    fn a_move_from_a_card_that_left_lands_on_the_next_in_the_old_order() {
        let snapshot = snapshot_of(&[vec![sent(1, 10.0), sent(3, 50.0)]]);
        let order = vec![id(1), id(2), id(3)];
        assert_eq!(
            land(&snapshot, Some(id(2)), Move::Down, &order),
            Some(id(3))
        );
        assert_eq!(
            land(&snapshot, Some(id(1)), Move::Down, &order),
            Some(id(3)),
            "a card still drawn moves as usual"
        );
    }

    #[test]
    fn the_keys_reach_only_a_mouse_page_and_pass_only_the_drawer() {
        assert!(NAV_JS.contains("matchMedia('__POINTER__')"), "{NAV_JS}");
        assert!(
            NAV_JS.contains("e.shiftKey && key.startsWith('Arrow')"),
            "{NAV_JS}"
        );
        assert!(
            NAV_JS.contains("(key === 'Home' || key === 'End') && !shows"),
            "Home and End only while the cursor shows: {NAV_JS}"
        );
        assert!(NAV_JS.contains("e.preventDefault()"), "{NAV_JS}");
    }

    #[test]
    fn focus_dropped_by_a_redraw_goes_back_to_the_cursor_after_a_key() {
        assert!(TRACK_JS.contains("new MutationObserver("), "{TRACK_JS}");
        assert!(
            TRACK_JS
                .contains("window.__zwipeDeckInput === 'key' && (!now || now === document.body)"),
            "only after a key and only from the page: {TRACK_JS}"
        );
        assert!(
            TRACK_JS.contains("if (live(find(id))) return window.__zwipeDeckFocus(id);"),
            "a card still drawn takes focus back: {TRACK_JS}"
        );
    }

    #[test]
    fn a_page_scrolling_under_a_still_mouse_is_not_mouse_input() {
        assert!(
            TRACK_JS.contains("if (e.movementX || e.movementY) window.__zwipeDeckInput = 'mouse'"),
            "{TRACK_JS}"
        );
    }

    #[test]
    fn the_card_keys_read_the_focused_card_only_after_a_key() {
        let js = keyed_js();
        assert!(
            js.starts_with("(window.__zwipeDeckInput === 'key' ?"),
            "{js}"
        );
        assert!(js.contains(STOP_SELECTOR), "{js}");
        assert!(js.contains("__zwipeDeckHandOn()"), "{js}");
    }
}
