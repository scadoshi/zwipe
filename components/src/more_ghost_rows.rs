//! Show more's ghost rows on a card list, which [`MoreGhosts`] steps through: the rows' skeleton
//! easing in under the list while Show more's page is out, then fading under the rows it deals.

use dioxus::prelude::*;
use zwipe_ui::commander::more_ghosts::{GHOST_ROWS, MoreGhosts};

/// Show more's ghost rows, drawn between a list's held rows and the rows Show more dealt while
/// `ghosts` is drawn. Each row is the card rows' skeleton, its thumb and its bar, with
/// `row_class` added for the client's row height. Hidden from screen readers, which hear the rows
/// themselves.
#[component]
pub fn MoreGhostRows(
    ghosts: Signal<MoreGhosts>,
    /// Added to each ghost row's classes.
    #[props(default)]
    row_class: &'static str,
) -> Element {
    let mut ghosts = ghosts;
    let class = ghosts.read().class();
    rsx! {
        div {
            class,
            "aria-hidden": "true",
            onanimationend: move |event: AnimationEvent| {
                let mut next = ghosts.peek().clone();
                next.animation_ended(&event.animation_name());
                if next != *ghosts.peek() {
                    ghosts.set(next);
                }
            },
            div {
                for i in 0..GHOST_ROWS {
                    div { key: "{i}", class: "skeleton-card-row {row_class}",
                        div { class: "skeleton-bar skeleton-card-thumb" }
                        div { class: "skeleton-bar skeleton-card-bar-row" }
                    }
                }
            }
        }
    }
}
