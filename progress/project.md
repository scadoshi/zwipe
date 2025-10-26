---
description: Tracks project development status and provides development context for AI assistants to understand current progress and guide next steps appropriately
alwaysApply: true
---

## Update Instructions for AI Assistants

**Status Categories**: COMPLETE = Production-ready implementation. IN PROGRESS = Currently working on. NEXT = Planned immediate priorities. BACKLOG = Future planned work.

**When to Update**: After major feature completions, architectural decisions, or priority changes. Move items between categories as development progresses.

**Development Strategy**: Focus on completing current priorities before starting new work. Maintain clean architecture and comprehensive error handling throughout.

---

**Last Updated**: Completed full deck profile CRUD (Create/View/Edit/Delete) with conditional updates and comprehensive error handling.

**Current Focus**: Build confirmation dialog for deck deletion. Design card addition flow for adding cards to decks with quantity management.

**Recent Achievement**: Built complete EditDeckProfile screen with pre-populated form fields, debounced commander search, copy-max selection, and change tracking. Implemented conditional update requests sending only modified fields. Fixed critical ownership validation bug (missing negation) affecting 5 operations. Separated view and edit concerns with dedicated ViewDeckProfile and EditDeckProfile screens. Added delete deck functionality. Fixed Scryfall ID bug sending card_profile.id instead of scryfall_data.id. Implemented comprehensive error handling for load errors, submission errors, and delete errors. Updated UpdateDeckProfile domain model to support commander_id and copy_max fields with proper validation.

**Current Success**: Complete deck profile CRUD flow from creation through deletion. Proper view/edit separation. Conditional updates with change tracking. Commander search integration. CopyMax domain modeling. Comprehensive error handling at all layers. Working delete functionality. Fixed CSS overflow issues with commander images.

### 🎯 Currently Working On (Top 5)
1. **Delete Confirmation Dialog** - Build "are you sure?" confirmation before deck deletion
2. **Deck Card Addition Flow** - UI/UX for browsing cards and adding them to decks with quantities
3. **Card Browse Screen** - Swipeable card-by-card navigation through search results
4. **Deck Card Display** - Visual representation of cards in decks with quantities
5. **Frontend Deck Analytics** - Calculate mana curve, color distribution from card data

### 🤔 Next Immediate Priorities (Top 5)
1. **Swipeable Card Browsing** - Implement card-by-card swipe navigation through search results
2. **Frontend Deck Analytics** - Calculate mana curve, color distribution, type breakdown from card data
3. **Card Detail Screen** - Full card view with image and all attributes
4. **Deck Validation** - Client-side validation for deck legality and card limits
5. **Deck Sharing** - Export/import deck lists in standard formats

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
- **Dioxus Component Validation**: Secure login with generic error messaging and smart registration with real-time validation
- **UX-Focused Error Handling**: Security-first login validation vs user-friendly registration feedback patterns
- **Swipe Detection System**: Complete touch and mouse event handling with Delta struct for distance/velocity tracking
- **Direction Resolution**: Threshold-based detection using 50px distance OR 3.0 px/ms velocity with allowed-direction filtering
- **Cross-Platform Input**: OnTouch and OnMouse traits with proper coordinate system handling and state management
- **Multi-Screen Navigation**: Position-based screen tracking with BasicPoint coordinates enabling vertical/horizontal swipe navigation
- **Swipe-to-Submit Pattern**: Detection of disallowed swipe directions to trigger form submissions without buttons
- **Always-Render Architecture**: All auth screens render simultaneously with CSS calc transforms controlling visibility
- **Minimal UI Approach**: Arrow-in-title hints and simplified CSS removing unnecessary flexbox complexity
- **Frontend-Backend HTTP Integration**: Complete authentication flow with reqwest POST requests to /api/auth endpoints
- **Async Dioxus Patterns**: spawn() for calling async HTTP functions from sync event handlers
- **HTTP Error Handling**: Custom error types mapping status codes (401/422/500) to user-facing messages
- **Response Deserialization**: JSON to AuthenticateUserSuccess with custom Deserialize maintaining newtype validation
- **Signal Async Patterns**: Proper cloning of Signals and client instances for async block move semantics
- **Loading State UX**: Spinning card animation during HTTP requests with automatic error clearing on successful retry
- **Route Function Architecture**: Backend route paths exported as functions for frontend import (single source of truth)
- **Swipe Abstraction**: Consolidated onswipestart/move/end methods for identical touch/mouse behavior
- **Environment Configuration**: Separate .env files with OnceLock caching for efficient config loading
- **WSL2 Display Resolution**: Resolved GUI rendering issues in development environment
- **Session Domain Modeling**: Complete session.rs module with Session struct (user + access_token + refresh_token + both expirations)
- **Session Request Types**: CreateSession, RefreshSession, RevokeSessions with comprehensive error handling
- **Token Terminology Refactoring**: Renamed Jwt → AccessToken throughout codebase while preserving JwtSecret for technical accuracy
- **RefreshToken Architecture**: Self-contained struct with value + expires_at fields, 14-day lifespan constant, hex-encoded 32-byte random generation
- **Sha256Hash Trait**: Flexible trait implementation for both RefreshToken and String types enabling secure token storage
- **Authentication Flow Diagram**: Comprehensive visual mapping of frontend/backend flows including token validation, refresh, and 401 handling
- **Session Port Definitions**: AuthService and AuthRepository traits updated with session management operations
- **Session Repository Implementation**: Complete SQLx outbound layer with create_refresh_token, use_refresh_token, revoke_sessions, enforce_session_maximum, delete_expired_tokens
- **Token Rotation Pattern**: use_refresh_token deletes old token and creates new one atomically for security
- **Session Error System**: Comprehensive error variants (NotFound, Expired, Revoked, Forbidden) with proper SQLx error mapping
- **Session Maximum Enforcement**: SQL window functions maintaining 5 token limit per user with automatic cleanup of oldest tokens
- **Token Validation Logic**: Expiration checking, ownership verification, revocation status in use_refresh_token method
- **Expired Token Cleanup**: delete_expired_tokens method for scheduled cleanup job in sync binary (removes tokens where expires_at < NOW())
- **Session Service Layer**: Complete implementation of create_session, refresh_session, revoke_sessions with cross-domain orchestration
- **Atomic Registration**: create_user_and_refresh_token wraps user + token creation in single transaction preventing orphaned accounts
- **Transaction Helper Pattern**: create_refresh_token_with_tx and enforce_refresh_token_max_with_tx reused across repository methods
- **Dual-Generic AuthService**: AuthService<AR: AuthRepository, UR: UserRepository> pattern enabling cross-domain data composition
- **Service Atomicity Audit**: Confirmed all service layers properly delegate atomic operations to repositories
- **Request Type Simplification**: Removed unnecessary wrappers (CreateSession/RevokeSessions use Uuid directly)
- **AccessToken API Update**: generate() method now accepts User struct for cleaner service layer code
- **Middleware AccessToken Integration**: Updated AuthenticatedUser extractor to construct Jwt from bearer token string and validate
- **Jwt Validation Refactor**: Moved validate() method from AccessToken to Jwt type for cleaner separation of concerns
- **Username in AuthenticatedUser**: Added username field to middleware extractor for consistent user identification across handlers
- **Session Response Migration**: Register and login handlers now return Session struct (user + access_token + refresh_token)
- **Refresh Endpoint Implementation**: Complete /api/auth/refresh POST endpoint with HttpRefreshSession types and route function
- **Logout Endpoint Implementation**: Complete /api/user/logout POST endpoint with revoke_sessions handler, AuthenticatedUser middleware, proper RESTful design (POST not GET for side effects)
- **Enhanced Session Error Logging**: RefreshSessionError variants (NotFound, Expired, Revoked, Forbidden) now include user_id for security audit trails
- **Exhaustive Error Mapping**: Removed all catch-all patterns from ApiError From impls across auth, user, card, deck, and deck_card handlers ensuring explicit handling of every error variant
- **Scheduled Token Cleanup Job**: CheckSessions trait in zync binary with weekly cleanup, memory-tracked timestamps using map_or() pattern, WasAgo trait for time checks
- **Frontend Compile-Time Configuration**: build.rs with cargo:rustc-env directives passing BACKEND_URL to compiler, env!() macro for compile-time environment variable access (required for desktop/iOS apps)
- **Background Job Architecture**: Unified zync binary loop handling both card sync (CheckCards) and session cleanup (CheckSessions) with hourly execution and appropriate time thresholds
- **Modular Architecture Refactoring**: Complete restructuring of auth, user, deck, and card domains into per-operation files with consistent error/models/helpers pattern in SQLx layer
- **Ownership Validation System**: OwnsDeck trait on Uuid type providing ownership checks across create_deck_card, get_deck, update_deck_profile, update_deck_card, and delete_deck operations
- **Direct Domain Serialization**: Custom Serialize implementations on domain types (DeckCard, Quantity, DeckProfile) eliminating unnecessary HTTP wrapper layer
- **Card Domain Modularization**: Split 900+ line card.rs into modular structure - extracted error mappings (35 lines), insertion helpers (198 lines), scryfall field binding (324 lines)
- **Frontend Logging Infrastructure**: Integrated tracing_subscriber in Dioxus app with configurable log levels via RUST_LOG environment variable
- **Infallible Config Pattern**: Changed frontend Config to panic on invalid environment variables rather than returning Result (config is deployment requirement)
- **Session Persistence Trait**: PersistentSession trait on Session with keyring crate for iOS Keychain/Android KeyStore (requires entitlements for production)
- **Dioxus Context API**: Global session state with use_context_provider in App root, use_context in components eliminating prop drilling
- **Context-Based State Management**: Session and AuthClient provided via context, props reserved for parent-child relationships (swipe_state pattern)
- **Route Optimization**: MainHome at root redirecting to /auth when no session (optimizes common case of returning authenticated users)
- **Development Mock Utilities**: Spoof trait for generating mock Session data enabling rapid frontend development without authentication dependencies
- **Screen Structure Architecture**: Profile, MainHome, and Decks screens with vertical swipe navigation integrated
- **Session Management System**: Complete ActiveSession wrapper with GetActiveSession trait for token validation/refresh
- **Deck Profiles API**: Backend endpoint for fetching user's deck profiles with proper authentication
- **HTTP Client Architecture**: Session-aware request patterns with automatic token refresh handling
- **Session Domain Modularization**: Split session operations into separate files (create_session, refresh_session, revoke_sessions, etc.)
- **ActiveSession Trait**: Frontend session validation with get_active_session (fallible) and infallible_get_active_session (collapses errors to Option)
- **Main Screen Scaffolding**: Profile, MainHome, and Decks screens with session validation on mount
- **Session Checking Pattern**: spawn() async blocks that validate session and update signal, triggering reactive re-render
- **Deck Loading Implementation**: Complete use_resource pattern with three-state rendering (None/Ok/Err), honest error handling (SessionExpired vs empty results)
- **Resource Mental Model**: Clarified use_resource for "fetch and display" vs Signal for "already have, just validate" patterns
- **Async Closure Pattern**: Async closures returning Futures (`move || async move {}`) for reusable async operations that can be .await'ed
- **Frontend Hexagonal Refactoring**: Complete restructuring into inbound/ui/, outbound/client/, domain/ layers matching backend architecture
- **GlobalSignal Migration**: Switched from Context API to static GlobalSignal for session and auth client (Signal::global pattern)
- **Swipe Modularization**: Split 378-line monolithic swipe.rs into 6 focused modules for better maintainability
- **Swipeable Component Foundation**: Reusable wrapper with children prop and Vec<Direction> parameter, proper clone pattern for closure ownership
- **Separate Screen Hierarchies**: Established auth/ and app/ screen directories with home orchestrator files
- **Swipeable Integration**: All auth screens using shared SwipeState signal passed from home orchestrator
- **Axis Locking**: traversing_axis (X/Y) prevents diagonal swiping, set on first movement based on SwipeConfig allowed directions
- **Dynamic Screen Positioning**: CSS transforms combining xpx/ypx (finger delta) + xvw/yvh (screen_displacement * gap constants)
- **Smart Screen Displacement**: update_position() only called when direction in navigation_swipes, preventing displacement from submission swipes
- **Smooth Animation System**: is_swiping flag sets return_animation_seconds to 0.0 during active swipe, non-zero after release
- **Swipe-to-Submit Detection**: use_effect pattern watching latest_swipe signal to trigger form submissions without buttons
- **Gesture Separation Architecture**: Horizontal swipes (left/right) for submission, vertical swipes (up/down) for navigation to avoid conflicts
- **Extended Axis Locking**: Axis locking includes submission_swipe directions alongside navigation_swipes for comprehensive gesture control
- **Complete Swipeable Integration**: All auth (Login, Register, Home) and app (Profile, MainHome, Decks) screens using unified Swipeable component
- **SwipeConfig Refinement**: Removed constructor method, using struct literal initialization for clarity
- **Real Session Flow**: Switched from spoofed sessions to Session::infallible_load() with actual authentication
- **HomeGuard Component**: Route guard using use_memo to conditionally render AppHome/AuthHome without navigation effects
- **use_memo Pattern**: Derived state that only updates when session.is_some() boolean changes, preventing re-renders on session content updates
- **Single Route Architecture**: Replaced multi-route structure with single HomeGuard route for cleaner conditional rendering
- **Reactivity Loop Debugging**: Diagnosed and fixed infinite HTTP connection spawning (use_effect spawning infinite background loops)
- **use_future Pattern**: Replaced use_effect with use_future for background session validation (runs once, doesn't track dependencies)
- **Conditional Signal Updates**: Pattern of `if new_value != old_value { signal.set(new_value) }` to prevent infinite resource refetches
- **Centralized Session Refresh**: Moved background session validation to home.rs, removed duplicate logic from profile.rs and decks.rs
- **Session Refresh in Resources**: Pattern for use_resource to check token expiration, refresh if needed, and update signal only when changed
- **Fire-and-Forget spawn()**: Direct spawn() in component body for one-time background tasks (home.rs session refresh loop)
- **ScreenOffset Type System**: Flexible Point2D<i32> coordinate system for screen positioning replacing single-direction Option<Direction>
- **Multi-Dimensional Screen Positioning**: Screens can now be positioned at arbitrary x/y coordinates (up twice, diagonal positions, etc)
- **Chaining Screen Offset Methods**: Helper methods (up_again, down_again) allow progressive screen offset calculations
- **Position-Aware Form Submission**: Form submission guards check screen_offset to prevent accidental submissions from wrong screens
- **Simplified Auth UI**: Removed swipe-to-submit from auth screens, using traditional button-based forms for MVP acceleration
- **SessionProvider Architecture**: Centralized session context provider wrapping Router for global session/auth_client access
- **Guard Route Pattern**: Root route component conditionally redirecting to /home or /login based on session presence
- **Profile Management Screens**: Complete change username/email/password forms with minimal lowercase UI aesthetic
- **Logout Implementation**: Frontend session clearing with Persist trait delete(), navigation to login, TODO for backend revocation
- **Deck List Screen**: Resource-based deck fetching with proper three-state rendering (loading/success/error)
- **Resource Lifetime Handling**: `.value().with()` pattern extracting owned data to avoid temporary value borrowing errors
- **Clickable Deck Items**: Deck list items with onclick handlers and hover styling, TODO for navigation to detail screens
- **Minimal UI Consistency**: Lowercase text, centered layouts, clean spacing across auth and app screens
- **Profile Change Operations**: Complete HTTP client methods for change_username, change_email, change_password with bearer token authentication
- **Authenticated Request Pattern**: Session validation, token refresh, bearer header injection for all authenticated endpoints
- **Success Message System**: Random success messages on successful operations with get_random_success_message() utility
- **Logout Implementation**: Full backend integration with POST /api/auth/logout, server-side refresh token revocation, local session clearing
- **Universal Swipeable UI**: All screens (login, profile changes, deck list) wrapped in Swipeable component for consistent moveable interactions
- **Form Validation Refinement**: Proper error display timing (after first submit), separate validation for each field, submission_error clearing on success
- **Resource Pattern Mastery**: Match on Resource using `.value().with(|result| match result {...})` to avoid temporary value lifetime errors
- **Change Password Special Handling**: Current password not validated (legacy password policy compatibility), only new password validated
- **Deck Creation UI**: CreateDeck component with searchable commander field, singleton toggle, and form inputs for deck name
- **Searchable Commander Field**: Debounced card search (300ms) with HttpSearchCards::by_name() helper, shows top 5 results
- **Deck Creation Save**: Complete save functionality with session upkeep, error handling, loading states, and navigation to deck list
- **Card Search Refactoring**: Reversed search flow to query ScryfallData first, then fetch CardProfiles by scryfall_data_id for correct results
- **Type Naming Clarity**: Renamed GetCardProfiles → CardProfileIds, GetCards → ScryfallDataIds for better clarity
- **Bidirectional Sleeve Traits**: Created SleeveScryfallData and SleeveCardProfile traits for flexible data combination
- **UUID Database Handling**: Refactored DatabaseUser and DatabaseDeckProfile to return Uuid directly from PostgreSQL, eliminated string conversion errors
- **Card Field Access**: Made Card struct fields public for frontend consumption
- **Deck Constraint Error Fix**: Corrected unique constraint violation detection (is_check_constraint_violation → is_unique_constraint_violation)
- **Signal Simplification**: Cleaned up signal usage replacing .read() with signal() syntax throughout frontend (17+ changes)
- **HTTP Client Error Refactoring**: Eliminated 215 lines by creating centralized ApiError enum replacing LoginError, RegisterError, ChangeUsernameError, ChangeEmailError, ChangePasswordError, LogoutError, RefreshError, CreateDeckError, DeleteDeckError, GetDeckProfilesError
- **From Trait Error Conversion**: Implemented `From<(StatusCode, String)> for ApiError` enabling automatic `.into()` conversions throughout client layer
- **Request Pattern Standardization**: All POST/PUT requests use `.json()` method (auto-serialization + Content-Type), all authenticated requests use `.bearer_auth()` helper
- **Client Method Consistency**: All 12+ client methods follow identical pattern: `Result<T, ApiError>` returns, success case + wildcard match with `.into()`, clean error flow
- **Get Deck Client**: Built complete get_deck client method from scratch with proper authentication and error handling
- **Update Deck Profile Client**: Built complete update_deck_profile client method with HttpUpdateDeckProfileBody struct and proper validation
- **Backend Serialization Updates**: Added Deserialize to Deck domain type, Serialize to HttpUpdateDeckProfileBody for frontend/backend sharing
- **Complete HTTP Client Suite**: Built all 19 client methods covering every backend endpoint across auth, user, deck, deck_card, and card domains
- **Unified ApiError Architecture**: Moved ApiError enum to shared library (zerver/src/lib/inbound/http.rs), eliminated duplicate frontend error.rs file
- **Frontend/Backend Error Sharing**: Single ApiError type used by both frontend and backend with Network variant for client-side errors
- **Reqwest Version Alignment**: Synchronized reqwest versions between frontend (0.12) and backend (0.11 → 0.12) for clean From trait implementations
- **Client Method Organization**: Refactored auth client methods - moved user profile operations (change_username, change_email, change_password, delete_user, get_user) from auth/ to user/ folder
- **Delete User Security Enhancement**: Added password confirmation requirement to delete_user operation with HttpDeleteUser request type
- **HttpSearchCards Serialization**: Added `skip_serializing_if = "Option::is_none"` to all optional fields for clean query parameter URLs
- **Card Client Methods**: Implemented get_card (by UUID) and search_cards (with complex query parameters) following established patterns
- **Deck Card Client Methods**: Implemented create_deck_card, update_deck_card, delete_deck_card for full deck composition management
- **HTTP Verb Consistency**: Audited and corrected all client methods to use proper REST verbs (GET for reads, POST for creates, PUT for updates, DELETE for deletes)
- **Logo System Refactoring**: Created modular logo system with Zwipe, Zerver, Zervice, Zwiper variants using const + struct pattern
- **Code Reduction Victory**: Net reduction of 263 lines (711 deleted, 448 added) through architectural improvements and elimination of duplicate code
- **CopyMax Domain Type**: Replaced is_singleton boolean with CopyMax newtype validating 1 (singleton) or 4 (standard) for MTG deck rules
- **GetDeckProfile Separation**: Split GetDeck into GetDeckProfile (profile-only) and GetDeck (full deck with cards) for cleaner API design
- **Backend CopyMax Integration**: Updated domain models, ports, services, handlers, and repository to use CopyMax throughout deck operations
- **Frontend CopyMax UI**: Three-option selection (standard/singleton/none) with centered layout and visual feedback
- **View/Edit Screen Separation**: Renamed GetDeck → ViewDeckProfile (read-only view), UpdateDeck → EditDeckProfile (editable form) for clear separation
- **Commander Image Display**: Scryfall image_uris integration showing large card images with text fallback for missing images
- **Image Fallback Pattern**: Qualified match on `ImageUris { large: Some(url), .. }` providing graceful degradation to text display
- **CSS Commander Styling**: Responsive card images (max-height: 40vh) with rounded corners, overflow-y handling for tall content
- **Ownership Validation Bug Fix**: Fixed critical negation error in owns_deck checks affecting 5 operations (get_deck_card, update_deck_profile, update_deck_card, delete_deck, delete_deck_card)
- **EditDeckProfile Implementation**: Complete 362-line edit screen with pre-populated fields, debounced commander search, copy-max selection, change tracking
- **Conditional Update Pattern**: Only send changed fields to backend by tracking original vs current values, reducing unnecessary database writes
- **Comprehensive Error Handling**: Separate load_error, submission_error, and delete_error signals for granular error display
- **UpdateDeckProfile Domain Refactor**: Added commander_id and copy_max fields with Option<Option<T>> pattern for distinguishing "no update" vs "set to None"
- **Dynamic SQL Updates**: QueryBuilder conditional push for name, commander_id, and copy_max fields based on request contents
- **Scryfall ID Bug Fix**: Corrected card reference from scryfall_data.id to card_profile.id (database UUID) for proper foreign key relationships
- **Delete Deck Functionality**: Implemented delete_deck button with async error handling and navigation to deck list on success
- **ZwipeClient Architecture**: Unified client consolidating AuthClient operations under single trait-based interface
- **Client Method Cleanup**: Removed deprecated auth client methods, consolidated user operations under unified client pattern

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
- **Session Architecture Design**: Comprehensive domain modeling for rotating refresh token system with flow diagrams
- **Token Strategy Decision**: Access tokens (JWT, 24hr) vs Refresh tokens (opaque 32-byte random, hashed with SHA-256)
- **Multi-Device Session Support**: Architecture allowing up to 5 concurrent refresh tokens per user for cross-device auth
- **RefreshToken Self-Containment**: Refactored to struct with value + expires_at fields for tight coupling and consistency
- **Session Repository Implementation**: Complete SQLx implementation with token rotation, validation, and enforcement patterns
- **Sha256Hash Trait Pattern**: Flexible trait for hashing both RefreshToken and String types, enabling secure database storage
- **Token Rotation Security**: Delete-then-create pattern in use_refresh_token ensures old tokens invalidated atomically
- **Window Function Enforcement**: SQL-based session maximum using ROW_NUMBER() OVER(PARTITION BY user_id) for automatic cleanup
- **Expired Token Cleanup Job**: delete_expired_tokens repository method for scheduled cleanup via sync binary
- **Transaction Helper Pattern Discovery**: Built reusable create_refresh_token_with_tx and enforce_refresh_token_max_with_tx for atomic operations
- **Atomicity Bug Fix**: Discovered and fixed critical race condition in register_user where user creation could succeed but token creation fail
- **Cross-Repository Orchestration**: Implemented AuthService<AR, UR> dual-generic pattern for clean user data + token operation coordination
- **Service Layer Atomicity Audit**: Comprehensive review of all service implementations confirming proper separation of concerns
- **Tuple Return Pattern**: Using (User, RefreshToken) returns to maintain atomicity when operations span auth + user tables

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

### 🌐 Frontend HTTP Client Evolution & Refactoring
- **Initial Client Implementation**: Built AuthClient with operation-specific error types (LoginError, RegisterError, etc.) - verbose but functional
- **Error Type Proliferation**: Each operation had dedicated error enum with From impls, manual header construction, manual JSON serialization
- **Refactoring Revelation**: Realized backend returns standardized (StatusCode, String) tuples - frontend duplicating unnecessary mapping logic
- **Centralized Error Design**: Created single ApiError enum with variants (Unauthorized, Forbidden, NotFound, UnprocessableEntity, InternalServerError, Unknown)
- **From Trait Implementation**: Built `From<(StatusCode, String)> for ApiError` centralizing all status code → error variant mapping in one location
- **Request Pattern Standardization**: Discovered `.json()` convenience method (auto-serializes + sets Content-Type), replaced manual `.header()` + `.body(serde_json::to_string())`
- **Authentication Helper Discovery**: Standardized on `.bearer_auth()` replacing manual `format!("Bearer {}")` header construction
- **Massive Code Reduction**: Eliminated 215 lines (360 deleted, 145 added) - removed 10+ error type definitions with associated From impls
- **Client Method Unification**: Refactored login, register, logout, refresh, change_username, change_email, change_password, create_deck, delete_deck, get_deck_profiles - all follow identical pattern
- **New Methods from Scratch**: Built get_deck and update_deck_profile following established patterns - proved architecture is learnable and consistent
- **Call Site Simplification**: UI components now handle single ApiError type with `.to_string()` for display - no complex matching needed
- **Backend Alignment**: Added Serialize to HttpUpdateDeckProfileBody, Deserialize to Deck for frontend consumption
- **Production Readiness**: HTTP client layer now feels production-ready with minimal boilerplate and consistent error handling

### 🎨 Session: Complete Deck Profile CRUD Implementation
**Independent Achievements:**
- **EditDeckProfile Screen**: Built complete 362-line edit form with pre-population, debounced search, change tracking
- **Conditional Updates**: Implemented pattern tracking original vs current values, sending only changed fields
- **Critical Bug Fix**: Discovered and fixed ownership validation negation error affecting 5 repository operations
- **Error Handling**: Separate signals for load_error, submission_error, delete_error providing granular user feedback
- **Domain Model Extension**: Updated UpdateDeckProfile to support commander_id and copy_max with Option<Option<T>> pattern
- **Dynamic SQL**: Extended QueryBuilder with conditional push for name, commander_id, copy_max fields
- **ID Bug Resolution**: Debugged commander blanking issue, fixed by sending card_profile.id instead of scryfall_data.id
- **CSS Problem Solving**: Fixed button cutoff by adding max-height to commander images and overflow-y to swipeable

**Strengths:**
- **Debugging Persistence**: Traced commander issue through full stack (frontend → domain → SQLx) to identify root cause
- **Change Detection Logic**: Implemented clean pattern comparing current vs original state before building update requests
- **Error Granularity**: Recognized need for separate error states (load vs submission vs delete) for better UX
- **View/Edit Separation**: Understood value of separating read-only viewing from editable form concerns
- **Option Semantics**: Grasped Option<Option<T>> pattern for distinguishing "no change" from "set to None"

**Key Learnings:**
- **Pre-Population Pattern**: use_effect watching resources, extracting data, populating form signals with current values
- **Negation Bugs**: Missing `!` in ownership checks can silently invert security logic - critical to test authorization paths
- **Foreign Key IDs**: Must use database primary keys (card_profile.id), not external IDs (scryfall_data.id) for relationships
- **Conditional SQL Building**: QueryBuilder allows elegant dynamic queries without string concatenation vulnerabilities
- **CSS Overflow Management**: overflow-y + max-height constraints prevent content from pushing elements off-screen

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
- **Token Cleanup Job Integration**: Add delete_expired_tokens call to sync binary for scheduled cleanup (daily/weekly)

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

### 🚢 iOS/Android Deployment
- **iOS Keychain Entitlements**: Configure Dioxus.toml with bundle identifier and keychain-access-groups for persistent session storage
- **Android KeyStore Configuration**: Verify keyring crate configuration for Android secure storage
- **App Signing**: Set up iOS/Android code signing for device deployment
- **Store Submission**: Prepare assets and metadata for App Store/Play Store submission

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