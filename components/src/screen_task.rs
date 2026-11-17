//! Server writes a screen starts that must land whatever the screen does next.
//!
//! A task from `spawn` belongs to the screen that started it and ends when the
//! screen goes, dropping a request it has not sent yet. [`spawn_write`] runs a
//! write in the root instead, so it is sent exactly once and runs to its end. A
//! screen's own signals end with it, so such a task checks [`Mounted`] before it
//! touches them; root state and toasts it may touch at any time.

use std::future::Future;

use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use futures_channel::oneshot;
use uuid::Uuid;

/// Runs `write` in the root, so it is sent and runs to its end even when the screen
/// that started it goes first. For writes only: a read whose screen is gone has no one
/// to answer.
pub fn spawn_write(write: impl Future<Output = ()> + 'static) {
    let _task = spawn_forever(write);
}

/// Runs `write` as [`spawn_write`] does, after every earlier write given the same `key` has
/// run to its end: writes for one card reach the server in the order they were made, so an
/// undo can never land before the write it undoes. Writes under different keys run at once.
pub fn spawn_write_in_order(key: Uuid, write: impl Future<Output = ()> + 'static) {
    spawn_write(in_order(key, write));
}

/// Runs `write` as a task of the current screen, after every earlier write given the same
/// `key` has run to its end or been dropped with its screen, so writes for one card reach the
/// server in the order they were made.
pub fn spawn_in_order(key: Uuid, write: impl Future<Output = ()> + 'static) {
    let _task = spawn(in_order(key, write));
}

/// The per-thread state that orders each key's writes.
#[cfg_attr(
    target_os = "android",
    expect(
        clippy::missing_const_for_thread_local,
        reason = "fires on Android for the const-initialized NEXT, which is already const"
    )
)]
mod slots {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;

    use futures_channel::oneshot;
    use uuid::Uuid;

    thread_local! {
        /// Each key's latest write while it is out, by its number: the next write for the key
        /// waits on it. The app's tasks all run on one thread, the web's and the phone's alike.
        pub(super) static LATEST: RefCell<HashMap<Uuid, (u64, oneshot::Receiver<()>)>> = RefCell::new(HashMap::new());
        /// The number the next write takes.
        pub(super) static NEXT: Cell<u64> = const { Cell::new(0) };
    }
}

use slots::{LATEST, NEXT};

/// `write`, once the key's previous write has ended. A write ends when it finishes or when its
/// task is dropped, since dropping it drops its sender, so a write whose screen or root went
/// never holds the next one back.
fn in_order(key: Uuid, write: impl Future<Output = ()> + 'static) -> impl Future<Output = ()> {
    let number = NEXT.with(|next| {
        let number = next.get();
        next.set(number.wrapping_add(1));
        number
    });
    let (done, finished) = oneshot::channel::<()>();
    let previous = LATEST.with(|latest| latest.borrow_mut().insert(key, (number, finished)));
    async move {
        let _done = done;
        if let Some((_, previous)) = previous {
            let _ = previous.await;
        }
        write.await;
        LATEST.with(|latest| {
            let mut latest = latest.borrow_mut();
            if latest.get(&key).is_some_and(|(at, _)| *at == number) {
                latest.remove(&key);
            }
        });
    }
}

/// Whether the screen that made it is still up, for a [`spawn_write`] task that
/// touches the screen's own signals once its answer lands. It holds a signal of the
/// screen's own, which ends with the screen, so it reads false from then on.
#[derive(Clone, Copy, Debug)]
pub struct Mounted(Signal<()>);

impl Mounted {
    /// True until the screen goes.
    #[must_use]
    pub fn get(self) -> bool {
        self.0.try_peek().is_ok()
    }
}

/// This screen's [`Mounted`].
pub fn use_mounted() -> Mounted {
    Mounted(use_signal(|| ()))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::time::Duration;

    use dioxus::dioxus_core::NoOpMutations;

    use super::*;

    thread_local! {
        static LOG: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
        static HOLD: RefCell<Option<tokio::sync::oneshot::Receiver<()>>> = const { RefCell::new(None) };
    }

    fn log(entry: &'static str) {
        LOG.with(|l| l.borrow_mut().push(entry));
    }

    /// Lets `dom`'s tasks run for a little while.
    fn run(rt: &tokio::runtime::Runtime, dom: &mut VirtualDom) {
        rt.block_on(async {
            for _ in 0..10 {
                tokio::select! {
                    () = dom.wait_for_work() => {}
                    () = tokio::time::sleep(Duration::from_millis(5)) => {}
                }
            }
        });
        dom.render_immediate(&mut NoOpMutations);
    }

    #[test]
    fn a_later_write_for_the_same_card_waits_for_the_earlier_one() {
        fn app() -> Element {
            use_hook(|| {
                let card = Uuid::from_u128(1);
                let hold = HOLD.with(|h| h.borrow_mut().take());
                spawn_write_in_order(card, async move {
                    if let Some(hold) = hold {
                        let _ = hold.await;
                    }
                    log("dismiss");
                });
                spawn_write_in_order(card, async { log("undismiss") });
                spawn_write_in_order(Uuid::from_u128(2), async { log("other card") });
            });
            rsx! {}
        }
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let (release, hold) = tokio::sync::oneshot::channel();
        HOLD.with(|h| *h.borrow_mut() = Some(hold));
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        run(&rt, &mut dom);
        assert_eq!(
            LOG.with(|l| l.borrow().clone()),
            ["other card"],
            "the undo waits while the dismiss is out; another card's write does not"
        );
        release.send(()).unwrap();
        run(&rt, &mut dom);
        assert_eq!(
            LOG.with(|l| l.borrow().clone()),
            ["other card", "dismiss", "undismiss"]
        );
    }

    #[test]
    fn a_card_s_writes_still_run_after_the_runner_of_its_earlier_ones_is_dropped() {
        thread_local! {
            static RAN: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
        }
        fn holding() -> Element {
            use_hook(|| {
                spawn_write_in_order(Uuid::from_u128(7), async {
                    std::future::pending::<()>().await;
                });
            });
            rsx! {}
        }
        fn saving() -> Element {
            use_hook(|| {
                spawn_write_in_order(Uuid::from_u128(7), async {
                    RAN.with(|r| r.borrow_mut().push("save"));
                });
            });
            rsx! {}
        }
        let card = Uuid::from_u128(7);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let _guard = rt.enter();
        let mut first = VirtualDom::new(holding);
        first.rebuild_in_place();
        run(&rt, &mut first);
        drop(first);
        let mut second = VirtualDom::new(saving);
        second.rebuild_in_place();
        run(&rt, &mut second);
        assert_eq!(
            RAN.with(|r| r.borrow().clone()),
            ["save"],
            "the dropped runner let go of card {card}"
        );
    }
}
