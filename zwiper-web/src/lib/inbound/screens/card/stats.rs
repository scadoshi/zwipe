//! One of the card screen's stats widgets: the shared widget with the web's ghost chips while the
//! stats load.

use std::collections::HashSet;

use components::CardStatsWidget;
use dioxus::prelude::*;
use zwipe_ui::screens::card_lookup::stats::Widget;

use crate::inbound::chrome::ghost::GhostText;

/// `widget` as a fold, a ghost chip in each figure's place while the stats load. `folded` holds
/// the folds the user shut.
#[component]
pub(super) fn StatsWidget(widget: Widget, folded: Signal<HashSet<String>>) -> Element {
    rsx! {
        CardStatsWidget {
            widget,
            folded,
            ghost: rsx! {
                GhostText { chars: 3 }
            },
            ghost_class: "web-ghost-chip",
        }
    }
}
