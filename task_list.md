# MTG Deck Builder - Task List

## Project Overview
Building a Magic: The Gathering deck-building mobile app with Tinder-like swiping interface.
- **Backend**: Rust (Axum framework) + PostgreSQL
- **Frontend**: Flutter (iOS/Android)
- **Goal**: MVP with core swiping functionality

---

## Phase 1: Development Environment Setup

### ✅ System Dependencies
- [x] Rust (latest stable) - confirmed installed
- [x] PostgreSQL 16.9 (installed and running)
- [x] Diesel CLI (installed)
- [x] Development headers and build tools

### ✅ Rust Project Creation
- [x] Created `deck_builder_api` Rust project with Cargo
- [x] Configured Axum web framework
- [x] Added all necessary dependencies (Diesel, tokio, serde, etc.)
- [x] CORS configuration for Flutter frontend

### ✅ Database Setup
- [x] Create development database (`diesel setup`)
- [x] Verify database connection
- [x] Set up database credentials (PostgreSQL auth with user `scottyrayfermo`)

### ✅ Basic Server Setup
- [x] Created basic Axum server with health check
- [x] Configured environment variables
- [x] Server compiles and runs successfully on port 8080
- [x] Basic API routes structure in place

---

## Phase 2: Database Models & Schema

### ✅ Database Schema Design 
- [x] Create User table migration with Diesel
- [x] Create Card table migration (MTG card data)
- [x] Create Deck table migration (user's deck collections)
- [x] Create DeckCard join table migration (cards in decks with quantities)
- [x] Run all migrations
- [x] Fix migration syntax errors (semicolons, table references, constraints)

### ✅ Rust Model Structs (COMPLETE! 🎉)
- [x] Define User struct with Diesel derives (complete with security features)
- [x] Define Card struct with all MTG fields (production-ready with docs)
- [x] Define Deck struct with user relationship (complete with foreign key!)
- [x] Define DeckCard struct for join table (dual foreign keys working!)
- [x] Set up proper serialization with serde (All models complete)
- [x] Custom MtgFormat enum with database integration (strum + Diesel)

### 🎯 Database Relationships
```rust
// User -> has many Decks
// Deck -> belongs to User, has many DeckCards
// Card -> has many DeckCards
// DeckCard -> belongs to Deck and Card
```

---

## Phase 3: Authentication & Core API

### ✅ Database Connection Pool (COMPLETE! 🎉)
- [x] Configure Diesel connection pool (r2d2 with PostgreSQL)
- [x] Add database state to Axum app (State<DbPool>)
- [x] Handle database errors properly (StatusCode mapping)
- [x] Test database connectivity (working perfectly!)
- [x] Organize imports properly (categorized by std/external/internal)
- [x] Build production-ready handlers (DB vs non-DB separation)

### ✅ API Endpoints Infrastructure (COMPLETE! 🎉)
- [x] `GET /` - Root endpoint (no DB, static info)
- [x] `GET /health` - Shallow health check (fast, no DB)
- [x] `GET /health/deep` - Deep health check (with DB connection test)
- [x] `GET /api/v1/decks` - List user's decks (REAL database query working!)
- [x] `GET /api/v1/cards` - Cards endpoint (placeholder, ready for implementation)
- [x] All endpoints tested and working via curl
- [x] Clean JSON responses with proper structure
- [x] Error handling with appropriate HTTP status codes

### ✅ Code Organization & Module Structure (COMPLETE! 🎉)
- [x] Refactor handlers into separate modules (`handlers/cards.rs`, `handlers/decks.rs`, `handlers/health.rs`)
- [x] Create proper `handlers/mod.rs` with module exports
- [x] Move health endpoints from main.rs to dedicated module
- [x] Clean main.rs (reduced from 138 to 69 lines)
- [x] Implement consistent import patterns across all handlers
- [x] Use explicit route naming (`handlers::cards::list_cards`)
- [x] Establish production-ready module architecture

### 🎯 User Authentication (NEXT UP!)
- [ ] Add password hashing with argon2
- [ ] Create user registration endpoint: `POST /api/v1/users`
- [ ] Create login endpoint with JWT generation: `POST /api/v1/auth/login`
- [ ] Add JWT middleware for protected routes
- [ ] Add user validation (unique email/username)
- [ ] Extract user_id from JWT tokens in handlers
- [ ] Test authentication flow

### 🎯 API Endpoints Enhancement
- [ ] Add JWT authentication to protected routes
- [ ] Implement proper user context extraction
- [ ] Add request/response logging middleware
- [ ] Implement rate limiting
- [ ] Add API documentation endpoints

---

## Phase 4: Card Data Management

### 🎯 MTG Card Data Integration
- [ ] Research Scryfall API integration
- [ ] Create card data seeding script
- [ ] Seed initial card set (Standard-legal cards)
- [ ] Add card image URL handling
- [ ] Test card data retrieval

### 🎯 Card API Endpoints
- [ ] `GET /api/v1/cards` - List cards with pagination
- [ ] `GET /api/v1/cards/:id` - Get specific card
- [ ] `GET /api/v1/cards/random` - Random cards for swiping
- [ ] `GET /api/v1/cards/search` - Search cards by name/type
- [ ] Add proper filtering and sorting

---

## Phase 5: Deck Management API

### 🎯 Deck CRUD Operations
- [ ] `GET /api/v1/decks` - List user's decks
- [ ] `POST /api/v1/decks` - Create new deck
- [ ] `GET /api/v1/decks/:id` - Show deck details
- [ ] `PUT /api/v1/decks/:id` - Update deck
- [ ] `DELETE /api/v1/decks/:id` - Delete deck

### 🎯 Card-Deck Management
- [ ] `POST /api/v1/decks/:id/cards` - Add card to deck (right swipe)
- [ ] `DELETE /api/v1/decks/:id/cards/:card_id` - Remove card from deck
- [ ] `PUT /api/v1/decks/:id/cards/:card_id` - Update card quantity
- [ ] Implement deck validation rules

### 🎯 Card Swiping Logic
- [ ] Implement card queue system (pre-load next 10 cards)
- [ ] Add format filtering for card selection
- [ ] Track user preferences for better card suggestions
- [ ] Optimize for mobile performance

### 🚀 Future Enhancements (Post-MVP)
- [ ] **Advanced Search & Recommendations**
  - [ ] Add keywords column to cards table (TEXT[] array)
  - [ ] Implement keyword extraction from oracle text
  - [ ] Add synergy_tags for deck building suggestions
  - [ ] Create GIN indexes for fast keyword searches
  - [ ] Build recommendation algorithm based on deck synergies
  - [ ] Add power_level and complexity_rating columns

---

## Phase 6: Testing & Validation

### 🎯 API Testing
- [ ] Test all endpoints with curl/Postman
- [ ] Verify JSON response formats
- [ ] Test authentication flows
- [ ] Test error handling and validation
- [ ] Performance testing for card swiping

### 🎯 Data Validation
- [ ] Validate deck size limits (60 card minimum for Standard)
- [ ] Validate card quantities (max 4 of same card)
- [ ] Test user authorization for deck operations
- [ ] Validate MTG format legality

---

## Phase 7: Flutter Integration (Future)

### 🎯 Flutter Project Setup
- [ ] Install Flutter SDK
- [ ] Create new Flutter project
- [ ] Set up HTTP client for Rust API calls
- [ ] Configure JWT token storage

### 🎯 Flutter UI Development
- [ ] Create login/register screens
- [ ] Build card swiping interface
- [ ] Implement deck list and detail views
- [ ] Add card image loading and caching

---

## Current Status

**✅ COMPLETED**: 
- Rust development environment fully set up
- Axum web server created and running  
- Database connection configured (PostgreSQL)
- Basic project structure with proper dependencies
- Core decisions updated to reflect Rust choice
- **All database migrations created and executed successfully**
- **Complete database schema with proper foreign keys, indexes, and constraints**
- **User model complete** (4 structs: User, NewUser, UpdateUser, UserResponse)
- **Card model complete** (4 structs: Card, NewCard, UpdateCard, CardResponse)
- **Deck model complete** (4 structs + foreign key to User + MtgFormat enum)
- **DeckCard model complete** (4 structs + dual foreign keys to Deck and Card)
- **Custom MtgFormat enum** (strum + manual Diesel ToSql/FromSql implementations)
- **Production-quality documentation and security features**
- **All models use proper check_for_backend validation**
- **ALL 4 CORE DATABASE MODELS COMPLETE AND COMPILING! 🎉**
- **🔥 DATABASE CONNECTION POOL (r2d2) - PRODUCTION READY! 🔥**
- **🔥 API ENDPOINTS WITH REAL DATABASE QUERIES WORKING! 🔥**
  - `GET /` - Static API info (optimized, no DB)
  - `GET /health` - Fast health check (no DB)
  - `GET /health/deep` - Full health check with DB connectivity test
  - `GET /api/v1/decks` - Real Diesel query: `SELECT * FROM decks WHERE user_id = 1`
  - `GET /api/v1/cards` - Ready for card data integration
- **Organized imports by category (std/external/internal)**
- **Professional error handling with proper HTTP status codes**
- **All endpoints tested via curl and returning clean JSON**
- **🏗️ PRODUCTION-READY MODULE ARCHITECTURE COMPLETE! 🏗️**
  - Clean main.rs (69 lines, focused on server setup)
  - Organized handlers into modules (`handlers/cards.rs`, `handlers/decks.rs`, `handlers/health.rs`)
  - Proper module exports with `handlers/mod.rs`
  - Consistent import patterns across all modules
  - Explicit route naming for clarity (`handlers::cards::list_cards`)
  - Scalable architecture ready for authentication and new features

**🎯 NEXT**: User Authentication (registration, login, JWT tokens)!
**📍 YOU ARE HERE**: PRODUCTION-READY API WITH CLEAN MODULE ARCHITECTURE! 🚀

---

## Quick Start Commands - Current State

```bash
# Test current server
cargo run
curl http://localhost:8080/health

# Test all endpoints
curl http://localhost:8080/              # Root info
curl http://localhost:8080/health        # Health check
curl http://localhost:8080/health/deep   # DB connectivity
curl http://localhost:8080/api/v1/decks  # User decks
curl http://localhost:8080/api/v1/cards  # Cards (placeholder)
```

**Tech Stack Confirmed:** 
- ✅ **Backend**: Rust + Axum + Diesel + PostgreSQL
- ✅ **Frontend**: Flutter (unchanged)
- ✅ **Database**: PostgreSQL with Diesel ORM
- ✅ **Auth**: JWT tokens with argon2 password hashing (ready to implement)
- ✅ **Architecture**: Production-ready module structure

Ready to build fast, type-safe MTG deck builder! 🦀⚡ 