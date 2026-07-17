use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_components::CardRow as SharedCardRow;
use zwipe_core::domain::{
    card::{
        Card, scryfall_data::ScryfallData, search_card::card_filter::price_currency::PriceCurrency,
    },
    deck::Board,
};

/// Context: resolve an oracle tag's description for the expandable otag reveal.
/// Provided by the deck-cards view (which holds the catalog cache); absent
/// elsewhere, so those rows keep plain, non-expandable otag chips.
#[derive(Clone, Copy)]
pub(crate) struct OtagDescribe(pub(crate) Callback<String, Option<String>>);

/// Context: open the example-cards browse for a tag slug, from the otag reveal's
/// "Examples" button. Provided by the deck-cards view, which owns the overlay.
#[derive(Clone, Copy)]
pub(crate) struct OtagExamplesOpen(pub(crate) Callback<String>);

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
    mut preview_card: Signal<Option<(ScryfallData, usize)>>,
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
    // Which face the expanded detail is flipped to (0 = front), so the fullscreen
    // preview opens on the side the user was viewing. Kept in sync via the shared
    // detail's `on_face_change`.
    let mut current_face = use_signal(|| 0usize);
    // Deck's price-target currency, shared by the parent view via context so
    // every row prices in the same currency without threading a prop through
    // each call site. Falls back to USD outside that provider.
    let price_currency = try_use_context::<Signal<PriceCurrency>>()
        .map(|c| c())
        .unwrap_or_default();
    // Otag reveal plumbing (deck-cards view provides it): a description lookup and
    // an examples opener. Absent elsewhere, so those rows keep plain otag chips.
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    rsx! {
        SharedCardRow {
            card,
            qty,
            expanded_card,
            on_image: move |()| {
                preview_card.set(Some((scryfall_data_for_preview.clone(), current_face())));
                preview_dismissing.set(false);
            },
            on_face_change: move |face: usize| current_face.set(face),
            on_qty_change,
            on_move_to,
            current_board,
            on_printing,
            mvp,
            on_toggle_mvp,
            price_currency,
            show_classification: true,
            describe_tag,
            on_examples,
        }
    }
}
