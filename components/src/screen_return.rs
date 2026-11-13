//! A screen another screen covers, as a route change looks: while a screen is on top of it (a
//! deck's Cards over its Profile, a card's details over Cards on a phone) the one underneath is
//! not painted, focused or tapped ([`UNDER_LAYER`] with `inert`), and when the top one closes it
//! comes back as if entered fresh: its entrances run again ([`replay_entrances`]) and anything
//! that deals on entry deals again, through its [`ScreenReturns`]. Its state, scroll and data
//! stay as they were, since it never unmounts.

use dioxus::prelude::*;

/// The class a covered screen carries: not painted. It also takes `inert`, so it is neither
/// focused nor tapped.
pub const UNDER_LAYER: &str = "under-layer";

/// The kit's entrances, which [`replay_entrances`] runs again: the content's rise and fade
/// (`content-enter`, `content-enter-delayed`) and the chips' (`chip-in`).
pub const ENTRANCES: &str = ".content-enter, .content-enter-delayed, .chip-in";

/// Runs the entrances under `__ROOT__` again: their animation goes off and back on across a
/// reflow, which starts each from its first frame.
const REPLAY_JS: &str = "{ const root = document.querySelector('__ROOT__'); if (root) { const runs = [...root.querySelectorAll('__ENTRANCES__')]; if (root.matches('__ENTRANCES__')) runs.unshift(root); for (const el of runs) el.style.animation = 'none'; void root.offsetHeight; for (const el of runs) el.style.animation = ''; } }";

/// How many times a screen has come back from under another, for whatever deals on entry (the
/// stacks, the featured strip) to deal again on each change. Provided by the screen's host.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenReturns(pub ReadSignal<u32>);

/// Counts the times `covered` turns false after being true: each a return of the screen it
/// covered.
pub fn use_screen_returns(covered: bool) -> ReadSignal<u32> {
    let mut returns = use_signal(|| 0u32);
    let mut was = use_signal(|| covered);
    use_effect(use_reactive!(|covered| {
        let before = *was.peek();
        if before == covered {
            return;
        }
        was.set(covered);
        if before && !covered {
            let next = returns.peek().wrapping_add(1);
            returns.set(next);
        }
    }));
    returns.into()
}

/// Runs the entrances under the element matching `root` again, as [`ScreenReturns`] describes.
pub fn replay_entrances(root: &str) {
    document::eval(&replay_js(root));
}

/// [`REPLAY_JS`] for `root`.
fn replay_js(root: &str) -> String {
    REPLAY_JS
        .replace("__ROOT__", root)
        .replace("__ENTRANCES__", ENTRANCES)
}

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::NoOpMutations;

    use super::*;

    #[test]
    fn the_replay_restarts_only_the_entrances_under_the_root() {
        let js = replay_js(".deck-profile");
        assert!(
            js.contains("document.querySelector('.deck-profile')"),
            "{js}"
        );
        assert!(
            js.contains(&format!("root.querySelectorAll('{ENTRANCES}')")),
            "{js}"
        );
        assert!(
            js.contains("el.style.animation = 'none'; void root.offsetHeight;"),
            "{js}"
        );
    }

    thread_local! {
        static COVER: std::cell::Cell<Option<Signal<bool>>> = const { std::cell::Cell::new(None) };
        static RETURNS: std::cell::Cell<Option<ReadSignal<u32>>> = const { std::cell::Cell::new(None) };
    }

    fn app() -> Element {
        let cover = use_signal(|| false);
        COVER.with(|c| c.set(Some(cover)));
        rsx! { Counted { covered: cover() } }
    }

    #[component]
    fn Counted(covered: bool) -> Element {
        let returns = use_screen_returns(covered);
        RETURNS.with(|r| r.set(Some(returns)));
        rsx! {}
    }

    fn settle(dom: &mut VirtualDom) {
        for _ in 0..6 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
    }

    #[test]
    fn a_return_counts_each_time_the_cover_comes_off() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        settle(&mut dom);
        let cover = COVER.with(std::cell::Cell::get).unwrap();
        let returns = RETURNS.with(std::cell::Cell::get).unwrap();
        let count = |dom: &VirtualDom| dom.in_runtime(|| *returns.peek());
        assert_eq!(count(&dom), 0, "never covered");
        for (covered, want) in [(true, 0), (false, 1), (true, 1), (false, 2)] {
            dom.in_runtime(|| {
                let mut cover = cover;
                cover.set(covered);
            });
            settle(&mut dom);
            assert_eq!(count(&dom), want, "after covered = {covered}");
        }
    }
}
