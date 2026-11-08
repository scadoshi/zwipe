//! The shared deck's preview image: its layout against golden PNGs, its
//! route's statuses and cache headers, and the share payload's version and
//! theme.
//!
//! Golden images live in `tests/golden`. After a deliberate layout change,
//! regenerate them with `ZWIPE_UPDATE_GOLDEN=1 cargo test -p zerver --test
//! share_preview` and look at the diff before committing.
//!
//! `samples` (ignored) draws the sample decks with real Scryfall art from a
//! folder and prints render times:
//! `SHARE_PREVIEW_SAMPLES=<dir> cargo test -p zerver --test share_preview -- --ignored samples --nocapture`,
//! where `<dir>` holds `Tymna.jpg`, `Kraum.jpg`, `Chandra.jpg` and `Bolt.jpg`.
//!
//! The route tests need `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "a test: fixtures unwrap, index and invert colors, the sample run prints its timings, and a panic is the failure report"
)]

mod common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use axum::body::Bytes;
use axum::http::{Method, StatusCode, header};
use common::TestApp;
use resvg::tiny_skia;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zwipe::inbound::http::share_preview::SharePreview;
use zwipe::inbound::http::share_preview::layout::{Preview, Tone};
use zwipe::outbound::scryfall_art::ArtSource;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_core::domain::deck::{Board, DeckCard, DeckEntry, Format, PowerLevel, Quantity};
use zwipe_core::domain::user::theme::ThemeConfig;
use zwipe_core::http::contracts::deck::HttpSharedDeck;
use zwipe_core::http::paths::{
    DECK_ROUTE, SHARE_PREVIEW_LAYOUT, share_deck_route, shared_deck_preview_route,
    shared_deck_preview_version, shared_deck_route,
};

/// A real card's JSON, the template every fixture card is made from.
const CARD_TEMPLATE: &str = include_str!("fixtures/share_preview_card.json");

/// A fixture card: `name` with this mana value, type, color identity, USD
/// price and art crop URL.
fn card(
    name: &str,
    cmc: f64,
    type_line: &str,
    colors: &[&str],
    usd: Option<&str>,
    art: Option<(&str, &str)>,
) -> Card {
    let mut value: Value = serde_json::from_str(CARD_TEMPLATE).unwrap();
    let id = name_id(name, 1);
    let oracle = name_id(name, 2);
    value["card_profile"]["scryfall_data_id"] = json!(id);
    let data = &mut value["scryfall_data"];
    data["id"] = json!(id);
    data["oracle_id"] = json!(oracle);
    data["name"] = json!(name);
    data["cmc"] = json!(cmc);
    data["type_line"] = json!(type_line);
    data["color_identity"] = json!(colors);
    data["colors"] = json!(colors);
    data["prices"]["usd"] = json!(usd);
    match art {
        Some((url, artist)) => {
            data["image_uris"]["art_crop"] = json!(url);
            data["artist"] = json!(artist);
        }
        None => data["image_uris"]["art_crop"] = Value::Null,
    }
    serde_json::from_value(value).unwrap()
}

/// A stable id for `name`: FNV-1a over the name, seeded by `salt`, so a
/// fixture card keeps its id between runs.
fn name_id(name: &str, salt: u8) -> Uuid {
    let hash = |seed: u64| {
        name.bytes().chain([salt]).fold(seed, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        })
    };
    let high = hash(0xcbf2_9ce4_8422_2325);
    let low = hash(0x8422_2325_cbf2_9ce4);
    Uuid::from_u128((u128::from(high) << 64) | u128::from(low))
}

/// `card` on the main board, `qty` copies.
fn entry(card: Card, qty: i32) -> DeckEntry {
    DeckEntry {
        deck_card: DeckCard {
            deck_id: Uuid::nil(),
            scryfall_data_id: card.scryfall_data.id,
            oracle_id: card.scryfall_data.oracle_id.unwrap(),
            quantity: Quantity::new(qty).unwrap(),
            board: Board::Deck,
            mvp_at: None,
        },
        card,
    }
}

/// A main board with a spread of mana values and some lands.
fn filler(colors: &[&str]) -> Vec<DeckEntry> {
    let spread = [
        (1.0, 8),
        (2.0, 12),
        (3.0, 10),
        (4.0, 7),
        (5.0, 4),
        (6.0, 2),
        (7.0, 1),
    ];
    let mut entries: Vec<DeckEntry> = spread
        .iter()
        .flat_map(|(cmc, n)| (0..*n).map(move |i| (cmc, i)))
        .map(|(cmc, i)| {
            entry(
                card(
                    &format!("Spell {cmc} {i}"),
                    *cmc,
                    "Creature",
                    colors,
                    Some("0.50"),
                    None,
                ),
                1,
            )
        })
        .collect();
    entries.push(entry(
        card("Basic Land", 0.0, "Basic Land", &[], Some("0.10"), None),
        30,
    ));
    entries
}

/// A shared deck with the fields every sample shares.
fn deck(name: &str, format: Option<Format>, theme: ThemeConfig) -> HttpSharedDeck {
    let mut deck: HttpSharedDeck = serde_json::from_value(json!({
        "name": name, "format": null, "commander": null, "partner_commander": null,
        "background": null, "signature_spell": null, "entries": []
    }))
    .unwrap();
    deck.format = format;
    deck.updated_at = chrono::DateTime::from_timestamp(1_762_560_000, 0);
    deck.theme = Some(theme);
    deck
}

const TYMNA_ART: &str =
    "https://cards.scryfall.io/art_crop/front/b/c/bc7cbe9b-324e-42b8-94e2-36e91cb32163.jpg";
const KRAUM_ART: &str =
    "https://cards.scryfall.io/art_crop/front/5/5/557fcd17-6cb3-414a-b2b1-ea9ae32e5aec.jpg";
const CHANDRA_ART: &str =
    "https://cards.scryfall.io/art_crop/front/4/0/40cb22c8-cb03-45c9-bb0e-b8cabdcc43cd.jpg";
const BOLT_ART: &str =
    "https://cards.scryfall.io/art_crop/front/7/6/7673784e-db4b-43a1-8d55-1bb9fc1e284f.jpg";

fn gruvbox() -> ThemeConfig {
    ThemeConfig::default()
}

fn github_light() -> ThemeConfig {
    ThemeConfig {
        name: "github".to_owned(),
        is_dark: false,
    }
}

/// Commander with partners: Tymna and Kraum, a bracket, three tags.
fn partners(theme: ThemeConfig) -> HttpSharedDeck {
    let mut d = deck("Tymna Kraum Breakfast Club", Some(Format::Commander), theme);
    d.power_level = Some(PowerLevel::Optimized);
    d.tags = vec![
        "aggro".to_owned(),
        "card-draw".to_owned(),
        "tempo".to_owned(),
    ];
    d.commander = Some(card(
        "Tymna the Weaver",
        3.0,
        "Legendary Creature — Human Cleric",
        &["W", "B"],
        Some("1.85"),
        Some((TYMNA_ART, "Winona Nelson")),
    ));
    d.partner_commander = Some(card(
        "Kraum, Ludevic's Opus",
        5.0,
        "Legendary Creature — Zombie Horror",
        &["U", "R"],
        Some("0.95"),
        Some((KRAUM_ART, "Aaron Miller")),
    ));
    d.entries = filler(&["W", "U", "B", "R"]);
    d
}

/// Oathbreaker: Chandra with Lightning Bolt as the signature spell.
fn oathbreaker(theme: ThemeConfig) -> HttpSharedDeck {
    let mut d = deck("Chandra Burns It All", Some(Format::Oathbreaker), theme);
    d.tags = vec!["burn".to_owned()];
    d.commander = Some(card(
        "Chandra, Torch of Defiance",
        4.0,
        "Legendary Planeswalker — Chandra",
        &["R"],
        Some("2.10"),
        Some((CHANDRA_ART, "Kieran Yanner")),
    ));
    d.signature_spell = Some(card(
        "Lightning Bolt",
        1.0,
        "Instant",
        &["R"],
        Some("0.90"),
        Some((BOLT_ART, "Milivoj Ćeran")),
    ));
    d.entries = filler(&["R"])[..30].to_vec();
    d
}

/// A Modern deck: no command zone, so no art and the text full width.
fn modern(theme: ThemeConfig) -> HttpSharedDeck {
    let mut d = deck(
        "Mono Red Burn With An Unreasonably Long Name For Testing",
        Some(Format::Modern),
        theme,
    );
    d.tags = vec![
        "burn".to_owned(),
        "aggro".to_owned(),
        "spellslinger".to_owned(),
        "tokens".to_owned(),
        "stax".to_owned(),
    ];
    d.entries = filler(&["R"])[..20].to_vec();
    d
}

/// A flat two-tone image standing in for card art, so goldens don't depend
/// on Scryfall.
fn synthetic_art(r: u8, g: u8, b: u8) -> Bytes {
    let mut pixmap = tiny_skia::Pixmap::new(626, 457).unwrap();
    pixmap.fill(tiny_skia::Color::from_rgba8(r, g, b, 255));
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(255 - r, 255 - g, 255 - b, 255);
    let rect = tiny_skia::Rect::from_xywh(0.0, 300.0, 626.0, 157.0).unwrap();
    pixmap.fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
    Bytes::from(pixmap.encode_png().unwrap())
}

fn synthetic_source() -> ArtSource {
    ArtSource::by_url(HashMap::from([
        (TYMNA_ART.to_owned(), synthetic_art(40, 80, 160)),
        (KRAUM_ART.to_owned(), synthetic_art(160, 40, 40)),
        (CHANDRA_ART.to_owned(), synthetic_art(200, 90, 20)),
        (BOLT_ART.to_owned(), synthetic_art(30, 30, 30)),
    ]))
}

/// Renders `deck` and checks it against `tests/golden/{name}.png`, allowing
/// a few pixels of drift from font rasterization; rewrites the golden under
/// `ZWIPE_UPDATE_GOLDEN=1`.
async fn assert_golden(name: &str, deck: &HttpSharedDeck) {
    let preview = SharePreview::new(synthetic_source());
    let png = preview.png(Uuid::nil(), deck).await.unwrap();
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "golden",
        &format!("{name}.png"),
    ]
    .iter()
    .collect();
    if std::env::var_os("ZWIPE_UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &png).unwrap();
        return;
    }
    let golden = std::fs::read(&path).unwrap_or_else(|_| {
        panic!(
            "no golden at {}; run with ZWIPE_UPDATE_GOLDEN=1",
            path.display()
        )
    });
    let got = tiny_skia::Pixmap::decode_png(&png).unwrap();
    let want = tiny_skia::Pixmap::decode_png(&golden).unwrap();
    assert_eq!(
        (got.width(), got.height()),
        (want.width(), want.height()),
        "{name}: size"
    );
    let differing = got
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .zip(want.data().as_chunks::<4>().0)
        .filter(|(a, b)| a.iter().zip(b.iter()).any(|(x, y)| x.abs_diff(*y) > 16))
        .count();
    let limit = got.data().len() / 4 / 1000;
    assert!(
        differing <= limit,
        "{name}: {differing} pixels differ from the golden (limit {limit})"
    );
}

/// The layout the goldens show, with the SHA-256 of the three golden files
/// in order. Regenerating them means the drawing changed, so the layout
/// must move with it: bump `SHARE_PREVIEW_LAYOUT` in zwipe-core and
/// `PREVIEW_LAYOUT` in the Pages Function, then record both here.
const GOLDEN_LAYOUT: (u32, &str) = (
    2,
    "24d4decf94a12c66e5a09994edabfce27c583a81cc6291c414500134d39812a0",
);

#[test]
fn a_new_drawing_bumps_the_layout_version() {
    let mut hasher = Sha256::new();
    for name in [
        "commander-partners-gruvbox",
        "oathbreaker-gruvbox",
        "modern-github-light",
    ] {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "golden",
            &format!("{name}.png"),
        ]
        .iter()
        .collect();
        hasher.update(std::fs::read(&path).unwrap());
    }
    let goldens = hex::encode(hasher.finalize());
    assert_eq!(
        (SHARE_PREVIEW_LAYOUT, goldens.as_str()),
        GOLDEN_LAYOUT,
        "the goldens changed: bump SHARE_PREVIEW_LAYOUT (and the Function's PREVIEW_LAYOUT) and record the new layout and hash in GOLDEN_LAYOUT"
    );
}

#[tokio::test]
async fn golden_commander_with_partners() {
    assert_golden("commander-partners-gruvbox", &partners(gruvbox())).await;
}

#[tokio::test]
async fn golden_oathbreaker() {
    assert_golden("oathbreaker-gruvbox", &oathbreaker(gruvbox())).await;
}

#[tokio::test]
async fn golden_no_command_zone_in_a_light_theme() {
    assert_golden("modern-github-light", &modern(github_light())).await;
}

/// `deck`'s tags as the preview draws them: each label with its tone.
fn tags(deck: &HttpSharedDeck) -> Vec<(String, Tone)> {
    Preview::of(deck)
        .tags
        .into_iter()
        .map(|t| (t.label, t.tone))
        .collect()
}

/// The deck tags' labels, all muted.
fn deck_tags(deck: &HttpSharedDeck) -> Vec<(String, Tone)> {
    deck.tags
        .iter()
        .map(|t| (deck_tag_label(t), Tone::Muted))
        .collect()
}

#[test]
fn the_tags_follow_the_deck_list() {
    let d = partners(gruvbox());
    let mut want = vec![
        (format!("{} cards", d.card_count()), Tone::Accent),
        ("Commander".to_owned(), Tone::Palette3),
        ("Optimized (4)".to_owned(), Tone::Palette5),
        (format!("${:.2}", d.price_usd()), Tone::Money),
        ("Tymna the Weaver".to_owned(), Tone::Palette3),
        ("Kraum, Ludevic's Opus".to_owned(), Tone::Palette3),
    ];
    want.extend(deck_tags(&d));
    assert_eq!(tags(&d), want, "partners: both commanders");

    let d = oathbreaker(gruvbox());
    let mut want = vec![
        (format!("{} cards", d.card_count()), Tone::Accent),
        ("Oathbreaker".to_owned(), Tone::Palette3),
        (format!("${:.2}", d.price_usd()), Tone::Money),
        ("Chandra, Torch of Defiance".to_owned(), Tone::Palette3),
        ("Lightning Bolt".to_owned(), Tone::Palette3),
    ];
    want.extend(deck_tags(&d));
    assert_eq!(tags(&d), want, "oathbreaker and signature spell");

    let d = modern(gruvbox());
    let mut want = vec![
        (format!("{} cards", d.card_count()), Tone::Accent),
        ("Modern".to_owned(), Tone::Palette3),
        (format!("${:.2}", d.price_usd()), Tone::Money),
    ];
    want.extend(deck_tags(&d));
    assert_eq!(tags(&d), want, "no command zone");
}

#[test]
fn a_deck_with_no_priced_card_has_no_price_tag() {
    let mut d = modern(gruvbox());
    d.entries.clear();
    assert!(
        tags(&d).iter().all(|(_, tone)| *tone != Tone::Money),
        "{:?}",
        tags(&d)
    );
    assert_eq!(tags(&d)[0], ("0 cards".to_owned(), Tone::Accent));
}

#[tokio::test]
async fn a_deck_whose_art_fails_still_draws() {
    let preview = SharePreview::new(ArtSource::fixed(None));
    let png = preview
        .png(Uuid::nil(), &partners(gruvbox()))
        .await
        .unwrap();
    let decoded = tiny_skia::Pixmap::decode_png(&png).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (1200, 630));
}

/// Builds one sample deck in a theme.
type Make = fn(ThemeConfig) -> HttpSharedDeck;

/// Draws the sample decks with real art and prints each render's time.
#[tokio::test]
#[ignore = "needs SHARE_PREVIEW_SAMPLES, a folder of real art crops"]
async fn samples() {
    let dir = PathBuf::from(std::env::var("SHARE_PREVIEW_SAMPLES").unwrap());
    let art = |file: &str| Bytes::from(std::fs::read(dir.join(file)).unwrap());
    let source = ArtSource::by_url(HashMap::from([
        (TYMNA_ART.to_owned(), art("Tymna.jpg")),
        (KRAUM_ART.to_owned(), art("Kraum.jpg")),
        (CHANDRA_ART.to_owned(), art("Chandra.jpg")),
        (BOLT_ART.to_owned(), art("Bolt.jpg")),
    ]));
    let preview = SharePreview::new(source);
    let decks: [(&str, Make, ThemeConfig); 6] = [
        ("commander-partners-gruvbox-dark", partners, gruvbox()),
        ("commander-partners-github-light", partners, github_light()),
        ("oathbreaker-gruvbox-dark", oathbreaker, gruvbox()),
        ("oathbreaker-github-light", oathbreaker, github_light()),
        ("modern-no-command-zone-gruvbox-dark", modern, gruvbox()),
        (
            "modern-no-command-zone-github-light",
            modern,
            github_light(),
        ),
    ];
    for (i, (name, make, theme)) in decks.into_iter().enumerate() {
        let deck = &make(theme);
        let token = Uuid::from_u128(u128::try_from(i).unwrap());
        let started = Instant::now();
        let png = preview.png(token, deck).await.unwrap();
        let first = started.elapsed();
        let started = Instant::now();
        preview.png(token, deck).await.unwrap();
        let cached = started.elapsed();
        std::fs::write(dir.join(format!("{name}.png")), &png).unwrap();
        println!(
            "{name}: first render {first:?}, cached {cached:?}, {} KB",
            png.len() / 1024
        );
    }
}

/// A verified user's shared deck, with its share token.
async fn shared_deck(app: &TestApp, username: &str) -> String {
    let (token, uid) = app.register(username).await;
    app.verify_email(&uid).await;
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": "Shared Deck", "format": "modern" }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{deck}");
    let deck_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    let (status, share) = app
        .post(&share_deck_route(deck_id), json!({}), Some(&token))
        .await;
    assert!(status.is_success(), "{share}");
    share["share_token"].as_str().unwrap().to_owned()
}

#[sqlx::test]
async fn the_share_payload_carries_its_version_and_the_owners_theme(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let share = shared_deck(&app, "sharer").await;
    let (status, body) = app
        .get(&shared_deck_route(share.parse().unwrap()), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["updated_at"].is_string(), "{body}");
    assert_eq!(
        body["theme"],
        json!({ "name": "gruvbox", "is_dark": true }),
        "the default theme"
    );
    assert!(
        body.get("user_id").is_none() && body.get("username").is_none(),
        "no identity: {body}"
    );
}

#[sqlx::test]
async fn the_preview_route_caches_the_current_version_for_a_year(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let share = shared_deck(&app, "previewer").await;
    let token: Uuid = share.parse().unwrap();
    let (_, body) = app.get(&shared_deck_route(token), None).await;
    let version = chrono::DateTime::parse_from_rfc3339(body["updated_at"].as_str().unwrap())
        .unwrap()
        .timestamp();

    let path = format!(
        "{}?v={}",
        shared_deck_preview_route(token),
        shared_deck_preview_version(version)
    );
    let (status, headers, png) = app.send_bytes(Method::GET, &path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "image/png");
    assert_eq!(
        headers[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    let decoded = tiny_skia::Pixmap::decode_png(&png).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (1200, 630));

    for stale in [
        format!("{}?v=1", shared_deck_preview_route(token)),
        // The deck's version under an older layout.
        format!("{}?v={version}", shared_deck_preview_route(token)),
        shared_deck_preview_route(token),
    ] {
        let (status, headers, _) = app.send_bytes(Method::GET, &stale).await;
        assert_eq!(status, StatusCode::OK, "{stale}");
        assert_eq!(
            headers[header::CACHE_CONTROL],
            "public, max-age=300",
            "{stale}"
        );
    }
}

#[sqlx::test]
async fn an_unknown_or_malformed_token_has_no_preview(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _, _) = app
        .send_bytes(Method::GET, &shared_deck_preview_route(Uuid::new_v4()))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = app
        .send_bytes(Method::GET, "/api/share/deck/not-a-token/preview.png")
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn the_preview_route_is_rate_limited_per_address(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let share = shared_deck(&app, "limited").await;
    let path = shared_deck_preview_route(share.parse().unwrap());
    // The first request draws the image, long enough in a debug build for
    // the bucket to refill, so the burst starts once it is cached.
    let (status, _, _) = app.send_bytes(Method::GET, &path).await;
    assert_eq!(status, StatusCode::OK, "warm-up");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    // A burst of 10 passes; the 11th from the same address is refused.
    for i in 0..10 {
        let (status, _, _) = app.send_bytes(Method::GET, &path).await;
        assert_eq!(status, StatusCode::OK, "request {i}");
    }
    let (status, headers, _) = app.send_bytes(Method::GET, &path).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(headers.contains_key(header::RETRY_AFTER), "{headers:?}");
}
