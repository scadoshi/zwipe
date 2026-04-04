# Architecture Decisions

Key technical decisions made during development. Context for why things are built the way they are.

---

## Frontend Framework: Dioxus (Rust)

**Decided: 2026-03-26 — shipped to real iPhone same day.**

Single language (Rust) across backend and frontend. Same types, same error handling, same mental model. For a solo developer this eliminates context switching — shared domain types between zerver and zwiper, compile-time safety on both sides.

- **Framework**: Dioxus 0.7.3 (`dx` CLI)
- **Target**: iOS physical device (`aarch64-apple-ios`)
- **Session storage**: `keyring` crate → iOS Keychain via `keychain-access-groups` entitlement

### Critical build flag

Dioxus 0.7 does NOT generate an Xcode project — it produces a `.app` bundle directly. Without `--device`, `dx` targets the simulator and crashes on real hardware:

```bash
dx build --platform ios --device "scotland-mobile"
```

Full iOS signing + deploy reference: `ops/ios/`.

---

## Shared Models: zwipe-core as shared domain crate

**Decided: early development. Revised: 2026-04-02.**

Both zerver and zwiper need the same domain types (User, Deck, Card, request/response structs). Originally zwiper imported directly from zerver with `default-features = false`. This worked but created a backwards dependency (client → server) pulling ~17 unnecessary transitive deps.

**Current approach**: `zwipe-core` is a pure shared domain crate. Both zerver and zwiper depend on it:

```
zwiper ──→ zwipe-core ←── zerver
zite   ──→ zwipe-core
```

**Rules for zwipe-core:**
- No feature flags, no conditional compilation
- No server-only dependencies (sqlx, anyhow, argon2, axum, tokio)
- Only types genuinely shared between frontend and backend
- All domain validation and tests live in core; zerver re-exports via `pub use`
- Service-layer errors (wrapping `anyhow::Error`) stay in zerver — the frontend never sees them

**What lives in zwipe-core:** domain entities (User, DeckProfile, DeckCard), value objects (Username, DeckName, Quantity, Format, DeckWarning), request types (CreateDeckProfile, UpdateDeckCard, etc.), validation errors, content moderation, password validation.

**What stays in zerver:** service-layer errors, port traits, service implementations, database adapters, HTTP handlers/routes, `ApiError`, any type that requires server-only dependencies.

Zerver files for extracted types become one-liners: `pub use zwipe_core::domain::deck::format::*;`

---

## ApiError Must Stay in Zerver

**Decided: 2026-04-02.**

`ApiError` is the HTTP error enum that maps domain errors to status codes. It lives in `zerver/src/lib/inbound/http/mod.rs`. It was considered for extraction to zwipe-core but **cannot move** due to Rust's orphan rule.

**Why:** Zerver has ~10 `impl From<DomainError> for ApiError` conversions across its handler files (e.g., `impl From<InvalidCreateDeckProfile> for ApiError`). If `ApiError` moves to zwipe-core, both the error type AND the domain error type become foreign to zerver — Rust's orphan rule forbids implementing a foreign trait (`From`) for two foreign types. Every handler-level error mapping would break.

**Consequence:** Zwiper must keep zerver as a dependency (with `default-features = false`) to access `ApiError`. This is acceptable — `ApiError` is an inbound HTTP adapter type, not domain logic. Its `From` impls are handler-level glue that maps domain errors to HTTP status codes, which is exactly where adapter logic belongs.

**All server-only code in zerver must be gated with `#[cfg(feature = "zerver")]`** so zwiper's build doesn't pull in axum, sqlx, jsonwebtoken, etc.

---

## Database Adapter Pattern: No Custom SQLx Impls on Domain Types

**Decided: 2026-04-02.**

Domain types must NOT have custom `impl Type<Postgres>`, `impl Encode`, or `impl Decode` — even if the impl code lives in the adapter layer (`outbound/sqlx/`). This is both an architectural choice and a Rust compiler requirement.

### Why

**Rust's orphan rule** prevents implementing a foreign trait (like `sqlx::Type`) on a type from another crate. If `Format` lives in `zwipe-core`, zerver cannot `impl Type<Postgres> for Format` — neither the trait nor the type is local to zerver.

But even without the orphan rule, custom SQLx impls on domain types are the wrong pattern. Domain types shouldn't know how they're persisted. The database is an adapter — it should handle its own serialization.

### Pattern

Use intermediate `Database*` structs with primitive fields that SQLx understands natively, then convert at the boundary:

```rust
// outbound/sqlx/deck/models.rs — the adapter layer
#[derive(FromRow)]
struct DatabaseDeckProfile {
    pub format: Option<String>,   // ← primitive, SQLx handles natively
    pub name: String,
    // ...
}

impl TryFrom<DatabaseDeckProfile> for DeckProfile {
    fn try_from(db: DatabaseDeckProfile) -> Result<Self, _> {
        let format = db.format.map(Format::try_from).transpose()?;
        let name = DeckName::new(db.name)?;
        // ...
    }
}
```

For **enums** (Format, Rarity): store as `String` (TEXT column), convert with `TryFrom<&str>` / `to_legality_key()`.

For **JSONB types** (Colors, Legalities, Prices, CardFaces): use `sqlx::types::Json<T>` wrapper in queries, which works automatically with any `Serialize + Deserialize` type. No custom impls needed.

### Result

- Domain types are portable across crates (no orphan rule conflicts)
- Database serialization is explicit and visible in the adapter layer
- Correct hexagonal architecture — the domain doesn't depend on infrastructure
- Existing `Database*` wrapper types (DatabaseUser, DatabaseDeckProfile, etc.) already followed this pattern; custom SQLx impls were redundant

---

## Hosting: Ubuntu Server via Cloudflare Tunnel

See `architecture/hosting.md`.
