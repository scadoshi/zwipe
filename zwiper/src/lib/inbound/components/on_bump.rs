//! Runs a closure each time a counter moves, after the screen has mounted.

use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;

/// Calls `on_bump` every time `counter` changes after the first render. The first render's own run is skipped, since a screen loads its first state another way.
///
/// The mounted flag is shared through `Rc`. `use_effect` runs the newest render's closure, so a plain `Cell` from `use_hook` is a fresh copy on every render and would read as the first run forever.
pub(crate) fn use_on_bump(counter: Signal<u32>, mut on_bump: impl FnMut() + 'static) {
    let mounted = use_hook(|| Rc::new(Cell::new(false)));
    use_effect(move || {
        let _ = counter();
        if !mounted.replace(true) {
            return;
        }
        on_bump();
    });
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    /// The root's counter, a signal that re-renders it without a bump, and how many times `on_bump` ran.
    type RootSignals = (Signal<u32>, Signal<u32>, Signal<u32>);

    thread_local! {
        static SIGNALS: RefCell<Option<RootSignals>> = const { RefCell::new(None) };
    }

    #[component]
    fn Root() -> Element {
        let counter = use_signal(|| 0u32);
        let unrelated = use_signal(|| 0u32);
        let mut calls = use_signal(|| 0u32);
        // Read during render, so writing it re-renders the root and hands the effect a new closure.
        let _ = unrelated();
        use_on_bump(counter, move || {
            let next = calls.peek().wrapping_add(1);
            calls.set(next);
        });
        SIGNALS.with(|s| *s.borrow_mut() = Some((counter, unrelated, calls)));
        rsx! {}
    }

    fn bump(dom: &mut VirtualDom, mut signal: Signal<u32>) {
        dom.in_runtime(|| {
            let next = signal.peek().wrapping_add(1);
            signal.set(next);
        });
        dom.render_immediate_to_vec();
    }

    #[test]
    fn the_first_render_does_not_call_it() {
        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();
        dom.render_immediate_to_vec();
        let (_, _, calls) = SIGNALS.with(|s| s.borrow().expect("the root rendered"));
        assert_eq!(dom.in_runtime(|| *calls.peek()), 0);
    }

    #[test]
    fn every_bump_after_mount_calls_it_even_across_rerenders() {
        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();
        dom.render_immediate_to_vec();
        let (counter, unrelated, calls) = SIGNALS.with(|s| s.borrow().expect("the root rendered"));

        bump(&mut dom, counter);
        assert_eq!(dom.in_runtime(|| *calls.peek()), 1, "the first bump");

        bump(&mut dom, unrelated);
        assert_eq!(
            dom.in_runtime(|| *calls.peek()),
            1,
            "a re-render alone is not a bump"
        );

        bump(&mut dom, counter);
        assert_eq!(
            dom.in_runtime(|| *calls.peek()),
            2,
            "a bump after a re-render"
        );
    }
}
