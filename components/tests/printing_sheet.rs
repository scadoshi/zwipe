//! The printing sheet draws the printings its host's [`PrintingsSource`] fetched, and offers Save
//! only to a host that saves.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::collections::{HashMap, HashSet};

use components::{MeldFetch, MeldSource, PrintingSheet, PrintingsFetch, PrintingsSource};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

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
        .split_once(r#"class="stat-chip stat-chip-p6 printing-artist""#)
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
    assert!(!html.contains("printing-artist"), "{html}");
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
    let artist = at("stat-chip stat-chip-p6 printing-artist");
    let price = at(r#"class="stat-chip stat-chip-money""#);
    assert!(
        set < number && number < place && place < artist && artist < price,
        "set, number, place, artist, then prices: {row}"
    );
    assert!(!html.contains("printing-facts"), "no second line: {html}");
}

/// Each face's large image in [`two_faced`].
const FRONT_LARGE: &str = "https://cards.example/front-large.jpg";
const BACK_LARGE: &str = "https://cards.example/back-large.jpg";

/// The shared card made a transform card, each face with its own images.
fn two_faced() -> Card {
    let mut card: serde_json::Value = serde_json::from_str(CARD_JSON).unwrap();
    let sd = &mut card["scryfall_data"];
    sd["layout"] = "transform".into();
    sd["image_uris"] = serde_json::Value::Null;
    sd["oracle_text"] = serde_json::Value::Null;
    let face = |side: &str, name: &str| {
        let uris: serde_json::Map<String, serde_json::Value> =
            ["small", "normal", "large", "png", "art_crop", "border_crop"]
                .iter()
                .map(|size| {
                    (
                        (*size).to_owned(),
                        format!("https://cards.example/{side}-{size}.jpg").into(),
                    )
                })
                .collect();
        serde_json::json!({
            "object": "card_face", "name": name, "mana_cost": "", "type_line": "Creature",
            "oracle_text": format!("{name} rules."), "image_uris": uris,
        })
    };
    sd["card_faces"] = serde_json::json!([face("front", "Front Face"), face("back", "Back Face")]);
    serde_json::from_value(card).unwrap()
}

/// The melded card's Scryfall id, for [`meld_half`]'s first printing.
const RESULT_ID: &str = "00000000-0000-0000-0000-00000000beef";
/// The melded card's Scryfall id for the other printing.
const OTHER_RESULT_ID: &str = "00000000-0000-0000-0000-00000000cafe";
/// The other printing's Scryfall id.
const OTHER_PRINTING_ID: &str = "00000000-0000-0000-0000-0000000000a2";

/// The shared card made a meld half under `id` whose result is `result`.
fn meld_half_as(id: &str, result: &str) -> Card {
    let mut card: serde_json::Value = serde_json::from_str(CARD_JSON).unwrap();
    card["card_profile"]["scryfall_data_id"] = id.into();
    let sd = &mut card["scryfall_data"];
    sd["id"] = id.into();
    sd["layout"] = "meld".into();
    sd["all_parts"] = serde_json::json!([
        { "object": "related_card", "id": id, "component": "meld_part",
          "name": "This Half", "type_line": "Creature", "uri": "https://api.scryfall.com/cards/x" },
        { "object": "related_card", "id": result, "component": "meld_result",
          "name": "The Melded", "type_line": "Creature", "uri": "https://api.scryfall.com/cards/y" },
    ]);
    serde_json::from_value(card).unwrap()
}

/// The shared card made a meld half whose result is [`RESULT_ID`].
fn meld_half() -> Card {
    meld_half_as(&card().scryfall_data.id.to_string(), RESULT_ID)
}

/// The melded card under [`RESULT_ID`], with its own images.
fn melded() -> ScryfallData {
    let mut card: serde_json::Value = serde_json::from_str(CARD_JSON).unwrap();
    let sd = &mut card["scryfall_data"];
    sd["id"] = RESULT_ID.into();
    sd["name"] = "The Melded".into();
    for size in ["small", "normal", "large", "png", "art_crop", "border_crop"] {
        sd["image_uris"][size] = format!("https://cards.example/melded-{size}.jpg").into();
    }
    serde_json::from_value(sd.clone()).unwrap()
}

/// The sheet's foot row, where Select (Save) sits.
fn foot_row(html: &str) -> &str {
    let (_, row) = html
        .split_once(r#"class="web-printing-foot-row""#)
        .unwrap_or_else(|| panic!("a foot row: {html}"));
    row
}

/// The dock in the sheet's foot row, where Select (Save) sits.
fn dock(html: &str) -> &str {
    let row = foot_row(html);
    let (_, dock) = row
        .split_once(r#"class="action-dock web-printing-save""#)
        .unwrap_or_else(|| panic!("a dock in the foot row: {row}"));
    dock
}

#[test]
fn a_two_faced_card_s_flip_sits_in_save_s_dock_behind_a_rule() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        use_context_provider(|| source(PrintingsFetch::Ready(vec![two_faced()]), asked));
        let open = use_signal(|| true);
        rsx! {
            PrintingSheet { card: two_faced(), open, on_save: |_: Card| {} }
        }
    }
    let html = render(app);
    let dock = dock(&html);
    let flip = dock
        .find(r#"data-action="flip""#)
        .unwrap_or_else(|| panic!("Flip in the dock: {dock}"));
    let rule = dock
        .find(r#"class="action-dock-divider""#)
        .unwrap_or_else(|| panic!("a rule in the dock: {dock}"));
    let save = dock
        .find(r#"aria-label="Save""#)
        .unwrap_or_else(|| panic!("Save in the dock: {dock}"));
    assert!(
        flip < rule && rule < save,
        "Flip, the rule, then Save: {dock}"
    );
    assert_eq!(
        dock.matches(r#"class="action-dock-btn""#).count(),
        2,
        "Flip is a dock button the size of Save: {dock}"
    );
    assert!(
        dock.contains(components::icons::FLIP),
        "the Flip icon: {dock}"
    );
}

#[test]
fn a_sheet_that_only_looks_docks_flip_alone_with_no_rule() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        use_context_provider(|| source(PrintingsFetch::Ready(vec![two_faced()]), asked));
        let open = use_signal(|| true);
        rsx! {
            PrintingSheet { card: two_faced(), open }
        }
    }
    let html = render(app);
    let dock = dock(&html);
    assert!(dock.contains(r#"data-action="flip""#), "{dock}");
    assert!(!dock.contains("action-dock-divider"), "{dock}");
    assert!(!dock.contains(r#"aria-label="Save""#), "{dock}");
}

#[test]
fn a_two_faced_card_s_sheet_opens_on_the_face_its_row_shows() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        use_context_provider(|| source(PrintingsFetch::Ready(vec![two_faced()]), asked));
        let open = use_signal(|| true);
        rsx! {
            PrintingSheet { card: two_faced(), open, face: 1 }
        }
    }
    let html = render(app);
    assert!(html.contains(BACK_LARGE), "the back: {html}");
    assert!(!html.contains(FRONT_LARGE), "not the front: {html}");
}

#[test]
fn a_meld_half_s_sheet_offers_meld() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        use_context_provider(|| source(PrintingsFetch::Ready(vec![meld_half()]), asked));
        use_context_provider(|| MeldSource {
            results: Signal::new(HashMap::new()),
            on: Signal::new(HashSet::new()),
            request: Callback::new(|_: Uuid| {}),
        });
        let open = use_signal(|| true);
        rsx! {
            PrintingSheet { card: meld_half(), open }
        }
    }
    let html = render(app);
    let dock = dock(&html);
    assert!(dock.contains(r#"data-action="meld""#), "{dock}");
    assert!(dock.contains(r#"aria-pressed="false""#), "off: {dock}");
    assert!(!dock.contains("action-dock-on"), "unlit: {dock}");
}

#[test]
fn a_melded_row_s_sheet_opens_melded_and_melds_every_printing() {
    fn app() -> Element {
        let half = meld_half();
        let other = meld_half_as(OTHER_PRINTING_ID, OTHER_RESULT_ID);
        let asked = use_signal(Vec::new);
        let printings = vec![half.clone(), other];
        use_context_provider(move || source(PrintingsFetch::Ready(printings.clone()), asked));
        let result = Uuid::parse_str(RESULT_ID).unwrap();
        let mut melds_asked: Signal<Vec<Uuid>> = use_signal(Vec::new);
        let id = half.scryfall_data.id;
        use_context_provider(move || MeldSource {
            results: Signal::new(HashMap::from([(
                result,
                MeldFetch::Ready(Box::new(melded())),
            )])),
            on: Signal::new(HashSet::from([id])),
            request: Callback::new(move |id: Uuid| melds_asked.write().push(id)),
        });
        let open = use_signal(|| true);
        rsx! {
            PrintingSheet { card: half, open }
            p { class: "melds-asked", "{melds_asked.read().len()}:{melds_asked.read().first().copied().unwrap_or_default()}" }
        }
    }
    let html = render(app);
    assert!(
        html.contains("https://cards.example/melded-large.jpg"),
        "the melded card in the opened printing's place: {html}"
    );
    assert!(
        html.contains(&format!(
            r#"<p class="melds-asked">1:{OTHER_RESULT_ID}</p>"#
        )),
        "the other printing's melded card is asked for: {html}"
    );
    let dock = dock(&html);
    assert!(dock.contains(r#"aria-pressed="true""#), "Meld on: {dock}");
    assert!(
        dock.contains(r#"class="action-dock-btn action-dock-on""#),
        "lit as the dock lights a toggle: {dock}"
    );
}

#[test]
fn a_single_faced_card_s_sheet_offers_neither_flip_nor_meld() {
    let html = render(|| open_sheet(true));
    assert!(!html.contains(r#"data-action="flip""#), "{html}");
    assert!(!html.contains(r#"data-action="meld""#), "{html}");
}
