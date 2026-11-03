//! Bottom sheet for browsing and selecting card printings.
//!
//! What the sheet knows and does is `zwipe_ui::screens::printing_sheet`;
//! this file draws its state, keeps the carousel, and performs its effects.

use components::{
    ActionBar, Button, ButtonVariant, TOAST_NORMAL, ToastOptions, Toasts, use_overlay_back_action,
    use_toast,
};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_ui::screens::printing_sheet::{self as model, Action, Effect, Mode, State, Status};

use super::card_info::{CardInfoDisplay, PrintingSheetSkeleton};
use super::flippable_card_image::FlippableCardImage;
use crate::inbound::components::interactions::carousel::Carousel;
use crate::inbound::components::interactions::carousel::dots::CarouselDots;
use crate::inbound::components::interactions::carousel::state::CarouselState;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::component;
use crate::inbound::components::ui_toast;

/// Page width in pixels when the viewport can't be measured.
const FALLBACK_PAGE_WIDTH_PX: f64 = 375.0;

/// The sheet's model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    carousel: Signal<CarouselState>,
    /// The viewport width measured after the last fetch, for laying out its carousel.
    page_width: Signal<f64>,
    open: Signal<bool>,
    on_save: EventHandler<Card>,
    host_screen: &'static str,
    client: Signal<ZwipeClient>,
    toast: Toasts,
    usage_buffer: Signal<UsageBuffer>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        match effect {
            Effect::FetchPrintings { ticket, oracle_id } => {
                let client = self.client;
                let mut page_width = self.page_width;
                spawn(async move {
                    match client().fetch_printings(oracle_id).await {
                        Ok(printings) => {
                            // Measure viewport width (sheet-content uses no-pad-x so carousel is full-width)
                            let mut eval = document::eval("dioxus.send(window.innerWidth)");
                            page_width
                                .set(eval.recv::<f64>().await.unwrap_or(FALLBACK_PAGE_WIDTH_PX));
                            self.dispatch(Action::Loaded { ticket, printings });
                        }
                        Err(e) => {
                            self.usage_buffer.peek().report_error(
                                self.host_screen,
                                component::PRINTING_SHEET,
                                "load_printings",
                                &e,
                            );
                            self.toast.error(
                                e.to_user_message(),
                                ToastOptions::default().duration(TOAST_NORMAL),
                            );
                            self.dispatch(Action::Failed { ticket });
                        }
                    }
                });
            }
            Effect::Show(at) => {
                let count = self.model.peek().printings().len();
                let width = *self.page_width.peek();
                let mut carousel = self.carousel;
                carousel.set(CarouselState::new(count, at, width));
            }
            Effect::Rewind(at) => {
                let mut carousel = self.carousel;
                carousel.with_mut(|s| {
                    s.current_index = at;
                    s.drag_offset_px = 0.0;
                    s.snap_transition_ms = 0;
                });
            }
            Effect::Pick(card) => self.on_save.call(*card),
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
            Effect::Close => {
                let mut open = self.open;
                open.set(false);
            }
        }
    }
}

/// Bottom sheet for browsing all printings of a card and selecting one.
#[component]
pub(crate) fn PrintingSheet(
    card: Card,
    open: Signal<bool>,
    on_save: EventHandler<Card>,
    /// Host screen const (`vocabulary::screen`) for error-report breadcrumbs,
    /// this sheet serves several screens, and the report's aggregation axis is
    /// always the host, never the component.
    host_screen: &'static str,
    /// Browse-only: hide Save and never fire `on_save`. Used on the remove
    /// screen, where changing a card's printing would break the delete (which is
    /// keyed on the exact `scryfall_data_id` already on the deck).
    #[props(default)]
    read_only: bool,
    /// Confirmation shown after Save. Host-specific because only some hosts
    /// persist: the deck view sends a `PATCH` for the deck card, while the add screen just
    /// re-skins the card still sitting in the swipe stack, where "saved" would
    /// claim a deck change that never happened.
    #[props(default = model::SAVED)]
    saved_message: &'static str,
) -> Element {
    let mode = if read_only {
        Mode::ReadOnly
    } else {
        Mode::Pick
    };
    let driver = Driver {
        model: use_signal(|| State::new(mode, saved_message)),
        carousel: use_signal(CarouselState::empty),
        page_width: use_signal(|| FALLBACK_PAGE_WIDTH_PX),
        open,
        on_save,
        host_screen,
        client: use_context(),
        toast: use_toast(),
        usage_buffer: use_context(),
    };
    let carousel_state = driver.carousel;

    let current_scryfall_id = card.scryfall_data.id;
    let oracle_id = card.scryfall_data.oracle_id;

    // Fetch printings when the sheet opens; the model reuses the last fetch
    // when it was for this oracle_id.
    use_effect(move || {
        if !open() {
            return;
        }
        driver.dispatch(Action::Open {
            printing_id: current_scryfall_id,
            oracle_id,
        });
    });

    let current_idx = carousel_state().current_index;
    let (status, printings, visible_card, can_save, shows_save) = {
        let sheet = driver.model.read();
        (
            sheet.status(),
            sheet.printings().to_vec(),
            sheet.visible(current_idx).cloned(),
            sheet.can_save(current_idx),
            sheet.shows_save(),
        )
    };

    // OS back closes the sheet before the router sees it. This one hand-rolls
    // its own backdrop + `.bottom-sheet` markup instead of using the shared
    // `BottomSheet`, so it needs its own registration, and it mirrors the
    // backdrop exactly, warning when an unsaved printing is about to be lost.
    let dismiss = use_callback(move |(): ()| driver.dispatch(Action::Dismiss { at: current_idx }));
    use_overlay_back_action(open.into(), dismiss);

    rsx! {
        // Modal backdrop. `sheet-over` lifts the backdrop and sheet above a
        // details sheet already open under them.
        div {
            class: if open() { "sheet-backdrop sheet-over show" } else { "sheet-backdrop sheet-over" },
            onclick: move |_| driver.dispatch(Action::Dismiss { at: current_idx }),
        }

        // Bottom sheet
        div {
            class: if open() { "bottom-sheet sheet-over show" } else { "bottom-sheet sheet-over" },

            // The same backdrop orangui's BottomSheet carries, since this sheet is
            // built by hand on the same classes.
            ::components::GridCells { still: true }
            div { class: "sheet-header",
                span { style: "font-size: 1rem; color: var(--accent-primary);", {model::TITLE} }
            }

            div { class: "sheet-content no-pad-x",
                if status == Status::Loading {
                    PrintingSheetSkeleton {}
                } else if status == Status::Several {
                    // Multi-printing: carousel
                    Carousel { state: carousel_state,
                        for printing in printings.iter() {
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

                    CarouselDots { current: current_idx, total: printings.len() }

                    // Info row for currently visible printing
                    if let Some(card) = visible_card {
                        CardInfoDisplay { card }
                    }
                } else if let Some(card) = visible_card {
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
                    variant: ButtonVariant::Action,
                    onclick: move |_| driver.dispatch(Action::Back { at: current_idx }),
                    {model::BACK}
                }

                // Always there when saving is possible at all, grayed until
                // the carousel has moved off the printing it opened on.
                if shows_save {
                    Button {
                        variant: ButtonVariant::Action,
                        disabled: !can_save,
                        onclick: move |_| driver.dispatch(Action::Save { at: current_idx }),
                        {model::SAVE}
                    }
                }
            }
        }
    }
}
