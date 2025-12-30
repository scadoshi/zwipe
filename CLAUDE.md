# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ZWIPE is a mobile-first Magic: The Gathering deck builder with swipe-based navigation. Full-stack Rust application with hexagonal architecture.

- **Backend (zerver/)**: Axum REST API with PostgreSQL, SQLx, JWT auth
- **Frontend (zwiper/)**: Dioxus cross-platform app (web/iOS/Android)
- **Database**: 35k+ MTG cards synced from Scryfall API

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
./zcripts/denv/mac/setup.sh     # macOS
./zcripts/denv/fedora/reset.sh  # Linux
```

## Architecture

### Hexagonal (Ports & Adapters) Pattern

Both packages follow this structure:
```
src/lib/
├── domain/           # Pure business logic, newtypes, no external deps
│   └── {entity}/
│       └── models/   # Per-operation request/response types
├── inbound/          # Entry points (HTTP handlers, UI screens)
│   ├── http/         # Backend: Axum handlers, routes, middleware
│   └── ui/           # Frontend: screens/, components/
└── outbound/         # External systems (database, APIs, HTTP client)
    └── sqlx/         # Backend: SQLx repositories
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

Backend (`zerver/.env`): `DATABASE_URL`, `JWT_SECRET`, `BIND_ADDRESS`, `ALLOWED_ORIGINS`
Frontend (`zwiper/.env`): `BACKEND_URL`, `RUST_LOG`, `RUST_BACKTRACE`

## Commit Guidelines

- Concise, one-line messages (multi-line only when many changes)
- Group related files logically
- No emojis
- Use `git diff` to understand changes before committing
