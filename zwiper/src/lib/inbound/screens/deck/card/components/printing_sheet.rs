//! Bottom sheet for browsing and selecting card printings.
//!
//! What the sheet knows and does is `zwipe_ui::screens::printing_sheet`;
//! this file draws its state as components' cover flow, keeps the shown
//! index, and performs its effects.

use components::{
    BottomSheet, CoverFlow, DockAction, LoadFailed, TOAST_NORMAL, ToastOptions, Toasts, icons,
    use_toast,
};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_ui::screens::printing_sheet::{self as model, Action, Effect, Mode, State, Status};

use super::card_info::{CardInfoDisplay, PrintingSheetSkeleton};
use super::flippable_card_image::FlippableCardImage;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::component;
use crate::inbound::components::ui_toast;

/// The sheet's model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    /// The printing the cover flow shows in its middle.
    index: Signal<usize>,
    open: Signal<bool>,
    on_save: Option<EventHandler<Card>>,
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
                spawn(async move {
                    match client().fetch_printings(oracle_id).await {
                        Ok(printings) => {
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
            Effect::Show(at) | Effect::Rewind(at) => {
                let mut index = self.index;
                index.set(at);
            }
            Effect::Pick(card) => {
                if let Some(on_save) = self.on_save {
                    on_save.call(*card);
                }
            }
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
    /// Takes the picked printing. Without one the sheet only browses: no
    /// Save and no toast, as where nothing would keep the pick (Remove cards,
    /// whose delete is keyed on the printing already on the deck, a staple,
    /// Activity, Examples).
    #[props(default)]
    on_save: Option<EventHandler<Card>>,
    /// Host screen const (`vocabulary::screen`) for error-report breadcrumbs,
    /// this sheet serves several screens, and the report's aggregation axis is
    /// always the host, never the component.
    host_screen: &'static str,
    /// Confirmation shown after Save. Host-specific because only some hosts
    /// persist: the deck view sends a `PATCH` for the deck card, while the add screen just
    /// re-skins the card still sitting in the swipe stack, where "saved" would
    /// claim a deck change that never happened.
    #[props(default = model::SAVED)]
    saved_message: &'static str,
) -> Element {
    let mode = if on_save.is_some() {
        Mode::Pick
    } else {
        Mode::ReadOnly
    };
    let driver = Driver {
        model: use_signal(|| State::new(mode, saved_message)),
        index: use_signal(|| 0usize),
        open,
        on_save,
        host_screen,
        client: use_context(),
        toast: use_toast(),
        usage_buffer: use_context(),
    };
    let index = driver.index;

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

    let current_idx = index();
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

    rsx! {
        // `sheet-over` lifts the backdrop and sheet above a details sheet
        // already open under them. A backdrop tap, the ×, a drag down or the
        // OS back gesture leave the same way, warning when an unsaved printing
        // is about to be lost.
        div { class: "sheet-over",
            BottomSheet {
                open,
                title: model::TITLE.to_string(),
                closable: true,
                on_dismiss: move |()| driver.dispatch(Action::Dismiss { at: current_idx }),

                div { class: "printing-sheet-body",
                    if status == Status::Loading {
                        PrintingSheetSkeleton {}
                    } else if status == Status::Failed {
                        LoadFailed { message: model::LOAD_FAILED, on_retry: move |()| driver.dispatch(Action::Retry) }
                    } else if status == Status::Several {
                        CoverFlow { printings, index }

                        // Info row for the printing in the middle
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

                // Save floats at the sheet's bottom right whenever saving is
                // possible at all, grayed until the carousel has moved off the
                // printing it opened on.
                if shows_save {
                    div { class: "action-dock action-dock-right sheet-dock",
                        DockAction {
                            label: model::SAVE,
                            icon: icons::CHECK,
                            disabled: !can_save,
                            onclick: move |()| driver.dispatch(Action::Save { at: current_idx }),
                        }
                    }
                }
            }
        }
    }
}
