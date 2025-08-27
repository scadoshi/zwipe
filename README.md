# Zwipe 🃏

A mobile-first Magic: The Gathering deck building app with a **Tinder-like swiping interface**. Swipe right to add cards to your deck, left to skip - making deck building fun and intuitive.

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

✅ **Complete - User Domain Pipeline**
- Full CRUD operations (Create, Read, Update, Delete)
- Comprehensive error handling with request/operation separation
- Type-safe API design with proper HTTP status mapping
- Enterprise-level hexagonal architecture

✅ **Complete - Authentication System**  
- JWT token generation and validation
- Argon2 password hashing with salt
- Custom middleware for route protection
- Production-ready security boundaries

✅ **Complete - Card Data Integration**
- 35,400+ MTG cards from Scryfall API
- Complex nested types (prices, legalities, card faces)
- Bulk operations with optimized batching
- JSONB storage for flexible card data

🚧 **In Progress - Auth HTTP Handlers**
- Registration and login endpoints
- Following established User domain patterns
- Comprehensive error mapping

🔄 **Next - Mobile Frontend**
- Dioxus mobile app development
- Card swiping interface
- Offline-capable deck building

## 🎓 Learning-Focused Development

This project **prioritizes learning over speed**, featuring:

### 📚 Documentation & Progress Tracking
- **[Learning Progress](/.cursor/rules/brain-progress.mdc)**: Neural pathway mapping for Rust/web development concepts
- **[Project Status](/.cursor/rules/project-progress.mdc)**: Detailed implementation progress and architecture decisions
- **[Core Decisions](/core_decisions/)**: Technical strategy documents and trade-off analysis
- **[Quizzes](/quizzes/)**: Regular knowledge assessments to solidify learning

### 🤖 AI-Assisted Learning
- AI assistants help update progress documentation
- Learning-focused code reviews and architecture discussions  
- Quiz generation based on implemented concepts
- Pattern recognition and best practice guidance

### 🧠 Knowledge Tracking
The project maintains detailed learning maps across confidence levels:
- **Confident**: Could teach others (Rust fundamentals, basic web dev)
- **Developing**: Successfully implemented but still learning (hexagonal architecture, advanced error handling)
- **Learning**: Recently introduced concepts needing guidance

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ (for async fn in traits)
- PostgreSQL 14+
- sqlx-cli (`cargo install sqlx-cli`)

### Setup
```bash
# Clone and setup
git clone <repository-url>
cd deck-builder

# Database setup
createdb deck_builder
cd deck_builder
sqlx migrate run

# Environment setup
cp .env.example .env
# Edit .env with your database URL and JWT secret

# Run the server
cargo run --bin server
```

### Test the API
```bash
# Health check
curl http://localhost:8080/health

# Register a user
curl --json '{"username": "player1", "email": "player1@example.com", "password": "secure123"}' \
  http://localhost:8080/api/v1/auth/register

# Create a user (alternative endpoint)
curl --json '{"username": "player2", "email": "player2@example.com"}' \
  http://localhost:8080/api/v1/users
```

## 🏛️ Architecture Highlights

### Hexagonal Architecture (Ports & Adapters)
```
📁 Domain Layer (Business Logic)
├── User domain: CRUD operations with validation
├── Auth domain: JWT + password security  
├── Card domain: MTG card data management
└── Deck domain: Deck building logic

📁 Ports Layer (Interfaces)
├── Repository traits for data access
├── Service traits for business operations
└── Clean dependency boundaries

📁 Adapters Layer (Implementation)
├── HTTP handlers (Axum)
├── Database operations (SQLx)
├── External APIs (Scryfall)
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

**Built with Rust 🦀 | Powered by Learning 🧠 | Mobile-First 📱**

*This project demonstrates enterprise-grade Rust development with hexagonal architecture, comprehensive error handling, and production-ready patterns - all while maintaining a focus on learning and knowledge transfer.*
