//! Show more's ghost rows on a card list, which [`MoreGhosts`] steps through: the rows' skeleton
//! easing in under the list while Show more's next rows are on their way, then fading under them.

use dioxus::prelude::*;
use zwipe_ui::commander::more_ghosts::{GHOST_ROWS, MoreGhosts};

/// Show more's ghost rows, drawn between a list's held rows and the rows Show more dealt while
/// `ghosts` is drawn. Each row is the card rows' skeleton, its thumb and its bar, with
/// `row_class` added for the client's row height. `on_animation_end` hears the name of each
/// animation that ends on them, for the model that steps them. Hidden from screen readers, which
/// hear the rows themselves.
#[component]
pub fn MoreGhostRows(
    ghosts: MoreGhosts,
    on_animation_end: EventHandler<String>,
    /// Added to each ghost row's classes.
    #[props(default)]
    row_class: &'static str,
) -> Element {
    rsx! {
        div {
            class: ghosts.class(),
            "aria-hidden": "true",
            onanimationend: move |event: AnimationEvent| on_animation_end.call(event.animation_name()),
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
