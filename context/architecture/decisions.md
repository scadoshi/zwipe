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

Full iOS signing + deploy reference: `ops/ios.md`.

---

## Shared Models: zwiper imports from zerver

**Decided: early development.**

Both zerver and zwiper need the same domain types (User, Deck, Card, request/response structs). Three options were considered:

1. Separate shared library (`ztructs`) — clean but breaks the domain-first workflow, requires three-way coordination
2. Shared library with feature flags — same problems
3. **zwiper imports directly from zerver** ← chosen

The insight: zerver IS the domain layer. In hexagonal architecture, adapters depend on the domain — and zwiper is just another adapter (the UI adapter). The `#[cfg(feature = "zerver")]` flag hides server-only code (SQLx, Axum) from the frontend build.

**Result**: single source of truth, no extraction work, no duplication. `zwiper/Cargo.toml` depends on `zerver` without the `zerver` feature.

---

## Hosting: Ubuntu Server via Cloudflare Tunnel

See `architecture/hosting.md`.
