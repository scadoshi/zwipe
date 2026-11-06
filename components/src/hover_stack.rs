//! The hover preview a card list shows a mouse: a stack of up to
//! [`HOVER_STACK_CAP`] full-art cards pinned to a top corner, newest on top.
//!
//! Hovering a [`CardRow`](crate::CardRow) pushes the face it shows onto the
//! stack and holds it there while the cursor stays on the row. Leaving the
//! row starts that card's life: `PREVIEW_LIFE_MS` later it fades out over
//! `PREVIEW_FADE_MS` and drops off, each card on its own timer. Flipping a
//! row's face pushes the new face with its life already running. The stack
//! sits in the corner away from the cursor.
//!
//! A screen calls [`use_hover_stack`], mounts [`HoverStackHost`] outside
//! any transformed ancestor and calls [`HoverStack::track`] from its page's
//! `onmousemove`; its rows find the stack by context. The stack is
//! hover-only, so off the browser nothing in it is timed: a card that is
//! let go drops at once.

use std::cell::Cell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use dioxus::core::Runtime;
use dioxus::prelude::*;

/// The most cards the stack holds; a push past it drops the oldest.
const HOVER_STACK_CAP: usize = 5;

/// How long a card stays up once its row is left or flipped.
#[cfg(target_arch = "wasm32")]
const PREVIEW_LIFE_MS: u32 = 2000;

/// A card's fade out, the `hover-stack-out` animation in `magic.css`.
#[cfg(target_arch = "wasm32")]
const PREVIEW_FADE_MS: u32 = 400;

/// How far each older card sits under the one above it, in rem.
const HOVER_STACK_STEP_REM: f64 = 2.6;

/// Where a card is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoverPhase {
    /// The cursor is still on the row that pushed it; its life has not
    /// started.
    Held,
    /// Let go, counting down its life (`PREVIEW_LIFE_MS`).
    Living,
    /// Its life is over and it is fading out (`PREVIEW_FADE_MS`).
    Leaving,
}

/// One card in the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HoverEntry {
    /// Unique per push, so a card hovered twice stacks twice.
    id: u64,
    /// The image the card shows.
    url: String,
    /// Where the card is in its life.
    phase: HoverPhase,
}

/// Puts `entry` on top of the stack and drops what falls past the cap.
fn push(entries: &mut Vec<HoverEntry>, entry: HoverEntry) {
    entries.insert(0, entry);
    entries.truncate(HOVER_STACK_CAP);
}

/// Lets go of a held entry, true when `id` was held (and is still in the
/// stack), so its life starts once.
fn release(entries: &mut [HoverEntry], id: u64) -> bool {
    match entries
        .iter_mut()
        .find(|e| e.id == id && e.phase == HoverPhase::Held)
    {
        Some(entry) => {
            entry.phase = HoverPhase::Living;
            true
        }
        None => false,
    }
}

/// Starts an entry's fade out.
#[cfg(any(target_arch = "wasm32", test))]
fn mark_leaving(entries: &mut [HoverEntry], id: u64) {
    if let Some(entry) = entries.iter_mut().find(|e| e.id == id) {
        entry.phase = HoverPhase::Leaving;
    }
}

/// Drops an entry.
fn remove(entries: &mut Vec<HoverEntry>, id: u64) {
    entries.retain(|e| e.id != id);
}

/// Whether the stack belongs in the right corner: while the cursor works
/// the window's left half. `None` before the window's width is known.
fn pins_right(x: f64, width: f64) -> Option<bool> {
    (width > 0.0).then_some(x < width / 2.0)
}

/// A screen's hover stack. `Copy`; from [`use_hover_stack`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverStack {
    entries: Signal<Vec<HoverEntry>>,
    next_id: Signal<u64>,
    on_right: Signal<bool>,
    width: Signal<f64>,
    host: ScopeId,
}

impl HoverStack {
    /// Pushes a hovered row's image, held until [`Self::leave`] with the id
    /// it returns.
    pub(crate) fn enter(self, url: String) -> u64 {
        let id = self.take_id();
        let mut entries = self.entries;
        push(
            &mut entries.write(),
            HoverEntry {
                id,
                url,
                phase: HoverPhase::Held,
            },
        );
        id
    }

    /// Starts the life of the held card `id`; nothing if it is not held.
    pub(crate) fn leave(self, id: u64) {
        let mut entries = self.entries;
        let released = entries.try_write().is_ok_and(|mut e| release(&mut e, id));
        if released {
            self.retire(id);
        }
    }

    /// Starts the life of every held card, as when a row starts to drag.
    pub fn leave_held(self) {
        let held: Vec<u64> = self
            .entries
            .peek()
            .iter()
            .filter(|e| e.phase == HoverPhase::Held)
            .map(|e| e.id)
            .collect();
        for id in held {
            self.leave(id);
        }
    }

    /// Pushes an image with its life already running, as for a flipped face.
    pub(crate) fn flash(self, url: String) {
        let id = self.take_id();
        let mut entries = self.entries;
        push(
            &mut entries.write(),
            HoverEntry {
                id,
                url,
                phase: HoverPhase::Living,
            },
        );
        self.retire(id);
    }

    /// Moves the stack away from the cursor at window x `x`, writing only
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

    /// Runs out the life of `id`, then its fade, then drops it. The timers
    /// run in the host's scope, so a row that unmounts meanwhile still
    /// clears its card.
    #[cfg(target_arch = "wasm32")]
    fn retire(self, id: u64) {
        let mut entries = self.entries;
        Runtime::current().spawn(self.host, async move {
            gloo_timers::future::TimeoutFuture::new(PREVIEW_LIFE_MS).await;
            if let Ok(mut e) = entries.try_write() {
                mark_leaving(&mut e, id);
            }
            gloo_timers::future::TimeoutFuture::new(PREVIEW_FADE_MS).await;
            if let Ok(mut e) = entries.try_write() {
                remove(&mut e, id);
            }
        });
    }

    /// Off the browser nothing is timed: the card drops at once.
    #[cfg(not(target_arch = "wasm32"))]
    fn retire(self, id: u64) {
        let mut entries = self.entries;
        if let Ok(mut e) = entries.try_write() {
            remove(&mut e, id);
        }
    }
}

/// Creates the screen's hover stack and provides it to its rows. On the
/// browser it reads the window's width once, for [`HoverStack::track`].
pub fn use_hover_stack() -> HoverStack {
    let width = use_signal(|| 0.0_f64);
    let host = use_hook(dioxus::core::current_scope_id);
    let stack = HoverStack {
        entries: use_signal(Vec::new),
        next_id: use_signal(|| 0),
        on_right: use_signal(|| false),
        width,
        host,
    };
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

/// The stack's cards, newest on top. Mount it outside any transformed
/// ancestor, where `position: fixed` means the window.
#[component]
pub fn HoverStackHost(stack: HoverStack) -> Element {
    let entries = (stack.entries)();
    if entries.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: if (stack.on_right)() { "hover-stack pin-right" } else { "hover-stack" },
            for (i, entry) in entries.into_iter().enumerate() {
                div {
                    key: "{entry.id}",
                    class: if entry.phase == HoverPhase::Leaving { "hover-stack-card leaving" } else { "hover-stack-card" },
                    style: "top: calc({i} * {HOVER_STACK_STEP_REM}rem); z-index: {200_usize.saturating_sub(i)};",
                    img { src: "{entry.url}", alt: "Card preview" }
                }
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

    fn entry(id: u64) -> HoverEntry {
        HoverEntry {
            id,
            url: format!("card-{id}"),
            phase: HoverPhase::Held,
        }
    }

    fn ids(entries: &[HoverEntry]) -> Vec<u64> {
        entries.iter().map(|e| e.id).collect()
    }

    #[test]
    fn the_newest_card_goes_on_top() {
        let mut entries = Vec::new();
        push(&mut entries, entry(0));
        push(&mut entries, entry(1));
        push(&mut entries, entry(2));
        assert_eq!(ids(&entries), vec![2, 1, 0]);
    }

    #[test]
    fn the_stack_holds_five_and_drops_the_oldest() {
        let mut entries = Vec::new();
        for id in 0..7 {
            push(&mut entries, entry(id));
        }
        assert_eq!(entries.len(), HOVER_STACK_CAP);
        assert_eq!(ids(&entries), vec![6, 5, 4, 3, 2]);
    }

    #[test]
    fn the_same_card_hovered_twice_stacks_twice() {
        let mut entries = Vec::new();
        push(
            &mut entries,
            HoverEntry {
                url: "same".into(),
                ..entry(0)
            },
        );
        push(
            &mut entries,
            HoverEntry {
                url: "same".into(),
                ..entry(1)
            },
        );
        assert_eq!(ids(&entries), vec![1, 0]);
    }

    #[test]
    fn a_held_card_is_let_go_once() {
        let mut entries = vec![entry(3)];
        assert!(release(&mut entries, 3));
        assert!(entries.iter().all(|e| e.phase == HoverPhase::Living));
        assert!(!release(&mut entries, 3));
        assert!(!release(&mut entries, 9));
    }

    #[test]
    fn a_card_fades_then_drops() {
        let mut entries = vec![entry(1), entry(0)];
        mark_leaving(&mut entries, 0);
        assert!(
            entries
                .iter()
                .any(|e| e.id == 0 && e.phase == HoverPhase::Leaving)
        );
        remove(&mut entries, 0);
        assert_eq!(ids(&entries), vec![1]);
    }

    #[test]
    fn the_stack_dodges_to_the_side_the_cursor_is_not_on() {
        assert_eq!(pins_right(100.0, 1000.0), Some(true));
        assert_eq!(pins_right(900.0, 1000.0), Some(false));
        assert_eq!(pins_right(100.0, 0.0), None);
    }
}
