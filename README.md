# zwipe

a mobile-first magic: the gathering deck building app with a **tinder-like swiping interface**. swipe right to add cards to your deck, left to skip - making deck building fun and intuitive.

**this is a learning first project** designed to develop production-ready rust skills for professional work in the rust ecosystem, with the hope of becoming a real application someday.

## what this app does

**core vision**: transform mtg deck building from a complex, desktop-focused task into an engaging, mobile-first experience.

- swipe-to-build: browse cards with smooth swiping gestures
- mobile-first: optimized for ios and android  
- fast & offline: local card database for instant performance
- user accounts: save and sync decks across devices with rotating refresh tokens
- complete card database: access to all standard-legal mtg cards

## architecture

**full-stack rust** 🦀 with enterprise-grade hexagonal architecture:

```
backend (rust)
├── axum web framework
├── sqlx with postgresql 
├── session-based authentication with rotating refresh tokens
├── hexagonal architecture (ports/adapters)
└── production-ready error handling

frontend (rust + dioxus)
├── dioxus cross-platform framework
├── shared rust types with backend
├── swipe gesture detection (touch + mouse)
├── secure token storage (keychain/keystore)
└── native mobile performance
```

### current implementation status

**complete - enterprise hexagonal architecture**
- production-ready ports/adapters pattern with clean domain separation
- multi-domain architecture (auth, user, card, deck domains)
- advanced newtype patterns with smart constructors
- comprehensive error handling with dual-layer mapping

**complete - session-based authentication**  
- rotating refresh token architecture with sha-256 hashing
- jwt access tokens with custom middleware validation
- argon2 password hashing with enterprise-level validation
- multi-device session support (up to 5 concurrent sessions)
- route protection through type-based authentication

**complete - card data integration**
- 35,400+ mtg cards from scryfall api
- complex nested types with jsonb storage
- bulk operations with optimized batching
- background sync job with metrics tracking

**complete - deck management system**
- full deck crud operations with proper relationship modeling
- nested resource api endpoints with restful design
- cross-domain service orchestration with hashmap join optimization

**in progress - dioxus mobile frontend**
- swipe gesture detection system (touch + mouse support)
- auth screens with real-time validation
- http client integration with backend
- secure token storage architecture designed
- next: token refresh on 401, loading screens, main app ui

## learning-focused development & portfolio project

this project serves as both a **learning laboratory** and **portfolio demonstration** for advancing rust development skills with the goal of working professionally in the rust ecosystem.

### learning philosophy
rather than rushing to market, this project **prioritizes deep learning and clean architecture** to build industry-ready skills:
- enterprise patterns for portfolio demonstration (hexagonal architecture, comprehensive error handling)
- pragmatic choices for learning efficiency (strategic simplification over theoretical purity)
- production-ready practices that employers value (type safety, security-first design, thorough documentation)

### ai-enhanced learning management
the project leverages ai assistants to optimize the learning experience:

- [learning progress tracking](/progress/brain.md): detailed skill mapping across confidence levels
- [project status management](/progress/project.md): implementation progress with architectural decisions
- [architecture guidelines](/.cursor/rules/): hexagonal patterns and learning optimization strategies
- guided development: ai maintains focus on learning objectives while building production-quality code

### skill development tracking
the project maintains granular learning maps to demonstrate growth:
- **confident**: ready to teach others and use in production
- **developing**: successfully implemented but still expanding understanding  
- **learning**: recently introduced concepts with guided practice
- **unexplored**: future learning targets identified

this approach creates a **living portfolio** that shows not just what was built, but **how skills were systematically developed** - valuable for technical interviews and demonstrating learning ability to potential employers.

## quick start

### prerequisites
- rust 1.75+ (for async fn in traits)
- postgresql 14+
- sqlx-cli (`cargo install sqlx-cli`)

### setup
```bash
# clone and setup
git clone <repository-url>
cd zwipe

# setup using provided scripts
./scripts/dev_env_init/

# available binaries
cargo run --bin zerver    # main api server
cargo run --bin zync      # background card sync job
cargo run --bin zync_test # sync testing utility

# health check
curl http://localhost:3000/health/database
```

## architecture highlights

### hexagonal architecture (ports & adapters)
```
domain layer (business logic)
├── auth: session management + rotating refresh tokens + password security
├── user: read-only user profile access
├── card: mtg card data management + search
├── deck: deck building + card management
└── health: system monitoring

ports layer (interfaces)
├── repository traits for data access
├── service traits for business operations
└── clean dependency boundaries

adapters layer (implementation)
├── http handlers (axum) with nested resource routes
├── database operations (sqlx) with composite keys
├── external apis (scryfall) with sync jobs
└── mobile client (dioxus) - in progress
```

### production-ready patterns
- **type safety**: comprehensive newtype patterns for domain validation
- **error handling**: dual-layer error mapping (request validation vs operation errors)
- **security**: sha-256 token hashing, argon2 password hashing, rotating refresh tokens
- **performance**: bulk operations, connection pooling, optimized queries
- **testing**: comprehensive test coverage with organized categories

## data strategy

**hybrid architecture** 📦 for optimal performance:

- **local storage**: complete card database (~400mb) for instant swiping
- **server storage**: user accounts and deck sync for cross-device access  
- **smart caching**: popular card images cached locally
- **offline capability**: full deck building without internet

## mvp scope

**phase 1: backend api foundation** (complete)
- user management with session-based authentication
- card database with scryfall integration (35,400+ cards)
- deck crud operations with nested resource api
- production-ready error handling and security

**phase 2: mobile app development** (in progress)
- dioxus cross-platform mobile app
- swipe gesture detection system (complete)
- auth screens with validation (complete)
- token refresh and loading screens (next)
- card browsing and deck building ui

**phase 3: polish & deploy**
- performance optimization
- app store deployment
- user onboarding flow

## success metrics

### user engagement targets
- 50+ cards swiped per session
- 2+ decks created per user  
- 5+ minute session length
- 30%+ 7-day return rate

### technical performance
- <3s app launch time
- <100ms card swipe response
- <2s image load time
- <500ms api response time

## target audience

- **primary**: mtg players wanting mobile deck building
- **secondary**: new players seeking simple deck creation
- **demographics**: 18-35 years old, mobile-first users

## platform strategy

1. **android** (native linux testing)
2. **ios** (cloud builds)
3. **web** (bonus dioxus target)

## license

mit license - see [license](license) for details.

---

**built with rust 🦀 | powered by learning | portfolio-ready**

*this project demonstrates enterprise-grade rust development with hexagonal architecture, comprehensive error handling, and production-ready patterns - all developed through ai-guided learning to build industry-relevant skills. the codebase serves as both a learning laboratory and a portfolio piece showcasing systematic skill development in the rust ecosystem.*
