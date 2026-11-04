//! The HTML gate: the components zwiper and zite render keep their exact
//! markup. Each case renders a component with props a host passes today and
//! compares the result to a saved copy in `tests/html/`. Run with
//! `BLESS_HTML=1` to rewrite the copies after a deliberate markup change.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use components::{
    Bar, BarChart, BarOrientation, ChangelogFilter, ChangelogFilters, ChangelogLook,
    ChangelogReleases, ColorChart, ColorMap, ColorMapCounts, ColorMapGhost, DeckCharts, DrawOdds,
    FlippableCardImage, LabelCase,
};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};
use zwipe_core::http::contracts::changelog::{HttpChangelog, HttpRelease};

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

/// Compares `html` with the saved copy `name`, or saves it under `BLESS_HTML`.
fn golden(name: &str, html: &str) {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "html",
        &format!("{name}.html"),
    ]
    .iter()
    .collect();
    if std::env::var_os("BLESS_HTML").is_some() {
        fs::write(&path, html).unwrap();
        return;
    }
    let saved =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("no saved copy at {}", path.display()));
    assert_eq!(html, saved, "{name} renders different markup");
}

/// Mounts `app`, lets its effects run, and renders it.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

const SEED: u64 = 0x5eed_c0de_1234_5678;

#[test]
fn color_map_sample() {
    golden(
        "color_map_sample",
        &dioxus_ssr::render_element(rsx! {
            ColorMap {
                counts: ColorMapCounts {
                    cards: [180, 200, 70, 190, 80],
                    colorless: 189,
                },
                seed: SEED,
            }
        }),
    );
}

#[test]
fn color_map_small_with_key_action() {
    golden(
        "color_map_small",
        &dioxus_ssr::render_element(rsx! {
            ColorMap {
                counts: ColorMapCounts {
                    cards: [3, 0, 0, 1, 0],
                    colorless: 1,
                },
                seed: 7,
                key_action: rsx! {
                    button { class: "btn-action", "Open" }
                },
            }
        }),
    );
}

#[test]
fn color_map_capped() {
    golden(
        "color_map_capped",
        &dioxus_ssr::render_element(rsx! {
            ColorMap {
                counts: ColorMapCounts {
                    cards: [9000, 9000, 9000, 9000, 9000],
                    colorless: 0,
                },
                seed: 42,
            }
        }),
    );
}

#[test]
fn color_map_colorless_only() {
    golden(
        "color_map_colorless",
        &dioxus_ssr::render_element(rsx! {
            ColorMap {
                counts: ColorMapCounts {
                    cards: [0; 5],
                    colorless: 5,
                },
                seed: 3,
            }
        }),
    );
}

#[test]
fn color_map_ghost() {
    golden(
        "color_map_ghost",
        &dioxus_ssr::render_element(rsx! {
            ColorMapGhost {}
        }),
    );
}

fn release(version: &str, date: &str) -> HttpRelease {
    HttpRelease {
        version: version.to_string(),
        date: date.to_string(),
        released_on: None,
        entries: vec![format!("Entry for {version}")],
    }
}

/// Dec 2026 into Jan 2027 with an upcoming release, and one release whose
/// day cannot be read.
fn two_years() -> HttpChangelog {
    HttpChangelog {
        upcoming: vec![release("1.16.0", "")],
        releases: vec![
            release("1.15.1", "Jan 20, 2027"),
            release("1.15.0", "Jan 4, 2027"),
            release("1.14.2", "Dec 30, 2026"),
            release("1.14.1", "Dec 9, 2026"),
            release("1.14.0", "Nov 2, 2026"),
            release("0.9.0", "someday"),
        ],
    }
}

#[test]
fn changelog_latest() {
    golden(
        "changelog_latest",
        &render(|| {
            let selection = use_signal(|| ChangelogFilter::Latest);
            rsx! {
                ChangelogFilters { data: two_years(), selection }
                ChangelogReleases { data: two_years(), selection }
            }
        }),
    );
}

#[test]
fn changelog_anchored_with_links() {
    golden(
        "changelog_anchored",
        &render(|| {
            let selection = use_signal(|| ChangelogFilter::Latest);
            rsx! {
                ChangelogFilters { data: two_years(), selection, anchor: "#v1-14-1" }
                ChangelogReleases { data: two_years(), selection, link_versions: true }
            }
        }),
    );
}

#[test]
fn changelog_app_panels() {
    golden(
        "changelog_app_panels",
        &render(|| {
            let selection = use_signal(|| ChangelogFilter::Latest);
            rsx! {
                ChangelogFilters { data: two_years(), selection, look: ChangelogLook::App }
                ChangelogReleases { data: two_years(), selection, look: ChangelogLook::App }
            }
        }),
    );
}

#[test]
fn changelog_anchored_upcoming() {
    golden(
        "changelog_anchored_upcoming",
        &render(|| {
            let selection = use_signal(|| ChangelogFilter::Latest);
            rsx! {
                ChangelogFilters { data: two_years(), selection, anchor: "v1-16-0" }
                ChangelogReleases { data: two_years(), selection }
            }
        }),
    );
}

fn bars() -> Vec<Bar> {
    vec![
        Bar::new("Kept", 12, 100),
        Bar::new("Passed", 7, 58),
        Bar::new("Maybe", 0, 0),
    ]
}

#[test]
fn bar_chart_vertical() {
    golden(
        "bar_chart_vertical",
        &dioxus_ssr::render_element(rsx! {
            BarChart {
                label: "Totals",
                bars: bars(),
                hint: rsx! {
                    button { "?" }
                },
            }
        }),
    );
}

#[test]
fn bar_chart_vertical_tappable() {
    golden(
        "bar_chart_vertical_tappable",
        &render(|| {
            rsx! {
                BarChart {
                    bars: bars(),
                    on_bar: Callback::new(|_: String| {}),
                    selected: "Passed",
                }
            }
        }),
    );
}

#[test]
fn bar_chart_rows_with_suffix() {
    golden(
        "bar_chart_rows",
        &dioxus_ssr::render_element(rsx! {
            BarChart {
                label: "Keep rate",
                bars: bars(),
                orientation: BarOrientation::Horizontal,
                label_width: "8ch",
                count_suffix: "%",
            }
        }),
    );
}

#[test]
fn bar_chart_chip_rows() {
    golden(
        "bar_chart_chip_rows",
        &render(|| {
            let bars = vec![
                Bar {
                    key: "card-draw".to_string(),
                    label: "draw".to_string(),
                    count: 9,
                    pct: 100,
                    revealable: true,
                },
                Bar::new("ramp", 4, 44),
                Bar::new("lands", 2, 22).with_no_reveal(),
            ];
            rsx! {
                BarChart {
                    label: "Roles",
                    bars,
                    orientation: BarOrientation::Horizontal,
                    label_case: LabelCase::AsIs,
                    on_bar: Callback::new(|_: String| {}),
                    selected: "card-draw",
                    reveal: Callback::new(|key: String| rsx! {
                        span { "Tags for {key}" }
                    }),
                }
            }
        }),
    );
}

#[test]
fn deck_charts_with_role_tags() {
    golden(
        "deck_charts",
        &render(|| {
            let role_tags = BTreeMap::from([
                ("ramp", vec!["mana-rock".to_string()]),
                ("draw", Vec::new()),
            ]);
            rsx! {
                DeckCharts {
                    type_bars: vec![("creature", 30, 100), ("instant", 10, 33)],
                    category_bars: vec![("ramp", 10, 100), ("draw", 6, 60)],
                    role_tags,
                }
                DeckCharts {
                    type_bars: vec![("land", 36, 100)],
                    category_bars: vec![("ramp", 10, 100)],
                }
                ColorChart { color_bars: vec![("W", 12, 100), ("U", 0, 0)] }
            }
        }),
    );
}

/// Seven roles: five show and two fold behind See more, in the role grid and
/// in the oracle tag grid under it.
#[test]
fn deck_charts_fold_past_five_rows() {
    golden(
        "deck_charts_folded",
        &render(|| {
            let roles = vec![
                ("ramp", 12, 100),
                ("draw", 10, 83),
                ("remov", 8, 67),
                ("wipe", 4, 33),
                ("tutor", 3, 25),
                ("recur", 2, 17),
                ("stax", 1, 8),
            ];
            let tags = vec![
                ("mana-rock".to_string(), 9, 100),
                ("cantrip".to_string(), 6, 67),
                ("destroy-creature".to_string(), 4, 44),
                ("exile-creature".to_string(), 2, 22),
                ("counterspell".to_string(), 2, 22),
                ("wrath".to_string(), 1, 11),
            ];
            rsx! {
                DeckCharts {
                    type_bars: vec![("creature", 30, 100)],
                    category_bars: roles,
                    oracle_tag_bars: tags,
                    role_tags: BTreeMap::from([("ramp", vec!["mana-rock".to_string()])]),
                }
            }
        }),
    );
}

/// Six buckets: the stepper, five rows, then the sixth behind See more.
#[test]
fn draw_odds_fold_past_five_rows() {
    golden(
        "draw_odds_folded",
        &render(|| {
            rsx! {
                DrawOdds {
                    deck_size: 99,
                    buckets: vec![
                        ("Land", 36),
                        ("ramp", 12),
                        ("draw", 10),
                        ("remov", 8),
                        ("wipe", 4),
                        ("tutor", 3),
                    ],
                }
                DrawOdds { deck_size: 60, buckets: vec![("Land", 24), ("ramp", 4)] }
            }
        }),
    );
}

/// The fixture card's Scryfall data.
fn card() -> ScryfallData {
    let value: serde_json::Value = serde_json::from_str(CARD_JSON).unwrap();
    serde_json::from_value(value["scryfall_data"].clone()).unwrap()
}

/// The fixture card with two faces, each with its own art.
fn double_faced() -> ScryfallData {
    let mut value: serde_json::Value = serde_json::from_str(CARD_JSON).unwrap();
    let data = &mut value["scryfall_data"];
    let art = data["image_uris"].clone();
    data["image_uris"] = serde_json::Value::Null;
    data["card_faces"] = serde_json::json!([
        {
            "object": "card_face",
            "name": "Front Face",
            "mana_cost": "{1}{U}",
            "image_uris": art,
        },
        {
            "object": "card_face",
            "name": "Back Face",
            "mana_cost": "",
            "image_uris": art,
        },
    ]);
    serde_json::from_value(data.clone()).unwrap()
}

/// The fixture card with no art, rules or stats on it.
fn no_image() -> ScryfallData {
    let mut data = card();
    data.image_uris = None;
    data.mana_cost = Some("{2}{G}".to_string());
    data.oracle_text = Some("Draw a card.".to_string());
    data.flavor_text = Some("A quiet page.".to_string());
    data.power = Some("2".to_string());
    data.toughness = Some("3".to_string());
    data
}

#[test]
fn flippable_card_image_single_face() {
    golden(
        "flip_single",
        &render(|| {
            let sd = use_signal(card);
            rsx! {
                FlippableCardImage { sd, size: ImageSize::Large, class: "sd-image-overlay-img".to_string(), draggable: false }
            }
        }),
    );
}

#[test]
fn flippable_card_image_double_faced() {
    golden(
        "flip_double",
        &render(|| {
            let sd = use_signal(double_faced);
            rsx! {
                FlippableCardImage { sd, size: ImageSize::Normal }
                FlippableCardImage { sd, size: ImageSize::Normal, initial_face: 1 }
                FlippableCardImage { sd, size: ImageSize::Normal, flippable: false }
            }
        }),
    );
}

#[test]
fn flippable_card_image_no_art() {
    golden(
        "flip_no_image",
        &render(|| {
            let sd = use_signal(no_image);
            rsx! {
                FlippableCardImage { sd, size: ImageSize::Large }
            }
        }),
    );
}

#[test]
fn flippable_card_image_controlled_face() {
    golden(
        "flip_controlled",
        &render(|| {
            let sd = use_signal(double_faced);
            let face = use_signal(|| 1);
            let past_the_end = use_signal(|| 7);
            rsx! {
                FlippableCardImage { sd, size: ImageSize::Normal, face }
                FlippableCardImage { sd, size: ImageSize::Normal, face: past_the_end }
            }
        }),
    );
}
