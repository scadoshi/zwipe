//! Renders [`CardProxy`] at the sizes it fills, for a look in a browser: a
//! creature, a long-text card and a long-named card at swipe, Featured,
//! stack, phone Featured and thumbnail width, each in Gruvbox dark and
//! light. Ignored in the suite; run with
//! `cargo test -p components --test card_proxy_render -- --ignored`, which
//! writes `target/card-proxy-renders/<theme>.html` over the web app's sheets.

#![expect(
    clippy::unwrap_used,
    reason = "a test helper: a failed write fails the test"
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use components::{CARD_PROXY_FIT_JS, CardProxy};
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

/// The fixture card renamed and retexted.
fn card(name: &str, cost: &str, type_line: &str, text: &str, stats: (&str, &str)) -> Card {
    let mut card: Card = serde_json::from_str(CARD_JSON).unwrap();
    let sd = &mut card.scryfall_data;
    name.clone_into(&mut sd.name);
    sd.mana_cost = Some(cost.to_owned());
    sd.type_line = Some(type_line.to_owned());
    sd.oracle_text = Some(text.to_owned());
    sd.power = Some(stats.0.to_owned());
    sd.toughness = Some(stats.1.to_owned());
    sd.card_faces = None;
    card.card_profile.card_roles = vec!["token_maker".to_owned(), "aggression".to_owned()];
    card.card_profile.oracle_tags_by_role = BTreeMap::from([(
        "token_maker".to_owned(),
        vec!["repeatable-token-maker".to_owned()],
    )]);
    card
}

/// A creature with a short text and stats.
fn creature() -> Card {
    let mut krenko = card(
        "Krenko, Mob Boss",
        "{2}{R}{R}",
        "Legendary Creature \u{2014} Goblin Warrior",
        "{T}: Create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control.",
        ("3", "3"),
    );
    "Commander Masters".clone_into(&mut krenko.scryfall_data.set_name);
    krenko
}

/// A card with a long rules text.
fn long_text() -> Card {
    let mut urza = card(
        "Urza, Lord High Artificer",
        "{2}{U}{U}",
        "Legendary Creature \u{2014} Human Artificer",
        "When Urza, Lord High Artificer enters, create a 0/0 colorless Construct artifact creature token with \"This creature gets +1/+1 for each artifact you control.\"\nTap an untapped artifact you control: Add {U}.\n{5}: Shuffle your library, then exile the top card. Until end of turn, you may play that card without paying its mana cost.",
        ("1", "4"),
    );
    "Modern Horizons".clone_into(&mut urza.scryfall_data.set_name);
    urza.scryfall_data.keywords = Some(vec![]);
    urza
}

/// A card whose name holds one long word.
fn long_name() -> Card {
    let mut card = card(
        "Thembercha Overwatcher",
        "{3}{W}",
        "Creature \u{2014} Bird Soldier",
        "Flying",
        ("2", "3"),
    );
    "Commander Masters".clone_into(&mut card.scryfall_data.set_name);
    card
}

/// The widths a proxy fills: a swipe card, Featured's preview, a stack card,
/// the app deck page's Featured card on a phone, and a list row's thumbnail
/// on the web and in the app.
const SIZES: [(&str, u32); 6] = [
    ("Swipe", 320),
    ("Featured", 300),
    ("Stack", 210),
    ("Phone featured", 150),
    ("Thumbnail", 44),
    ("App thumbnail", 64),
];

#[test]
#[ignore = "writes renders for a look in a browser"]
fn render_the_proxy_at_every_size() {
    let sheets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../zwiper-web/assets/sheets");
    let mut links = String::new();
    for name in [
        "01-fonts.css",
        "02-themes.css",
        "03-components.css",
        "05-magic.css",
        "07-screens.css",
    ] {
        write!(
            links,
            r#"<link rel="stylesheet" href="file://{}">"#,
            sheets.join(name).display()
        )
        .unwrap();
    }
    let mut cells = String::new();
    for (label, card) in [
        ("Creature", creature()),
        ("Long text", long_text()),
        ("Long name", long_name()),
    ] {
        write!(
            cells,
            r#"<h2 class="r-label">{label}</h2><div class="r-row">"#
        )
        .unwrap();
        for (size, width) in SIZES {
            let html = dioxus_ssr::render_element(rsx! {
                CardProxy { sd: card.scryfall_data.clone(), profile: Some(card.card_profile.clone()) }
            });
            write!(
                cells,
                r#"<figure class="r-cell"><div style="width:{width}px">{html}</div><figcaption>{size} {width}px</figcaption></figure>"#
            )
            .unwrap();
        }
        cells.push_str("</div>");
    }
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/card-proxy-renders");
    fs::create_dir_all(&out).unwrap();
    for theme in ["gruvbox-dark", "gruvbox-light"] {
        let page = format!(
            r#"<!doctype html><html><head><meta charset="utf-8">{links}<style>
body {{ margin: 0; padding: 24px; background: var(--bg-secondary, var(--bg-primary)); color: var(--text-primary); font-family: 'JetBrains Mono', monospace; }}
.r-row {{ display: flex; gap: 28px; align-items: flex-start; margin-bottom: 18px; }}
.r-cell {{ margin: 0; }}
.r-cell figcaption, .r-label {{ font-size: 13px; color: var(--text-muted); margin: 6px 0; font-weight: 400; }}
</style></head><body class="theme-{theme}">{cells}<script>{CARD_PROXY_FIT_JS}</script></body></html>"#
        );
        fs::write(out.join(format!("{theme}.html")), page).unwrap();
    }
}
