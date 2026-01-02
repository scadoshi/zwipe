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
# prerequisites: rust 1.75+, postgresql 14+, dx cli

# setup database
./zcripts/denv/mac/setup.sh      # macos
./zcripts/denv/fedora/reset.sh   # linux

# run backend
cd zerver && cargo run --bin zerver

# run frontend
cd zwiper && dx serve

# background card sync (optional)
cargo run --bin zync
```

## architecture

hexagonal architecture with domain-driven design:
- clean domain layer with type-safe newtypes
- port/adapter separation between business logic and infrastructure
- shared rust types between frontend and backend

## license

cc by-nc 4.0 - see [license](LICENSE)
