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
