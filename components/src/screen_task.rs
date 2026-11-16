//! Server writes a screen starts that must land whatever the screen does next.
//!
//! A task from `spawn` belongs to the screen that started it and ends when the
//! screen goes, dropping a request it has not sent yet. [`spawn_write`] runs a
//! write in the root instead, so it is sent exactly once and runs to its end. A
//! screen's own signals end with it, so such a task checks [`Mounted`] before it
//! touches them; root state and toasts it may touch at any time.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::mem;
use std::pin::Pin;

use dioxus::core::spawn_forever;
use dioxus::prelude::*;
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
    let idle = LANES.with(|lanes| {
        let mut lanes = lanes.borrow_mut();
        let lane = lanes.entry(key).or_default();
        lane.waiting.push_back(Box::pin(write));
        !mem::replace(&mut lane.running, true)
    });
    if idle {
        spawn_write(run_lane(key));
    }
}

/// One key's writes: those waiting their turn, and whether one is out.
#[derive(Default)]
struct Lane {
    waiting: VecDeque<Pin<Box<dyn Future<Output = ()>>>>,
    running: bool,
}

thread_local! {
    /// Each key's lane, for as long as it has a write out or waiting. The app's tasks all run
    /// on one thread, the web's and the phone's alike.
    static LANES: RefCell<HashMap<Uuid, Lane>> = RefCell::new(HashMap::new());
}

/// Runs `key`'s writes one after another until none waits, then closes its lane.
async fn run_lane(key: Uuid) {
    loop {
        let next = LANES.with(|lanes| {
            let mut lanes = lanes.borrow_mut();
            let next = lanes
                .get_mut(&key)
                .and_then(|lane| lane.waiting.pop_front());
            if next.is_none() {
                lanes.remove(&key);
            }
            next
        });
        let Some(write) = next else {
            return;
        };
        write.await;
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
}
