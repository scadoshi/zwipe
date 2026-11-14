//! The printing sheet draws the printings its host's [`PrintingsSource`] fetched, and offers Save
//! only to a host that saves.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::collections::HashMap;

use components::{PrintingSheet, PrintingsFetch, PrintingsSource};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

fn card() -> Card {
    serde_json::from_str(CARD_JSON).unwrap()
}

/// Renders `app` once its effects have run, inside the runtime the kit's sheets time their
/// entrance on.
fn render(app: fn() -> Element) -> String {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let _runtime = runtime.enter();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// The card's printings as `fetch`, under a source that records each request in `asked`.
fn source(fetch: PrintingsFetch, asked: Signal<Vec<Uuid>>) -> PrintingsSource {
    let oracle = card().scryfall_data.oracle_id.unwrap();
    let mut asked = asked;
    PrintingsSource {
        results: Signal::new(HashMap::from([(oracle, fetch)])),
        request: Callback::new(move |id| asked.write().push(id)),
    }
}

/// The card's open sheet with its printings fetched, saving when `saves`.
fn open_sheet(saves: bool) -> Element {
    let asked = use_signal(Vec::new);
    use_context_provider(|| source(PrintingsFetch::Ready(vec![card()]), asked));
    let open = use_signal(|| true);
    let on_save = saves.then(|| EventHandler::new(|_: Card| {}));
    rsx! {
        PrintingSheet { card: card(), open, on_save }
        p { class: "asked", "{asked.read().len()}" }
    }
}

#[test]
fn an_open_sheet_asks_its_host_for_the_printings_and_draws_them() {
    let html = render(|| open_sheet(true));
    assert!(
        html.contains(r#"<p class="asked">1</p>"#),
        "one request: {html}"
    );
    assert!(
        html.contains("stat-chip stat-chip-money"),
        "the printing's prices: {html}"
    );
}

#[test]
fn a_sheet_that_saves_offers_save() {
    let html = render(|| open_sheet(true));
    assert!(html.contains(r#"aria-label="Save""#), "{html}");
}

#[test]
fn a_sheet_that_only_looks_offers_no_save() {
    let html = render(|| open_sheet(false));
    assert!(!html.contains(r#"aria-label="Save""#), "{html}");
    assert!(
        html.contains("printing-chips"),
        "it still shows the printing: {html}"
    );
}

#[test]
fn a_failed_fetch_offers_retry() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        use_context_provider(|| source(PrintingsFetch::Failed, asked));
        let open = use_signal(|| true);
        rsx! {
            PrintingSheet { card: card(), open }
        }
    }
    let html = render(app);
    assert!(html.contains("Retry"), "{html}");
}

#[test]
fn the_header_names_the_card() {
    let html = render(|| open_sheet(true));
    assert!(
        html.contains(r#"<span class="sheet-title">Research Assistant</span>"#),
        "the card's name in the title bar: {html}"
    );
    assert!(
        !html.contains(r#"<span class="sheet-title">Printings</span>"#),
        "{html}"
    );
}

#[test]
fn the_foot_credits_the_printing_s_artist_beside_the_image_icon() {
    let html = render(|| open_sheet(true));
    let (_, chip) = html
        .split_once(r#"class="detail-chip detail-chip-artist printing-artist""#)
        .unwrap();
    let (chip, _) = chip.split_once("</span>").unwrap();
    assert!(
        chip.contains(components::IMAGE_ICON),
        "the Image icon: {chip}"
    );
    assert!(chip.contains("Svetlin Velinov"), "the artist: {chip}");
}

#[test]
fn a_printing_with_no_artist_has_no_artist_chip() {
    fn app() -> Element {
        let mut nameless = card();
        nameless.scryfall_data.artist = None;
        let asked = use_signal(Vec::new);
        let fetched = nameless.clone();
        use_context_provider(move || source(PrintingsFetch::Ready(vec![fetched.clone()]), asked));
        let open = use_signal(|| true);
        rsx! {
            PrintingSheet { card: nameless, open }
        }
    }
    let html = render(app);
    assert!(html.contains("printing-chips"), "{html}");
    assert!(!html.contains("detail-chip-artist"), "{html}");
}

#[test]
fn the_narrow_sheet_shows_its_printing_as_one_row_of_chips() {
    let html = render(|| open_sheet(true));
    let (_, row) = html.split_once(r#"class="printing-chips""#).unwrap();
    let (row, _) = row.split_once("</div>").unwrap();
    let data = card().scryfall_data;
    let at = |needle: &str| {
        row.find(needle)
            .unwrap_or_else(|| panic!("{needle:?} in the row: {row}"))
    };
    let set = at(&format!(
        r#"class="stat-chip stat-chip-p3 printing-chips-set">{}"#,
        data.set_name
    ));
    let number = at(&format!(
        r#"class="stat-chip stat-chip-muted">#{}"#,
        data.collector_number
    ));
    let place = at(r#"class="stat-chip stat-chip-muted">1 of 1"#);
    let artist = at("detail-chip detail-chip-artist printing-artist");
    let price = at(r#"class="stat-chip stat-chip-money""#);
    assert!(
        set < number && number < place && place < artist && artist < price,
        "set, number, place, artist, then prices: {row}"
    );
    assert!(!html.contains("printing-facts"), "no second line: {html}");
}
