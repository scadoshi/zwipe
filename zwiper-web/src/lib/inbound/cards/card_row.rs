//! The app's card row over the kit's, with the screen-level overlays its
//! `Image` and `Printing` buttons open. The kit's row feeds the screen's
//! [`HoverStack`](components::HoverStack) itself.

use components::CardRow as SharedCardRow;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Board;

use crate::inbound::cards::image_preview::ImagePreview;
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::cards::printing_sheet::PrintingSheet;

/// Whether compact rows show their art thumbnail; a screen's `Art` chip
/// toggles it. Rows without the context show no art.
#[derive(Debug, Clone, Copy)]
pub struct ShowRowArt(pub Signal<bool>);

/// The overlays a screen's rows open: the full image and the printing
/// sheet. `Copy`; provided by [`use_card_overlays`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardOverlays {
    /// The card and face the image preview shows.
    pub preview: Signal<Option<(ScryfallData, usize)>>,
    /// The card whose printings are open.
    pub printing: Signal<Option<Card>>,
}

/// Provides the screen's [`CardOverlays`]. Mount [`CardOverlaysHost`]
/// beside the screen frame.
pub fn use_card_overlays() -> CardOverlays {
    let overlays = CardOverlays {
        preview: use_signal(|| None),
        printing: use_signal(|| None),
    };
    use_context_provider(|| overlays)
}

/// The image preview and printing sheet the screen's rows open. A saved
/// printing goes to `on_printing_save`, which swaps it into the screen's
/// list; a `read_only` sheet only browses the printings.
#[component]
pub fn CardOverlaysHost(
    overlays: CardOverlays,
    on_printing_save: EventHandler<Card>,
    #[props(default = "Printing saved")] saved_message: &'static str,
    #[props(default)] read_only: bool,
) -> Element {
    let CardOverlays {
        preview,
        mut printing,
    } = overlays;
    let mut open = use_signal(|| false);
    use_effect(move || open.set(printing().is_some()));
    use_effect(move || {
        if !open() && printing.peek().is_some() {
            printing.set(None);
        }
    });
    rsx! {
        ImagePreview { card: preview }
        if let Some(card) = printing() {
            PrintingSheet {
                key: "{card.scryfall_data.id}",
                card,
                open,
                on_save: on_printing_save,
                read_only,
                saved_message,
            }
        }
    }
}

/// A card row: the compact line, and on a click the card's detail with the
/// host's `footer`, then `Image`, `Printing` and the host's
/// `extra_actions`. One row per screen is open at a time, by
/// `expanded_card`. Under a screen's hover stack, hovering the compact
/// line shows the card's image there.
///
/// A host that keeps the shown face passes it as `face` and takes each
/// flip from `on_face_change`; the row's art, hover preview, detail and
/// Image then follow it.
///
/// The deck cards view adds the editing actions: the quantity stepper
/// (`on_qty_change`), the board moves (`on_move_to` from `current_board`)
/// and the MVP star (`mvp`, `on_toggle_mvp`), with prices in
/// `price_currency`.
#[component]
pub fn CardRow(
    card: Card,
    qty: i32,
    expanded_card: Signal<Option<Uuid>>,
    #[props(default)] extra_actions: Option<Element>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] qty_tags: Vec<String>,
    #[props(default)] on_qty_change: Option<EventHandler<i32>>,
    #[props(default)] on_move_to: Option<EventHandler<Board>>,
    #[props(default)] current_board: Option<Board>,
    #[props(default)] mvp: Option<bool>,
    #[props(default)] on_toggle_mvp: Option<EventHandler<()>>,
    #[props(default)] price_currency: PriceCurrency,
    #[props(default)] on_printing: Option<EventHandler<Card>>,
    #[props(default)] face: Option<usize>,
    #[props(default)] on_face_change: Option<EventHandler<usize>>,
) -> Element {
    let overlays = try_use_context::<CardOverlays>();
    let mut own_face = use_signal(|| 0usize);
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let show_art = try_use_context::<ShowRowArt>().map(|s| (s.0)());
    let sd_for_image = card.scryfall_data.clone();
    let on_image = overlays.map(|CardOverlays { mut preview, .. }| {
        EventHandler::new(move |()| {
            preview.set(Some((
                sd_for_image.clone(),
                face.unwrap_or_else(|| *own_face.read()),
            )));
        })
    });
    let on_printing = on_printing.or_else(|| {
        overlays.map(|CardOverlays { mut printing, .. }| {
            EventHandler::new(move |picked: Card| printing.set(Some(picked)))
        })
    });
    rsx! {
        div {
            class: "web-card-row",
            SharedCardRow {
                card,
                qty,
                expanded_card,
                show_art,
                on_image,
                on_face_change: move |f: usize| {
                    own_face.set(f);
                    if let Some(handler) = on_face_change {
                        handler.call(f);
                    }
                },
                face,
                on_qty_change,
                on_move_to,
                current_board,
                on_printing,
                mvp,
                on_toggle_mvp,
                price_currency,
                extra_actions,
                footer,
                qty_tags,
                show_classification: true,
                describe_tag,
                on_examples,
            }
        }
    }
}
