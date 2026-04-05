# Architecture Overview

Full-stack Rust application using hexagonal architecture. One language across all crates — shared types via `zwipe-core`, compile-time safety everywhere.

---

## The Family

```
┌─────────┐     ┌─────────────┐     ┌─────────┐
│  zwiper  │────→│  zwipe-core  │←────│  zerver  │
│ (mobile) │     │   (domain)   │     │  (api)   │
└─────────┘     └─────────────┘     └────┬─────┘
                       ↑                  │
                ┌──────┘            ┌─────┴─────┐
                │                   │  zervice   │
            ┌───┴──┐               │  (sync)    │
            │ zite  │               └───────────┘
            │ (web) │
            └──────┘               ┌───────────┐
                                   │   zort     │
                                   │ (classify) │
                                   └───────────┘
```

| Crate | Binary | Role | Depends on |
|-------|--------|------|-----------|
| **zwipe-core** | — (library) | Shared domain types, validation, HTTP contracts | serde, uuid, chrono, thiserror |
| **zerver** | `zerver` | Axum REST API, PostgreSQL, JWT auth | zwipe-core, axum, sqlx, tokio |
| **zerver** | `zervice` | Background sync (Scryfall card data) | zwipe-core (via zerver lib) |
| **zwiper** | `zwiper` | Dioxus cross-platform mobile app | zwipe-core, zerver (feature-gated for ApiError only), dioxus |
| **zite** | `zite` | Dioxus static website (zwipe.net) | zwipe-core, dioxus |
| **zort** | `zort` (future) | AI card classification client | Postgres direct, LLM API |

---

## zwipe-core — Shared Domain

Pure Rust library. No feature flags. No server-only dependencies. The single source of truth for all types shared across the ecosystem.

```
zwipe-core/src/
├── lib.rs
├── test_utils.rs
│
├── domain/
│   ├── auth/
│   │   └── models/
│   │       ├── session.rs          — Session, claims
│   │       ├── access_token.rs     — AccessToken newtype
│   │       ├── refresh_token.rs    — RefreshToken newtype
│   │       └── password.rs         — Password validation rules
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
│   │   │       ├── card_filter/    — CardFilter + CardFilterBuilder (30+ fields)
│   │   │       │   ├── mod.rs      — CardFilter struct
│   │   │       │   ├── builder/    — Fluent builder with setters/getters
│   │   │       │   ├── error.rs    — InvalidCardFilter
│   │   │       │   └── order_by_option.rs
│   │   │       ├── filter_cards.rs — In-memory filter + sort
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
│   │   │   ├── deck_card.rs        — DeckCard (quantity, maybeboard)
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
│   │   │   └── preferences.rs      — Theme, dark mode
│   │   └── requests/
│   │       └── get_user.rs
│   │
│   ├── moderation.rs               — Profanity filter
│   └── logo/                       — ASCII art logos
│
└── http/
    ├── paths.rs                    — API route path constants
    ├── helpers.rs                  — Opdate<T> (partial update semantics)
    └── contracts/
        ├── auth.rs                 — HttpLogin, HttpRegister, etc.
        ├── deck.rs                 — HttpCreateDeckProfile, HttpUpdateDeckProfile
        ├── deck_card.rs            — HttpCreateDeckCard, HttpUpdateDeckCard
        └── user.rs                 — HttpChangeEmail, HttpChangePassword, etc.
```

**Purity rules:** No sqlx, axum, tokio, anyhow, argon2, jsonwebtoken. No `#[derive(FromRow)]`. No `#[cfg(feature)]`. See `decisions.md` for rationale.

---

## zerver — API Server

Axum REST API with PostgreSQL. Hexagonal architecture — domain is the center, HTTP handlers and database repositories are adapters.

**Binaries:**
- `zerver` — HTTP API server (systemd service in production)
- `zervice` — Run-once nightly job: Scryfall sync, card classification, materialized view refresh, session cleanup

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
    │   └── health/             — Health check service
    │
    ├── inbound/                — Entry points
    │   ├── http/
    │   │   ├── routes.rs       — All API route definitions
    │   │   ├── mod.rs          — AppState, ApiError, middleware setup
    │   │   ├── middleware/      — JWT auth extraction, rate limiting
    │   │   └── handlers/
    │   │       ├── auth/       — Login, register, refresh, verify, reset
    │   │       ├── card/       — Search, get card, filter metadata
    │   │       ├── deck/       — Deck CRUD, get deck with entries
    │   │       ├── deck_card/  — Add/update/delete/import cards
    │   │       └── user/       — Profile, preferences, delete account
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
        └── resend/             — Transactional email via Resend API
```

**Database (PostgreSQL, 10 tables + 1 materialized view):**

| Table | Purpose |
|-------|---------|
| `users` | Accounts (email, username, hashed password, lockout) |
| `user_preferences` | Theme, dark mode |
| `scryfall_data` | All card printings (~110k rows, ~100 columns) |
| `card_profiles` | Internal card metadata (is_token, mechanical_categories) |
| `latest_cards` | **Materialized view** — deduplicated to latest printing per oracle_id (~35k rows). Refreshed by zervice after sync. All search queries read from this view. |
| `decks` | Deck profiles (name, format, commander_id, partner_commander_id, background_id, signature_spell_id) |
| `deck_cards` | Deck-card join (quantity, board) |
| `refresh_tokens` | Rotating refresh tokens (SHA-256 hashed, max 5/user) |
| `email_verification_tokens` | One-time email verification |
| `password_reset_tokens` | One-time password reset |
| `scryfall_data_sync_metrics` | Sync job audit trail |

---

## zwiper — Mobile App

Dioxus cross-platform app. Primary target: iOS. Same hexagonal structure — screens are inbound adapters, API client is the outbound adapter.

```
zwiper/src/
├── bin/                        — App entrypoint
│
└── lib/
    ├── domain/
    │   ├── error.rs            — Client error types
    │   ├── theme.rs            — 9-theme system
    │   └── language.rs         — i18n support
    │
    ├── inbound/
    │   ├── router.rs           — Screen routing
    │   ├── components/         — Reusable UI
    │   │   ├── interactions/
    │   │   │   └── swipe/      — Swipeable component, SwipeState, SwipeConfig, Direction
    │   │   ├── auth/           — Bouncer (auth guard), session upkeep
    │   │   ├── accordion/      — Collapsible sections
    │   │   ├── toast/          — Toast notifications
    │   │   ├── alert_dialog/   — Confirmation dialogs
    │   │   └── fields/         — Reusable form inputs
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
        ├── buy_links.rs        — TCGplayer, CardKingdom URL builders
        └── client/             — HTTP API client
            ├── auth/           — Login, register, refresh, logout, forgot password
            ├── card/           — Search, get card, filter metadata
            ├── deck/           — CRUD, get tokens, get profile
            ├── deck_card/      — Add, update, delete, import
            └── user/           — Profile, preferences, delete
```

**Platforms:** iOS (primary), Android (near ready), Web (preview), Desktop

---

## zite — Static Website

Dioxus static site deployed to GitHub Pages at [zwipe.net](https://zwipe.net). Handles marketing pages and auth flows that require a web browser.

```
zite/src/
├── main.rs                 — Router, nav bar, footer, API base URL
└── pages/
    ├── home.rs             — Landing page with feature grid
    ├── about.rs            — Developer bio, tech stack, architecture
    ├── contribute.rs       — Stripe, Buy Me a Coffee, GitHub Sponsors
    ├── discord.rs          — Community invite
    ├── ios.rs              — App Store download (pending)
    ├── android.rs          — Play Store download (pending)
    ├── privacy.rs          — Privacy policy
    ├── verify.rs           — Email verification (token from URL)
    └── reset.rs            — Password reset form (shared validation from zwipe-core)
```

**Deploy:** Push to main → GitHub Actions → `dx build --release --platform web` → GitHub Pages

---

## zort — AI Classification Client (Future)

Standalone binary for mechanical category classification. Connects directly to PostgreSQL, classifies cards via LLM, writes tags back.

```
zort/                       (future crate)
├── Cargo.toml
└── src/
    └── main.rs             — Subcommands: classify, reclassify, delta, audit
```

**Not embedded in zervice** — keeps deterministic sync separate from non-deterministic AI. See `plans/mechanical-category.md` for full design.

---

## Key Patterns

**Hexagonal architecture:** Domain logic has no external dependencies. Inbound adapters (HTTP handlers, UI screens) and outbound adapters (database repositories, API clients) are swappable.

**Newtypes for type safety:** `UserId`, `DeckId`, `EmailAddress`, `Password`, `Quantity`, `DeckName` — prevents mixing IDs, enforces validation at construction.

**Database adapter pattern:** Domain types never have SQLx derives. `Database*` wrapper structs with primitive fields convert to domain types via `TryFrom`. See `decisions.md`.

**Session auth:** JWT access tokens (24h) + rotating refresh tokens (14d, SHA-256 hashed, max 5 per user).

**Card filtering:** 30+ filter fields with builder pattern. Backend uses SQLx QueryBuilder with PostgreSQL jsonb/array operators. Frontend has modular filter screens synced via Dioxus signals.

**Deck validation:** Pure function in zwipe-core. Warnings are informational (not blocking). `WarningAction` enum tells the UI what fix to offer per warning type.

**Maybeboard:** Boolean flag on deck_cards. Excluded from metrics, validation, card count. Toggle via update_deck_card. Export/import supports `// Maybeboard` headers.

**Commander system:** Supports all partner variants (Partner, Partner with [Name], Friends Forever, Doctor's Companion), backgrounds (Choose a Background), and Oathbreaker (signature spell). Color identity = union of command zone. Eligibility filtering per format.
