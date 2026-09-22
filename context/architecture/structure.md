# Architecture Overview

Full-stack Rust application using hexagonal architecture. One language across all crates: shared types via `zwipe-core`, compile-time safety everywhere.

Every section below describes the code as it stands (last verified against the tree 2026-09-21). The one exception is zort, which is a sketch and says so.

---

## The Family

```
   ┌──────────┐              ┌──────────┐
   │  zwiper  │              │   zite   │
   │ (mobile) │              │  (web)   │
   └────┬─────┘              └────┬─────┘
        │                         │
        └───────────┬─────────────┘
                    │  both depend on both
        ┌───────────┴───────────┐
        ↓                       ↓
┌──────────────┐      ┌──────────────────┐
│ zwipe-client │      │ zwipe-components │
│  (api calls) │      │ (UI + themes.css)│
└───────┬──────┘      └─────────┬────────┘
        └──────────┬────────────┘
                   ↓
          ┌──────────────┐        ┌──────────┐
          │  zwipe-core  │←───────│  zerver  │
          │   (domain)   │        │  (api)   │
          └──────────────┘        └────┬─────┘
                                       │
                                 ┌─────┴──────┐
                                 │  zervice   │
                                 │  (sync)    │
                                 └────────────┘
```

Both clients also depend on `zwipe-core` directly, for the domain types they pass around. `zervice` is a second binary in the `zerver` crate, not a crate of its own.

| Crate | Binary | Role | Depends on |
|-------|--------|------|-----------|
| **zwipe-core** | — (library) | Shared domain types, validation, HTTP contracts | serde, uuid, chrono, thiserror |
| **zerver** | `zerver` | Axum REST API, PostgreSQL, JWT auth | zwipe-core, axum, sqlx, tokio |
| **zerver** | `zervice` | Background sync (Scryfall card data) | zwipe-core (via zerver lib) |
| **zwiper** | `zwiper` | Dioxus cross-platform mobile app | zwipe-core, zwipe-client, zwipe-components, dioxus |
| **zite** | `zite` | Dioxus static website (zwipe.net) | zwipe-core, zwipe-client, zwipe-components, dioxus |
| **zwipe-client** | — (library) | Typed API client: one `call` over the core `Endpoint` descriptions | zwipe-core, reqwest |
| **zwipe-components** | — (library) | Shared Dioxus UI components + `themes.css`/`components.css` | zwipe-core, dioxus |
| **zort** | — (hypothetical) | AI card classification client. Sketched only: no crate, no directory, nothing built | Postgres direct, LLM API |

---

## zwipe-core: Shared Domain

Pure Rust library. No feature flags. No server-only dependencies. The single source of truth for all types shared across the ecosystem.

```
zwipe-core/src/
├── lib.rs
├── test_utils.rs
├── serde_helpers.rs
├── version.rs                      — semver parsing for the min-client-version gate
├── content/
│   └── changelog/                  — the changelog itself, compiled into every surface + served at /api/changelog
├── legal/                          — legal text shared by app and site
│
├── domain/
│   ├── auth/
│   │   └── models/
│   │       ├── session.rs          — Session, claims
│   │       ├── access_token.rs     — AccessToken newtype
│   │       ├── refresh_token.rs    — RefreshToken newtype
│   │       ├── password.rs         — Password validation rules
│   │       └── secret.rs           — Secret (plaintext that redacts in Debug/Display)
│   │
│   ├── card/
│   │   ├── models/
│   │   │   ├── card_profile.rs     — CardProfile (is_token, timestamps)
│   │   │   ├── mod.rs              — Card aggregate (CardProfile + ScryfallData)
│   │   │   ├── scryfall_data/
│   │   │   │   ├── mod.rs          — ScryfallData (~100 fields)
│   │   │   │   ├── colors.rs       — Color, Colors set
│   │   │   │   ├── legalities.rs   — Format legality map
│   │   │   │   ├── rarity.rs       — Rarity enum
│   │   │   │   ├── prices.rs       — USD/EUR/TIX pricing
│   │   │   │   ├── image_uris.rs   — Image URLs at various sizes
│   │   │   │   ├── card_faces.rs   — Double-faced card data
│   │   │   │   └── all_parts.rs    — Related tokens/parts
│   │   │   └── search_card/
│   │   │       ├── card_filter/    — CardQuery + CardQueryBuilder (~50 criteria fields)
│   │   │       │   ├── mod.rs      — Module docs (query vs in-memory split)
│   │   │       │   ├── query.rs    — CardQuery (criteria + Limit + ordering)
│   │   │       │   ├── criteria/   — CardCriteria predicate core + matches()
│   │   │       │   ├── builder/    — Fluent builder with setters/getters
│   │   │       │   ├── error.rs    — InvalidCardCriteria
│   │   │       │   ├── card_sort_key.rs — CardSortKey (name, CMC, rarity, etc.)
│   │   │       │   └── price_currency.rs — PriceCurrency (USD/EUR/TIX)
│   │   │       ├── cards.rs        — Cards collection: matching(), sorted(), sort_deck_entries()
│   │   │       ├── group_cards.rs  — GroupByOption (type/cmc/color), CardGroup
│   │   │       ├── card_type.rs    — CardType enum (7 types)
│   │   │       ├── commander_eligibility.rs — Per-format eligibility + partner validation
│   │   │       └── stop_words.rs   — Filter extraction stop words
│   │   └── mod.rs
│   │
│   ├── deck/
│   │   ├── models/
│   │   │   ├── deck.rs             — Deck aggregate (DeckProfile + entries + warnings)
│   │   │   ├── deck_profile.rs     — DeckProfile (commander, partner, background, sig spell)
│   │   │   ├── deck_card.rs        — DeckCard (quantity, board, mvp_at)
│   │   │   ├── board.rs            — Board enum (Deck, Maybeboard, Sideboard)
│   │   │   ├── deck_metrics.rs     — DeckMetrics (mana curve, type/color dist, prices)
│   │   │   ├── deck_warning.rs     — DeckWarning + WarningAction (FixQuantity, ClearCommander, Remove)
│   │   │   ├── validate_deck.rs    — Pure validation (count, legality, copies, color identity, commander, partner, background, spell)
│   │   │   ├── format.rs           — Format enum (23 formats), rules per format
│   │   │   ├── deck_name.rs        — DeckName newtype
│   │   │   └── quantity.rs         — Quantity newtype (1-99)
│   │   ├── requests/               — Operation request types + validation errors
│   │   │   ├── create_deck_profile.rs
│   │   │   ├── update_deck_profile.rs
│   │   │   ├── create_deck_card.rs
│   │   │   ├── update_deck_card.rs
│   │   │   ├── delete_deck_card.rs
│   │   │   ├── import_deck_cards.rs — Parser for plain-text decklists (// Maybeboard support)
│   │   │   ├── delete_deck.rs
│   │   │   ├── get_deck_profile.rs
│   │   │   ├── get_deck_profiles.rs
│   │   │   └── get_deck_card.rs
│   │   └── mod.rs
│   │
│   ├── user/
│   │   ├── models/
│   │   │   ├── username.rs         — Username newtype
│   │   │   ├── email.rs            — Email handling
│   │   │   ├── hints.rs            — One-time UI hint tracking
│   │   │   ├── theme.rs            — ThemeConfig
│   │   │   └── preferences.rs      — Theme (ALLOWED_THEMES, 31), dark mode, franchises
│   │   └── requests/
│   │       └── get_user.rs
│   │
│   ├── moderation.rs               — Profanity filter
│   ├── site.rs                     — Base URLs + contact points shared by every surface
│   └── logo/                       — ASCII art logos
│
└── http/
    ├── paths.rs                    — API route path constants
    ├── helpers.rs                  — Opdate<T> (partial update semantics)
    └── contracts/
        ├── auth.rs                 — HttpLogin, HttpRegister, etc.
        ├── changelog.rs            — HttpChangelog, HttpRelease
        ├── client.rs               — HttpMinClientVersion (force-update gate)
        ├── deck.rs                 — HttpCreateDeckProfile, HttpUpdateDeckProfile
        ├── deck_card.rs            — HttpCreateDeckCard, HttpPatchDeckCard
        ├── metrics.rs              — HttpCrashReport, ClientErrorReport, usage/signal batches, public metrics
        └── user.rs                 — HttpChangeEmail, HttpChangePassword, etc.
```

**Purity rules:** No sqlx, axum, tokio, anyhow, argon2, jsonwebtoken. No `#[derive(FromRow)]`. No `#[cfg(feature)]`. See `decisions.md` for rationale.

---

## zerver: API Server

Axum REST API with PostgreSQL. Hexagonal architecture: domain is the center, HTTP handlers and database repositories are adapters.

**Binaries:**
- `zerver`: HTTP API server (systemd service in production)
- `zervice`: Run-once nightly job: Scryfall sync, card classification, materialized view refresh, session cleanup

```
zerver/src/
├── bin/
│   ├── zerver.rs               — API server entrypoint
│   └── zervice.rs              — Scryfall sync entrypoint
│
└── lib/
    ├── lib.rs
    ├── config.rs               — Environment config
    │
    ├── domain/                 — Server-specific domain layer
    │   ├── auth/
    │   │   ├── services.rs     — Login, register, refresh, password reset
    │   │   ├── ports.rs        — AuthRepository, AuthService traits
    │   │   ├── models/         — Password (argon2), AccessToken (JWT)
    │   │   ├── requests/       — CreateSession, RefreshSession, VerifyEmail, etc.
    │   │   └── email_templates/
    │   ├── card/
    │   │   ├── services.rs     — Search, sync, card profile operations
    │   │   ├── ports.rs        — CardRepository, CardService traits
    │   │   ├── models/         — SyncMetrics, helpers
    │   │   └── requests/       — GetCard, GetArtists, GetSets, etc.
    │   ├── deck/
    │   │   ├── services.rs     — Deck CRUD, card management, import
    │   │   ├── ports.rs        — DeckRepository, DeckService traits
    │   │   └── models/         — Server-specific deck models
    │   ├── user/               — User services, ports
    │   ├── email/              — Email dispatch models
    │   ├── health/             — Health check service
    │   ├── metrics/            — Usage counters, events, crash/error intake (origin of the erased-service pattern, see decisions.md)
    │   └── upkeep/             — Nightly maintenance, zervice-only (retention sweeps, expired-session cleanup)
    │
    ├── inbound/                — Entry points
    │   ├── http/
    │   │   ├── routes.rs       — All API route definitions
    │   │   ├── mod.rs          — AppState, ApiError, middleware setup
    │   │   ├── cache.rs        — TtlSlot serving-layer caches
    │   │   ├── middleware.rs   — JWT auth extraction, last-active tracking
    │   │   └── handlers/
    │   │       ├── auth/       — Login, register, refresh, verify, reset
    │   │       ├── card/       — Search, get card, filter metadata
    │   │       ├── deck/       — Deck CRUD, get deck with entries
    │   │       ├── deck_card/  — Add/update/delete/import cards
    │   │       ├── user/       — Profile, preferences, delete account
    │   │       ├── metrics/    — Usage/event/crash/error intake, public metrics
    │   │       └── changelog.rs, client.rs, health.rs
    │   └── external/
    │       └── scryfall/       — Scryfall bulk data API client
    │
    └── outbound/               — External system adapters
        ├── sqlx/
        │   ├── postgres.rs     — Connection pool
        │   ├── auth/           — Auth repository (sessions, tokens, users)
        │   ├── card/           — Card repository (search, sync, upsert)
        │   │   └── helpers/    — Batch delta upsert
        │   ├── deck/           — Deck repository (CRUD, card management)
        │   │   ├── models.rs   — DatabaseDeckProfile, DatabaseDeckCard
        │   │   └── helper.rs   — Ownership verification
        │   └── user/           — User repository (preferences, profile)
        ├── archidekt/          — Fetches public Archidekt decks for import by URL
        └── resend/             — Transactional email via Resend API
```

**Database (PostgreSQL, 30 tables + 3 materialized views):** `zerver/migrations/` is the source of truth for the full schema. The ones you touch most:

| Table | Purpose |
|-------|---------|
| `users` | Accounts (email, username, hashed password, lockout) |
| `user_preferences` | Theme, dark mode |
| `scryfall_data` | All card printings (~110k rows, ~100 columns) |
| `card_profiles` | Internal card metadata (is_token, card_roles) |
| `decks` | Deck profiles (name, format, commander_id, partner_commander_id, background_id, signature_spell_id) |
| `deck_cards` | Deck-card join (quantity, board, mvp_at) |
| `otags` / `card_otags` | Oracle tag catalog and card-to-tag join |
| `refresh_tokens` | Rotating refresh tokens (SHA-256 hashed, max 5/user) |
| `email_verification_tokens` | One-time email verification |
| `password_reset_tokens` | One-time password reset |
| `zervice_metrics` | Sync job audit trail |

The rest are signal/analytics tables (user and commander signal, weekly facets, lifetime counters, events) plus client error and crash reporting.

| Materialized view | Purpose |
|-------|---------|
| `latest_cards` | Deduplicated to latest printing per oracle_id (~35k rows). Refreshed by zervice after sync. All search queries read from this view. |
| `card_signal_rollup` | Aggregated per-card signal |
| `otag_context_signal_rollup` | Aggregated per-otag contextual signal |

---

## zwiper: Mobile App

Dioxus cross-platform app. Primary target: iOS. Same hexagonal structure: screens are inbound adapters, the API client is the outbound adapter and lives in `zwipe-client`. UI building blocks and the theme CSS come from `zwipe-components`; the theme list lives in zwipe-core's preferences.

```
zwiper/src/
├── bin/                        — App entrypoint
│
└── lib/
    ├── config.rs               — Compile-time env config (backend URL etc., baked in by build.rs)
    ├── domain/
    │   ├── error.rs            — Client error types
    │   └── language.rs         — i18n support
    │
    ├── inbound/
    │   ├── router.rs           — Screen routing
    │   ├── components/         — Reusable UI
    │   │   ├── interactions/
    │   │   │   └── swipe/      — Swipeable component, SwipeState, SwipeConfig, Direction
    │   │   ├── auth/           — Bouncer (auth guard), session upkeep
    │   │   ├── navigation/     — Back handler, overlay stack
    │   │   ├── telemetry/      — Usage buffer (batched counters/events to the API)
    │   │   ├── accordion/      — Collapsible sections
    │   │   ├── toast/          — Toast notifications
    │   │   ├── alert_dialog/   — Confirmation dialogs
    │   │   ├── fields/         — Reusable form inputs
    │   │   └── (single files)  — bottom sheet, chips, hint dialogs, catalog cache, update-required gate, …
    │   │
    │   └── screens/
    │       ├── home.rs
    │       ├── auth/           — Login, register, forgot password
    │       ├── profile/        — User settings, change email/password/username
    │       │   └── components/ — Preferences UI
    │       └── deck/
    │           ├── list.rs     — Deck list
    │           ├── create.rs   — Create deck (DeckFields with commander/partner/background/spell)
    │           ├── edit.rs     — Edit deck metadata
    │           ├── view.rs     — Deck overview (stats, charts, warnings, buy links)
    │           ├── export.rs   — Export as text (maybeboard toggle)
    │           ├── import.rs   — Import from text (// Maybeboard header)
    │           ├── components/ — DeckFields, DeckStats, DeckCharts, DeckWarnings, MoreButtons
    │           └── card/
    │               ├── add.rs      — Swipe to add (right=add, left=skip, up=maybeboard, down=undo)
    │               ├── view.rs     — Card list (grouping, maybeboard toggle, command zone, qty controls)
    │               ├── remove.rs   — Swipe to remove (tri-state maybeboard filter)
    │               ├── components/ — CardRow, CardInfo, ImagePreview, ActionHistory
    │               └── filter/     — CardFilterSheet with 12+ accordion sections
    │                   ├── card_filter_sheet.rs — Bottom sheet with per-section clear buttons
    │                   ├── name.rs, format.rs, rarity.rs, set.rs, artist.rs, sort.rs, config.rs
    │                   ├── types/      — Card type chips
    │                   ├── mana/       — CMC, color identity, produced mana
    │                   ├── combat/     — Power, toughness
    │                   ├── oracle_text/ — Text contains, keywords, oracle words
    │                   └── flavor_text.rs
    │
    └── outbound/
        ├── session.rs          — JWT + refresh token (keychain storage)
        ├── keyring_entry.rs    — Keychain/keystore access
        ├── theme_store.rs      — Persisted theme choice
        ├── crash_store.rs      — Panic hook writes a crash file; next launch reports it
        ├── android_fs.rs       — Android app-files dir via JNI
        ├── open_url.rs         — Open links in the system browser
        └── buy_links.rs        — TCGplayer, CardKingdom URL builders
```

**Platforms:** iOS (primary), Android, Web (preview), Desktop

---

## zite: Website

Dioxus site deployed to GitHub Pages at [zwipe.net](https://zwipe.net). Marketing pages, the auth flows that need a browser (verify, reset), and a handful of pages that read the public API: changelog, guides, and the shared-deck viewer. Statically hosted, not entirely static content.

No login, no deck building today. `decisions.md` (2026-04-06) commits zite to growing into the full authenticated deck builder eventually; that surface lives only in zwiper for now, but the client layer it needs is already wired up: zite's six API calls all go through `zwipe-client`, so the authed endpoints are a method call away rather than a second implementation.

```
zite/src/
├── api.rs              — The shared zwipe-client, pointed at the backend
├── main.rs                 — Router, nav bar, footer, API base URL
└── pages/
    ├── home.rs             — Landing page with feature grid
    ├── about.rs            — Developer bio, tech stack, architecture
    ├── contribute.rs       — Stripe, Buy Me a Coffee, GitHub Sponsors
    ├── discord.rs          — Community invite
    ├── ios.rs              — App Store download
    ├── android.rs          — Play Store download
    ├── privacy.rs          — Privacy policy
    ├── changelog.rs        — Release notes from /api/changelog
    ├── guides/             — How-to guides
    ├── shared_deck.rs      — Public deck share viewer (reads /api/share/deck/{token})
    ├── not_found.rs        — 404
    ├── verify.rs           — Email verification (token from URL)
    └── reset.rs            — Password reset form (shared validation from zwipe-core)
```

**Deploy:** Push to main → GitHub Actions → `dx build --release --platform web` → GitHub Pages

---

## zwipe-client: API Client

The typed client for the backend, depending on zwipe-core and reqwest and nothing else. No Dioxus, no platform code: crash reporting, session storage and URL config stay in the apps. Both clients use it, and neither builds a request by hand.

`call.rs` is the only transport code. It reads an `Endpoint` from zwipe-core for method, path, auth and body, sends it, and decodes the success body. Every other file is a thin method describing one call:

```rust
/// Fetches a complete deck with all cards.
pub async fn get_deck(&self, deck_id: Uuid, session: &Session) -> Result<Deck, ClientError> {
    self.call(GetDeck(deck_id), Some(session)).await
}
```

```
zwipe-client/src/
├── lib.rs          — ZwipeClient (reqwest client + base URL)
├── call.rs         — The one place a request is built, sent and decoded
├── error.rs        — ClientError: status vocabulary plus user-facing copy
├── auth/           — Login, register, refresh, logout, forgot password
├── card/           — Search, get card, filter metadata
├── changelog/      — Release history
├── deck/           — CRUD, tokens, profiles, sharing, skips
├── deck_card/      — Add, update, delete, import
├── metrics/        — Usage batches, crash reports, anonymous events
├── user/           — Profile, preferences, maybeboard
└── version/        — Minimum supported client version
```

`ZwipeClient::new` takes the base URL, so the crate reads no environment and owns no default. zwiper passes its build-time config value, which is what lets a debug build point at prod for migration testing.

The wasm target needs reqwest without rustls-tls and getrandom on `wasm_js`, the same split zite's manifest carries. No `Send` bounds anywhere: reqwest's wasm futures are `!Send`, and nothing here needs a work-stealing spawner.

---

## zwipe-components: Shared UI

Dioxus component library both clients depend on; the owner's portfolio consumes parts of it too. Ships `themes.css` (31 themes, each with a dark and a light palette) and `components.css`. CSS load order matters: themes first, then components, then app styles.

The source is flat, one file per component: card_details.rs and card_row.rs (shared card rendering), changelog.rs (renders the compiled-in changelog), charts.rs, theme_picker.rs, nav_bar.rs, nav_dropdown.rs, oracle_text.rs, page_meta.rs, and assorted smaller pieces (buttons, chips, banners, panels). The allowed-theme list itself lives in zwipe-core (`ALLOWED_THEMES`); this crate owns the palettes.

---

## zort: AI Classification Client (Hypothetical)

Nothing here exists. No `zort/` directory, no workspace member, no code. The sketch is kept because the shape is still the plan if card-role classification ever moves out of zervice.

Standalone binary for card role classification. Connects directly to PostgreSQL, classifies cards via LLM, writes tags back.

```
zort/                       (future crate)
├── Cargo.toml
└── src/
    └── main.rs             — Subcommands: classify, reclassify, delta, audit
```

**Not embedded in zervice**: keeps deterministic sync separate from non-deterministic AI.

---

## Key Patterns

**Hexagonal architecture:** Domain logic has no external dependencies. Inbound adapters (HTTP handlers, UI screens) and outbound adapters (database repositories, API clients) are swappable.

**Newtypes for type safety:** `Username`, `DeckName`, `Quantity` (zwipe-core) and `Password` (zerver) enforce validation at construction. IDs stay bare `Uuid` on purpose: `DeckProfile { id: Uuid, user_id: Uuid }`. There are no `UserId`/`DeckId` wrappers.

**Database adapter pattern:** Domain types never have SQLx derives. `Database*` wrapper structs with primitive fields convert to domain types via `TryFrom`. See `decisions.md`.

**Session auth:** JWT access tokens (24h) + rotating refresh tokens (14d, SHA-256 hashed, max 5 per user).

**Card filtering:** 30+ filter fields with builder pattern. Backend uses SQLx QueryBuilder with PostgreSQL jsonb/array operators. Frontend has modular filter screens synced via Dioxus signals.

**Deck validation:** Pure function in zwipe-core. Warnings are informational (not blocking). `WarningAction` enum tells the UI what fix to offer per warning type.

**Boards:** `DeckCard.board` is a `Board` enum (`Deck`, `Maybeboard`, `Sideboard`), stored as text on `deck_cards` behind a CHECK constraint. Maybeboard and sideboard cards are excluded from metrics, validation, and card count. Set via update_deck_card. Export/import supports `// Maybeboard` headers.

**Commander system:** Supports all partner variants (Partner, Partner with [Name], Friends Forever, Doctor's Companion), backgrounds (Choose a Background), and Oathbreaker (signature spell). Color identity = union of command zone. Eligibility filtering per format.
