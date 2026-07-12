use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_components::CardRow as SharedCardRow;
use zwipe_core::domain::{
    card::{
        Card, scryfall_data::ScryfallData, search_card::card_filter::price_currency::PriceCurrency,
    },
    deck::Board,
};

/// Thin wrapper over the shared [`SharedCardRow`]: wires the app's fullscreen
/// [`ImagePreview`] signals into the shared row's `on_image` action so call
/// sites keep passing `preview_card`/`preview_dismissing` unchanged.
///
/// [`ImagePreview`]: super::image_preview::ImagePreview
#[component]
pub(crate) fn CardRow(
    card: Card,
    qty: i32,
    expanded_card: Signal<Option<Uuid>>,
    mut preview_card: Signal<Option<ScryfallData>>,
    mut preview_dismissing: Signal<bool>,
    on_qty_change: Option<EventHandler<i32>>,
    on_move_to: Option<EventHandler<Board>>,
    current_board: Option<Board>,
    on_printing: Option<EventHandler<Card>>,
    /// MVP star state: `Some(true)` filled, `Some(false)` outline, `None` = no
    /// star rendered (non-mainboard rows, tokens, command zone).
    mvp: Option<bool>,
    on_toggle_mvp: Option<EventHandler<()>>,
) -> Element {
    let scryfall_data_for_preview = card.scryfall_data.clone();
    // Deck's price-target currency, shared by the parent view via context so
    // every row prices in the same currency without threading a prop through
    // each call site. Falls back to USD outside that provider.
    let price_currency = try_use_context::<Signal<PriceCurrency>>()
        .map(|c| c())
        .unwrap_or_default();
    rsx! {
        SharedCardRow {
            card,
            qty,
            expanded_card,
            on_image: move |()| {
                preview_card.set(Some(scryfall_data_for_preview.clone()));
                preview_dismissing.set(false);
            },
            on_qty_change,
            on_move_to,
            current_board,
            on_printing,
            mvp,
            on_toggle_mvp,
            price_currency,
            show_classification: true,
        }
    }
}
