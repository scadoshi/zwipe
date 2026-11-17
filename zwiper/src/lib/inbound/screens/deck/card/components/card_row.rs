use std::collections::HashMap;

use components::{CardRow as SharedCardRow, PrintingsFace, share_face, use_shared_face};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Board;

/// Context: resolve an oracle tag's description for the expandable otag reveal.
/// Provided by the deck-cards view (which holds the catalog cache); absent
/// elsewhere, so those rows keep plain, non-expandable otag chips.
#[derive(Clone, Copy)]
pub(crate) struct OtagDescribe(pub(crate) Callback<String, Option<String>>);

/// Context: open the example-cards browse for a tag slug, from the otag reveal's
/// "Examples" button. Provided by the deck-cards view, which owns the overlay.
#[derive(Clone, Copy)]
pub(crate) struct OtagExamplesOpen(pub(crate) Callback<String>);

/// Context: whether compact rows show their art-crop thumbnail. Provided by
/// the deck-cards view (its "Art" toggle chip); absent elsewhere, so other
/// hosts keep art-free rows.
#[derive(Clone, Copy)]
pub(crate) struct ShowRowArt(pub(crate) Signal<bool>);

/// Context: every card's shown face on the deck page, so a row, its stack card, its details
/// drawer and its featured card turn together. Provided by the deck cards view; absent
/// elsewhere, so other hosts' rows keep their own face.
#[derive(Clone, Copy)]
pub(crate) struct DeckFaces(pub(crate) Signal<HashMap<Uuid, usize>>);

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
    /// Host-specific buttons appended to the shared row's expanded action bar.
    #[props(default)]
    extra_actions: Option<Element>,
    /// Host content under the expanded detail's chips, before its action bar.
    #[props(default)]
    footer: Option<Element>,
    /// Tags in place of the leading qty, forwarded to the shared row.
    #[props(default)]
    qty_tags: Vec<String>,
) -> Element {
    let scryfall_data_for_preview = card.scryfall_data.clone();
    // Which face the expanded detail is flipped to (0 = front), so the fullscreen
    // preview opens on the side the user was viewing. Kept in sync via the shared
    // detail's `on_face_change`.
    let mut current_face = use_signal(|| 0usize);
    // The deck page's shared face for the card, where it keeps them.
    let deck_faces = try_use_context::<DeckFaces>().map(|DeckFaces(faces)| faces);
    let shared_face = use_shared_face(deck_faces, scryfall_data_for_preview.id);
    let share = share_face(deck_faces, scryfall_data_for_preview.id);
    // Deck's price-target currency, shared by the parent view via context so
    // every row prices in the same currency without threading a prop through
    // each call site. Falls back to USD outside that provider.
    let price_currency = try_use_context::<Signal<PriceCurrency>>().map_or_default(|c| c());
    // Otag reveal plumbing (deck-cards view provides it): a description lookup and
    // an examples opener. Absent elsewhere, so those rows keep plain otag chips.
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    // `None` where the view doesn't provide the toggle (rows outside the
    // deck-cards screen render art-free); `Some(visible)` keeps the thumb
    // mounted so the toggle eases it in and out.
    let show_art = try_use_context::<ShowRowArt>().map(|s| (s.0)());
    // The sheet the row opens starts on the face the row shows.
    let printings_face = try_use_context::<PrintingsFace>();
    let on_printing = on_printing.map(|open| {
        EventHandler::new(move |picked: Card| {
            if let Some(PrintingsFace(mut face)) = printings_face {
                face.set(*current_face.peek());
            }
            open.call(picked);
        })
    });
    rsx! {
        SharedCardRow {
            card,
            qty,
            expanded_card,
            show_art,
            on_image: move |()| {
                let face = shared_face.unwrap_or(current_face)();
                preview_card.set(Some((scryfall_data_for_preview.clone(), face)));
                preview_dismissing.set(false);
            },
            on_face_change: move |face: usize| {
                current_face.set(face);
                if let Some(share) = share {
                    share.call(face);
                }
            },
            face: shared_face.map(|face| face()),
            on_qty_change,
            on_move_to,
            current_board,
            on_printing,
            mvp,
            on_toggle_mvp,
            extra_actions,
            footer,
            qty_tags,
            price_currency,
            show_classification: true,
            describe_tag,
            on_examples,
            // The deck page's rows, the one place that keeps the shared faces, carry Send to.
            send_to: deck_faces.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    /// A transform card: the fixture with Delver of Secrets' two faces.
    fn delver() -> Card {
        let json = include_str!("../../../../../../../../zwipe-ui/src/test_support/card.json");
        let mut card: Value = serde_json::from_str(json).expect("the fixture parses");
        let sd = card.get_mut("scryfall_data").expect("its data");
        sd["oracle_text"] = Value::Null;
        sd["card_faces"] = json!([
            { "object": "card_face", "name": "Delver of Secrets", "mana_cost": "{U}",
              "type_line": "Creature — Human Wizard", "oracle_text": "Upkeep." },
            { "object": "card_face", "name": "Insectile Aberration", "mana_cost": "",
              "type_line": "Creature — Human Insect", "oracle_text": "Flying" }
        ]);
        serde_json::from_value(card).expect("the two-faced card parses")
    }

    #[test]
    fn an_open_row_shows_the_face_its_deck_page_shares() {
        fn app() -> Element {
            let card = delver();
            let id = card.scryfall_data.id;
            let faces = use_signal(|| HashMap::from([(id, 1_usize)]));
            use_context_provider(|| DeckFaces(faces));
            let expanded_card = use_signal(|| Some(id));
            rsx! {
                CardRow {
                    card,
                    qty: 1,
                    expanded_card,
                    preview_card: use_signal(|| None),
                    preview_dismissing: use_signal(|| false),
                    on_qty_change: None,
                    on_move_to: None,
                    current_board: None,
                    on_printing: None,
                    mvp: None,
                    on_toggle_mvp: None,
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("Creature — Human Insect"),
            "the shared back: {html}"
        );
    }
}
