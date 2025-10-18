# zwipe

mobile-first magic: the gathering deck builder with swipe-based navigation. a learning-focused project building production rust skills.

## status

**backend** 🦀 - complete
- hexagonal architecture with ports/adapters pattern
- session-based auth with rotating refresh tokens (jwt + sha-256)
- postgresql with sqlx, 35k+ cards from scryfall api
- full crud for users, decks, cards with nested resource routes

**frontend** 🦀 - in progress  
- dioxus cross-platform framework
- swipe gesture system (touch + mouse)
- auth screens with real-time validation
- http client integration complete
- debugging screen freeze on login (session persistence)

## tech stack

```
backend
├── axum web framework
├── sqlx + postgresql
├── jwt access tokens + rotating refresh tokens
└── argon2 password hashing

frontend
├── dioxus (web + ios + android)
├── shared rust types via feature flags
├── swipe detection with velocity tracking
└── keychain/keystore session storage (pending entitlements)
```

## quick start

```bash
# prerequisites: rust 1.75+, postgresql 14+, sqlx-cli

# setup database
./zcripts/denv/mac/setup.sh  # or fedora/reset.sh

# run backend
cd zerver && cargo run --bin zerver

# run frontend  
cd zwiper && dx serve

# background sync job
cargo run --bin zync
```

## architecture

hexagonal architecture with clean domain separation:

- **domain layer**: business logic (auth, user, card, deck domains)
- **ports layer**: repository and service trait definitions
- **adapters layer**: http handlers, sqlx repos, scryfall client

type-safe newtypes throughout (userid, deckid, emailaddress, password) with smart constructor validation.

## learning focus

this project prioritizes deep learning over rapid deployment. comprehensive progress tracking in `/progress`:

- **brain.md** - skill confidence levels across rust topics
- **project.md** - implementation status and architectural decisions  
- **rules/** - hexagonal patterns and learning strategies

built to demonstrate production-ready rust skills for professional work.

## license

mit - see [license](LICENSE)

---

**learning rust 🦀 building clean architecture 🦀 mobile-first**
