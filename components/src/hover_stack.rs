//! The hover preview a card list shows a mouse: one full-art card pinned to
//! a top corner.
//!
//! Hovering a collapsed [`CardRow`](crate::CardRow) shows the face it shows
//! and holds it while the cursor stays on the row. Entering another row
//! swaps the card at once; leaving the row fades it out over
//! `PREVIEW_FADE_MS`, and a row that scrolls out from under a still cursor
//! counts as left. An expanded row shows none. The card sits in the corner
//! away from the cursor.
//!
//! A screen calls [`use_hover_stack`], mounts [`HoverStackHost`] outside
//! any transformed ancestor and calls [`HoverStack::track`] from its page's
//! `onmousemove`; its rows find the stack by context. The preview is
//! hover-only, so off the browser nothing in it is timed: a card that is
//! let go drops at once.

use std::cell::Cell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use dioxus::core::Runtime;
use dioxus::prelude::*;

/// A card's fade out, the `hover-stack-out` animation in `magic.css`.
#[cfg(target_arch = "wasm32")]
const PREVIEW_FADE_MS: u32 = 400;

/// Sends a hovered row a `mouseleave` when a scroll moves it out from under
/// a still cursor, which the browser does not do itself. Installed once per
/// page; the row's own `onmouseleave` then lets its card go.
const SCROLL_LEAVE_JS: &str = r"
if (!window.__zwipeHoverScroll) {
    window.__zwipeHoverScroll = true;
    let x = 0, y = 0, row = null, queued = false;
    const rowOf = (el) => (el instanceof Element ? el.closest('.card-row-compact') : null);
    document.addEventListener('mousemove', (event) => {
        x = event.clientX; y = event.clientY; row = rowOf(event.target);
    }, { capture: true, passive: true });
    document.addEventListener('scroll', () => {
        if (!row || queued) return;
        queued = true;
        requestAnimationFrame(() => {
            queued = false;
            const under = rowOf(document.elementFromPoint(x, y));
            if (row && under !== row) {
                const left = row; row = under;
                left.dispatchEvent(new MouseEvent('mouseleave', { clientX: x, clientY: y }));
            }
        });
    }, { capture: true, passive: true });
}
";

/// Where the card is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoverPhase {
    /// The cursor is still on the row that showed it.
    Held,
    /// Let go and fading out (`PREVIEW_FADE_MS`).
    Leaving,
}

/// The card on show.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HoverEntry {
    /// Unique per hover, so a late leave or fade never touches a newer card.
    id: u64,
    /// The image the card shows.
    url: String,
    /// Where the card is in its life.
    phase: HoverPhase,
}

/// Lets go of the held card `id`, true when it was the one held, so its fade
/// starts once.
fn release(entry: &mut Option<HoverEntry>, id: u64) -> bool {
    match entry {
        Some(e) if e.id == id && e.phase == HoverPhase::Held => {
            e.phase = HoverPhase::Leaving;
            true
        }
        _ => false,
    }
}

/// Drops the card when it is still `id`.
fn remove(entry: &mut Option<HoverEntry>, id: u64) {
    if entry.as_ref().is_some_and(|e| e.id == id) {
        *entry = None;
    }
}

/// Whether the card belongs in the right corner: while the cursor works
/// the window's left half. `None` before the window's width is known.
fn pins_right(x: f64, width: f64) -> Option<bool> {
    (width > 0.0).then_some(x < width / 2.0)
}

/// A screen's hover preview: the one card previewed and the corner it
/// pins to, which [`HoverStackHost`] draws outside the rows. The page's
/// pointer moves go to [`track`](Self::track), and a row that starts to drag
/// fades the preview through [`leave_held`](Self::leave_held).
/// `Copy`; from [`use_hover_stack`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverStack {
    entry: Signal<Option<HoverEntry>>,
    next_id: Signal<u64>,
    on_right: Signal<bool>,
    width: Signal<f64>,
    host: ScopeId,
}

impl HoverStack {
    /// Shows a hovered row's image in place of any card on show, held until
    /// [`Self::leave`] with the id it returns.
    pub(crate) fn enter(self, url: String) -> u64 {
        let id = self.take_id();
        let mut entry = self.entry;
        entry.set(Some(HoverEntry {
            id,
            url,
            phase: HoverPhase::Held,
        }));
        id
    }

    /// Fades out the held card `id`; nothing if it is not the one held.
    pub(crate) fn leave(self, id: u64) {
        let mut entry = self.entry;
        let released = entry.try_write().is_ok_and(|mut e| release(&mut e, id));
        if released {
            self.retire(id);
        }
    }

    /// Fades out the held card, as when a row starts to drag.
    pub fn leave_held(self) {
        let held = self
            .entry
            .peek()
            .as_ref()
            .filter(|e| e.phase == HoverPhase::Held)
            .map(|e| e.id);
        if let Some(id) = held {
            self.leave(id);
        }
    }

    /// Moves the card away from the cursor at window x `x`, writing only
    /// on a change.
    pub fn track(self, x: f64) {
        let Some(right) = pins_right(x, *self.width.peek()) else {
            return;
        };
        let mut on_right = self.on_right;
        if *on_right.peek() != right {
            on_right.set(right);
        }
    }

    /// A fresh id.
    fn take_id(self) -> u64 {
        let mut next_id = self.next_id;
        let id = *next_id.peek();
        next_id.set(id.wrapping_add(1));
        id
    }

    /// Drops `id` once its fade has run. The timer runs in the host's scope,
    /// so a row that unmounts meanwhile still clears its card.
    #[cfg(target_arch = "wasm32")]
    fn retire(self, id: u64) {
        let mut entry = self.entry;
        Runtime::current().spawn(self.host, async move {
            gloo_timers::future::TimeoutFuture::new(PREVIEW_FADE_MS).await;
            if let Ok(mut e) = entry.try_write() {
                remove(&mut e, id);
            }
        });
    }

    /// Off the browser nothing is timed: the card drops at once.
    #[cfg(not(target_arch = "wasm32"))]
    fn retire(self, id: u64) {
        let mut entry = self.entry;
        if let Ok(mut e) = entry.try_write() {
            remove(&mut e, id);
        }
    }
}

/// Creates the screen's hover preview and provides it to its rows. On the
/// browser it reads the window's width once, for [`HoverStack::track`], and
/// installs the scroll check that lets a card go when its row scrolls away.
pub fn use_hover_stack() -> HoverStack {
    let width = use_signal(|| 0.0_f64);
    let host = use_hook(dioxus::core::current_scope_id);
    let stack = HoverStack {
        entry: use_signal(|| None),
        next_id: use_signal(|| 0),
        on_right: use_signal(|| false),
        width,
        host,
    };
    use_hook(|| {
        document::eval(SCROLL_LEAVE_JS);
    });
    use_future(move || async move {
        #[cfg(target_arch = "wasm32")]
        {
            let mut width = width;
            let mut eval = document::eval("dioxus.send(window.innerWidth)");
            if let Ok(w) = eval.recv::<f64>().await {
                width.set(w);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = width;
    });
    use_context_provider(|| stack)
}

/// The card on show. Mount it outside any transformed ancestor, where
/// `position: fixed` means the window.
#[component]
pub fn HoverStackHost(stack: HoverStack) -> Element {
    let Some(entry) = (stack.entry)() else {
        return rsx! {};
    };
    rsx! {
        div { class: if (stack.on_right)() { "hover-stack pin-right" } else { "hover-stack" },
            div {
                key: "{entry.id}",
                class: if entry.phase == HoverPhase::Leaving { "hover-stack-card leaving" } else { "hover-stack-card" },
                img { src: "{entry.url}", alt: "Card preview" }
            }
        }
    }
}

/// The entry a row holds while hovered, kept outside the row's signals so
/// the row's drop can still read it.
pub(crate) type HeldEntry = Rc<Cell<Option<u64>>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn held(id: u64) -> HoverEntry {
        HoverEntry {
            id,
            url: format!("card-{id}"),
            phase: HoverPhase::Held,
        }
    }

    #[test]
    fn a_held_card_is_let_go_once() {
        let mut entry = Some(held(3));
        assert!(release(&mut entry, 3));
        assert_eq!(entry.as_ref().map(|e| e.phase), Some(HoverPhase::Leaving));
        assert!(!release(&mut entry, 3));
    }

    #[test]
    fn a_late_leave_never_touches_a_newer_card() {
        let mut entry = Some(held(4));
        assert!(!release(&mut entry, 3));
        remove(&mut entry, 3);
        assert_eq!(
            entry.as_ref().map(|e| (e.id, e.phase)),
            Some((4, HoverPhase::Held))
        );
    }

    #[test]
    fn a_faded_card_drops() {
        let mut entry = Some(held(0));
        assert!(release(&mut entry, 0));
        remove(&mut entry, 0);
        assert!(entry.is_none());
    }

    #[test]
    fn the_card_dodges_to_the_side_the_cursor_is_not_on() {
        assert_eq!(pins_right(100.0, 1000.0), Some(true));
        assert_eq!(pins_right(900.0, 1000.0), Some(false));
        assert_eq!(pins_right(100.0, 0.0), None);
    }
}
