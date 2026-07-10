//! Integration-test harness.
//!
//! Each test gets a fresh, migrated, isolated database via `#[sqlx::test]`
//! (requires `DATABASE_URL` in the env — `set -a; source zerver/.env`). We build
//! the real Axum router with [`build_router`] over that pool + a capturing fake
//! email sender, and drive it in-process with `tower::ServiceExt::oneshot` — no
//! socket, full middleware stack.

#![allow(clippy::unwrap_used, clippy::indexing_slicing, dead_code)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode, header};
use chrono::NaiveDate;
use dashmap::DashMap;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::types::Json;
use sqlx::{PgPool, QueryBuilder};
use tower::ServiceExt; // oneshot
use uuid::Uuid;

use zwipe::domain::auth::models::access_token::JwtSecret;
use zwipe_core::domain::card::scryfall_data::rarity::Rarity;
use zwipe::domain::email::models::{SendEmail, SendEmailError};
use zwipe::domain::email::ports::EmailSender;
use zwipe::domain::{auth, card, deck, health, metrics, user};
use zwipe::inbound::http::{AppState, build_router};
use zwipe::outbound::sqlx::postgres::Postgres;

const TEST_JWT_SECRET: &str = "test-jwt-secret-that-is-at-least-32-characters-long";

/// Captures every outbound email instead of hitting Resend. Later slices read
/// the verify/reset token out of the last captured body.
#[derive(Clone, Default)]
pub struct FakeEmailSender {
    pub sent: Arc<Mutex<Vec<SendEmail>>>,
}

impl EmailSender for FakeEmailSender {
    async fn send_email(&self, email: SendEmail) -> Result<(), SendEmailError> {
        self.sent.lock().unwrap().push(email);
        Ok(())
    }
}

impl FakeEmailSender {
    /// HTML body of the most recently sent email.
    pub fn last_body(&self) -> Option<String> {
        self.sent.lock().unwrap().last().map(|e| e.html_body.clone())
    }

    /// Extracts the raw token from a `/{segment}/{token}` link in the most
    /// recent email body — the verify/reset flows embed the raw token there
    /// (`{web_base_url}/verify/{raw}`, `.../reset/{raw}`), exactly as a user
    /// would receive it. Returns `None` if no such link is present.
    pub fn last_token(&self, segment: &str) -> Option<String> {
        let body = self.last_body()?;
        let marker = format!("/{segment}/");
        let start = body.find(&marker)? + marker.len();
        let rest = &body[start..];
        let end = rest
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '/'))
            .unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

// Distinct fake peer IP per TestApp so the per-IP governor limiter never shares
// state across tests running in parallel.
static IP_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct TestApp {
    pub router: axum::Router,
    pub pool: PgPool,
    pub emails: FakeEmailSender,
    fake_ip: SocketAddr,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        let db = Postgres { pool: pool.clone() };
        let emails = FakeEmailSender::default();
        let jwt_secret = JwtSecret::new(TEST_JWT_SECRET).unwrap();

        // Mirrors zerver.rs service construction, with the fake email sender and
        // fixed test config.
        let auth_service = auth::services::Service::new(
            db.clone(),
            db.clone(),
            emails.clone(),
            jwt_secret.clone(),
            "http://localhost".to_string(),
            "support@test.local".to_string(),
        );
        let user_service = user::services::Service::new(db.clone());
        let health_service = health::services::Service::new(db.clone());
        let card_service = card::services::Service::new(db.clone());
        let deck_service = deck::services::Service::new(db.clone(), db.clone());
        let metrics_service: Arc<dyn metrics::ports::ErasedMetricsService> =
            Arc::new(metrics::services::Service::new(db));

        let state = AppState {
            auth_service: Arc::new(auth_service),
            user_service: Arc::new(user_service),
            health_service: Arc::new(health_service),
            card_service: Arc::new(card_service),
            deck_service: Arc::new(deck_service),
            metrics_service,
            last_active_cache: Arc::new(DashMap::new()),
            min_client_version: Arc::from("0.0.0"),
            web_base_url: Arc::from("http://localhost"),
        };

        let router = build_router(state, jwt_secret, vec![]);

        let n = IP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let o = n.to_be_bytes();
        let fake_ip = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(10, o[1], o[2], o[3].max(1))),
            40000 + (n as u16 & 0x0fff),
        );

        Self { router, pool, emails, fake_ip }
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        json: Option<Value>,
        token: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(t) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let body = match &json {
            Some(v) => {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                Body::from(serde_json::to_vec(v).unwrap())
            }
            None => Body::empty(),
        };
        let mut request = builder.body(body).unwrap();
        // The public routes' governor key-extractor reads the peer IP from
        // ConnectInfo; oneshot requests have none, so insert a fake one.
        request.extensions_mut().insert(ConnectInfo(self.fake_ip));

        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, value)
    }

    pub async fn post(&self, path: &str, json: Value, token: Option<&str>) -> (StatusCode, Value) {
        self.send(Method::POST, path, Some(json), token).await
    }

    pub async fn get(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        self.send(Method::GET, path, None, token).await
    }

    pub async fn put(&self, path: &str, json: Value, token: Option<&str>) -> (StatusCode, Value) {
        self.send(Method::PUT, path, Some(json), token).await
    }

    pub async fn delete(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        self.send(Method::DELETE, path, None, token).await
    }

    /// DELETE carrying a JSON body (e.g. `delete-user`, which re-auths on a
    /// password in the request body).
    pub async fn delete_json(
        &self,
        path: &str,
        json: Value,
        token: Option<&str>,
    ) -> (StatusCode, Value) {
        self.send(Method::DELETE, path, Some(json), token).await
    }

    /// Mark a user's email verified via a direct write. Unverified accounts hit
    /// the deck/card caps; tests that need the higher limits call this.
    pub async fn verify_email(&self, user_id: &str) {
        let uid: uuid::Uuid = user_id.parse().unwrap();
        sqlx::query("UPDATE users SET email_verified_at = now() WHERE id = $1")
            .bind(uid)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    /// Register a fresh user; returns `(access_token, user_id)`.
    pub async fn register(&self, username: &str) -> (String, String) {
        let body = serde_json::json!({
            "username": username,
            "email": format!("{username}@test.local"),
            "password": "TestPass123!",
        });
        let (status, v) = self.post("/api/auth/register", body, None).await;
        assert_eq!(status, StatusCode::CREATED, "register failed: {v}");
        (
            v["access_token"]["value"].as_str().unwrap().to_string(),
            v["user"]["id"].as_str().unwrap().to_string(),
        )
    }
}

// =====================================================================
//  Card fixtures
// =====================================================================
//
// `cards` (the `scryfall_data` table) is a ~90-column wire mirror of the
// Scryfall API with ~35 NOT NULL columns and no domain `Default`. Building a
// full `ScryfallData` per test would drown the intent, so `card(name)` is a
// small builder over only the fields search / serve / ordering assert on;
// everything else gets a realistic constant. `seed_cards` writes the row plus
// its required `card_profiles` mate (the `latest_cards` view JOINs it) and
// refreshes both materialized views the migrations create empty.
//
// The raw INSERT is an unchecked `sqlx` query on purpose — test-only columns
// stay out of the committed `.sqlx` offline data. The defaults are chosen so
// every row round-trips back through `DatabaseScryfallData::try_from`: colors
// are WUBRG short names, rarity is a valid variant, and the JSONB columns
// deserialize into `Legalities` / `Prices` (both all-`Option`, so `{}` works).

// Distinct, stable ids in allocation order — no `Uuid::new_v4()` (feature) and
// no `Math.random`-style nondeterminism, so ordering assertions are repeatable.
static CARD_SEQ: AtomicU64 = AtomicU64::new(1);

const ID_NS: u128 = 0x1000_0000_0000_0000_0000_0000_0000_0000;
const ORACLE_NS: u128 = 0x2000_0000_0000_0000_0000_0000_0000_0000;

/// A card row to seed. Build with [`card`] and the chainable setters; only the
/// fields tests care about are exposed, the rest are sane constants.
#[derive(Clone)]
pub struct CardFixture {
    id: Uuid,
    oracle_id: Option<Uuid>,
    name: String,
    layout: String,
    cmc: Option<f64>,
    colors: Vec<String>,
    color_identity: Vec<String>,
    keywords: Option<Vec<String>>,
    mana_cost: Option<String>,
    oracle_text: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    produced_mana: Option<Vec<String>>,
    type_line: Option<String>,
    rarity: String,
    edhrec_rank: Option<i32>,
    usd: Option<String>,
    set: String,
    set_name: String,
    set_id: Uuid,
    collector_number: String,
    legalities: Value,
    flavor_text: Option<String>,
    artist: Option<String>,
    lang: String,
    digital: bool,
    oversized: bool,
    promo: bool,
    content_warning: Option<bool>,
    mechanical_categories: Vec<String>,
}

/// Start a card fixture. Distinct `id` and `oracle_id` are assigned up front so
/// tests can reference them (`.id()`, `.oracle_id()`) for `/api/card/{id}`
/// lookups and ordering checks before seeding.
pub fn card(name: &str) -> CardFixture {
    let n = u128::from(CARD_SEQ.fetch_add(1, Ordering::Relaxed));
    CardFixture {
        id: Uuid::from_u128(ID_NS | n),
        oracle_id: Some(Uuid::from_u128(ORACLE_NS | n)),
        name: name.to_string(),
        layout: "normal".to_string(),
        cmc: Some(0.0),
        colors: Vec::new(),
        color_identity: Vec::new(),
        keywords: None,
        mana_cost: None,
        oracle_text: None,
        power: None,
        toughness: None,
        produced_mana: None,
        type_line: Some("Creature".to_string()),
        rarity: "common".to_string(),
        edhrec_rank: None,
        usd: None,
        set: "TST".to_string(),
        set_name: "Test Set".to_string(),
        set_id: Uuid::from_u128(0x5E7),
        collector_number: n.to_string(),
        legalities: json!({}),
        flavor_text: None,
        artist: None,
        lang: "en".to_string(),
        digital: false,
        oversized: false,
        promo: false,
        content_warning: None,
        mechanical_categories: Vec::new(),
    }
}

/// Splits a compact color string like `"WU"` into Scryfall short names
/// (`["W", "U"]`), ignoring anything that isn't a WUBRG letter. Pass `""` for
/// colorless.
fn colors_of(s: &str) -> Vec<String> {
    s.chars()
        .filter_map(|c| match c.to_ascii_uppercase() {
            l @ ('W' | 'U' | 'B' | 'R' | 'G') => Some(l.to_string()),
            _ => None,
        })
        .collect()
}

impl CardFixture {
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn oracle_id(&self) -> Option<Uuid> {
        self.oracle_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set both `id` and `oracle_id` — use when a test needs two printings of
    /// the same card (same `oracle_id`, different `id`).
    pub fn with_ids(mut self, id: Uuid, oracle_id: Option<Uuid>) -> Self {
        self.id = id;
        self.oracle_id = oracle_id;
        self
    }
    pub fn oracle(mut self, oracle_id: Option<Uuid>) -> Self {
        self.oracle_id = oracle_id;
        self
    }
    pub fn layout(mut self, layout: &str) -> Self {
        self.layout = layout.to_string();
        self
    }
    pub fn cmc(mut self, cmc: f64) -> Self {
        self.cmc = Some(cmc);
        self
    }
    /// Card colors, compact form (`"R"`, `"WU"`, `""` for colorless).
    pub fn colors(mut self, colors: &str) -> Self {
        self.colors = colors_of(colors);
        self
    }
    /// Color identity, compact form. Defaults to whatever `colors` was set to
    /// is NOT assumed — set both explicitly when they differ.
    pub fn color_identity(mut self, ci: &str) -> Self {
        self.color_identity = colors_of(ci);
        self
    }
    /// Shorthand: set `colors` and `color_identity` to the same value.
    pub fn mono(self, colors: &str) -> Self {
        let v = colors_of(colors);
        let mut s = self.colors(colors);
        s.color_identity = v;
        s
    }
    pub fn keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = Some(keywords.iter().map(|k| k.to_string()).collect());
        self
    }
    pub fn mana_cost(mut self, mana_cost: &str) -> Self {
        self.mana_cost = Some(mana_cost.to_string());
        self
    }
    pub fn oracle_text(mut self, text: &str) -> Self {
        self.oracle_text = Some(text.to_string());
        self
    }
    pub fn power(mut self, power: &str) -> Self {
        self.power = Some(power.to_string());
        self
    }
    pub fn toughness(mut self, toughness: &str) -> Self {
        self.toughness = Some(toughness.to_string());
        self
    }
    pub fn produced_mana(mut self, colors: &str) -> Self {
        self.produced_mana = Some(colors_of(colors));
        self
    }
    pub fn type_line(mut self, type_line: &str) -> Self {
        self.type_line = Some(type_line.to_string());
        self
    }
    pub fn rarity(mut self, rarity: &str) -> Self {
        self.rarity = rarity.to_string();
        self
    }
    pub fn edhrec_rank(mut self, rank: i32) -> Self {
        self.edhrec_rank = Some(rank);
        self
    }
    pub fn usd(mut self, usd: &str) -> Self {
        self.usd = Some(usd.to_string());
        self
    }
    pub fn set(mut self, code: &str, name: &str) -> Self {
        self.set = code.to_string();
        self.set_name = name.to_string();
        self
    }
    /// Mark this card legal in a format (e.g. `"commander"`). Repeatable.
    pub fn legal(mut self, format: &str) -> Self {
        self.legalities[format] = json!("legal");
        self
    }
    pub fn commander_legal(self) -> Self {
        self.legal("commander")
    }
    pub fn flavor_text(mut self, text: &str) -> Self {
        self.flavor_text = Some(text.to_string());
        self
    }
    pub fn artist(mut self, artist: &str) -> Self {
        self.artist = Some(artist.to_string());
        self
    }
    pub fn lang(mut self, lang: &str) -> Self {
        self.lang = lang.to_string();
        self
    }
    pub fn digital(mut self, digital: bool) -> Self {
        self.digital = digital;
        self
    }
    pub fn oversized(mut self, oversized: bool) -> Self {
        self.oversized = oversized;
        self
    }
    pub fn promo(mut self, promo: bool) -> Self {
        self.promo = promo;
        self
    }
    pub fn content_warning(mut self, warning: bool) -> Self {
        self.content_warning = Some(warning);
        self
    }
    /// A token card (`layout = "token"`); the seeded `card_profiles.is_token`
    /// tracks the layout, so this also flips the `is_token` flag.
    pub fn token(mut self) -> Self {
        self.layout = "token".to_string();
        self
    }
    /// Mechanical categories (snake_case, e.g. `"ramp"`, `"graveyard_hate"`) —
    /// written to `card_profiles.mechanical_categories`.
    pub fn categories(mut self, cats: &[&str]) -> Self {
        self.mechanical_categories = cats.iter().map(|c| c.to_string()).collect();
        self
    }
}

/// Seed `cards` with the given fixtures, write their `card_profiles` rows, and
/// refresh the `latest_cards` / `card_signal_rollup` materialized views (both
/// start empty — the views must be refreshed before any card query, even with
/// zero rows, or Postgres errors that they are unpopulated).
pub async fn seed_cards(pool: &PgPool, cards: &[CardFixture]) {
    if !cards.is_empty() {
        let released = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let mut qb = QueryBuilder::new(
            "INSERT INTO scryfall_data (\
             id, lang, object, layout, oracle_id, \
             prints_search_uri, rulings_uri, scryfall_uri, uri, \
             cmc, color_identity, colors, keywords, legalities, mana_cost, name, \
             oracle_text, power, produced_mana, reserved, toughness, type_line, edhrec_rank, \
             artist, flavor_text, content_warning, \
             border_color, booster, collector_number, digital, finishes, frame, full_art, \
             highres_image, image_status, oversized, prices, promo, rarity, related_uris, \
             released_at, reprint, scryfall_set_uri, set_name, set_search_uri, set_type, \
             set_uri, set, set_id, story_spotlight, textless, variation) ",
        );
        qb.push_values(cards.iter(), |mut b, c| {
            let prices = json!({ "usd": c.usd });
            // Production stores rarity as the short code ("R"/"C"/…) — the SQL
            // rarity filters compare against `to_short_name()`. Storing the long
            // word here would read back fine (try_from accepts both) but silently
            // break those filters, so mirror the real write form.
            let rarity = Rarity::try_from(c.rarity.as_str())
                .map(|r| r.to_short_name())
                .unwrap_or_else(|_| c.rarity.clone());
            b.push_bind(c.id)
                .push_bind(c.lang.as_str())
                .push_bind("card")
                .push_bind(c.layout.as_str())
                .push_bind(c.oracle_id)
                .push_bind("https://scryfall.test/prints")
                .push_bind("https://scryfall.test/rulings")
                .push_bind("https://scryfall.test/card")
                .push_bind("https://scryfall.test/uri")
                .push_bind(c.cmc)
                .push_bind(c.color_identity.as_slice())
                .push_bind(c.colors.as_slice())
                .push_bind(c.keywords.as_deref())
                .push_bind(Json(&c.legalities))
                .push_bind(c.mana_cost.as_deref())
                .push_bind(c.name.as_str())
                .push_bind(c.oracle_text.as_deref())
                .push_bind(c.power.as_deref())
                .push_bind(c.produced_mana.as_deref())
                .push_bind(false) // reserved
                .push_bind(c.toughness.as_deref())
                .push_bind(c.type_line.as_deref())
                .push_bind(c.edhrec_rank)
                .push_bind(c.artist.as_deref())
                .push_bind(c.flavor_text.as_deref())
                .push_bind(c.content_warning)
                .push_bind("black") // border_color
                .push_bind(true) // booster
                .push_bind(c.collector_number.as_str())
                .push_bind(c.digital)
                .push_bind(vec!["nonfoil".to_string()]) // finishes
                .push_bind("2015") // frame
                .push_bind(false) // full_art
                .push_bind(true) // highres_image
                .push_bind("highres_scan") // image_status
                .push_bind(c.oversized)
                .push_bind(Json(prices))
                .push_bind(c.promo)
                .push_bind(rarity)
                .push_bind(Json(json!({}))) // related_uris
                .push_bind(released)
                .push_bind(false) // reprint
                .push_bind("https://scryfall.test/set")
                .push_bind(c.set_name.as_str())
                .push_bind("https://scryfall.test/set-search")
                .push_bind("expansion")
                .push_bind("https://scryfall.test/set-uri")
                .push_bind(c.set.as_str())
                .push_bind(c.set_id)
                .push_bind(false) // story_spotlight
                .push_bind(false) // textless
                .push_bind(false); // variation
        });
        qb.build().execute(pool).await.unwrap();

        let mut pb = QueryBuilder::new(
            "INSERT INTO card_profiles (scryfall_data_id, is_token, mechanical_categories) ",
        );
        pb.push_values(cards.iter(), |mut b, c| {
            b.push_bind(c.id)
                .push_bind(c.layout == "token")
                .push_bind(Json(c.mechanical_categories.clone()));
        });
        pb.build().execute(pool).await.unwrap();
    }

    refresh_card_views(pool).await;
}

/// Refresh the card materialized views. Call after mutating
/// `commander_card_signal` in a signal test (`seed_cards` already refreshes).
pub async fn refresh_card_views(pool: &PgPool) {
    sqlx::query("REFRESH MATERIALIZED VIEW latest_cards")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("REFRESH MATERIALIZED VIEW card_signal_rollup")
        .execute(pool)
        .await
        .unwrap();
}
