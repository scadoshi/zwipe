//! The Stacks view's shared pieces: the List | Stacks switch, the stack's
//! reveal and details drawer state, and the faces the rows and stacks show.
//! The deck page draws one stack per group with its own drawer; the
//! single-list card screens (Playstyle's lists, Activity's Most skipped)
//! draw one `CardsStack` per list with a `StackDrawer`.

use std::collections::{HashMap, HashSet};

use components::{CardStack, Chip, DeckGroupSection, StackCard};
pub(crate) use components::{DRAWER_CLASSES, StackDetails, use_stack_details_sync};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_ui::deck_cards::stacks::{CardsView, Gesture, Reveal, VIEW_LABEL};

use crate::inbound::cards::card_info::CardDetailsSheet;
use crate::inbound::cards::card_row::CardOverlays;
use crate::outbound::cards_view_store::{CardsViewPlace, CardsViewStore};

/// A screen's [`StackDetails`], provided as context and kept in step with
/// `listed`, the ids of every card the screen lists.
pub(crate) fn use_stack_details(listed: Memo<Vec<Uuid>>) -> StackDetails {
    let reveal = use_signal(Reveal::default);
    let details_open = use_signal(|| false);
    let details = use_context_provider(|| StackDetails {
        reveal,
        details_open,
    });
    use_stack_details_sync(reveal, details_open, listed);
    details
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

/// The List | Stacks choice at `place`, as last left there; the list when it
/// never was.
pub(crate) fn use_cards_view(place: CardsViewPlace) -> Signal<CardsView> {
    use_signal(|| CardsViewStore::browser(place).load())
}

/// The View chip row: List and Stacks, the one in `view` selected. A tap
/// stores the choice for `place` and calls `on_change`.
#[component]
pub(crate) fn CardsViewSwitch(
    mut view: Signal<CardsView>,
    place: CardsViewPlace,
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
                        CardsViewStore::browser(place).save(choice);
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

/// `cards` as one stack on the page, as the deck page draws a group: the
/// group's header, `title` and its count with `trailing` at its end, as the
/// card-width chip over the cards, and the cards floating under it, each on
/// its face in the screen's [`RowFaces`]. Hover reveals a card in place, and a
/// click opens its details in the screen's [`StackDrawer`].
#[component]
pub(crate) fn CardsStack(
    cards: Vec<Card>,
    title: String,
    #[props(default)] trailing: Option<Element>,
) -> Element {
    let collapsed: Signal<HashSet<String>> = use_signal(HashSet::new);
    let details = use_context::<StackDetails>();
    let faces = try_use_context::<RowFaces>();
    let mut reveal = details.reveal;
    let shown = reveal();
    let ids: Vec<Uuid> = cards.iter().map(|c| c.scryfall_data.id).collect();
    let cards: Vec<StackCard> = cards
        .into_iter()
        .map(|card| {
            let face = faces.map_or(0, |f| f.of(card.scryfall_data.id));
            StackCard { card, qty: 1, face }
        })
        .collect();
    let count = cards.len();
    let fold_key = title.clone();
    rsx! {
        div { class: "deck-stacks cards-stack",
            div { class: "deck-stack", "data-fold": "{fold_key}",
                DeckGroupSection {
                    group_key: title.clone(),
                    header: title,
                    pips: None,
                    count: Some(count),
                    collapsed,
                    trailing: trailing.map(|t| rsx! { span { class: "deck-stack-tail", {t} } }),
                    CardStack {
                        cards,
                        revealed: shown.revealed_in(&ids),
                        open: shown.open(),
                        on_hover: move |id| reveal.write().apply(Gesture::Hover(id)),
                        on_leave: move |id| reveal.write().apply(Gesture::Leave(id)),
                        on_select: move |id| details.click(id),
                    }
                }
            }
        }
    }
}

/// The open stack card's details drawer, from `cards` (every card the screen stacks): Image and
/// Printings as a row's open them, then what `extras` draws for the card, the actions and notes
/// its row shows. It stays mounted on the drawn card, else the first, so it slides in and out;
/// nothing when `cards` is empty.
#[component]
pub(crate) fn StackDrawer(
    cards: Vec<Card>,
    #[props(default)] extras: Option<Callback<Card, Element>>,
) -> Element {
    let StackDetails {
        reveal,
        details_open,
    } = use_context::<StackDetails>();
    let overlays = try_use_context::<CardOverlays>();
    let faces = try_use_context::<RowFaces>();
    let drawn = reveal().drawn();
    let Some(card) = cards
        .iter()
        .find(|c| Some(c.scryfall_data.id) == drawn)
        .or_else(|| cards.first())
        .cloned()
    else {
        return rsx! {};
    };
    let card_id = card.scryfall_data.id;
    let shown_face = faces.map(|f| f.of(card_id));
    let on_image = overlays.map(|CardOverlays { mut preview, .. }| {
        let image_of = card.scryfall_data.clone();
        EventHandler::new(move |face: usize| preview.set(Some((image_of.clone(), face))))
    });
    let on_printings = overlays.map(|CardOverlays { mut printing, .. }| {
        let print_card = card.clone();
        EventHandler::new(move |()| printing.set(Some(print_card.clone())))
    });
    let footer = extras.map(|draw| draw.call(card.clone()));
    rsx! {
        CardDetailsSheet {
            open: details_open,
            card,
            on_image,
            on_printings,
            footer,
            class: DRAWER_CLASSES,
            face: shown_face,
            on_face_change: move |face: usize| {
                if let Some(faces) = faces {
                    faces.set(card_id, face);
                }
            },
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
