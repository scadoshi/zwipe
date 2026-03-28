# zwipe

mobile-first magic: the gathering deck builder with swipe-based navigation.

**status:** shipped to ios — approaching app store submission

## tech stack

full-stack rust application:
- **zerver**: axum rest api, postgresql, sqlx, jwt auth, scryfall sync
- **zwiper**: dioxus mobile/ios/android app, swipe gesture system
- **zweb**: dioxus web client (zwipe.net)
- **architecture**: hexagonal/ports-and-adapters pattern

## quick start

```bash
# prerequisites
# - rust (https://rustup.rs)
# - macos: xcode command line tools (xcode-select --install)

# run setup script (installs postgres, dx, sqlx-cli, creates database)
./zcripts/denv/mac/setup.sh      # macos
./zcripts/denv/fedora/setup.sh   # linux

# start development
cargo run --bin zerver            # backend api
cd zwiper && dx serve             # mobile app (web preview)
cargo run --bin zervice           # scryfall card sync (run once to seed)
```

## architecture

hexagonal architecture with domain-driven design:
- clean domain layer with type-safe newtypes
- port/adapter separation between business logic and infrastructure
- shared rust types between frontend and backend (zwiper depends on zerver domain)

see [contributing](CONTRIBUTING.md) for project structure detail.

## license

cc by-nc 4.0 - see [license](LICENSE)
