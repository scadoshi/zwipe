# Contributing

## Prerequisites

- Rust (stable)
- PostgreSQL
- [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started) (`cargo install dioxus-cli`)
- iOS: Xcode + `ios-deploy` (`brew install ios-deploy`)

## Setup

```bash
# macOS
./zcripts/denv/mac/setup.sh

# Runs the backend
cargo run --bin zerver

# Runs the frontend (web)
cd zwiper && dx serve
```

## Environment

```
zerver/.env   — DATABASE_URL, JWT_SECRET, BIND_ADDRESS, ALLOWED_ORIGINS, RESEND_API_KEY, RESEND_EMAIL_FROM
zwiper/.env   — BACKEND_URL, RUST_LOG, RUST_BACKTRACE
```

Copy `.env.example` if present, or refer to `context/ops/server.md`.

## Project Structure

The workspace has three packages: `zerver` (backend), `zwiper` (mobile/iOS/Android), and `zweb` (web client).

### zerver

Axum REST API and background sync job. Hexagonal architecture — domain is the center, inbound/outbound are adapters.

```
zerver/src/
├── bin/
│   ├── zerver/         — HTTP server entrypoint
│   └── zervice/        — Scryfall background sync job
└── lib/
    ├── domain/         — Pure business logic, newtypes, service traits
    │   ├── auth/       — Sessions, JWT, password hashing, email verification
    │   ├── card/       — Card profiles, search filters, Scryfall data models
    │   ├── deck/       — Deck and deck_card models, limits
    │   ├── user/       — User profile models
    │   └── email/      — Email dispatch models
    ├── inbound/
    │   ├── http/       — Axum routes and handlers (auth, card, deck, deck_card, user)
    │   └── external/   — Scryfall API client (inbound data source)
    └── outbound/
        ├── sqlx/       — PostgreSQL repositories (auth, card, deck, user)
        └── resend/     — Email delivery via Resend API
```

**Key pattern**: domain types are defined in `zerver` and shared with `zwiper` via a feature flag — `zwiper` depends on `zerver` without the `zerver` feature, which strips out server-only deps (SQLx, Axum).

### zwiper

Dioxus cross-platform app (iOS, Android, web preview). Same hexagonal structure as the backend.

```
zwiper/src/
├── bin/                — App entrypoint
└── lib/
    ├── inbound/
    │   ├── screens/    — Top-level app screens (auth, deck list, card swipe, profile)
    │   │   └── deck/card/filter/  — Card filter screens (color, type, mana, etc.)
    │   └── components/ — Reusable UI (swipe gesture, accordion, toast, alert_dialog, fields)
    └── outbound/
        └── client/     — HTTP client modules for each domain (auth, card, deck, deck_card, user)
```

### zweb

Static Dioxus web client hosted on Cloudflare Pages (`zwipe.net`). Handles email verification, password reset, privacy policy, and App Store redirects.

## Making Changes

### If you change a SQL query

SQLx macros (`query!`, `query_scalar!`, `query_as!`) are verified at compile time. After
modifying any query, regenerate the offline cache from the workspace root:

```bash
cargo sqlx prepare --workspace
```

Commit the updated `.sqlx/` directory alongside your changes. CI builds use this cache
instead of a live database.

### Tests

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Commits

- Concise one-line messages
- Group related files logically
- No emojis, no AI signatures

## Deployment

See `context/ops/server.md` for server setup and `context/ops/ios.md` for iOS builds.
