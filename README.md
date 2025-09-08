# Zwipe 🃏

```
▎▎▉▉▉▉▉▉▉            ▎▎▉▉
  ▎▎▉▉▉▉▉▉▎▎▉▉    ▎▎▉▉▎▎▉▉▎▎▉▉▉▉▉▉ ▎▎▉▉▉▉▉▉▉
      ▎▎▉▉ ▎▎▉▉    ▎▎▉▉    ▎▎▉▉▉▉▉▉ ▎▎▉▉ ▎▎▉▉
     ▎▎▉▉   ▎▎▉▉▎▎▉▉▎▎▉▉▎▎▉▉▎▎▉▉ ▎▎▉▉▎▎▉▉▉▉▉▉▉
    ▎▎▉▉     ▎▎▉▉▎▎▉▉▎▎▉▉▎▎▉▉▎▎▉▉ ▎▎▉▉▎▎▉▉
   ▎▎▉▉▉▉▉▉▉▉ ▎▎▉▉▉▉▉▉▉▉  ▎▎▉▉▎▎▉▉▉▉▉▉ ▎▎▉▉▉ ▎▎▉▉
    ▎▎▉▉▉▉▉▉▉▉ ▎▎▉▉▎▎▉▉    ▎▎▉▉▎▎▉▉▉▉   ▎▎▉▉▉▉▉▉
                                ▎▎▉▉
                                 ▎▎▉▉
                                  ▎▎▉▉
                                   ▎▎▉▉
```

A mobile-first Magic: The Gathering deck building app with a **Tinder-like swiping interface**. Swipe right to add cards to your deck, left to skip - making deck building fun and intuitive.

**🦀 This is primarily a learning project** designed to develop production-ready Rust skills for professional work in the Rust ecosystem, with the hope of becoming a real application someday.

## 🎯 What This App Does

**Core Vision**: Transform MTG deck building from a complex, desktop-focused task into an engaging, mobile-first experience.

- **🔄 Swipe-to-Build**: Browse cards with smooth swiping gestures
- **📱 Mobile-First**: Optimized for iOS and Android  
- **⚡ Fast & Offline**: Local card database for instant performance
- **🔐 User Accounts**: Save and sync decks across devices
- **🃏 Complete Card Database**: Access to all Standard-legal MTG cards

## 🏗️ Architecture

**Full-Stack Rust** with enterprise-grade hexagonal architecture:

```
🦀 Backend (Rust)
├── Axum web framework
├── SQLx with PostgreSQL 
├── JWT authentication
├── Hexagonal architecture (ports/adapters)
└── Production-ready error handling

📱 Frontend (Future)
├── Dioxus cross-platform framework
├── Shared Rust types with backend
├── Offline-first with smart sync
└── Native mobile performance
```

### Current Implementation Status

✅ **Complete - Enterprise Hexagonal Architecture**
- Production-ready ports/adapters pattern with clean domain separation
- Multi-domain architecture (Auth, User, Card, Deck domains)
- Advanced newtype patterns with smart constructors
- Comprehensive error handling with dual-layer mapping

✅ **Complete - Authentication & Security System**  
- JWT token generation/validation with custom middleware
- Argon2 password hashing with enterprise-level validation
- Route protection through type-based authentication
- User ownership validation and privilege escalation prevention

✅ **Complete - Card Data Integration**
- 35,400+ MTG cards from Scryfall API
- Complex nested types (prices, legalities, card faces, image URIs)
- Bulk operations with optimized batching and composite key architecture
- JSONB storage with custom SQLx type implementations

✅ **Complete - Deck Management System**
- Full deck CRUD operations with proper relationship modeling
- Deck card management with composite key architecture
- Nested resource API endpoints with RESTful design
- Cross-domain service orchestration with HashMap join optimization

✅ **Complete - Production HTTP API**
- Complete REST API with proper HTTP semantics
- Comprehensive error mapping and status code handling
- Tuple path parameter extraction for multi-parameter routes
- Background sync job with metrics tracking

🔄 **Next - Mobile Frontend**
- Dioxus mobile app development
- Card swiping interface implementation
- Offline-capable deck building with sync

## 🎓 Learning-Focused Development & Portfolio Project

This project serves as both a **learning laboratory** and **portfolio demonstration** for advancing Rust development skills with the goal of working professionally in the Rust ecosystem.

### 🎯 Learning Philosophy
Rather than rushing to market, this project **prioritizes deep learning and clean architecture** to build industry-ready skills. You'll see a deliberate mix of:
- **Enterprise patterns** for portfolio demonstration (hexagonal architecture, comprehensive error handling)
- **Pragmatic choices** for learning efficiency (strategic simplification over theoretical purity)
- **Production-ready practices** that employers value (type safety, security-first design, thorough documentation)

### 🤖 AI-Enhanced Learning Management
The project leverages AI assistants to optimize the learning experience:

- **[Learning Progress Tracking](/progress/brain.md)**: Detailed neural pathway mapping across confidence levels (Confident → Developing → Learning → Unexplored)
- **[Project Status Management](/progress/project.md)**: Implementation progress with architectural decisions and learning achievements
- **[Architecture Guidelines](/.cursor/rules/)**: Hexagonal patterns and learning optimization strategies
- **Guided Development**: AI helps maintain focus on learning objectives while building production-quality code

### 🧠 Skill Development Tracking
The project maintains granular learning maps to demonstrate growth:
- **Confident**: Ready to teach others and use in production
- **Developing**: Successfully implemented but still expanding understanding  
- **Learning**: Recently introduced concepts with guided practice
- **Unexplored**: Future learning targets identified

This approach creates a **living portfolio** that shows not just what was built, but **how skills were systematically developed** - valuable for technical interviews and demonstrating learning ability to potential employers.

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ (for async fn in traits)
- PostgreSQL 14+
- sqlx-cli (`cargo install sqlx-cli`)

### Setup
```bash
# Clone and setup
git clone <repository-url>
cd zwipe

# Setup using provided scripts
./scripts/dev_env_init/

# Available binaries
cargo run --bin zerver    # Main API server (Zwipe Server)
cargo run --bin zync      # Background card sync job (Zwipe Sync)  
cargo run --bin zync_test # Sync testing utility

# Health check
curl http://localhost:3000/health/database
```

## 🏛️ Architecture Highlights

### Hexagonal Architecture (Ports & Adapters)
```
📁 Domain Layer (Business Logic)
├── Auth domain: JWT + password security + user lifecycle
├── User domain: Read-only user profile access
├── Card domain: MTG card data management + search
├── Deck domain: Deck building + card management
└── Health domain: System monitoring

📁 Ports Layer (Interfaces)
├── Repository traits for data access
├── Service traits for business operations
└── Clean dependency boundaries

📁 Adapters Layer (Implementation)
├── HTTP handlers (Axum) with nested resource routes
├── Database operations (SQLx) with composite keys
├── External APIs (Scryfall) with sync jobs
└── Future: Mobile client (Dioxus)
```

### Production-Ready Patterns
- **Type Safety**: Comprehensive newtype patterns for domain validation
- **Error Handling**: Dual-layer error mapping (request validation vs operation errors)
- **Security**: Domain-enforced password validation, JWT middleware
- **Performance**: Bulk operations, connection pooling, optimized queries
- **Testing**: Comprehensive test coverage with organized categories

## 📊 Data Strategy

**Hybrid Architecture** for optimal performance:

- **Local Storage**: Complete card database (~400MB) for instant swiping
- **Server Storage**: User accounts and deck sync for cross-device access  
- **Smart Caching**: Popular card images cached locally
- **Offline Capability**: Full deck building without internet

## 🎯 MVP Scope

**Phase 1**: Backend API Foundation ✅
- User management with authentication
- Card database with Scryfall integration
- Deck CRUD operations
- Production-ready error handling

**Phase 2**: Mobile App Development 🚧
- Dioxus cross-platform mobile app
- Card swiping interface
- Offline-first deck building
- Clean UI following mobile-first design

**Phase 3**: Polish & Deploy 🔄
- Performance optimization
- App store deployment
- User onboarding flow

## 📈 Success Metrics

### User Engagement Targets
- 50+ cards swiped per session
- 2+ decks created per user  
- 5+ minute session length
- 30%+ 7-day return rate

### Technical Performance
- <3s app launch time
- <100ms card swipe response
- <2s image load time
- <500ms API response time

## 🎮 Target Audience

- **Primary**: MTG players wanting mobile deck building
- **Secondary**: New players seeking simple deck creation
- **Demographics**: 18-35 years old, mobile-first users

## 📱 Platform Strategy

1. **Android** (native Linux testing)
2. **iOS** (cloud builds)
3. **Web** (bonus Dioxus target)

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

**Built with Rust 🦀 | Powered by Learning 🧠 | Portfolio-Ready 📋**

*This project demonstrates enterprise-grade Rust development with hexagonal architecture, comprehensive error handling, and production-ready patterns - all developed through AI-guided learning to build industry-relevant skills. The codebase serves as both a learning laboratory and a portfolio piece showcasing systematic skill development in the Rust ecosystem.*
