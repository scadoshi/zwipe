//! The deck cards page's keyboard cursor: the arrow keys
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
use orangui_core::page_keys::GG_WINDOW_MS;
use uuid::Uuid;
use zwipe_ui::deck_cards::cursor::{self, MOVE_KEYS, Move, Snapshot, Stop, Walk, move_for};

use crate::deck_cards_view::StackDetails;
use crate::keys::{PRESSED_KEY, STACK_DETAILS_CLASS, cursor_guard_js};

/// The featured cards in the deck head, the wide head's images and the phone strip's cards, which
/// the cursor stops on as one row above the cards ([`cursor::step_with_head`]). Each is also a
/// card in the List or Stacks, so its stop is named apart, as `head:` and its id.
pub const HEAD_SELECTOR: &str = ".deck-head-card-image[data-row], .deck-featured-card[data-row]";

/// The cards the cursor stops on: the deck page's List rows and stack cards, its featured cards
/// ([`HEAD_SELECTOR`]), and the card rows of the screens that list cards without editing them
/// ([`use_card_list_cursor`]). A deck row's own card row inside it is not a second stop.
pub const STOP_SELECTOR: &str = ".web-deck-row[data-row], .card-stack-slot[data-row], .web-card-row[data-row]:not(.web-deck-row > *), .deck-head-card-image[data-row], .deck-featured-card[data-row]";

/// Installs, once a tab, the page-wide trackers the cursor reads: the last input (`key` or
/// `mouse`), the card in focus, `__zwipeDeckFocus`, which focuses a card by id and keeps it in
/// view (at once while a key is held, `__zwipeDeckHeld`, else smoothly), and `__zwipeDeckHandOn`, which focuses the cursor's card again, or, once it has left the
/// page, the next card after it in the last reading order, else the one before. While a key moved
/// last, a page change that drops focus to the page (a redraw, an Undo, a removed card) hands it
/// back that way. `g` `g` and Shift+G, which move the page, let go of the cursor's card.
const TRACK_JS: &str = r"
if (!window.__zwipeDeckTrack) {
    window.__zwipeDeckTrack = true;
    window.__zwipeDeckInput = 'mouse';
    const stops = '__STOPS__';
    const head = '__HEAD__';
    // A featured card's stop is named apart from the same card's row or stack card.
    const spot = (s) => (s.matches(head) ? 'head:' : '') + s.getAttribute('data-row');
    let previewed = null;
    const unpreview = () => { if (previewed) { const line = previewed; previewed = null; line.dispatchEvent(new MouseEvent('mouseleave')); } };
    document.addEventListener('mousemove', (e) => { if (e.movementX || e.movementY) { window.__zwipeDeckInput = 'mouse'; unpreview(); } }, true);
    document.addEventListener('pointerdown', () => { window.__zwipeDeckInput = 'mouse'; }, true);
    document.addEventListener('focusin', (e) => {
        const stop = e.target.closest && e.target.closest(stops);
        if (stop) window.__zwipeDeckCursor = spot(stop);
        const line = e.target.closest && e.target.closest('.web-card-row .card-row-compact');
        if (line && line !== previewed && window.__zwipeCursorPreview && window.__zwipeDeckInput === 'key' && !document.querySelector('.web-card-row .card-row.expanded')) {
            unpreview();
            previewed = line;
            line.dispatchEvent(new MouseEvent('mouseenter'));
        }
    }, true);
    document.addEventListener('focusout', (e) => { if (e.target === previewed) unpreview(); }, true);
    // gg and Shift+G move the page, not the cursor: the cursor's card lets go, its id kept, so
    // the next arrow carries on from it. The key reads as the page keys read it, its case from
    // Shift alone.
    let lastG = 0;
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey || e.metaKey || e.altKey || e.repeat) return;
        const key = __KEY__;
        const now = Date.now();
        const gg = key === 'g' && now - lastG < __GG__;
        if (key === 'g') lastG = gg ? 0 : now;
        const jump = key === 'G' || gg;
        if (!jump) return;
        const at = document.activeElement;
        if (at && at.closest && at.closest(stops)) { window.__zwipeDeckInput = 'mouse'; at.blur(); }
    }, true);
    const find = (id) => [...document.querySelectorAll(stops)].find((s) => spot(s) === id);
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
/// draws cards, gathers the cards as drawn (in columns: the deck page's stacks or paired List
/// columns, else a screen's lists marked [`LIST_ATTR`] where it has more than one, else one) and
/// sends `[key, from, columns]` (a grid marked [`GRID_ATTR`] in its columns as laid out, each
/// column the cards that share a left edge), `from` the
/// card the move starts at (the hovered card when the mouse moved last, else the card in focus,
/// else the last cursor), each column `[[id, center_y, in_view], ...]`, `open`: `drawer`
/// while the details drawer shows, `row` while a List row is open, else empty, whether the
/// key repeats, the featured cards `[[spot, center_x], ...]` and each column's horizontal center. Home and End only
/// answer while a card has focus from a key.
const NAV_JS: &str = r"
window.__zwipeDeckNav = window.__zwipeDeckNav || [];
window.__zwipeDeckNav.push('__ID__');
const stops = '__STOPS__';
const headStops = '__HEAD__';
const onKey = (e) => {
    const stack = window.__zwipeDeckNav;
    if (!stack.length || stack[stack.length - 1] !== '__ID__') return;
    const key = __KEY__;
    if (!__MOVES__.includes(key) || e.shiftKey && key.startsWith('Arrow')) return;
    if (!__GUARD__) return;
    const box = (s) => (s && getComputedStyle(s).display === 'contents' && s.firstElementChild) || s;
    const all = [...document.querySelectorAll(stops)].filter((s) => box(s).offsetParent !== null && !s.closest('[inert]'));
    if (!all.length) return;
    const spot = (s) => (s.matches(headStops) ? 'head:' : '') + s.getAttribute('data-row');
    const cards = all.filter((s) => !s.matches(headStops));
    const focused = document.activeElement && document.activeElement.closest && document.activeElement.closest(stops);
    const shows = window.__zwipeDeckInput === 'key' && focused;
    if ((key === 'Home' || key === 'End') && !shows) return;
    const hovered = all.filter((s) => s.matches(':hover')).pop();
    const fromEl = window.__zwipeDeckInput === 'mouse' && hovered ? hovered : focused;
    const from = fromEl ? spot(fromEl) : (window.__zwipeDeckCursor || null);
    e.preventDefault();
    window.__zwipeDeckInput = 'key';
    window.__zwipeDeckHeld = e.repeat;
    const stacked = [...document.querySelectorAll('.deck-stacks-column')];
    const listed = document.querySelector('.deck-list-columns');
    const paired = listed && getComputedStyle(listed).display === 'grid';
    const lists = [...document.querySelectorAll('[__LIST__]')];
    const holders = stacked.length ? stacked : paired ? [...listed.querySelectorAll(':scope > .deck-list-column')] : lists.length > 1 ? lists : [];
    const grid = holders.length ? null : document.querySelector('[__GRID__]');
    const gridColumns = () => {
        const byLeft = new Map();
        for (const s of cards.filter((s) => grid.contains(s))) {
            const left = Math.round(box(s).getBoundingClientRect().left);
            if (!byLeft.has(left)) byLeft.set(left, []);
            byLeft.get(left).push(s);
        }
        return [...byLeft.entries()].sort((a, b) => a[0] - b[0]).map((entry) => entry[1]);
    };
    const laid = grid ? gridColumns() : null;
    const height = window.innerHeight;
    const as = (s) => { const r = box(s).getBoundingClientRect(); return [s.getAttribute('data-row'), r.top + r.height / 2, r.bottom > 0 && r.top < height]; };
    const columns = laid ? laid.map((column) => column.map(as)) : holders.length ? holders.map((h) => cards.filter((s) => h.contains(s)).map(as)) : [cards.map(as)];
    const centerX = (el) => { const r = box(el).getBoundingClientRect(); return r.left + r.width / 2; };
    const columnX = laid ? laid.map((column) => centerX(column[0])) : holders.length ? holders.map(centerX) : [cards.length ? centerX(cards[0]) : 0];
    const head = all.filter((s) => s.matches(headStops)).map((s) => [spot(s), centerX(s)]);
    window.__zwipeDeckOrder = columns.flat().map((s) => s[0]);
    const open = document.querySelector('.bottom-sheet.show.__DRAWER__') ? 'drawer' : document.querySelector('.web-deck-row.selected, .web-card-row .card-row.expanded') ? 'row' : '';
    dioxus.send([key, from, columns, open, e.repeat, head, columnX]);
};
window['__zwipeDeckNav__ID__'] = onKey;
document.addEventListener('keydown', onKey);
";

/// The attribute that marks one list of a screen of card lists side by side (Playstyle's
/// Suggested commanders and Staples): the cursor takes each as a column and walks them as
/// [`Walk::Lists`], at any width, stacked or side by side.
pub const LIST_ATTR: &str = "data-cursor-list";

/// The attribute that marks a grid of cards laid out in rows (the card search's results): the
/// cursor takes the cards that share a left edge as a column and walks them as [`Walk::Grid`].
pub const GRID_ATTR: &str = "data-cursor-grid";

/// Installs the hand-back the card search box calls as focus leaves it by Escape or `↓`
/// ([`HAND_BACK_JS`]): the cursor's card takes focus again, else the first card in view, and the
/// keys act on it.
const HOME_JS: &str = r"
window.__zwipeCursorHome = () => {
    window.__zwipeDeckInput = 'key';
    if (window.__zwipeDeckHandOn && window.__zwipeDeckHandOn()) return;
    const height = window.innerHeight;
    const first = [...document.querySelectorAll('__STOPS__')].find((s) => {
        const r = s.getBoundingClientRect();
        return s.offsetParent !== null && !s.closest('[inert]') && r.bottom > 0 && r.top < height;
    });
    if (first && window.__zwipeDeckFocus) window.__zwipeDeckFocus(first.getAttribute('data-row'));
};
";

/// Removes the hand-back.
const HOME_DROP_JS: &str = "delete window.__zwipeCursorHome;";

/// Hands the keyboard cursor back to the page's cards where the page has a hand-back
/// ([`use_card_grid_cursor`]), and reports whether it did, for the card search box as Escape or
/// `↓` lets it go.
pub(crate) const HAND_BACK_JS: &str =
    "if (window.__zwipeCursorHome) { window.__zwipeCursorHome(); true } else { false }";

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

/// Moves the cursor from card `__FROM__` to card `__TO__` where it sits on `__FROM__` or on its
/// featured card, in the last reading order too, so a redraw hands focus to `__TO__`.
const FOLLOW_JS: &str = "if (window.__zwipeDeckCursor === '__FROM__') window.__zwipeDeckCursor = '__TO__'; if (window.__zwipeDeckCursor === 'head:__FROM__') window.__zwipeDeckCursor = 'head:__TO__'; window.__zwipeDeckOrder = (window.__zwipeDeckOrder || []).map((id) => id === '__FROM__' ? '__TO__' : id);";

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

/// When the cursor's new List row opens, while a row is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowOpen {
    /// At once: a single press.
    Now,
    /// Once the key rests ([`ROW_SETTLE_MS`]): a held key's repeats only move the cursor, since
    /// each row opening lays out and paints the whole list again.
    Settled,
}

impl RowOpen {
    /// When a key that repeats or not (`held`) opens its row.
    fn of(held: bool) -> Self {
        if held { Self::Settled } else { Self::Now }
    }
}

/// How long a held key rests before its row opens, in milliseconds: a few repeats' worth.
#[cfg(target_arch = "wasm32")]
const ROW_SETTLE_MS: u32 = 150;

/// Opens list row `__ID__` where it is shut, as a click on its line does, which shuts the open one.
const OPEN_ROW_JS: &str = "const row = document.querySelector('.web-deck-row[data-row=\"__ID__\"], .web-card-row[data-row=\"__ID__\"]'); if (row && !row.classList.contains('selected') && !row.querySelector('.card-row.expanded')) row.querySelector('.card-row-compact')?.click();";

/// [`TRACK_JS`] with the cursor's stops, the pressed key and the page keys' gg window filled in.
fn track_js() -> String {
    TRACK_JS
        .replace("__STOPS__", STOP_SELECTOR)
        .replace("__HEAD__", HEAD_SELECTOR)
        .replace("__KEY__", PRESSED_KEY)
        .replace("__GG__", &GG_WINDOW_MS.to_string())
}

/// Marks the screen as one whose cursor card shows its image as hover does.
const PREVIEW_ON_JS: &str = "window.__zwipeCursorPreview = true;";

/// Unmarks it.
const PREVIEW_OFF_JS: &str = "window.__zwipeCursorPreview = false;";

/// The cursor on a screen that lists cards without editing them (the Commander maybeboard,
/// Activity's Most skipped, Playstyle's lists): the same keys as the deck page over its card rows,
/// in one column, or over each list marked [`LIST_ATTR`] as [`Walk::Lists`] walks them, so `h` and
/// `l` cross to the same place in the other list and `j` and `k` run on from one list into the
/// next. Enter opens or closes a row, an open row follows the cursor, and, when the last
/// input was a key, the cursor's card shows its image in the screen's hover stack as hovering its
/// row does. While a row is open none shows, as an open row shows none, so a move that opens the
/// next row in its place leaves no image behind.
pub fn use_card_list_cursor() {
    use_hook(|| {
        document::eval(PREVIEW_ON_JS);
    });
    use_drop(|| {
        document::eval(PREVIEW_OFF_JS);
    });
    use_deck_cursor(None, Walk::Lists);
}

/// The cursor on a grid of whole cards marked [`GRID_ATTR`] (the card search's results): the
/// deck page's keys over the grid as laid out at this width ([`Walk::Grid`]), Enter opening the
/// card as a click does. The card search box hands the cursor back as Escape or `↓` lets it go.
pub fn use_card_grid_cursor() {
    use_hook(|| {
        document::eval(&HOME_JS.replace("__STOPS__", STOP_SELECTOR));
    });
    use_drop(|| {
        document::eval(HOME_DROP_JS);
    });
    use_deck_cursor(None, Walk::Grid);
}

/// One card as the browser sends it: its id, vertical center and whether it is in view.
type SentStop = (String, f64, bool);

/// One featured card as the browser sends it: its stop's name ([`Spot`]) and horizontal center.
type SentHead = (String, f64);

/// A stop the cursor lands on: a card in the List or Stacks, or a featured card in the deck head,
/// named apart since a featured card is also one of the cards. The browser writes a featured
/// card's stop as `head:` and its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spot {
    /// A card in the List or Stacks, by its Scryfall id.
    Card(Uuid),
    /// A featured card in the deck head, by its Scryfall id.
    Head(Uuid),
}

impl Spot {
    /// The card the stop shows.
    fn card(self) -> Uuid {
        match self {
            Self::Card(id) | Self::Head(id) => id,
        }
    }
}

impl std::fmt::Display for Spot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Card(id) => write!(f, "{id}"),
            Self::Head(id) => write!(f, "head:{id}"),
        }
    }
}

impl std::str::FromStr for Spot {
    type Err = uuid::Error;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name.strip_prefix("head:") {
            Some(id) => Uuid::try_parse(id).map(Self::Head),
            None => Uuid::try_parse(name).map(Self::Card),
        }
    }
}

/// Answers the deck page's cursor keys, as the module describes, moving as `walk` says. `details`
/// is the page's Stacks reveal and drawer, which follows the cursor while it is open.
pub(crate) fn use_deck_cursor(details: Option<StackDetails>, walk: Walk) {
    let id = use_hook(Uuid::new_v4);
    let mut order = use_signal(Vec::<Spot>::new);
    // Each row move, so a settled opening that a later move overtook does nothing.
    let row_moves = use_signal(|| 0u32);
    use_hook(move || {
        document::eval(&track_js());
        let script = NAV_JS
            .replace("__ID__", &id.to_string())
            .replace("__STOPS__", STOP_SELECTOR)
            .replace("__HEAD__", HEAD_SELECTOR)
            .replace("__LIST__", LIST_ATTR)
            .replace("__GRID__", GRID_ATTR)
            .replace("__KEY__", PRESSED_KEY)
            .replace("__MOVES__", &format!("{MOVE_KEYS:?}"))
            .replace("__GUARD__", &cursor_guard_js(STACK_DETAILS_CLASS))
            .replace("__DRAWER__", STACK_DETAILS_CLASS);
        spawn(async move {
            let mut keys = document::eval(&script);
            while let Ok((key, from, columns, open, held, heads, column_x)) = keys
                .recv::<(
                    String,
                    Option<String>,
                    Vec<Vec<SentStop>>,
                    String,
                    bool,
                    Vec<SentHead>,
                    Vec<f64>,
                )>()
                .await
            {
                let Some(step) = move_for(&key) else {
                    continue;
                };
                let snapshot = snapshot_of(&columns);
                let head = cursor::Head {
                    column_x,
                    ..head_of(&heads)
                };
                let from: Option<Spot> = from.and_then(|from| from.parse().ok());
                let Some(target) = land(&snapshot, &head, from, step, walk, &order.peek()) else {
                    continue;
                };
                order.set(snapshot.order());
                document::eval(&format!(
                    "window.__zwipeDeckFocus('{target}', window.__zwipeDeckHeld);"
                ));
                match (Follow::of(&open), target) {
                    (Follow::Drawer, _) => {
                        let card = target.card();
                        if let Some(details) = details
                            && details.reveal.peek().open() != Some(card)
                        {
                            details.click(card);
                        }
                    }
                    (Follow::Row, Spot::Card(card)) => {
                        open_row(row_moves, card, RowOpen::of(held));
                    }
                    // A featured card has no row to open.
                    (Follow::Row, Spot::Head(_)) | (Follow::Nothing, _) => {}
                }
            }
        });
    });
    use_drop(move || {
        document::eval(&NAV_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// Opens List row `target` as `when` says: at once, or once no later row move has come for
/// [`ROW_SETTLE_MS`]. Off the browser nothing is timed and it opens at once.
fn open_row(mut moves: Signal<u32>, target: Uuid, when: RowOpen) {
    let count = moves.peek().wrapping_add(1);
    moves.set(count);
    let open = move || {
        document::eval(&OPEN_ROW_JS.replace("__ID__", &target.to_string()));
    };
    match when {
        RowOpen::Now => open(),
        #[cfg(target_arch = "wasm32")]
        RowOpen::Settled => {
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(ROW_SETTLE_MS).await;
                if *moves.peek() == count {
                    open();
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        RowOpen::Settled => open(),
    }
}

/// The browser's columns as a [`Snapshot`], dropping any card whose id doesn't parse.
fn snapshot_of(columns: &[Vec<SentStop>]) -> Snapshot<Spot> {
    Snapshot {
        columns: columns
            .iter()
            .map(|column| {
                column
                    .iter()
                    .filter_map(|(id, center_y, in_view)| {
                        Uuid::try_parse(id).ok().map(|id| Stop {
                            id: Spot::Card(id),
                            center_y: *center_y,
                            in_view: *in_view,
                        })
                    })
                    .collect()
            })
            .collect(),
    }
}

/// The browser's featured cards as the row above the columns, left to right as drawn, keeping
/// only the stops named as featured ([`Spot::Head`]); the columns' centers come separately.
fn head_of(stops: &[SentHead]) -> cursor::Head<Spot> {
    cursor::Head {
        stops: stops
            .iter()
            .filter_map(|(name, center_x)| match name.parse() {
                Ok(id @ Spot::Head(_)) => Some(cursor::HeadStop {
                    id,
                    center_x: *center_x,
                }),
                Ok(Spot::Card(_)) | Err(_) => None,
            })
            .collect(),
        column_x: Vec::new(),
    }
}

/// Where `step` from `from` lands: from a card no longer drawn, the card after it in `order` (the
/// last reading order) that still is, else the one before; else as [`cursor::step_with_head`]
/// moves as `walk`, the featured cards in `head` a row above the columns.
fn land(
    snapshot: &Snapshot<Spot>,
    head: &cursor::Head<Spot>,
    from: Option<Spot>,
    step: Move,
    walk: Walk,
    order: &[Spot],
) -> Option<Spot> {
    let drawn =
        |id: &Spot| snapshot.order().contains(id) || head.stops.iter().any(|stop| stop.id == *id);
    if let Some(gone) = from.filter(|id| !drawn(id))
        && let Some(next) = snapshot.hand_on(gone, order)
    {
        return Some(next);
    }
    cursor::step_with_head(snapshot, head, from, step, walk)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn card(n: u128) -> Spot {
        Spot::Card(id(n))
    }

    fn sent(n: u128, y: f64) -> SentStop {
        (id(n).to_string(), y, true)
    }

    #[test]
    fn the_featured_cards_are_stops_named_apart_from_their_list_rows() {
        for featured in [
            ".deck-head-card-image[data-row]",
            ".deck-featured-card[data-row]",
        ] {
            assert!(HEAD_SELECTOR.contains(featured), "{featured}");
            assert!(STOP_SELECTOR.contains(featured), "{featured}");
        }
        assert_eq!(
            "head:00000000-0000-0000-0000-000000000007".parse::<Spot>(),
            Ok(Spot::Head(id(7)))
        );
        assert_eq!(id(7).to_string().parse::<Spot>(), Ok(Spot::Card(id(7))));
        assert_eq!(Spot::Head(id(7)).to_string(), format!("head:{}", id(7)));
        assert!("head:nope".parse::<Spot>().is_err());
        assert!(
            TRACK_JS.contains("const spot = (s) =>"),
            "the browser names a featured stop apart from the same card's row: {TRACK_JS}"
        );
    }

    #[test]
    fn the_keys_send_the_featured_cards_and_each_column_s_center() {
        assert!(
            NAV_JS.contains("dioxus.send([key, from, columns, open, e.repeat, head, columnX]);"),
            "{NAV_JS}"
        );
        let head = head_of(&[
            (format!("head:{}", id(9)), 120.0),
            (id(3).to_string(), 50.0),
        ]);
        assert_eq!(
            head.stops,
            vec![cursor::HeadStop {
                id: Spot::Head(id(9)),
                center_x: 120.0
            }],
            "only head stops make the row"
        );
    }

    #[test]
    fn a_held_key_opens_its_row_only_once_it_settles() {
        assert!(
            NAV_JS.contains("dioxus.send([key, from, columns, open, e.repeat, head, columnX]);"),
            "the keys say whether they repeat: {NAV_JS}"
        );
        assert_eq!(
            RowOpen::of(false),
            RowOpen::Now,
            "a single press opens it at once"
        );
        assert_eq!(
            RowOpen::of(true),
            RowOpen::Settled,
            "a repeat waits for the key to rest"
        );
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
        assert_eq!(snapshot.order(), vec![card(1), card(2)]);
    }

    #[test]
    fn a_move_from_a_card_that_left_lands_on_the_next_in_the_old_order() {
        let snapshot = snapshot_of(&[vec![sent(1, 10.0), sent(3, 50.0)]]);
        let order = vec![card(1), card(2), card(3)];
        assert_eq!(
            land(
                &snapshot,
                &cursor::Head::default(),
                Some(card(2)),
                Move::Down,
                Walk::Columns,
                &order
            ),
            Some(card(3))
        );
        assert_eq!(
            land(
                &snapshot,
                &cursor::Head::default(),
                Some(card(1)),
                Move::Down,
                Walk::Columns,
                &order
            ),
            Some(card(3)),
            "a card still drawn moves as usual"
        );
    }

    #[test]
    fn a_screen_s_marked_lists_are_its_columns_and_walk_as_lists() {
        assert!(
            NAV_JS.contains("const lists = [...document.querySelectorAll('[__LIST__]')];"),
            "{NAV_JS}"
        );
        assert!(
            NAV_JS.contains(": lists.length > 1 ? lists : [];"),
            "two lists or more are columns, at any width; one list is one column: {NAV_JS}"
        );
        let snapshot = snapshot_of(&[
            vec![sent(1, 10.0), sent(2, 50.0), sent(3, 90.0)],
            vec![sent(11, 30.0)],
        ]);
        assert_eq!(
            land(
                &snapshot,
                &cursor::Head::default(),
                Some(card(3)),
                Move::Right,
                Walk::Lists,
                &[]
            ),
            Some(card(11)),
            "the third commander to the only staple"
        );
        assert_eq!(
            land(
                &snapshot,
                &cursor::Head::default(),
                Some(card(3)),
                Move::Down,
                Walk::Lists,
                &[]
            ),
            Some(card(11)),
            "the last commander on to the first staple"
        );
    }

    #[test]
    fn a_marked_grid_is_walked_in_its_columns_as_laid_out() {
        assert!(
            NAV_JS.contains(
                "const grid = holders.length ? null : document.querySelector('[__GRID__]');"
            ),
            "{NAV_JS}"
        );
        assert!(
            NAV_JS.contains("const left = Math.round(box(s).getBoundingClientRect().left);"),
            "a column is the cards that share a left edge: {NAV_JS}"
        );
        assert!(
            NAV_JS.contains("sort((a, b) => a[0] - b[0])"),
            "columns left to right: {NAV_JS}"
        );
        let snapshot = snapshot_of(&[
            vec![sent(1, 10.0), sent(4, 90.0)],
            vec![sent(2, 10.0)],
            vec![sent(3, 10.0)],
        ]);
        assert_eq!(
            land(
                &snapshot,
                &cursor::Head::default(),
                Some(card(1)),
                Move::Right,
                Walk::Grid,
                &[]
            ),
            Some(card(2))
        );
        assert_eq!(
            land(
                &snapshot,
                &cursor::Head::default(),
                Some(card(3)),
                Move::Down,
                Walk::Grid,
                &[]
            ),
            Some(card(4)),
            "down from a column with nothing below lands on the next row's last card"
        );
    }

    #[test]
    fn the_hand_back_focuses_the_cursor_card_else_the_first_in_view() {
        assert!(
            HOME_JS.contains("if (window.__zwipeDeckHandOn && window.__zwipeDeckHandOn()) return;"),
            "{HOME_JS}"
        );
        assert!(
            HOME_JS.contains("window.__zwipeDeckInput = 'key';"),
            "the keys act on it: {HOME_JS}"
        );
        assert!(
            HAND_BACK_JS.contains("if (window.__zwipeCursorHome)"),
            "a page with no hand-back keeps the box's own keys: {HAND_BACK_JS}"
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
    fn a_page_jump_lets_go_of_the_cursor_card() {
        let script = track_js();
        assert!(
            script.contains("const jump = key === 'G' || gg;"),
            "Shift+G and gg: {script}"
        );
        assert!(
            script.contains(&format!("now - lastG < {GG_WINDOW_MS}")),
            "the page keys' gg window: {script}"
        );
        assert!(
            script.contains("window.__zwipeDeckInput = 'mouse'; at.blur();"),
            "the card lets go and stays let go, as the cursor's hand-back only follows a key: {script}"
        );
        assert!(
            !script.contains("__STOPS__")
                && !script.contains("__GG__")
                && !script.contains("__KEY__"),
            "every placeholder filled: {script}"
        );
    }

    #[test]
    fn a_page_jump_reads_the_key_as_the_page_keys_do() {
        let script = track_js();
        assert!(
            script.contains(&format!("const key = {PRESSED_KEY};")),
            "a Shift+G whose KeyboardEvent.key comes lowercase (Caps Lock) is still Shift+G, as the page keys that scroll read it: {script}"
        );
        assert!(
            !script.contains("e.key ==="),
            "no raw KeyboardEvent.key comparison left: {script}"
        );
    }

    #[test]
    fn a_featured_card_that_changes_id_keeps_the_cursor_on_the_featured_card() {
        let js = follow_js(id(1), id(2));
        assert!(
            js.contains(&format!(
                "if (window.__zwipeDeckCursor === 'head:{}') window.__zwipeDeckCursor = 'head:{}';",
                id(1),
                id(2)
            )),
            "{js}"
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
    fn the_cursor_shows_no_image_while_a_row_is_open() {
        assert!(
            TRACK_JS.contains("!document.querySelector('.web-card-row .card-row.expanded')"),
            "an open row shows no image, and the next row opens in its place: {TRACK_JS}"
        );
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
