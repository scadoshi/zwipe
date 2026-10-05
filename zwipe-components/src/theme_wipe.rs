//! Sweeps a newly picked theme across the page.
//!
//! The picker writes a pending theme rather than the real one. A view
//! transition snapshots the page, the theme class is swapped on the host's
//! themed element inside it, and the real theme is committed only once the
//! swap is in, so the re-render lands on a page that already shows it. The
//! `theme-wipe` keyframes in components.css reveal the new snapshot over the
//! old: left to right, except a switch from light to dark, which runs right
//! to left (top to bottom and bottom to top at hamburger widths). Browsers without view transitions, and readers who ask for reduced
//! motion, get the instant swap.
//!
//! Every pick also replays the page's entrance, as the nav logo does: the
//! logo's animation restarts and [`Replay`] moves, so every count-up and
//! decode on the page starts over.

use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;
use zwipe_core::domain::user::models::theme::ThemeConfig;

use crate::Replay;

/// Receives the themed element's selector and the new theme class, and swaps
/// the element onto it inside a view transition. Inside the swap it tells the
/// app to commit the theme and waits for the answer that the app has rendered
/// it, so the new snapshot already holds the new picker label and the nav
/// laid out around it. Tells the app once more when the wipe has finished.
/// Without a wipe, the same exchange runs with no transition around it.
const WIPE_JS: &str = r#"
const [selector, next] = await dioxus.recv();
const target = document.querySelector(selector);
const root = document.documentElement;
const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
const commit = async () => {
    dioxus.send(true);
    await dioxus.recv();
    // Rendering is held during the swap, so wait on a timer, not a frame.
    await new Promise((r) => setTimeout(r, 20));
};
let wipe = null;
if (target && document.startViewTransition && !still) {
    const swap = async () => {
        Array.from(target.classList).forEach((c) => {
            if (/^theme-.+-(dark|light)$/.test(c)) target.classList.remove(c);
        });
        target.classList.add(next);
        await commit();
    };
    // Light to dark sweeps right to left; every other change left to right.
    const toDark = next.endsWith("-dark") && /theme-\S+-light/.test(target.className);
    root.dataset.wipe = toDark ? "left" : "right";
    // Tells the nav glide to stand down while the snapshots are in charge.
    root.dataset.wiping = "";
    wipe = document.startViewTransition(swap);
    wipe.finished.finally(() => delete root.dataset.wiping);
    try {
        await wipe.updateCallbackDone;
    } catch (e) {}
} else {
    await commit();
}
const logo = document.querySelector(".logo");
if (logo) {
    logo.style.animation = "none";
    void logo.offsetHeight;
    logo.style.animation = "";
}
if (wipe) {
    try {
        await wipe.finished;
    } catch (e) {}
}
dioxus.send(true);
"#;

/// The two signals to hand [`ThemePicker`](crate::ThemePicker): its `theme`
/// and its `shown`.
///
/// `target` is the selector of the element that carries the theme class
/// (`"body"`, `".theme-wrapper"`). A pick wipes in, and `theme` and `shown`
/// take it inside the transition, so the sweep reveals the picker's new label
/// with the nav already laid out around it; until then the picker keeps
/// showing the old theme. A change to `theme` from elsewhere, such as the
/// stored theme adopted after mount, moves both with it and does not wipe.
pub fn use_theme_wipe(
    mut theme: Signal<ThemeConfig>,
    target: &'static str,
) -> (Signal<ThemeConfig>, Signal<ThemeConfig>) {
    let mut picked = use_signal(|| theme.peek().clone());
    let mut shown = use_signal(|| theme.peek().clone());
    let armed = use_hook(|| Rc::new(Cell::new(false)));
    // Wipes still running; `shown` is theirs to move until they finish.
    let wiping = use_hook(|| Rc::new(Cell::new(0u32)));
    let wiping_in_sync = Rc::clone(&wiping);
    let replay = try_use_context::<Replay>();

    use_effect(move || {
        let next = picked.read().clone();
        // The first run only subscribes. It can see the stored theme already
        // adopted while `picked` still holds the default, which is not a pick.
        if !armed.replace(true) || *theme.peek() == next {
            return;
        }
        let wiping = Rc::clone(&wiping);
        wiping.set(wiping.get() + 1);
        spawn(async move {
            let mut js = document::eval(WIPE_JS);
            let _ = js.send((target, next.css_class()));
            let _ = js.recv::<bool>().await;
            // A later pick supersedes this one and commits itself.
            if *picked.peek() == next {
                theme.set(next.clone());
                shown.set(next);
                if let Some(Replay(mut count)) = replay {
                    count += 1;
                }
            }
            let _ = js.send(true);
            let _ = js.recv::<bool>().await;
            wiping.set(wiping.get() - 1);
        });
    });

    use_effect(move || {
        let current = theme.read().clone();
        if *picked.peek() != current {
            picked.set(current.clone());
        }
        if wiping_in_sync.get() == 0 && *shown.peek() != current {
            shown.set(current);
        }
    });

    (picked, shown)
}
