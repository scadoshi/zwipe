//! Undo on a swipe stack, coalesced while a card is entering.
//!
//! An Undo brings the newest swiped card back onto the stack with an enter animation. Pressed
//! again before that card is in, it would cut the card mid-flight and start the next, so a quick
//! run of Undos flickers through every card, the details pane with it. [`use_undo_burst`] holds
//! the Undos that come while a card is entering and runs them together as it lands: the stack
//! steps back over all of them in one render, and only the last card comes in.

use dioxus::prelude::*;

/// The Undo a stack's keys, buttons and gestures call. With no card entering it runs `undo` at
/// once, as a slow Undo always has. While `entering` is true it counts the press instead, and
/// when `entering` turns false it runs every counted Undo in the same tick, so the stack and the
/// pane show only where they end up.
pub fn use_undo_burst(entering: Memo<bool>, undo: Callback<()>) -> Callback<()> {
    let mut waiting = use_signal(|| 0_usize);
    // The card in: the waiting Undos run back to back. The first brings a card back, which
    // starts its enter, so the rest run straight through rather than waiting again.
    use_effect(move || {
        if entering() {
            return;
        }
        let count = *waiting.peek();
        if count == 0 {
            return;
        }
        waiting.set(0);
        for _ in 0..count {
            undo.call(());
        }
    });
    use_callback(move |()| {
        if *entering.peek() {
            let more = waiting.peek().saturating_add(1);
            waiting.set(more);
        } else {
            undo.call(());
        }
    })
}

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::NoOpMutations;

    use super::*;

    /// Settles `dom`'s effects.
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
    }

    #[test]
    fn undos_while_a_card_enters_wait_and_then_run_together() {
        #[derive(Clone, Copy)]
        struct Probe {
            entering: Signal<bool>,
            runs: Signal<usize>,
            gate: Callback<()>,
        }

        fn app() -> Element {
            let mut entering = use_signal(|| false);
            let mut runs = use_signal(|| 0_usize);
            let busy = use_memo(move || *entering.read());
            // Each Undo brings a card back, which starts its enter.
            let undo = use_callback(move |()| {
                *runs.write() += 1;
                entering.set(true);
            });
            let gate = use_undo_burst(busy, undo);
            use_context_provider(|| Probe {
                entering,
                runs,
                gate,
            });
            rsx! {}
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        let probe = dom.in_scope(ScopeId::APP, consume_context::<Probe>);

        dom.in_scope(ScopeId::APP, || probe.gate.call(()));
        settle(&mut dom);
        assert_eq!(
            *probe.runs.peek(),
            1,
            "an Undo with nothing entering runs at once"
        );

        dom.in_scope(ScopeId::APP, || {
            probe.gate.call(());
            probe.gate.call(());
            probe.gate.call(());
        });
        settle(&mut dom);
        assert_eq!(
            *probe.runs.peek(),
            1,
            "Undos while the card is entering wait for it"
        );

        let mut entering = probe.entering;
        dom.in_scope(ScopeId::APP, || entering.set(false));
        settle(&mut dom);
        assert_eq!(
            *probe.runs.peek(),
            4,
            "the card in, the three waiting Undos run together"
        );
        assert!(
            *probe.entering.peek(),
            "and the last card they bring back enters"
        );

        dom.in_scope(ScopeId::APP, || entering.set(false));
        settle(&mut dom);
        assert_eq!(*probe.runs.peek(), 4, "nothing more waits");
    }
}
