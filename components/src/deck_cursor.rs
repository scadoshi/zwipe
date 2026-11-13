//! The deck cards page's keyboard cursor (`context/plans/deck_keyboard_nav.md`): the arrow keys
//! and `h` `j` `k` `l` move focus between the cards, List rows and stack cards alike, and Home and
//! End go to the first and last card while the cursor shows. The cursor is the browser's focus on
//! a card's own focusable line or button, so Enter opens the card as it always did, a stack card
//! reveals as it does on hover, and the focus ring and a screen reader follow it.
//!
//! The browser gathers the cards as drawn, columns left to right and each card top to bottom, and
//! [`zwipe_ui::deck_cards::cursor`] decides where a move lands. The keys pass through the details
//! drawer, which follows the cursor to each new card, and never through any other sheet, dialog,
//! large modal or the image preview. They answer at any width, a narrow window or a tablet's
//! keyboard too: a key is the keyboard's own proof, and a touch never moves the cursor.
//!
//! Which card the card keys act on reads the cursor too (`KEYED_JS`): the card in focus when the
//! last input was a key, else the card under the mouse. A mouse move that scrolls nothing counts as
//! mouse input; a page scrolling under a still mouse does not.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::deck_cards::cursor::{MOVE_KEYS, Move, Snapshot, Stop, move_for};

use crate::deck_cards_view::StackDetails;
use crate::keys::{PRESSED_KEY, STACK_DETAILS_CLASS, cursor_guard_js};

/// The cards the cursor stops on: the deck page's List rows and stack cards, and the card rows of
/// the screens that list cards without editing them ([`use_card_list_cursor`]). A deck row's own
/// card row inside it is not a second stop, and a featured card in the deck head is not one.
pub const STOP_SELECTOR: &str = ".web-deck-row[data-row], .card-stack-slot[data-row], .web-card-row[data-row]:not(.web-deck-row > *)";

/// Installs, once a tab, the page-wide trackers the cursor reads: the last input (`key` or
/// `mouse`), the card in focus, `__zwipeDeckFocus`, which focuses a card by id and keeps it in
/// view (at once while a key is held, `__zwipeDeckHeld`, else smoothly), and `__zwipeDeckHandOn`, which focuses the cursor's card again, or, once it has left the
/// page, the next card after it in the last reading order, else the one before. While a key moved
/// last, a page change that drops focus to the page (a redraw, an Undo, a removed card) hands it
/// back that way.
const TRACK_JS: &str = r"
if (!window.__zwipeDeckTrack) {
    window.__zwipeDeckTrack = true;
    window.__zwipeDeckInput = 'mouse';
    const stops = '__STOPS__';
    let previewed = null;
    const unpreview = () => { if (previewed) { const line = previewed; previewed = null; line.dispatchEvent(new MouseEvent('mouseleave')); } };
    document.addEventListener('mousemove', (e) => { if (e.movementX || e.movementY) { window.__zwipeDeckInput = 'mouse'; unpreview(); } }, true);
    document.addEventListener('pointerdown', () => { window.__zwipeDeckInput = 'mouse'; }, true);
    document.addEventListener('focusin', (e) => {
        const stop = e.target.closest && e.target.closest(stops);
        if (stop) window.__zwipeDeckCursor = stop.getAttribute('data-row');
        const line = e.target.closest && e.target.closest('.web-card-row .card-row-compact');
        if (line && line !== previewed && window.__zwipeCursorPreview && window.__zwipeDeckInput === 'key') {
            unpreview();
            previewed = line;
            line.dispatchEvent(new MouseEvent('mouseenter'));
        }
    }, true);
    document.addEventListener('focusout', (e) => { if (e.target === previewed) unpreview(); }, true);
    const find = (id) => [...document.querySelectorAll(stops)].find((s) => s.getAttribute('data-row') === id);
    const box = (s) => (s && getComputedStyle(s).display === 'contents' && s.firstElementChild) || s;
    const live = (el) => el && box(el).offsetParent !== null && !el.closest('[inert]') ? el : null;
    window.__zwipeDeckFocus = (id, held) => {
        const stop = live(find(id));
        if (!stop) return null;
        const target = stop.querySelector('.card-row-compact, .card-stack-card') || stop;
        target.focus({ preventScroll: true });
        const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
        target.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: still || held ? 'auto' : 'smooth' });
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
";

/// Registers one page's cursor keys: a move key under the guard, at any width, while the page
/// draws cards, gathers the cards as drawn and sends `[key, from, columns]`, `from` the
/// card the move starts at (the hovered card when the mouse moved last, else the card in focus,
/// else the last cursor), each column `[[id, center_y, in_view], ...]`, and `open`: `drawer`
/// while the details drawer shows, `row` while a List row is open, else empty. Home and End only
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
    if (!__GUARD__) return;
    const box = (s) => (s && getComputedStyle(s).display === 'contents' && s.firstElementChild) || s;
    const all = [...document.querySelectorAll(stops)].filter((s) => box(s).offsetParent !== null && !s.closest('[inert]'));
    if (!all.length) return;
    const focused = document.activeElement && document.activeElement.closest && document.activeElement.closest(stops);
    const shows = window.__zwipeDeckInput === 'key' && focused;
    if ((key === 'Home' || key === 'End') && !shows) return;
    const hovered = all.filter((s) => s.matches(':hover')).pop();
    const fromEl = window.__zwipeDeckInput === 'mouse' && hovered ? hovered : focused;
    const from = fromEl ? fromEl.getAttribute('data-row') : (window.__zwipeDeckCursor || null);
    e.preventDefault();
    window.__zwipeDeckInput = 'key';
    window.__zwipeDeckHeld = e.repeat;
    const stacked = [...document.querySelectorAll('.deck-stacks-column')];
    const listed = document.querySelector('.deck-list-columns');
    const paired = listed && getComputedStyle(listed).display === 'grid';
    const holders = stacked.length ? stacked : paired ? [...listed.querySelectorAll(':scope > .deck-list-column')] : [];
    const height = window.innerHeight;
    const as = (s) => { const r = box(s).getBoundingClientRect(); return [s.getAttribute('data-row'), r.top + r.height / 2, r.bottom > 0 && r.top < height]; };
    const columns = holders.length ? holders.map((h) => all.filter((s) => h.contains(s)).map(as)) : [all.map(as)];
    window.__zwipeDeckOrder = columns.flat().map((s) => s[0]);
    const open = document.querySelector('.bottom-sheet.show.__DRAWER__') ? 'drawer' : document.querySelector('.web-deck-row.selected, .web-card-row .card-row.expanded') ? 'row' : '';
    dioxus.send([key, from, columns, open]);
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

/// A script expression for the card the card keys act on from the keyboard cursor: when the last
/// input was a key, the card in focus, else the card the cursor moves on to from one that left;
/// `null` when the mouse moved last.
#[must_use]
pub fn keyed_js() -> String {
    KEYED_JS.replace("__STOPS__", STOP_SELECTOR)
}

/// Moves the cursor from card `__FROM__` to card `__TO__` where it sits on `__FROM__`, in the
/// last reading order too, so a redraw hands focus to `__TO__`.
const FOLLOW_JS: &str = "if (window.__zwipeDeckCursor === '__FROM__') window.__zwipeDeckCursor = '__TO__'; window.__zwipeDeckOrder = (window.__zwipeDeckOrder || []).map((id) => id === '__FROM__' ? '__TO__' : id);";

/// Card `from` is now card `to`, as a printing change or its Undo makes it: the keyboard cursor
/// follows it, so focus lands on the card in its new printing rather than moving on.
pub fn follow_card(from: Uuid, to: Uuid) {
    document::eval(&follow_js(from, to));
}

/// [`FOLLOW_JS`] for `from` and `to`.
fn follow_js(from: Uuid, to: Uuid) -> String {
    FOLLOW_JS
        .replace("__FROM__", &from.to_string())
        .replace("__TO__", &to.to_string())
}

/// What follows the cursor to its new card: what the page shows open as it moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Follow {
    /// The details drawer shows: it swaps to the new card.
    Drawer,
    /// A List row is open: the new card's row opens in its place.
    Row,
    /// Nothing is open: only the cursor moves.
    Nothing,
}

impl Follow {
    /// What `open`, as the browser sends it, means.
    fn of(open: &str) -> Self {
        match open {
            "drawer" => Self::Drawer,
            "row" => Self::Row,
            _ => Self::Nothing,
        }
    }
}

/// Opens list row `__ID__` where it is shut, as a click on its line does, which shuts the open one.
const OPEN_ROW_JS: &str = "const row = document.querySelector('.web-deck-row[data-row=\"__ID__\"], .web-card-row[data-row=\"__ID__\"]'); if (row && !row.classList.contains('selected') && !row.querySelector('.card-row.expanded')) row.querySelector('.card-row-compact')?.click();";

/// Marks the screen as one whose cursor card shows its image as hover does.
const PREVIEW_ON_JS: &str = "window.__zwipeCursorPreview = true;";

/// Unmarks it.
const PREVIEW_OFF_JS: &str = "window.__zwipeCursorPreview = false;";

/// The cursor on a screen that lists cards without editing them (the Commander maybeboard,
/// Activity's Most skipped, Playstyle's lists): the same keys as the deck page over its card rows,
/// in one column. Enter opens or closes a row, an open row follows the cursor, and, when the last
/// input was a key, the cursor's card shows its image in the screen's hover stack as hovering its
/// row does.
pub fn use_card_list_cursor() {
    use_hook(|| {
        document::eval(PREVIEW_ON_JS);
    });
    use_drop(|| {
        document::eval(PREVIEW_OFF_JS);
    });
    use_deck_cursor(None);
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
            .replace("__GUARD__", &cursor_guard_js(STACK_DETAILS_CLASS))
            .replace("__DRAWER__", STACK_DETAILS_CLASS);
        spawn(async move {
            let mut keys = document::eval(&script);
            while let Ok((key, from, columns, open)) = keys
                .recv::<(String, Option<String>, Vec<Vec<SentStop>>, String)>()
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
                document::eval(&format!(
                    "window.__zwipeDeckFocus('{target}', window.__zwipeDeckHeld);"
                ));
                match Follow::of(&open) {
                    Follow::Drawer => {
                        if let Some(details) = details
                            && details.reveal.peek().open() != Some(target)
                        {
                            details.click(target);
                        }
                    }
                    Follow::Row => {
                        document::eval(&OPEN_ROW_JS.replace("__ID__", &target.to_string()));
                    }
                    Follow::Nothing => {}
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
    fn a_held_key_scrolls_the_cursor_at_once() {
        assert!(
            TRACK_JS.contains("window.__zwipeDeckFocus = (id, held) => {"),
            "{TRACK_JS}"
        );
        assert!(
            TRACK_JS.contains("behavior: still || held ? 'auto' : 'smooth'"),
            "{TRACK_JS}"
        );
        assert!(
            NAV_JS.contains("window.__zwipeDeckHeld = e.repeat;"),
            "the keys mark a repeat for the focus that follows"
        );
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
    fn the_keys_reach_any_width_and_pass_only_the_drawer() {
        assert!(
            !NAV_JS.contains("matchMedia") && !NAV_JS.contains("__POINTER__"),
            "a key reaches the cursor at any width, a keyboard's own proof: {NAV_JS}"
        );
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
    fn a_card_that_changes_id_keeps_the_cursor_and_its_place_in_the_order() {
        let js = follow_js(id(1), id(2));
        assert!(
            js.contains(&format!(
                "if (window.__zwipeDeckCursor === '{}') window.__zwipeDeckCursor = '{}';",
                id(1),
                id(2)
            )),
            "{js}"
        );
        assert!(
            js.contains(&format!("id === '{}' ? '{}' : id", id(1), id(2))),
            "{js}"
        );
    }

    #[test]
    fn a_screen_of_card_rows_stops_on_its_rows() {
        assert!(
            STOP_SELECTOR.contains(".web-card-row[data-row]"),
            "{STOP_SELECTOR}"
        );
        assert!(
            STOP_SELECTOR.contains(".web-card-row[data-row]:not(.web-deck-row > *)"),
            "a deck row's own card row is not a second stop for the same card: {STOP_SELECTOR}"
        );
        assert!(
            NAV_JS.contains(".web-card-row .card-row.expanded"),
            "an open card row counts as an open row: {NAV_JS}"
        );
        assert!(
            OPEN_ROW_JS.contains("row.querySelector('.card-row.expanded')"),
            "a card row that is shut opens: {OPEN_ROW_JS}"
        );
        for js in [NAV_JS, TRACK_JS] {
            assert!(
                js.contains("getComputedStyle(s).display === 'contents'"),
                "a card row draws no box of its own (display: contents), so its card row is measured: {js}"
            );
        }
    }

    #[test]
    fn a_screen_that_asks_shows_the_cursor_card_s_image_as_hover_does() {
        assert!(
            TRACK_JS.contains("window.__zwipeCursorPreview && window.__zwipeDeckInput === 'key'"),
            "only on a screen that asks, and only after a key: {TRACK_JS}"
        );
        assert!(
            TRACK_JS.contains("new MouseEvent('mouseenter')")
                && TRACK_JS.contains("new MouseEvent('mouseleave')"),
            "the row's own hover enter and leave: {TRACK_JS}"
        );
    }

    #[test]
    fn what_shows_open_decides_what_follows_the_cursor() {
        assert_eq!(Follow::of("drawer"), Follow::Drawer);
        assert_eq!(Follow::of("row"), Follow::Row);
        assert_eq!(Follow::of(""), Follow::Nothing);
        assert!(
            NAV_JS.contains("document.querySelector('.bottom-sheet.show.__DRAWER__') ? 'drawer' : document.querySelector('.web-deck-row.selected, .web-card-row .card-row.expanded') ? 'row' : ''"),
            "{NAV_JS}"
        );
        assert!(
            OPEN_ROW_JS.contains("!row.classList.contains('selected')"),
            "{OPEN_ROW_JS}"
        );
    }

    #[test]
    fn a_page_scrolling_under_a_still_mouse_is_not_mouse_input() {
        assert!(
            TRACK_JS
                .contains("if (e.movementX || e.movementY) { window.__zwipeDeckInput = 'mouse';"),
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
