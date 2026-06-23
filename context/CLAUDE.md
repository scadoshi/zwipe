# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ZWIPE is a mobile-first Magic: The Gathering deck builder with swipe-based navigation. Full-stack Rust application with hexagonal architecture.

- **Shared domain (zwipe-core/)**: Pure domain types, validation, and business rules — see rules below
- **Backend (zerver/)**: Axum REST API with PostgreSQL, SQLx, JWT auth
- **Frontend (zwiper/)**: Dioxus cross-platform app (web/iOS/Android)
- **Website (zite/)**: Dioxus static site
- **Database**: 35k+ MTG cards synced from Scryfall API

## zwipe-core Purity Rules

**zwipe-core is the shared domain crate. It must stay pure.** These rules are non-negotiable:

- **No feature flags.** No `#[cfg(feature = "...")]` on anything in this crate.
- **No server-only dependencies.** No sqlx, anyhow, axum, tokio, argon2, jsonwebtoken, or any crate only the server needs.
- **No SQLx derives or annotations.** No `#[derive(FromRow)]`, no `#[sqlx(...)]`. Domain types must not know about Postgres.
- **No service-layer errors.** Error types that wrap `anyhow::Error` (database failures, not-found, etc.) stay in zerver.
- **Only truly shared types.** If only the server needs it, it stays in zerver.
- **No types with `From` impls in zerver's handlers.** `ApiError` stays in zerver because its `From<DomainError>` impls would violate the orphan rule if both types were in core. See `architecture/decisions.md`.
- **All domain validation and tests live here.** Zerver re-exports via `pub use zwipe_core::...` — it adds only server-specific behavior.

**Allowed dependencies:** serde, thiserror, uuid, chrono, email_address, once_cell, serde_json, sha2, rand — crates that both frontend and backend legitimately use.

**Database adapter pattern:** Domain types are persisted via `Database*` wrapper structs in zerver's `outbound/sqlx/` layer. Wrappers use primitive fields (`String`, `Vec<String>`, `Json<T>`) that SQLx handles natively, then convert to domain types via `TryFrom`. See `architecture/decisions.md` for full rationale.

## Common Commands

### Backend (zerver)
```bash
cargo run --bin zerver          # Run web server
cargo run --bin zervice         # Run background sync job (Scryfall updates)
cargo test --workspace          # Run all tests
cargo clippy --workspace --all-targets -- -D warnings
```

### Frontend (zwiper)
```bash
dx serve                        # Web with hot reload (default)
dx serve --platform desktop     # Desktop app
dx serve --platform ios         # iOS (requires Xcode)
dx build --release --platform desktop
```

### Database Setup
```bash
./zcripts/dev-env/macos/setup.sh     # macOS
./zcripts/dev-env/fedora/reset.sh  # Linux
```

### SQLx Offline Mode (run after any query change)
```bash
# Run from workspace root whenever you add/modify a query_scalar!, query!, or query_as!
cargo sqlx prepare --workspace
# Commit the generated .sqlx/ directory — CI builds use it instead of a live DB
```

## Architecture

### Crate Dependency Graph

```
zwiper ──→ zwipe-core ←── zerver
zite   ──→ zwipe-core
```

zwipe-core owns all shared domain types. zerver re-exports them and adds server-specific layers (ports, services, database adapters, HTTP handlers). zwiper and zite import from zwipe-core for domain types and from zerver only for HTTP contract types (routes, ApiError, Http* structs).

### Hexagonal (Ports & Adapters) Pattern

All packages follow this structure where applicable:
```
src/lib/
├── domain/           # Pure business logic, newtypes, no external deps
│   └── {entity}/
│       ├── models/   # Entities and value objects
│       └── requests/ # Operation request types + validation errors
├── inbound/          # Entry points (HTTP handlers, UI screens)
│   ├── http/         # Backend: Axum handlers, routes, middleware
│   └── ui/           # Frontend: screens/, components/
└── outbound/         # External systems (database, APIs, HTTP client)
    └── sqlx/         # Backend: SQLx repositories (Database* wrappers here)
    └── client/       # Frontend: API client modules
```

### Key Patterns

**Newtypes for type safety**: `UserId`, `DeckId`, `EmailAddress`, `Password` - prevents mixing IDs, enforces validation at construction

**Module structure**: Uses `module/mod.rs` pattern (not monolithic `module.rs` files)

**Session auth**: JWT access tokens (24h) + rotating refresh tokens (14d, SHA-256 hashed, max 5 per user)

**Card filtering**: Backend uses SQLx QueryBuilder with PostgreSQL jsonb operators (`@>`, `<@`, `?|`). Frontend has modular filter screens synced via Dioxus signals.

## Linting

26 Clippy warnings configured in workspace `Cargo.toml`. Thresholds in `clippy.toml`:
- `too-many-arguments-threshold = 7` (refactor to builder pattern above this)
- `type-complexity-threshold = 250`
- `cognitive-complexity-threshold = 25`

## Environment Files

Backend (`zerver/.env`): `DATABASE_URL`, `JWT_SECRET`, `BIND_ADDRESS`, `ALLOWED_ORIGINS`, `RESEND_API_KEY`, `RESEND_EMAIL_FROM`, `LOG_DIR`
Frontend (`zwiper/.env`): `BACKEND_URL`, `RUST_LOG`, `RUST_BACKTRACE`

## Commit Guidelines

- Concise, one-line messages (multi-line only when many changes)
- Group related files logically
- No emojis
- Use `git diff` to understand changes before committing
- **Never** include AI-agent signatures in your commits.
    - Example: "Written with the help of Claude Opus 4.5"
    - Never commit with something like this in your message.

## Context Directory

```
context/
├── README.md               — start-here orientation (index of this directory)
├── CLAUDE.md               — this file
├── product/                — what we're building (prd, monetization, premium/ feature catalog)
├── architecture/           — why things are built the way they are
├── operations/             — how to build, deploy & ship (infrastructure/, ios/, android/)
├── development/            — coding standards (commits, docs, newtypes, dioxus)
├── progress/               — where we are (overview.md, todo.md, backlog.md)
└── archive/                — no longer active (brain, complete-*, learning framework)
```
