//! Bottom sheet for browsing and selecting card printings.

use super::{
    card_info::{CardInfoDisplay, PrintingSheetSkeleton},
    flippable_card_image::FlippableCardImage,
};
use crate::{
    inbound::components::interactions::carousel::{
        Carousel, dots::CarouselDots, state::CarouselState,
    },
    outbound::client::{ZwipeClient, card::get_printings::ClientGetPrintings},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::card::{Card, scryfall_data::ImageSize};

/// Bottom sheet for browsing all printings of a card and selecting one.
#[component]
pub(crate) fn PrintingSheet(
    card: Card,
    mut open: Signal<bool>,
    on_save: EventHandler<Card>,
) -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    let mut printings: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| false);
    let mut cached_oracle_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut carousel_state: Signal<CarouselState> = use_signal(CarouselState::empty);
    let mut initial_index: Signal<usize> = use_signal(|| 0);

    let current_scryfall_id = card.scryfall_data.id;
    let oracle_id = card.scryfall_data.oracle_id;

    // Fetch printings when sheet opens (cache by oracle_id)
    use_effect(move || {
        if !open() {
            return;
        }
        let Some(oid) = oracle_id else {
            return;
        };

        // Skip fetch if already cached for this oracle_id
        if cached_oracle_id() == Some(oid) {
            let idx = printings()
                .iter()
                .position(|p| p.scryfall_data.id == current_scryfall_id)
                .unwrap_or(0);
            initial_index.set(idx);
            carousel_state.with_mut(|s| {
                s.current_index = idx;
                s.drag_offset_px = 0.0;
                s.snap_transition_ms = 0;
            });
            return;
        }

        is_loading.set(true);

        spawn(async move {
            match client().get_printings(oid).await {
                Ok(cards) => {
                    let idx = cards
                        .iter()
                        .position(|p| p.scryfall_data.id == current_scryfall_id)
                        .unwrap_or(0);
                    let count = cards.len();
                    printings.set(cards);
                    initial_index.set(idx);
                    cached_oracle_id.set(Some(oid));

                    // Measure viewport width (modal-content uses no-pad-x so carousel is full-width)
                    let mut eval = document::eval("dioxus.send(window.innerWidth)");
                    let page_width: f64 = eval.recv::<f64>().await.unwrap_or(375.0);
                    carousel_state.set(CarouselState::new(count, idx, page_width));
                }
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
            is_loading.set(false);
        });
    });

    let current_idx = carousel_state().current_index;
    let has_changed = !printings().is_empty() && current_idx != initial_index();
    let visible_card = printings().get(current_idx).cloned();

    rsx! {
        // Modal backdrop
        div {
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| {
                if has_changed {
                    toast.warning(
                        "Printing discarded".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                open.set(false);
            },
        }

        // Bottom sheet
        div {
            class: if open() { "bottom-sheet show" } else { "bottom-sheet" },

            div { class: "modal-header",
                span { style: "font-size: 1rem; color: var(--accent-tertiary);", "Printings" }
            }

            div { class: "modal-content no-pad-x",
                if is_loading() {
                    PrintingSheetSkeleton {}
                } else if printings().len() > 1 {
                    // Multi-printing: carousel
                    Carousel { state: carousel_state,
                        for printing in printings().iter() {
                            {
                                let id = printing.scryfall_data.id;
                                let sd = printing.scryfall_data.clone();
                                rsx! {
                                    div { class: "carousel-page", key: "{id}",
                                        FlippableCardImage {
                                            sd,
                                            size: ImageSize::Large,
                                            class: "carousel-card-image".to_string(),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    CarouselDots { current: current_idx, total: printings().len() }

                    // Info row for currently visible printing
                    if let Some(card) = visible_card.clone() {
                        CardInfoDisplay { card }
                    }
                } else if let Some(card) = visible_card.clone() {
                    // Single printing: just show the image, no carousel
                    if card.scryfall_data.primary_image_url(ImageSize::Large).is_some() {
                        div { style: "display: flex; justify-content: center; margin-bottom: 0.75rem;",
                            FlippableCardImage {
                                sd: card.scryfall_data.clone(),
                                size: ImageSize::Large,
                                class: "carousel-card-image".to_string(),
                            }
                        }
                    }

                    CardInfoDisplay { card }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        if has_changed {
                            toast.info(
                                "Printing discarded".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        open.set(false);
                    },
                    "Close"
                }

                if has_changed {
                    if let Some(new_card) = visible_card {
                        {
                            rsx! {
                                Button {
                                    variant: ButtonVariant::Util,
                                    onclick: move |_| {
                                        on_save(new_card.clone());
                                        toast.info(
                                            "Printing saved".to_string(),
                                            ToastOptions::default().duration(Duration::from_millis(1500)),
                                        );
                                        open.set(false);
                                    },
                                    "Save"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
