# zerver

Axum REST API server for Zwipe. Re-exports domain types from `zwipe-core` and adds server-specific layers.

## Responsibilities

- **HTTP handlers** — Axum route handlers, middleware, JWT auth extraction
- **Database adapters** — SQLx repositories with `Database*` wrapper types for Postgres
- **Service layer** — business logic orchestration, port trait implementations
- **Port traits** — repository and service interfaces
- **Service-layer errors** — error types wrapping `anyhow::Error` (not in core)
- **Binaries** — `zerver` (web server) and `zervice` (background sync jobs)

## Key Pattern

Domain types live in `zwipe-core`. Zerver re-exports via `pub use zwipe_core::...` and adds only what requires server-only dependencies (sqlx, axum, argon2, jsonwebtoken, tokio).

Database persistence uses `Database*` wrapper structs with primitive fields — domain types never have SQLx derives.
