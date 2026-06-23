# Contributing

## Prerequisites

- Rust (stable)
- PostgreSQL
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started) (`cargo install dioxus-cli`) — project is on Dioxus 0.7
- iOS: Xcode + `ios-deploy` (`brew install ios-deploy`)
- Android: Android Studio (SDK + NDK) — see `context/operations/android/setup.md` (note the JDK 21 `JAVA_HOME` gotcha)

## Setup

```bash
# macOS
./zcripts/dev-env/macos/setup.sh

# Runs the backend
cargo run --bin zerver

# Runs the frontend (web)
cd zwiper && dx serve
```

## Environment

```
zerver/.env   — DATABASE_URL, JWT_SECRET, BIND_ADDRESS, ALLOWED_ORIGINS, RESEND_API_KEY, RESEND_EMAIL_FROM, LOG_DIR
zwiper/.env   — BACKEND_URL, RUST_LOG, RUST_BACKTRACE
```

Copy `.env.example` if present, or refer to `context/operations/infrastructure/server.md`.

## Project Structure

The workspace has four crates: `zwipe-core` (shared domain), `zerver` (backend), `zwiper` (mobile/iOS/Android), and `zite` (web client).

### zwipe-core

The shared domain crate — pure types, validation, and business rules used by every
other crate. It must stay **pure**: no feature flags, no server-only dependencies
(SQLx, Axum, tokio, argon2, …), no database derives/annotations. If only the server
needs a type, it stays in `zerver`. Everyone else (`zerver`, `zwiper`, `zite`)
depends on `zwipe-core` for domain types.

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

**Key pattern**: shared domain types live in `zwipe-core`, not `zerver`. `zerver` re-exports them (`pub use zwipe_core::…`) and layers server-only concerns on top (ports, services, SQLx adapters, HTTP handlers). `zwiper` and `zite` depend on `zwipe-core` directly for domain types, and on `zerver` only for HTTP-contract types (routes, `ApiError`, `Http*` structs).

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

### zite

Static Dioxus web client hosted on GitHub Pages (`zwipe.net`). Handles email verification, password reset, privacy policy, and App Store redirects.

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

See `context/operations/infrastructure/server.md` for server setup and `context/operations/ios/` for iOS builds (and `context/operations/android/setup.md` for Android).
