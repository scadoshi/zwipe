# zwipe

mobile-first magic: the gathering deck builder with swipe-based navigation.

**status:** in active development

## tech stack

full-stack rust application:
- **backend**: axum, postgresql, sqlx, jwt auth
- **frontend**: dioxus (web/ios/android), swipe gesture system
- **architecture**: hexagonal/ports-and-adapters pattern

## quick start

```bash
# prerequisites
# - rust 1.75+ (https://rustup.rs)
# - macos: xcode command line tools (xcode-select --install)

# run setup script (installs postgres, dx, sqlx-cli, creates database)
./zcripts/denv/macos/setup.sh    # macos
./zcripts/denv/fedora/setup.sh   # linux

# start development
cd zerver && cargo run --bin zerver  # backend
cd zwiper && dx serve                # frontend
cargo run --bin zervice              # background sync (optional)
```

## architecture

hexagonal architecture with domain-driven design:
- clean domain layer with type-safe newtypes
- port/adapter separation between business logic and infrastructure
- shared rust types between frontend and backend

## license

cc by-nc 4.0 - see [license](LICENSE)
