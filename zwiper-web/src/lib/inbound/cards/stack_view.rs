//! The Stacks view's shared pieces: the deck page's List | Stacks switch,
//! the stack's reveal and details drawer state, and the faces the rows and
//! stacks show.

use std::collections::HashMap;

use components::Chip;
pub(crate) use components::{StackDetails, use_stack_details_sync};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::deck_cards::stacks::{CardsView, VIEW_LABEL};

use crate::outbound::cards_view_store::CardsViewStore;

/// The face each card shows, by card id, the front where it has none: a
/// card screen provides it so its rows, its stack and the `v` key turn the
/// same face. A row with no face of its own here reads and writes this map.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RowFaces(pub(crate) Signal<HashMap<Uuid, usize>>);

impl RowFaces {
    /// The face card `id` shows.
    pub(crate) fn of(self, id: Uuid) -> usize {
        self.0.read().get(&id).copied().unwrap_or(0)
    }

    /// Shows card `id` on `face`.
    pub(crate) fn set(self, id: Uuid, face: usize) {
        let Self(mut faces) = self;
        faces.write().insert(id, face);
    }
}

/// Provides the screen's [`RowFaces`].
pub(crate) fn use_row_faces() -> RowFaces {
    let faces = use_signal(HashMap::new);
    use_context_provider(|| RowFaces(faces))
}

/// The deck page's List | Stacks choice, as last left; the list when it never
/// was.
pub(crate) fn use_cards_view() -> Signal<CardsView> {
    use_signal(|| CardsViewStore::browser().load())
}

/// The View chip row: List and Stacks, the one in `view` selected. A tap
/// stores the choice and calls `on_change`.
#[component]
pub(crate) fn CardsViewSwitch(
    mut view: Signal<CardsView>,
    #[props(default)] on_change: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        div { class: "chip-row deck-view-switch",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "{VIEW_LABEL}" }
            for choice in [CardsView::List, CardsView::Stacks] {
                Chip {
                    key: "{choice.key()}",
                    selected: view() == choice,
                    onclick: move |_| {
                        view.set(choice);
                        CardsViewStore::browser().save(choice);
                        if let Some(handler) = on_change {
                            handler.call(());
                        }
                    },
                    "{choice.label()}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use components::DRAWER_CLASSES;

    use crate::inbound::chrome::shortcuts::STACK_DETAILS_CLASS;

    #[test]
    fn the_drawer_names_the_class_the_keys_pass_through() {
        let classes: Vec<&str> = DRAWER_CLASSES.split_whitespace().collect();
        assert!(classes.contains(&"card-details-drawer"));
        assert!(classes.contains(&STACK_DETAILS_CLASS));
    }
}
