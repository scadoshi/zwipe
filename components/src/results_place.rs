//! Where the card search's results page was, kept for Back: each text's loaded pages
//! ([`Kept`]) and how far its list was scrolled, so returning from a card opens on the same
//! cards at the same place, the keyboard cursor on its card. The last [`KEPT_TEXTS`] texts are
//! kept for the visit, in memory only.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::screens::card_results::Kept;

/// How many texts' results are kept at once; the oldest goes first.
pub const KEPT_TEXTS: usize = 5;

/// One text's results and the list's scroll offset, in pixels from its top.
#[derive(Debug, Clone, PartialEq)]
pub struct KeptPlace {
    /// The results, every page loaded.
    pub kept: Kept,
    /// How far the list was scrolled.
    pub scroll: f64,
}

/// The kept places, oldest first.
static KEPT: GlobalSignal<Vec<KeptPlace>> = Signal::global(Vec::new);

/// The place kept for `query`, if any.
#[must_use]
pub fn kept_for(query: &str) -> Option<KeptPlace> {
    let query = query.trim();
    KEPT.peek()
        .iter()
        .find(|place| place.kept.query() == query)
        .cloned()
}

/// Keeps `place`, in place of any place for the same text, dropping the oldest past
/// [`KEPT_TEXTS`].
pub fn keep(place: KeptPlace) {
    keep_in(&mut KEPT.write(), place);
}

/// [`keep`] over `places`.
fn keep_in(places: &mut Vec<KeptPlace>, place: KeptPlace) {
    places.retain(|kept| kept.kept.query() != place.kept.query());
    places.push(place);
    let over = places.len().saturating_sub(KEPT_TEXTS);
    places.drain(..over);
}

/// The class of the grid whose box's scroll is kept.
const GRID: &str = ".card-list-results";

/// Finds the box that scrolls the results grid (the window's scroller where none does), and
/// sends its scroll offset as it scrolls, at most every 150ms and once more as it stops. A grid
/// drawn again is found again.
const WATCH_JS: &str = r"
const scroller = (el) => {
    for (let box = el.parentElement; box; box = box.parentElement) {
        const overflow = getComputedStyle(box).overflowY;
        if (overflow === 'auto' || overflow === 'scroll') return box;
    }
    return document.scrollingElement;
};
let at = null;
let last = 0;
let trailing = null;
const send = () => {
    last = Date.now();
    dioxus.send(at === document.scrollingElement ? window.scrollY : at.scrollTop);
};
// At most every 150ms, and once more as scrolling stops, so the last offset is the one kept.
const onScroll = () => {
    clearTimeout(trailing);
    const wait = 150 - (Date.now() - last);
    if (wait <= 0) send();
    else trailing = setTimeout(send, wait);
};
const watch = () => {
    const grid = document.querySelector('__GRID__');
    if (!grid) return;
    const box = scroller(grid);
    if (box === at) return;
    const from = at === document.scrollingElement ? window : at;
    if (from) from.removeEventListener('scroll', onScroll);
    at = box;
    (at === document.scrollingElement ? window : at).addEventListener('scroll', onScroll, { passive: true });
};
watch();
window['__zwipePlaceWatch__KEY__'] = setInterval(watch, 300);
window['__zwipePlaceStop__KEY__'] = () => {
    clearInterval(window['__zwipePlaceWatch__KEY__']);
    if (at) (at === document.scrollingElement ? window : at).removeEventListener('scroll', onScroll);
};
";

/// Stops watching.
const WATCH_DROP_JS: &str = "const stop = window['__zwipePlaceStop__KEY__']; if (stop) stop(); delete window['__zwipePlaceStop__KEY__']; delete window['__zwipePlaceWatch__KEY__'];";

/// Scrolls the results grid's box back to `__Y__` once the grid is tall enough to reach it, for
/// up to two seconds while its images lay out, then puts the keyboard cursor back on its card
/// where a key moved last.
const RESTORE_JS: &str = r"
const y = __Y__;
const scroller = (el) => {
    for (let box = el.parentElement; box; box = box.parentElement) {
        const overflow = getComputedStyle(box).overflowY;
        if (overflow === 'auto' || overflow === 'scroll') return box;
    }
    return document.scrollingElement;
};
const started = Date.now();
const settle = () => {
    const grid = document.querySelector('__GRID__');
    if (grid) {
        const box = scroller(grid);
        const reach = box.scrollHeight - box.clientHeight;
        box.scrollTop = Math.min(y, Math.max(reach, 0));
        if (reach >= y || Date.now() - started > 2000) {
            const id = window.__zwipeDeckCursor;
            if (window.__zwipeDeckInput === 'key' && id && window.__zwipeDeckFocus) {
                const card = document.querySelector('.card-stack-slot[data-row=' + JSON.stringify(id) + '] .card-stack-card');
                if (card) card.focus({ preventScroll: true });
            }
            return;
        }
    }
    if (Date.now() - started <= 2000) requestAnimationFrame(settle);
};
requestAnimationFrame(settle);
";

/// Calls `on_scroll` with the results list's scroll offset as it scrolls, while the caller is
/// mounted.
pub fn use_results_scroll(on_scroll: Callback<f64>) {
    let key = use_hook(Uuid::new_v4).simple().to_string();
    let watch_key = key.clone();
    use_hook(move || {
        let script = WATCH_JS
            .replace("__GRID__", GRID)
            .replace("__KEY__", &watch_key);
        spawn(async move {
            let mut scrolls = document::eval(&script);
            while let Ok(y) = scrolls.recv::<f64>().await {
                on_scroll.call(y);
            }
        });
    });
    use_drop(move || {
        document::eval(&WATCH_DROP_JS.replace("__KEY__", &key));
    });
}

/// Scrolls the results list back to `scroll` and the keyboard cursor back onto its card, as
/// Back from a card finds them.
pub fn restore_place(scroll: f64) {
    document::eval(&restore_js(scroll));
}

/// [`RESTORE_JS`] for `scroll`.
fn restore_js(scroll: f64) -> String {
    let y = if scroll.is_finite() {
        scroll.max(0.0)
    } else {
        0.0
    };
    RESTORE_JS
        .replace("__Y__", &format!("{y:.0}"))
        .replace("__GRID__", GRID)
}

#[cfg(test)]
mod tests {
    use zwipe_ui::screens::card_results::{Action, Effect, State};

    use super::*;

    /// Results kept for `query`: a page of one card, the lookup spent.
    fn kept(query: &str) -> Kept {
        let mut state = State::new();
        let effects = state.update(Action::Load(query.to_owned()));
        let Some(Effect::Lookup { ticket, .. }) = effects.first() else {
            panic!("a lookup: {effects:?}");
        };
        let card: zwipe_core::domain::card::Card =
            serde_json::from_str(include_str!("../../zwipe-ui/src/test_support/card.json"))
                .unwrap();
        let mut other = card.clone();
        other.scryfall_data.id = Uuid::new_v4();
        let _ = state.update(Action::Loaded {
            ticket: *ticket,
            cards: vec![card, other],
        });
        state.kept().unwrap()
    }

    #[test]
    fn a_text_keeps_one_place_and_the_oldest_goes_past_the_cap() {
        let mut places = Vec::new();
        keep_in(
            &mut places,
            KeptPlace {
                kept: kept("a"),
                scroll: 100.0,
            },
        );
        keep_in(
            &mut places,
            KeptPlace {
                kept: kept("a"),
                scroll: 900.0,
            },
        );
        assert_eq!(places.len(), 1, "one place per text");
        assert!(
            (places[0].scroll - 900.0).abs() < f64::EPSILON,
            "the newest"
        );
        for query in ["b", "c", "d", "e", "f"] {
            keep_in(
                &mut places,
                KeptPlace {
                    kept: kept(query),
                    scroll: 0.0,
                },
            );
        }
        assert_eq!(places.len(), KEPT_TEXTS);
        assert!(
            places.iter().all(|place| place.kept.query() != "a"),
            "the oldest went"
        );
    }

    #[test]
    fn the_restore_waits_for_the_grid_to_reach_the_place_then_puts_the_cursor_back() {
        let js = restore_js(1234.4);
        assert!(js.contains("const y = 1234;"), "{js}");
        assert!(js.contains("if (reach >= y || Date.now() - started > 2000) {"));
        assert!(js.contains("card.focus({ preventScroll: true })"));
        assert!(
            js.contains("window.__zwipeDeckInput === 'key'"),
            "only after a key"
        );
        assert!(restore_js(f64::NAN).contains("const y = 0;"));
    }

    #[test]
    fn the_watch_finds_the_box_that_scrolls_the_grid() {
        assert!(WATCH_JS.contains("const overflow = getComputedStyle(box).overflowY;"));
        assert!(
            WATCH_JS.contains("else trailing = setTimeout(send, wait);"),
            "the offset as scrolling stops is the one kept"
        );
        assert!(WATCH_DROP_JS.contains("stop()"));
    }
}
