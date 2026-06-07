# zwipe

mobile-first magic: the gathering deck builder with swipe-based navigation.

**status:** feature complete — ready for app store submission

## tech stack

full-stack rust application:
- **zwipe-core**: shared domain types, validation, HTTP contracts — the single source of truth
- **zerver**: axum rest api, postgresql, sqlx, jwt auth, scryfall sync
- **zwiper**: dioxus mobile/ios app, swipe gestures, 9 themes, dark mode
- **zite**: dioxus web client at [zwipe.net](https://zwipe.net)
- **zervice**: background jobs (scryfall sync, session cleanup)

```
zwiper ──→ zwipe-core ←── zerver
zite   ──→ zwipe-core
```

## quick start

```bash
# prerequisites: rust (https://rustup.rs), macos: xcode-select --install

./zcripts/dev-env/macos/setup.sh      # macos setup (postgres, dx, sqlx-cli, database)
./zcripts/dev-env/fedora/setup.sh   # linux setup

cargo run --bin zerver            # backend api
cd zwiper && dx serve             # mobile app (web preview)
cargo run --bin zervice           # scryfall card sync (run once to seed)
```

## architecture

hexagonal architecture with domain-driven design. `zwipe-core` owns all shared domain types — zerver re-exports them and adds server-specific layers (database adapters, HTTP handlers, service orchestration). See `context/architecture/decisions.md` for key decisions.

## license

cc by-nc 4.0 - see [license](LICENSE)
