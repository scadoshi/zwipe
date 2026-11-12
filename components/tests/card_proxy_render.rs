//! Renders [`CardProxy`] at the sizes it fills, for a look in a browser: a
//! creature and a long-text card at swipe, Featured, stack and thumbnail
//! width, each in Gruvbox dark and light. Ignored in the suite; run with
//! `cargo test -p components --test card_proxy_render -- --ignored`, which
//! writes `target/card-proxy-renders/<theme>.html` over the web app's sheets.

#![expect(
    clippy::unwrap_used,
    reason = "a test helper: a failed write fails the test"
)]

use std::fs;
use std::path::PathBuf;

use components::{CARD_PROXY_FIT_JS, CardProxy, ProxyFace};
use dioxus::prelude::*;

/// A creature with a short text and stats.
fn creature() -> ProxyFace {
    ProxyFace {
        name: "Krenko, Mob Boss".to_owned(),
        mana_cost: Some("{2}{R}{R}".to_owned()),
        type_line: Some("Legendary Creature \u{2014} Goblin Warrior".to_owned()),
        rarity: Some("Rare".to_owned()),
        set_name: Some("Commander Masters".to_owned()),
        oracle_text: Some(
            "{T}: Create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control."
                .to_owned(),
        ),
        stats: Some("3/3".to_owned()),
    }
}

/// A card with a long rules text.
fn long_text() -> ProxyFace {
    ProxyFace {
        name: "Urza, Lord High Artificer".to_owned(),
        mana_cost: Some("{2}{U}{U}".to_owned()),
        type_line: Some("Legendary Creature \u{2014} Human Artificer".to_owned()),
        rarity: Some("Mythic".to_owned()),
        set_name: Some("Modern Horizons".to_owned()),
        oracle_text: Some(
            "When Urza, Lord High Artificer enters, create a 0/0 colorless Construct artifact creature token with \"This creature gets +1/+1 for each artifact you control.\"\nTap an untapped artifact you control: Add {U}.\n{5}: Shuffle your library, then exile the top card. Until end of turn, you may play that card without paying its mana cost."
                .to_owned(),
        ),
        stats: Some("1/4".to_owned()),
    }
}

/// The widths a proxy fills: a swipe card, Featured's preview, a stack card
/// and a list row's thumbnail.
const SIZES: [(&str, u32); 4] = [
    ("Swipe", 320),
    ("Featured", 300),
    ("Stack", 210),
    ("Thumbnail", 44),
];

#[test]
#[ignore = "writes renders for a look in a browser"]
fn render_the_proxy_at_every_size() {
    let sheets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../zwiper-web/assets/sheets");
    let links: String = [
        "01-fonts.css",
        "02-themes.css",
        "03-components.css",
        "05-magic.css",
        "07-screens.css",
    ]
    .iter()
    .map(|name| {
        format!(
            r#"<link rel="stylesheet" href="file://{}">"#,
            sheets.join(name).display()
        )
    })
    .collect();
    let mut cells = String::new();
    for (label, face) in [("Creature", creature()), ("Long text", long_text())] {
        cells.push_str(&format!(
            r#"<h2 class="r-label">{label}</h2><div class="r-row">"#
        ));
        for (size, width) in SIZES {
            let card = dioxus_ssr::render_element(rsx! {
                CardProxy { face: face.clone(), roles: vec!["Tokens".to_owned(), "Aggression".to_owned()] }
            });
            cells.push_str(&format!(
                r#"<figure class="r-cell"><div style="width:{width}px">{card}</div><figcaption>{size} {width}px</figcaption></figure>"#
            ));
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
