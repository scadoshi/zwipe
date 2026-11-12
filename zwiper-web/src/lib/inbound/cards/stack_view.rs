//! The Stacks view's shared pieces: the deck page's List | Stacks switch,
//! the stack's reveal and details drawer state, and the faces the rows and
//! stacks show.

use std::collections::HashMap;

use components::Chip;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::deck_cards::stacks::{CardsView, Gesture, Reveal, VIEW_LABEL};

use crate::outbound::cards_view_store::CardsViewStore;

/// The stacks' reveal and whether the open card's drawer shows, which a click on a card and the
/// `d` key both move. A screen with stacks provides it as context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StackDetails {
    /// Which card shows in full and which card's details are open.
    pub(crate) reveal: Signal<Reveal>,
    /// Whether the open card's details drawer shows.
    pub(crate) details_open: Signal<bool>,
}

impl StackDetails {
    /// A click on card `id`: its details open, in place of any open one, or close where they are
    /// open.
    pub(crate) fn click(self, id: Uuid) {
        let Self {
            mut reveal,
            mut details_open,
        } = self;
        reveal.write().apply(Gesture::Click(id));
        details_open.set(reveal.peek().open().is_some());
    }
}

/// The details drawer's classes: the card details drawer's own, then
/// [`STACK_DETAILS_CLASS`](crate::inbound::chrome::shortcuts::STACK_DETAILS_CLASS), so a card
/// screen's keys reach the screen through it.
pub(crate) const DRAWER_CLASSES: &str = "card-details-drawer deck-stack-details";

/// Drops focus from the stack card the drawer opened from, so its focus
/// outline goes with the drawer.
const BLUR_STACK_CARD_JS: &str = "if (document.activeElement?.classList.contains('card-stack-card')) document.activeElement.blur();";

/// Keeps the drawer and the reveal state agreeing: a drawer closed by its
/// ×, Escape, the backdrop or the back gesture closes the open card and
/// clears its outline, and an open card that left the list closes the
/// drawer.
pub(crate) fn use_stack_details_sync(
    mut reveal: Signal<Reveal>,
    mut details_open: Signal<bool>,
    listed: Memo<Vec<Uuid>>,
) {
    use_effect(move || {
        let open = details_open();
        let card = reveal.peek().open();
        if !open && card.is_some() {
            reveal.write().apply(Gesture::Close);
            document::eval(BLUR_STACK_CARD_JS);
        }
        if open && card.is_some_and(|id| !listed().contains(&id)) {
            details_open.set(false);
        }
    });
}

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
    use super::*;
    use crate::inbound::chrome::shortcuts::STACK_DETAILS_CLASS;

    #[test]
    fn the_drawer_names_the_class_the_keys_pass_through() {
        let classes: Vec<&str> = DRAWER_CLASSES.split_whitespace().collect();
        assert!(classes.contains(&"card-details-drawer"));
        assert!(classes.contains(&STACK_DETAILS_CLASS));
    }
}
