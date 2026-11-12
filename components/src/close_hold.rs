//! Contents that stay drawn while their sheet slides closed or their section folds shut, so the
//! panel leaves full instead of blanking a frame into its close, as the kit's bottom sheet does.

use dioxus::prelude::*;

/// How long a closing panel keeps its contents: the kit's 300ms sheet slide and section fold.
pub const CLOSE_MS: u32 = 300;

/// Whether a panel's contents are drawn: from its opening until its latest close has run.
/// Each close is numbered, so the timer of a close that a reopening or a later close overtook
/// empties nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CloseHold {
    /// Whether the contents are drawn.
    kept: bool,
    /// The number of the latest open or close; a timer whose close is not this one is stale.
    turn: u32,
}

impl CloseHold {
    /// The panel opened: its contents are drawn, and no earlier close may empty it.
    pub fn open(&mut self) {
        self.kept = true;
        self.turn = self.turn.wrapping_add(1);
    }

    /// The panel closed: its contents stay until [`Self::settled`] is called with the number this
    /// returns.
    pub fn close(&mut self) -> u32 {
        self.turn = self.turn.wrapping_add(1);
        self.turn
    }

    /// The close numbered `close` has run its course: the contents go unless something has
    /// happened since.
    pub fn settled(&mut self, close: u32) {
        if close == self.turn {
            self.kept = false;
        }
    }

    /// Whether the contents are drawn.
    #[must_use]
    pub fn kept(self) -> bool {
        self.kept
    }
}

/// Whether a panel open while `open` draws its contents: while open and for [`CLOSE_MS`] after
/// it closes. Off the browser nothing is timed, and the contents go as it closes.
pub fn use_close_hold(open: ReadSignal<bool>) -> Memo<bool> {
    let mut hold = use_signal(|| {
        let mut hold = CloseHold::default();
        if *open.peek() {
            hold.open();
        }
        hold
    });
    let mut was_open = use_signal(|| *open.peek());
    use_effect(move || {
        let is_open = open();
        let was = *was_open.peek();
        was_open.set(is_open);
        if is_open && !was {
            hold.write().open();
        } else if was && !is_open {
            let close = hold.write().close();
            settle_after_close(hold, close);
        }
    });
    use_memo(move || hold.read().kept())
}

/// Settles `close` on `hold` once the close has run, for a host that keeps its own
/// [`CloseHold`], such as the one for whichever accordion section is folding shut.
#[cfg(target_arch = "wasm32")]
pub fn settle_after_close(mut hold: Signal<CloseHold>, close: u32) {
    spawn(async move {
        gloo_timers::future::TimeoutFuture::new(CLOSE_MS).await;
        if let Ok(mut held) = hold.try_write() {
            held.settled(close);
        }
    });
}

/// Off the browser nothing is timed: the close settles at once.
#[cfg(not(target_arch = "wasm32"))]
pub fn settle_after_close(mut hold: Signal<CloseHold>, close: u32) {
    hold.write().settled(close);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_close_keeps_the_contents_until_it_has_run() {
        let mut hold = CloseHold::default();
        assert!(!hold.kept());
        hold.open();
        assert!(hold.kept());
        let close = hold.close();
        assert!(hold.kept(), "the contents slide out with the panel");
        hold.settled(close);
        assert!(!hold.kept(), "and go once the slide has run");
    }

    #[test]
    fn reopening_mid_close_keeps_the_contents() {
        let mut hold = CloseHold::default();
        hold.open();
        let close = hold.close();
        hold.open();
        hold.settled(close);
        assert!(
            hold.kept(),
            "an earlier close's timer never empties an open panel"
        );
    }

    #[test]
    fn only_the_latest_close_drops_the_contents() {
        let mut hold = CloseHold::default();
        hold.open();
        let first = hold.close();
        hold.open();
        let second = hold.close();
        hold.settled(first);
        assert!(hold.kept());
        hold.settled(second);
        assert!(!hold.kept());
    }
}
