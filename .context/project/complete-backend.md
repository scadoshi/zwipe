# Complete - Backend ✅

Production-ready backend implementations.

---

## 🏗️ Core Architecture & Infrastructure
- **Hexagonal Architecture**: Complete ports/adapters pattern with clean domain separation
- **Multi-Domain Design**: Separate Auth, User, Card, Deck, and Health domains with clear boundaries
- **Database Foundation**: PostgreSQL with SQLx, connection pooling, migrations, and constraint management
- **Configuration Management**: Production-ready AppState with dependency injection patterns
- **Error Architecture**: Two-tier error handling (user-facing vs internal) with comprehensive domain error mapping
- **Security Infrastructure**: JWT middleware, password hashing, authentication flow, and route protection

## 🔐 Authentication & Security System
- **Complete Auth Domain**: Registration, login, password changes, and user lifecycle operations
- **JWT Implementation**: Token generation/validation with custom extractors and middleware
- **Password Security**: Argon2 hashing, salt generation, common password detection, complexity validation
- **Route Protection**: Type-safe authentication through handler parameters and middleware
- **Security Boundaries**: Information disclosure prevention and generic error responses

## 💾 Database & Data Management
- **SQLx Integration**: Raw SQL control with compile-time query verification and custom type integration
- **Advanced Query Patterns**: Dynamic QueryBuilder, bulk operations, transaction management
- **Constraint Handling**: PostgreSQL error code mapping, unique/check constraint violations
- **Production Data Pipeline**: Scryfall API integration with 35,400+ card processing capability
- **Composite Key Architecture**: Natural primary keys eliminating surrogate IDs where appropriate

## 📡 HTTP API & RESTful Design
- **Complete CRUD APIs**: User, Auth, Card, Deck, and DeckCard endpoints with proper HTTP semantics
- **RESTful Patterns**: Nested resource routes, path parameter extraction, status code precision
- **Advanced Middleware**: Custom extractors, type-safe authentication, generic handler patterns
- **Error Mapping**: Domain errors to appropriate HTTP status codes with information disclosure prevention
- **CORS Configuration**: Complete cross-origin setup for web application integration

## 🎮 Domain-Specific Implementation

### Card Management
- **Scryfall Integration**: Complete integration with dual color identity modes (equals/contains)
- **Comprehensive Search**: CMC/power/toughness ranges, text search, type filtering
- **Bulk Data Processing**: 35,400+ cards with efficient batch operations
- **Get Sets Endpoint**: Returns distinct set names for filtering UI
- **Get Card Types**: Extracts ~670 distinct subtypes for type filtering
- **Rarity Domain Type**: Complete Rarity enum with custom SQLx traits and flexible Scryfall parsing
- **Computed Card Fields**: `is_valid_commander` and `is_token` boolean columns calculated during upsert
- **Commander Validation**: 3-rule system (legendary creatures, legendary vehicles/spacecraft with P/T, oracle text "can be your commander")
- **Token Detection**: Authoritative `layout == "token"` check from Scryfall API
- **Optional Field Filtering**: CardFilter support for both flags enabling optional inclusion/exclusion

### Deck Management
- **Full CRUD Operations**: Complete deck lifecycle with card composition
- **Cross-Domain Orchestration**: Coordinating between multiple repositories
- **Nested Resource API**: RESTful routes for deck and deck card operations
- **CopyMax Domain Type**: Replaced is_singleton with validated 1 (singleton) or 4 (standard)
- **Commander Support**: Optional commander card with image display
- **Ownership Validation**: Complete OwnsDeck trait preventing unauthorized access

### Auth Domain Security
- **User Lifecycle Operations**: Complete username/email updates, password changes, account deletion
- **Session Management**: Create, refresh, and revoke sessions with token rotation
- **Token Architecture**: JWT access tokens (24hr) + rotating refresh tokens (14-day, SHA-256 hashed)
- **Multi-Device Support**: Up to 5 concurrent refresh tokens per user
- **Scheduled Cleanup**: Expired token cleanup in zync binary with weekly execution

### User Domain
- **Read-Only Profile**: All mutations moved to auth domain for proper security boundaries
- **Profile Data Access**: Get user information with authentication required

### Health Monitoring
- **Database Connectivity**: Health checks and system monitoring endpoints

## 🗄️ Database Evolution
- **PostgreSQL Migration**: Migrated scryfall_data schema from VARCHAR to TEXT for best practices
- **UPSERT Implementation**: ON CONFLICT DO UPDATE for both scryfall_data and card_profiles
- **Trigger Functions**: Auto-update card_profiles.updated_at on UPDATE operations
- **Foreign Key Strategy**: ON DELETE RESTRICT preserving deck integrity
- **Delta Sync**: PartialEq-based change detection with intelligent UPSERT
- **Empty Delta Guards**: Prevents invalid SQL generation from empty slices

## 🌐 Advanced HTTP & Middleware
- **Custom Middleware**: AuthenticatedUser extractor with FromRequestParts trait
- **Type-Safe Authentication**: Compile-time route protection
- **Generic Handler Patterns**: Complex generic state types across all handlers
- **Bearer Token Extraction**: JWT validation and claims parsing
- **Exhaustive Error Mapping**: Explicit per-variant matching preventing missing cases

## 🔄 Session & Token Architecture
- **Session Domain**: Complete session.rs module with rotating refresh tokens
- **Token Rotation**: Delete-then-create pattern ensuring old tokens invalidated
- **Session Maximum Enforcement**: SQL window functions maintaining 5 token limit
- **Transaction Helpers**: Reusable atomic operation patterns
- **Atomic Registration**: Single transaction preventing orphaned accounts
- **Cross-Domain Orchestration**: AuthService coordinating UserRepository and AuthRepository
- **Enhanced Error Logging**: Security audit trails with user_id in error variants
- **Refresh Endpoint**: Complete /api/auth/refresh with token rotation
- **Logout Endpoint**: POST /api/auth/logout with server-side token revocation

## 🏛️ Service Architecture
- **Generic Service Patterns**: Service<R> and Service<DR, CR> implementations
- **Repository Abstraction**: Trait-based dependency injection
- **Cross-Domain Coordination**: DeckService and AuthService patterns
- **Dual-Generic Pattern**: AuthService<AR, UR> for cross-domain composition
- **Transaction Management**: Services orchestrate, repositories handle atomicity

## 📊 Production API Design
- **Nested Resource Routes**: Hierarchical URL patterns
- **Tuple Path Extraction**: Multi-parameter route handling
- **Composite Key Architecture**: Natural primary keys
- **Comprehensive Search**: Advanced filtering with PostgreSQL operators
- **RESTful Patterns**: Proper HTTP verbs and status codes

## 🔧 Advanced Implementation
- **Modular Domain Architecture**: Per-operation files with error/models/helpers patterns
- **Ownership Validation Patterns**: Trait-based security checks
- **Direct Domain Serialization**: Eliminates unnecessary HTTP wrappers
- **Dynamic SQL Building**: QueryBuilder with conditional field updates
- **Option<Option<T>> Pattern**: Distinguishes "no update" from "set to None"
- **Scryfall Field Binding**: Production macro for 80+ field operations
- **PostgreSQL Advanced Queries**: JSONB operators (@>, <@, ?|), array operations, regex validation
- **Order By Filtering**: Dynamic ORDER BY with NULL exclusion for nullable fields (prices, power, toughness)
- **Sleeve Order Preservation**: Fixed HashMap iteration to preserve DB sort order by iterating source data
- **Computed Field Validation**: Pure helper functions (is_valid_commander, is_token) called during upsert for consistent data

## 🗂️ Code Organization
- **Workspace Configuration**: Multi-package setup with centralized dependencies
- **Clippy Configuration**: 26 workspace-level lints with thresholds
- **Modular File Organization**: Domain-specific separation with clear responsibilities
- **Feature Flags**: Server-only code properly gated with #[cfg(feature = "zerver")]
