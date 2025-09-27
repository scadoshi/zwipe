---
description: Tracks project development status and provides development context for AI assistants to understand current progress and guide next steps appropriately
alwaysApply: true
---

## Update Instructions for AI Assistants

**Status Categories**: COMPLETE = Production-ready implementation. IN PROGRESS = Currently working on. NEXT = Planned immediate priorities. BACKLOG = Future planned work.

**When to Update**: After major feature completions, architectural decisions, or priority changes. Move items between categories as development progresses.

**Development Strategy**: Focus on completing current priorities before starting new work. Maintain clean architecture and comprehensive error handling throughout.

---

**Last Updated**: After implementing complete frontend validation architecture and HTTP client infrastructure

**Current Focus**: Completing HTTP request implementation and mobile app integration

**Recent Achievement**: Successfully implemented complete frontend validation architecture with RawRegisterUser type for secure ephemeral password validation. Built AuthClient with AppConfig integration for clean HTTP client management. Enhanced backend authentication with fast-fail password policies to prevent invalid requests. Established clean separation between validation functions and network request functions for optimal architecture.

**Current Decision**: AuthClient contains full AppConfig for maximum flexibility and consistent patterns across future clients (CardClient, DeckClient). Fast-fail validation prevents network requests for passwords that could never match database hashes, optimizing both performance and user experience.

### 🎯 Currently Working On (Top 5)
1. **HTTP Request Implementation** - Completing register_user and authenticate_user network functions
2. **Response Parsing & Error Handling** - JSON parsing and ApiError handling for frontend
3. **JWT Token Storage** - Secure mobile token storage without SQLite overhead
4. **Dioxus Component Integration** - Connect validation and HTTP functions to UI components
5. **Mobile App Testing** - End-to-end authentication flow testing

### 🤔 Next Immediate Priorities (Top 5)
1. **Complete HTTP Functions** - Finish register_user and authenticate_user request/response handling
2. **Error Type Mapping** - Frontend-friendly error types from backend ApiError responses
3. **Token Storage Service** - Platform-specific secure JWT storage implementation
4. **Real-time Form Validation** - Integrate validation functions into Dioxus login/register components
5. **Mobile UI Polish** - Beautiful responsive design with validation feedback

---

## COMPLETE - Production Ready ✅

### 🏗️ Core Architecture & Infrastructure
- **Hexagonal Architecture**: Complete ports/adapters pattern with clean domain separation
- **Multi-Domain Design**: Separate Auth, User, Card, Deck, and Health domains with clear boundaries
- **Database Foundation**: PostgreSQL with SQLx, connection pooling, migrations, and constraint management
- **Configuration Management**: Production-ready AppState with dependency injection patterns
- **Error Architecture**: Two-tier error handling (user-facing vs internal) with comprehensive domain error mapping
- **Security Infrastructure**: JWT middleware, password hashing, authentication flow, and route protection

### 🔐 Authentication & Security System
- **Complete Auth Domain**: Registration, login, password changes, and user lifecycle operations
- **JWT Implementation**: Token generation/validation with custom extractors and middleware
- **Password Security**: Argon2 hashing, salt generation, common password detection, complexity validation
- **Route Protection**: Type-safe authentication through handler parameters and middleware
- **Security Boundaries**: Information disclosure prevention and generic error responses

### 💾 Database & Data Management
- **SQLx Integration**: Raw SQL control with compile-time query verification and custom type integration
- **Advanced Query Patterns**: Dynamic QueryBuilder, bulk operations, transaction management
- **Constraint Handling**: PostgreSQL error code mapping, unique/check constraint violations
- **Production Data Pipeline**: Scryfall API integration with 35,400+ card processing capability
- **Composite Key Architecture**: Natural primary keys eliminating surrogate IDs where appropriate

### 📡 HTTP API & RESTful Design
- **Complete CRUD APIs**: User, Auth, Card, Deck, and DeckCard endpoints with proper HTTP semantics
- **RESTful Patterns**: Nested resource routes, path parameter extraction, status code precision
- **Advanced Middleware**: Custom extractors, type-safe authentication, generic handler patterns
- **Error Mapping**: Domain errors to appropriate HTTP status codes with information disclosure prevention
- **CORS Configuration**: Complete cross-origin setup for web application integration

### 🎮 Domain-Specific Implementation
- **Card Management**: Complete Scryfall integration, comprehensive search with CMC/power/toughness ranges, dual color identity modes, bulk data processing
- **Deck Management**: Full CRUD operations with card composition, cross-domain orchestration, and nested resource API
- **Auth Domain Security**: Complete user lifecycle operations (username/email updates, account deletion) centralized for security
- **User Domain Simplification**: Read-only profile access, all mutations moved to auth domain for proper security boundaries
- **Health Monitoring**: Database connectivity checks and system health endpoints
- **Shared Models Architecture**: Complete feature flag system enabling frontend-backend code sharing through zwipe library
- **Frontend Validation Refinement**: Granular feature gating across ALL domains with perfect frontend/backend compilation separation
- **Frontend Validation Architecture**: RawRegisterUser type with ephemeral password handling and shared newtype validation
- **HTTP Client Infrastructure**: AuthClient with AppConfig integration and clean validation/network separation
- **Fast-fail Authentication**: Enhanced password validation preventing invalid network requests

---

## NEXT PRIORITIES 🎯

### 🚀 Production System Hardening
- **Rate Limiting Implementation**: Request throttling, abuse prevention, and API protection
- **Performance Optimization**: Query optimization, connection pool tuning, database indexing
- **Monitoring & Observability**: Structured logging, metrics collection, health monitoring
- **Caching Layer**: Redis integration for card data and query optimization

### 🎮 Advanced MTG Features
- **Enhanced Card Search**: Format legality, power/toughness filtering, advanced search operators
- **Deck Validation System**: Format legality checking, card limit enforcement
- **Collection Management**: User card ownership tracking, wishlist functionality
- **Deck Analytics**: Mana curve analysis, card type distribution

---

## DETAILED IMPLEMENTATION HISTORY 📚
*Complete log of trials, tribulations, and breakthroughs*

### 🏗️ Foundation & Architecture (Early Development)
- **Database Foundation**: Established 5 core tables (User, Card, ScryfallCard, Deck, DeckCard) with proper foreign key relationships
- **Connection Architecture**: Started with Diesel r2d2, later migrated to SQLx native pooling for better performance
- **Hexagonal Architecture**: Implemented complete ports/adapters pattern with clean domain separation across all domains
- **Multi-Domain Design**: Separated Auth, User, Card, Deck, and Health domains with clear boundaries and responsibilities
- **UUID Migration**: Strategic transition from i32 to Uuid for enterprise-grade scalability
- **Configuration Management**: Production-ready AppState with dependency injection patterns

### 🔐 Authentication & Security Implementation
- **JWT Implementation**: Complete token generation/validation with custom configuration and extractors
- **Password Security**: Argon2 hashing with salt generation, common password detection, complexity validation
- **Auth Domain**: Registration, login, password changes, and comprehensive user lifecycle operations
- **Route Protection**: Type-safe authentication through handler parameters and custom middleware
- **Security Boundaries**: Information disclosure prevention and generic error responses to prevent enumeration attacks
- **Current Password Verification**: Secure password change implementation requiring current password authentication
- **AuthenticatedUser Security**: All operations use AuthenticatedUser ID to prevent privilege escalation

### 💾 Database Evolution & Challenges
- **Diesel to SQLx Migration**: Complete transition from Diesel ORM to raw SQL control with custom type integration
- **Advanced Query Patterns**: Dynamic QueryBuilder, bulk operations, transaction management
- **Constraint Handling**: PostgreSQL error code mapping (23505 unique, 23514 check), unique/check constraint violations
- **Custom SQLx Types**: Successfully implemented custom types to replace Json<T> wrappers, navigated Rust orphan rule
- **JSONB Mastery**: Complex nested types (Prices, Legalities, ImageUris, CardFace) with JSONB storage
- **Production Debugging**: Solid troubleshooting of JSONB array constraints, identified NOT NULL issues

### 🎮 Scryfall Integration & Data Pipeline Trials
- **API Research**: Comprehensive endpoint analysis and field mapping for 80+ field ScryfallCard model
- **Bulk Data Processing**: Production-scale processing of 35,400+ MTG cards with resilient error handling
- **Performance Optimization**: Achieved ~140 cards/second insertion rate, optimized batch sizes for PostgreSQL parameter limits
- **HTTP Client Debugging**: Resolved double URL encoding issue in reqwest RequestBuilder.query()
- **Custom Serde Implementation**: Flexible type handling for inconsistent API data (attraction_lights integer vs string)
- **Advanced Macro Development**: Created production `bind_scryfall_card_fields!` macro for 80+ field operations
- **PostgreSQL Parameter Mastery**: Expert understanding of 65,535 parameter limits, batch optimization strategies

### 🏛️ Domain Architecture & Service Layer Evolution
- **User Domain Implementation**: Complete CRUD operations with production-ready SQLx implementation and advanced query patterns
- **Deck Domain Implementation**: Full create, read, update, delete for both Deck and DeckCard entities with composite constraints
- **Card Domain Architecture**: Strategic read-only API design with background bulk operations vs user-facing search
- **Health Domain**: Simple but production-ready database connectivity checks and system health endpoints
- **Service Layer Patterns**: Service<R> and dual-generic Service<DR, CR> with dependency injection and proper trait bounds
- **Cross-Domain Orchestration**: DeckService coordinating between multiple repositories with HashMap join optimization
- **The Great Domain Refactor**: Comprehensive Operation/OperationError/InvalidOperation naming pattern implementation
- **Shared Models Architecture Decision**: Resolved frontend-backend code sharing through feature flags in zwipe library
- **Feature Flag Implementation**: Granular `#[cfg(feature = "zerver")]` gating for server-only code while preserving domain models for frontend
- **Frontend Validation Refinement**: Complete architectural refinement enabling frontend access to validation types, HTTP request/response structures, and domain models across ALL domains (auth, card, deck, user) while maintaining server-only security boundaries

### 📡 HTTP API Development & RESTful Design Challenges
- **Axum Handler Evolution**: From basic handlers to complex generic state types with trait bound resolution
- **RESTful API Design**: Transitioned from JSON request bodies to path/query parameters for cleaner API design
- **Error Architecture**: Sophisticated ApiError with domain error mapping and two-tier error handling
- **CORS Configuration**: Complete cross-origin setup supporting all required HTTP methods
- **HTTP Module Refactoring**: Eliminated ApiSuccess wrapper, standardized response patterns, removed dead code
- **Path Parameter Extraction**: Advanced tuple extraction patterns for multi-parameter routes
- **Nested Resource Routes**: Hierarchical URL structure reflecting proper parent-child relationships

### 🔧 Advanced Implementation Challenges & Breakthroughs
- **Async Trait Compilation**: Battled lifetime and trait constraints, transitioned from #[async_trait] to impl Future patterns
- **QueryBuilder Macro Development**: Advanced macro for 80+ field operations with manual comma separation logic
- **Composite Key Architecture**: Removed surrogate IDs in favor of natural deck_id + card_profile_id composite keys
- **Sync Metrics & Background Jobs**: Independent sync binary with time-based scheduling and metrics tracking
- **Production Data Pipeline**: End-to-end Scryfall API → Database → Retrieval pipeline with real MTG card data
- **Security Enhancement**: Auth domain expansion, user domain simplification, ownership validation patterns
- **Auth Domain Security Consolidation**: Successfully moved all user lifecycle operations to auth domain for centralized security
- **User Domain Cleanup**: Simplified user domain to read-only profile access, removed all mutation operations
- **Production Security Architecture**: Complete security boundary establishment with proper authentication and authorization

---

## BACKLOG - Future Development 📋

### 🧪 Testing & Quality Assurance
- **Handler Test Suites**: Comprehensive unit tests for auth, health, deck, and card handlers
- **JWT Middleware Tests**: Security boundary validation, error response testing
- **Integration Test Framework**: Full HTTP request/response testing infrastructure
- **Performance Testing**: Load testing, connection pool optimization
- **End-to-End Test Suite**: Complete user workflow validation

### 🚀 Production Features
- **Rate Limiting**: Request throttling and abuse prevention mechanisms
- **Caching Layer**: Redis integration for card data and query optimization
- **Monitoring & Logging**: Structured logging, metrics collection, health monitoring
- **Database Optimization**: Query performance analysis, indexing strategy
- **Image Handling**: Card image serving, caching, and mobile optimization

### 🎮 MTG-Specific Features
- **Advanced Card Search**: Format legality, power/toughness filtering, advanced search operators
- **Deck Validation**: Format legality checking, card limit enforcement
- **Collection Management**: User card ownership tracking, wishlist functionality
- **Deck Analytics**: Mana curve analysis, card type distribution
- **Import/Export**: Support for various deck formats (MTGA, MTGO, etc.)

### 📱 Mobile Application Features
- **Offline Support**: PWA capabilities for offline deck management
- **Advanced Filtering**: Complex search queries, saved filters
- **Social Features**: Deck sharing, public deck browser, user profiles
- **Real-time Updates**: WebSocket integration for live deck collaboration

---

## Architectural Decision Guidelines

### ✅ Established Patterns
- **Service Layer**: Service<R> and Service<DR, CR> for single and cross-domain operations
- **Error Handling**: Two-tier strategy (user-facing vs internal) with comprehensive domain error mapping
- **Security**: AuthenticatedUser-based operations, information disclosure prevention
- **Database**: Composite keys where natural, SQLx with custom types, transaction consistency
- **HTTP**: RESTful design with path parameters, nested resources, proper status codes

### ⚠️ Key Principles
- **Services orchestrate, repositories persist** - Keep domain boundaries clean through composition
- **Domain-first security** - Validation at domain layer, not adapter layer
- **Defensive programming** - TryFrom at all boundaries, trust no external data
- **Strategic simplification** - Choose maintainable solutions over theoretical approaches

---

## Development Context for AI Assistants

### 🎯 Current Session Focus
- **Auth Domain Security**: Centralizing user lifecycle operations
- **User Domain Cleanup**: Simplifying to read-only operations
- **Production Hardening**: Comprehensive security and performance optimization

### 📚 Learning Context
- **Architecture Understanding**: Solid grasp of hexagonal patterns and service layer design
- **Implementation Confidence**: Strong SQLx, HTTP, and security implementation skills
- **Current Edge**: Advanced type systems, production deployment, monitoring

### 🛠️ Development Commands
```bash
# Start development server
cargo run --bin server

# Test current endpoints
curl http://localhost:3000/health/database
curl -H "Authorization: Bearer <token>" http://localhost:3000/api/user

# Run sync job
cargo run --bin sync
``` 