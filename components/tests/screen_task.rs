//! A server write a screen starts runs to its end even when the screen goes first, and a
//! screen can tell its tasks whether it is still up.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use components::{Mounted, spawn_write, use_mounted};
use dioxus::core::NoOpMutations;
use dioxus::prelude::*;

/// What the screen's tasks saw: writes finished, plain tasks finished, its mount flag, and
/// what its drop read from its own signal.
#[derive(Clone, Default)]
struct Seen {
    writes: Rc<Cell<u32>>,
    plain: Rc<Cell<u32>>,
    mounted: Rc<RefCell<Option<Mounted>>>,
    dropped_with: Rc<Cell<Option<u32>>>,
}

impl PartialEq for Seen {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.writes, &other.writes)
    }
}

/// A screen that starts one write and one plain task, each finishing after a short wait,
/// as an Undo's DELETE does while the card comes back.
#[component]
fn Screen(seen: Seen) -> Element {
    let mounted = use_mounted();
    let held = use_signal(|| 7_u32);
    use_hook({
        let seen = seen.clone();
        move || {
            *seen.mounted.borrow_mut() = Some(mounted);
            let writes = Rc::clone(&seen.writes);
            spawn_write(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                writes.set(writes.get().saturating_add(1));
            });
            let plain = Rc::clone(&seen.plain);
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                plain.set(plain.get().saturating_add(1));
            });
        }
    });
    let dropped_with = Rc::clone(&seen.dropped_with);
    use_drop(move || dropped_with.set(held.try_peek().ok().map(|v| *v)));
    rsx! { p { "screen" } }
}

/// Shows the screen, then leaves it as soon as it has drawn.
#[component]
fn Root(seen: Seen) -> Element {
    let mut shown = use_signal(|| true);
    use_effect(move || shown.set(false));
    rsx! {
        if shown() {
            Screen { seen }
        }
    }
}

/// Runs the screen, leaves it, and waits long enough for every task to finish.
fn run(seen: &Seen) -> std::io::Result<()> {
    // On a paused clock, which moves only when every task waits, so the screen always leaves
    // before its tasks' wait is up.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()?;
    runtime.block_on(async {
        let mut dom = VirtualDom::new_with_props(Root, RootProps { seen: seen.clone() });
        dom.rebuild_in_place();
        for _ in 0..4 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
        let Some(end) = tokio::time::Instant::now().checked_add(Duration::from_millis(300)) else {
            return;
        };
        while tokio::time::Instant::now() < end {
            let _ = tokio::time::timeout(Duration::from_millis(50), dom.wait_for_work()).await;
            dom.render_immediate(&mut NoOpMutations);
        }
    });
    Ok(())
}

#[test]
fn a_write_runs_to_its_end_after_the_screen_is_gone() {
    let seen = Seen::default();
    run(&seen).unwrap();
    assert_eq!(seen.writes.get(), 1, "the write finishes exactly once");
    assert_eq!(
        seen.plain.get(),
        0,
        "a plain task ends with its screen, so the screen left before the wait was up"
    );
}

#[test]
fn a_screen_tells_its_tasks_once_it_is_gone() {
    let seen = Seen::default();
    run(&seen).unwrap();
    let mounted = seen.mounted.borrow().unwrap();
    assert!(!mounted.get(), "the screen has left");
}

#[test]
fn a_screen_can_read_its_own_state_as_it_leaves() {
    let seen = Seen::default();
    run(&seen).unwrap();
    assert_eq!(
        seen.dropped_with.get(),
        Some(7),
        "its drop hands what it holds to the root"
    );
}
