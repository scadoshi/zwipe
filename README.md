# Zwipe

Mobile-first Magic: The Gathering deck builder with swipe-based navigation.

**Status:** v1.6.0 live on the iOS App Store; Android in closed testing (production launch in review); web client at [zwipe.net](https://zwipe.net).

## Tech stack

Full-stack Rust:
- **zwipe-core**: shared domain types, validation, business rules, HTTP contracts; the single source of truth
- **zwipe-components**: shared Dioxus UI components and CSS (themes, changelog, card details) consumed by the app, the site, and the owner's portfolio
- **zerver**: Axum REST API, PostgreSQL, SQLx, JWT auth, Scryfall sync
- **zwiper**: Dioxus iOS and Android app, swipe gestures, 31 themes, dark mode
- **zite**: Dioxus web client at [zwipe.net](https://zwipe.net)
- **zervice**: background jobs (Scryfall sync, session cleanup)

```
zwiper ──→ zwipe-core ←── zerver
  │           ↑            (zervice binary)
  └──→ zwipe-components ←── zite ──→ zwipe-core
```

## Quick start

```bash
# prerequisites: rust (https://rustup.rs), macos: xcode-select --install

./zcripts/dev-env/macos/setup.sh    # macos setup (postgres, dx, sqlx-cli, database)
./zcripts/dev-env/fedora/setup.sh   # linux setup

cargo run --bin zerver              # backend api
cd zwiper && dx serve               # mobile app (web preview)
cargo run --bin zervice             # scryfall card sync (run once to seed)
```

## Architecture

Hexagonal architecture with domain-driven design. `zwipe-core` owns all shared
domain types; zerver re-exports them and adds server-specific layers (database
adapters, HTTP handlers, service orchestration). See
`context/architecture/decisions.md` for key decisions and `context/README.md`
for the full documentation tree.

## License

CC BY-NC 4.0. See [LICENSE](LICENSE).
