# Integration tests — harness

## 1. Dependencies + layout

`zerver/Cargo.toml` gains dev-dependencies only:

```toml
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }   # ServiceExt::oneshot
http-body-util = "0.1"                             # collect response bodies
```

sqlx already carries default features, which include `migrate` — `#[sqlx::test]`
works as-is and resolves migrations from `zerver/migrations/` (the crate-root
`./migrations` default). Tests live in a new `zerver/tests/` integration-test
target:

```
zerver/tests/
├── common/mod.rs        # TestApp, fixtures, helpers (not a test file)
├── auth_flows.rs
├── deck_flows.rs
├── card_serving.rs
├── repo_deck.rs         # repo-level: clone, suppressions, import
├── repo_card.rs         # repo-level: band shuffle, signal rollup
└── metrics_flows.rs
```

`#[sqlx::test]` hands each test a `PgPool` pointing at a fresh,
migrated, isolated database (created via the `DATABASE_URL` server, dropped
on success). Tests run in parallel safely. Requires `DATABASE_URL` in the
env — same value dev already exports; document `set -a; source zerver/.env`
in the test module header.

## 2. Router extraction (the one production-code change)

`HttpServer::new` (`inbound/http/mod.rs:263`) builds the router and binds
the listener in one function — tests need the router without the socket.
Split it:

```rust
// new public fn — everything from mod.rs:274 (trace_layer) through :349
// (.with_state(state)) moves here verbatim
pub fn build_router(state: AppState, jwt_secret: JwtSecret,
                    allowed_origins: AllowedOrigins) -> axum::Router { ... }

// HttpServer::new becomes: erase services → AppState → build_router → bind
```

Behavior-preserving; `zerver.rs` unchanged. Tests call `build_router`
directly and drive it with `tower::ServiceExt::oneshot` — no port, no
tokio spawn, full middleware stack included.

## 3. `FakeEmailSender`

`domain/email/ports.rs:10` — implement the port in `tests/common`:

```rust
#[derive(Clone, Default)]
struct FakeEmailSender { sent: Arc<Mutex<Vec<SendEmail>>> }
// impl EmailSender: push + Ok(()); helper fn extracts the token
// (verify/reset URL) from the most recent captured body with a regex
```

This is what makes the verify-email and password-reset HTTP flows testable
end-to-end: the test reads the token out of the captured email exactly as a
user would from their inbox.

## 4. `TestApp`

The core helper in `tests/common/mod.rs`, mirroring `zerver.rs:57-88`
service construction but from a test pool and hardcoded test config:

```rust
pub struct TestApp {
    pub router: axum::Router,
    pub pool: PgPool,
    pub emails: FakeEmailSender,
}

impl TestApp {
    pub async fn new(pool: PgPool) -> Self {
        let db = Postgres { pool: pool.clone() };
        let emails = FakeEmailSender::default();
        // auth/user/health/card/deck/metrics services exactly as zerver.rs,
        // with `emails.clone()` where Resend goes and a fixed test
        // JwtSecret / web_base_url / support address
        // router = build_router(state, jwt_secret, permissive_origins)
    }

    // request helpers
    pub async fn post(&self, path, json, token: Option<&str>) -> (StatusCode, Value);
    pub async fn get(&self, path, token: Option<&str>) -> (StatusCode, Value);

    // flow helpers
    pub async fn register_and_login(&self, username) -> Session;  // returns bearer token + user_id
    pub async fn verify_email(&self, user_id);  // direct UPDATE users SET email_verified_at
                                                // (unverified accounts hit the deck cap)
}
```

**Governor caveat:** public routes rate-limit by peer IP
(`routes.rs:102-193`); `oneshot` requests carry no `ConnectInfo`, and
tower_governor's key extractor errors without it. The request helpers must
insert `ConnectInfo(SocketAddr)` into request extensions on every request.
Give each `TestApp` a distinct fake IP (atomic counter → `10.x.y.z`) so
per-test routers never share limiter state, and tests that intentionally
exercise many requests (lockout, rate-limit tests) vary the port/IP or
assert the 429 as the expected outcome.

## 5. Card fixtures

Card-serving tests need rows in `cards` and the `latest_cards` /
`card_signal_rollup` materialized views refreshed (migrations create the
views empty). Provide:

```rust
// tests/common: a builder with sane defaults over the wide cards schema
pub fn card(name: &str) -> CardFixture;   // .colors("R").cmc(3).type_line("Creature — Goblin")…
pub async fn seed_cards(pool, Vec<CardFixture>);  // INSERT + REFRESH latest_cards
                                                  // + REFRESH card_signal_rollup
```

Keep the fixture set small and purposeful (~15 cards: two colors, a
commander-legal legendary, lands, varied cmc/types/prices, one multi-face) —
enough to make search, filters, color-identity gating, and ordering
assertions meaningful without a Scryfall dump.

## 6. Conventions

- Tests assert on **status + wire JSON** (the `Http*` contracts), not on
  internals — they should survive refactors that keep behavior.
- Repo-level tests construct `Postgres { pool }` and call repository
  methods directly; no router.
- No sleeps; anything time-dependent (vesting, lockout windows) gets its
  timestamps written directly via the pool.
- Workspace lint rules apply, but `tests/` follows the existing test-module
  exception pattern for unwrap/indexing (`#![allow(clippy::unwrap_used, clippy::indexing_slicing)]`
  at file top, matching the unit-test convention).
