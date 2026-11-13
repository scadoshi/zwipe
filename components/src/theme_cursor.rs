//! The theme sheet's keyboard cursor on a computer: while the sheet is open, the arrow keys and
//! `h` `j` `k` `l` move focus through the theme rows as drawn, stopping at the edges, and Home and
//! End go to the first and last row. The cursor is the browser's focus on a row's button, so Enter
//! picks the theme as a click does and Esc closes the sheet as it always has. Focus starts on the
//! theme in use as the sheet opens and goes back to the opener as it closes.
//!
//! The browser gathers the rows with where each is drawn, and [`zwipe_ui::deck_cards::cursor`]
//! decides where a move lands, as it does for the deck page's cards. Only the computer layout
//! takes the keys (a mouse with hover, from 900px); elsewhere the kit's own keys stay, the up and
//! down arrows previewing the row above or below.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::deck_cards::cursor::{Cell, MOVE_KEYS, Snapshot, Stop, move_for};

use crate::keys::{PRESSED_KEY, top_sheet_guard_js, use_sheet_focus};
use crate::media::{COMPUTER_QUERY, use_computer};

/// What marks the theme sheet: the swatch dots its rows draw.
const THEME_SHEET_MARK: &str = ".theme-swatches";

/// The open theme sheet's row for the theme in use, which takes focus as the sheet opens.
const CURRENT_ROW: &str = ".bottom-sheet.show:has(.theme-swatches) .pref-row.selected";

/// Registers one sheet's cursor keys: a move key under the guard, on the computer layout, while
/// the theme sheet is the topmost open modal, sends `[key, from, cells]`, `from` the row in focus
/// (else the theme in use) by its place and each cell `[place, left, center_y, in_view]`. It
/// listens on the window before anything else hears the key, so the kit's arrow keys, which
/// preview a row, never see a move. Home and End answer only while a row has focus.
/// `__zwipeThemeFocus` focuses a row by its place and scrolls it into the sheet's view, and
/// `__zwipeThemeReveal` scrolls the row in focus into view once the sheet has given it focus.
const NAV_JS: &str = r"
window.__zwipeThemeNav = window.__zwipeThemeNav || [];
window.__zwipeThemeNav.push('__ID__');
const openSheet = () => [...document.querySelectorAll('.bottom-sheet.show')].filter((s) => s.querySelector('__MARK__')).pop();
const rowsOf = (sheet) => [...sheet.querySelectorAll('.pref-row')].filter((r) => !r.disabled);
const scroller = (row, sheet) => {
    for (let el = row.parentElement; el && el !== sheet.parentElement; el = el.parentElement) {
        const o = getComputedStyle(el).overflowY;
        if ((o === 'auto' || o === 'scroll') && el.scrollHeight > el.clientHeight) return el;
    }
    return null;
};
const reveal = (row, sheet, smooth) => {
    const box = scroller(row, sheet);
    if (!box) return;
    const b = box.getBoundingClientRect();
    const r = row.getBoundingClientRect();
    const pad = 8;
    let top = null;
    if (r.top < b.top + pad) top = box.scrollTop + r.top - b.top - pad;
    else if (r.bottom > b.bottom - pad) top = box.scrollTop + r.bottom - b.bottom + pad;
    const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (top !== null) box.scrollTo({ top, behavior: smooth && !still ? 'smooth' : 'auto' });
};
window.__zwipeThemeFocus = (at) => {
    const sheet = openSheet();
    const row = sheet && rowsOf(sheet)[at];
    if (!row) return;
    row.focus({ preventScroll: true });
    reveal(row, sheet, true);
};
window.__zwipeThemeReveal = () => {
    const until = Date.now() + 1000;
    const look = () => {
        const sheet = openSheet();
        const row = document.activeElement && document.activeElement.closest && document.activeElement.closest('.pref-row');
        if (sheet && row && sheet.contains(row)) { reveal(row, sheet, false); return; }
        if (Date.now() < until) setTimeout(look, 50);
    };
    setTimeout(look, 60);
};
const onKey = (e) => {
    const stack = window.__zwipeThemeNav;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    const key = __KEY__;
    if (!__MOVES__.includes(key) || e.shiftKey && key.startsWith('Arrow')) return;
    if (!matchMedia('__POINTER__').matches || !__GUARD__) return;
    const sheet = openSheet();
    if (!sheet) return;
    const rows = rowsOf(sheet);
    if (!rows.length) return;
    const focused = document.activeElement && document.activeElement.closest && document.activeElement.closest('.pref-row');
    const onRow = !!focused && rows.includes(focused);
    if ((key === 'Home' || key === 'End') && !onRow) return;
    e.preventDefault();
    e.stopPropagation();
    const fromEl = onRow ? focused : sheet.querySelector('.pref-row.selected');
    const from = fromEl ? rows.indexOf(fromEl) : -1;
    const box = scroller(rows[0], sheet);
    const view = box ? box.getBoundingClientRect() : { top: 0, bottom: window.innerHeight };
    const cells = rows.map((r, at) => { const b = r.getBoundingClientRect(); return [at, b.left, b.top + b.height / 2, b.bottom > view.top && b.top < view.bottom]; });
    dioxus.send([key, from < 0 ? null : from, cells]);
};
window['__zwipeThemeNav__ID__'] = onKey;
window.addEventListener('keydown', onKey, true);
";

/// Removes one sheet's cursor keys.
const NAV_DROP_JS: &str = r"
const onKey = window['__zwipeThemeNav__ID__'];
if (onKey) window.removeEventListener('keydown', onKey, true);
delete window['__zwipeThemeNav__ID__'];
window.__zwipeThemeNav = (window.__zwipeThemeNav || []).filter((id) => id !== '__ID__');
";

/// One row as the browser sends it: its place, left edge, vertical center and whether it is in
/// the sheet's view.
type SentCell = (usize, f64, f64, bool);

/// Answers the theme sheet's cursor keys, as the module describes, for the sheet `open` shows.
pub fn use_theme_cursor(open: Signal<bool>) {
    let computer = use_computer();
    let focus_open = use_memo(move || open() && computer());
    use_sheet_focus(focus_open.into(), CURRENT_ROW);
    use_effect(move || {
        if focus_open() {
            document::eval("window.__zwipeThemeReveal && window.__zwipeThemeReveal();");
        }
    });
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = nav_js(&id);
        spawn(async move {
            let mut keys = document::eval(&script);
            while let Ok((key, from, cells)) =
                keys.recv::<(String, Option<usize>, Vec<SentCell>)>().await
            {
                let Some(step) = move_for(&key) else {
                    continue;
                };
                if let Some(target) = snapshot_of(&cells).step(from, step) {
                    document::eval(&format!("window.__zwipeThemeFocus({target});"));
                }
            }
        });
    });
    use_drop(move || {
        document::eval(&NAV_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// [`NAV_JS`] for the registration `id`.
fn nav_js(id: &Uuid) -> String {
    NAV_JS
        .replace("__ID__", &id.to_string())
        .replace("__MARK__", THEME_SHEET_MARK)
        .replace("__KEY__", PRESSED_KEY)
        .replace("__MOVES__", &format!("{MOVE_KEYS:?}"))
        .replace("__POINTER__", COMPUTER_QUERY)
        .replace("__GUARD__", &top_sheet_guard_js(THEME_SHEET_MARK))
}

/// The browser's rows as a [`Snapshot`], in columns by where they are drawn.
fn snapshot_of(cells: &[SentCell]) -> Snapshot<usize> {
    let cells: Vec<Cell<usize>> = cells
        .iter()
        .map(|&(id, left, center_y, in_view)| Cell {
            stop: Stop {
                id,
                center_y,
                in_view,
            },
            left,
        })
        .collect();
    Snapshot::from_cells(&cells)
}

#[cfg(test)]
mod tests {
    use zwipe_ui::deck_cards::cursor::Move;

    use super::*;

    #[test]
    fn a_list_of_rows_steps_up_and_down_and_stops_at_its_ends() {
        let cells: Vec<SentCell> = vec![
            (0, 16.0, 20.0, true),
            (1, 16.0, 60.0, true),
            (2, 16.4, 100.0, false),
        ];
        let rows = snapshot_of(&cells);
        assert_eq!(rows.step(Some(0), Move::Down), Some(1));
        assert_eq!(rows.step(Some(2), Move::Down), None, "the last row stops");
        assert_eq!(rows.step(Some(0), Move::Up), None, "the first row stops");
        assert_eq!(rows.step(Some(1), Move::Right), None, "one column");
        assert_eq!(rows.step(Some(1), Move::Last), Some(2));
    }

    #[test]
    fn the_keys_reach_only_the_theme_sheet_on_top_on_a_computer() {
        let js = nav_js(&Uuid::nil());
        assert!(
            js.contains(&format!("matchMedia('{COMPUTER_QUERY}')")),
            "{js}"
        );
        assert!(js.contains(".querySelector('.theme-swatches')"), "{js}");
        assert!(
            js.contains("top.matches('.bottom-sheet.show')"),
            "only while the sheet is the topmost modal: {js}"
        );
        assert!(js.contains("e.shiftKey && key.startsWith('Arrow')"), "{js}");
        assert!(
            js.contains("(key === 'Home' || key === 'End') && !onRow"),
            "Home and End only from a row: {js}"
        );
    }

    #[test]
    fn a_move_never_reaches_the_kit_s_previewing_arrows() {
        let js = nav_js(&Uuid::nil());
        assert!(
            js.contains("window.addEventListener('keydown', onKey, true)"),
            "the window's capture runs before the kit's document listener: {js}"
        );
        assert!(js.contains("e.stopPropagation()"), "{js}");
        assert!(
            !js.contains("'Enter'"),
            "Enter stays the focused row's click: {js}"
        );
    }

    #[test]
    fn focus_starts_on_the_theme_in_use() {
        assert!(CURRENT_ROW.ends_with(".pref-row.selected"), "{CURRENT_ROW}");
        assert!(
            CURRENT_ROW.starts_with(".bottom-sheet.show"),
            "{CURRENT_ROW}"
        );
    }
}
