---
description: Tracks User's Rust/web development learning progress across confidence levels for AI teaching optimization
alwaysApply: true
---

## Update Instructions for AI Assistants

**Confidence Levels**: CONFIDENT = Could teach others without hesitation. DEVELOPING = Successfully implemented but still learning. LEARNING = Recently introduced, needs guidance. UNEXPLORED = Not yet encountered.

**When to Update**: After major implementations, quiz performance, or when user demonstrates clear confidence level changes. Move concepts between levels based on demonstrated understanding and teaching ability.

**Quiz Strategy**: Continue giving pop quizzes in chat to validate understanding and strengthen knowledge. Focus on recent implementations and concepts showing uncertainty.

---

## Current Learning Status

**Last Updated**: Successfully integrated Swipeable component across auth screens with axis locking and shared state architecture. Fixed multiple positioning and animation bugs.

**Next Learning Focus**: Implement swipe-to-submit detection using use_effect to watch latest_swipe signal, or extract custom onswipeend handlers in form screens to trigger submission logic.

**Recent Achievement**: Swipeable component now requires external Signal<SwipeState> (always passed from parent). Implemented axis locking with traversing_axis preventing diagonal swipes. Built dynamic CSS transform system combining finger deltas (xpx/ypx) with screen offsets (xvw/yvh). Fixed jittery swiping with is_swiping flag controlling animations. Debugged screen_displacement logic to only update for navigation_swipes, preventing unwanted offsets from submission swipes. All three auth screens move together with shared state.

### 🎯 Currently Working Towards (Top 5)
1. **Swipe-to-Submit Logic** - Watch latest_swipe signal with use_effect to trigger maybe_submit in Login/Register screens
2. **Alternative Submission Pattern** - Explore custom onswipeend handlers in form components if use_effect approach doesn't work
3. **App Screen Integration** - Roll out Swipeable to Profile/MainHome/Decks with proper shared state
4. **Session Management Consolidation** - Extract repeated check_session closure and use_effect validation loop into reusable pattern
5. **Profile/Home Screen Content** - Build actual functionality beyond "under construction" placeholders

### 🤔 Current Uncertainties (Top 5)
1. **Submission Detection Pattern** - Should use_effect watch latest_swipe, or extract onswipeend into form components?
2. **Signal Reactivity in use_effect** - Will reading swipe_state.read().latest_swipe trigger re-runs, or need different approach?
3. **Event Handler Extraction** - If need custom handlers, how to layer them on top of Swipeable's built-in handlers?
4. **Session Hook Pattern** - Can extract use_context + validation loop into reusable pattern without custom hooks?
5. **Component Abstraction Balance** - Finding right level between reusability and flexibility for different screen needs

---

## CONFIDENT - Could Teach Others 🎓

### 🦀 Core Rust Fundamentals
- **Basic Syntax**: Ownership, borrowing, basic pattern matching, error handling with Result
- **Type System**: Understanding of structs, enums, traits, basic generics
- **Memory Safety**: Conceptual understanding of Rust's ownership model
- **Module System**: Basic use of mod, pub, use statements for code organization
- **Cargo Workflow**: Creating projects, running tests, managing dependencies
- **Debugging**: Using println!, investigating compiler errors, reading documentation

### 🏗️ Hexagonal Architecture & Clean Design
- **Domain-First Design**: Business logic separation from external concerns, ports/adapters pattern
- **Dependency Inversion**: Core domain depends on abstractions, not concrete implementations
- **Clean Boundaries**: HTTP, database, external APIs properly separated from business logic
- **Architectural Decision Making**: When to break rules for convenience vs maintainability
- **Separation of Concerns**: Balancing responsibility with abstraction levels
- **YAGNI vs Future Planning**: Build for future but stay realistic about current needs
- **DRY Balance**: Code reuse without over-abstraction
- **Pragmatic API Design**: Route functions over constants when it preserves readability without adding ceremony
- **Domain Modeling Process**: Request types, error types, response types, and port definitions before implementation
- **Newtype Pattern**: Type-safe wrappers (RefreshToken, AccessToken) with validation and domain-specific methods
- **Modular File Organization**: Per-operation domain model files, error/models/helpers SQLx modules, separated HTTP handlers
- **Ownership Validation Patterns**: Trait-based ownership checking (OwnsDeck) preventing unauthorized resource access
- **Direct Domain Serialization**: Serialize domain types directly when HTTP shape matches, avoiding unnecessary wrapper boilerplate

### 🔒 Security & Authentication
- **JWT Security Flow**: Complete token generation/validation with proper configuration
- **Password Security**: Argon2 hashing, salts, rainbow table prevention, timing attack mitigation
- **Cryptographic Hashing Strategy**: SHA-256 for tokens (fast verification) vs Argon2 for passwords (slow, memory-hard)
- **Authentication Delays**: Why delays in auth are important for security
- **Information Disclosure Prevention**: Avoiding enumeration attacks through generic error responses
- **Security Spidey Senses**: When to step carefully with sensitive data and implementation details
- **Middleware Security**: Functional type-based authentication through authorization headers

### 🚨 Error Handling Architecture
- **Strategic Error Types**: When to use thiserror (decisive system responses) vs anyhow (console error stacks)
- **Error Flow Design**: Comprehensive error handling patterns throughout application layers
- **HTTP Status Mapping**: Business logic errors to appropriate status codes
- **Two-Tier Error Strategy**: User-facing vs internal error separation

### 🗄️ Database Design & Relationships
- **Relational Modeling**: Foreign keys, composite keys, constraints, and indices
- **Complex Joins**: Multi-table queries and relationship management
- **Database Constraints**: Strategic use for business rule enforcement
- **JSONB Confidence**: Comfortable storing JSON in PostgreSQL tables
- **Composite vs Surrogate Keys**: When to use natural vs artificial primary keys

### 📡 HTTP & RESTful API Design
- **Parameter Extraction**: Body, path, and query parameter handling
- **RESTful Patterns**: Proper HTTP verb usage, nested resource routes
- **Status Code Precision**: Correct HTTP status codes for different operations
- **Parameter Naming Consistency**: Aligned naming across request pipelines

### ⚙️ Basic Implementation Patterns
- **Environment Setup**: .env files, configuration management
- **macro_rules!**: Basic declarative macro creation and usage
- **Build Scripts**: build.rs with cargo directives (rustc-env, warning, rerun-if-changed) for compile-time configuration
- **Compile-Time Environment Variables**: env!() macro for baking config into binaries (required for desktop/mobile apps)
- **Infallible Config Pattern**: Using unwrap() in Config::from_env() when config is deployment requirement (better to crash than run with invalid config)
- **Frontend Logging**: tracing_subscriber::fmt() setup in main() with configurable log levels for development debugging

### 🎨 Dioxus Component Development & State Management
- **Component Architecture**: Function components with RSX macro for HTML-like syntax
- **State Management**: Signal types for reactive state with use_signal() patterns
- **Context API**: use_context_provider() at root, use_context() in components for global reactive state
- **Props vs Context**: Props for parent-child relationships, context for app-wide state (Session, AuthClient)
- **Development Utilities**: Spoof trait pattern for generating mock data enabling UI development without dependencies
- **Multi-Screen Navigation**: Vertical swipe navigation between Profile/Home/Decks with position-based transforms
- **Conditional Rendering**: Dynamic UI based on state with if expressions in RSX
- **Form Handling**: Input binding, event handlers (oninput, onsubmit), and form validation
- **Error Display Timing**: UX-focused validation that activates after first submit attempt
- **Security-Focused UI**: Generic error messaging to prevent information disclosure
- **Domain Type Integration**: Using shared backend types (Username, EmailAddress, Password) in frontend components
- **Navigation**: use_navigator() for programmatic routing between components
- **Touch/Mouse Events**: ontouchstart/move/end and onmousedown/move/up with ClientPoint coordinates
- **Event Data Access**: touches() vs touches_changed() for active vs ending touches
- **CSS Transform Integration**: Dynamic inline styles with signal-driven values for animations
- **Trait-Based Handlers**: Custom traits for Signal<State> to organize event handling logic
- **CSS Calc Transforms**: Combining pixel deltas with viewport units for responsive screen positioning
- **Animation Timing**: CSS transition-duration tied to swipe distance for natural-feeling motion
- **Inline Event Handlers**: Placing logic directly in ontouchend/onmouseup for submission detection
- **Async Event Patterns**: Using spawn() to call async functions from sync event handlers
- **Signal Cloning**: Understanding when to clone Signals vs values for move semantics in async blocks
- **HTTP Client Integration**: Building AuthClient with reqwest, handling network errors and response deserialization
- **Loading State Management**: is_loading Signal controlling spinner visibility during async operations
- **Conditional Error Display**: Showing errors only when present, clearing on successful retry
- **CSS Animation Integration**: Spinning card animation for visual feedback during HTTP requests
- **Route Function Pattern**: Exporting backend routes as functions for frontend import consistency
- **GlobalSignal Pattern**: Static global state with Signal::global() for session and auth client (alternative to Context API)
- **Dioxus Props Ownership**: Props must be owned values, can't pass references - use clone pattern for multiple closure captures

### 💾 SQLx Database Operations & Advanced Patterns
- **Connection Pooling**: Production-ready pool configuration with optimized settings
- **Error Handling**: Custom IsConstraintViolation trait with PostgreSQL error code mapping
- **Transaction Management**: Consistent transaction usage across all write operations
- **Advanced Query Building**: Dynamic QueryBuilder with range filtering, complex search parameters, bulk operations
- **Custom Type Integration**: Complex domain types with SQLx traits (Decode, Encode, Type)
- **Migration & Schema Management**: Forward-only migrations, database recreation workflows
- **Bulk Data Processing**: Production-scale processing (35,400+ cards) with resilient error handling
- **Constraint Management**: Advanced PostgreSQL constraint handling and violation detection
- **Modular Repository Architecture**: error/models/helpers module pattern for clean SQLx implementations
- **Ownership Validation Traits**: Custom traits on types (OwnsDeck for Uuid) enabling reusable security checks

### 🌐 Advanced HTTP & Middleware Patterns
- **Custom Middleware**: AuthenticatedUser extractor with FromRequestParts trait
- **Type-Safe Authentication**: Compile-time route protection through handler parameters
- **Generic Handler Patterns**: Complex generic state types across all handlers
- **Error Architecture**: Sophisticated ApiError with domain error mapping
- **Request/Response Conversion**: Clean TryFrom patterns for HTTP-to-domain conversion
- **Bearer Token Extraction**: JWT validation and claims parsing in middleware
- **Exhaustive Error Mapping**: Pattern of explicit per-variant matching in From<DomainError> for ApiError implementations, avoiding catch-all patterns to ensure compiler catches missing error cases when enums evolve

### 🔄 Async Programming & Trait Constraints
- **Async Function Design**: Building async functions with proper trait bounds
- **Send + Sync Constraints**: Understanding trait requirements for async service patterns
- **Generic Async Patterns**: Handler functions with generic state types and async operations
- **Future Handling**: Async trait implementations across hexagonal boundaries
- *Note: Manual thread synchronization (.lock(), message passing) not yet implemented*

### 📊 Production API Design
- **Nested Resource Routes**: Hierarchical /api/deck/{deck_id}/card/{card_profile_id} patterns
- **Tuple Path Extraction**: Path<(String, String)> for multi-parameter routes
- **Composite Key Architecture**: Natural primary keys eliminating surrogate IDs
- **Comprehensive Search System**: CMC/power/toughness ranges, dual color identity modes, input sanitization
- **PostgreSQL Advanced Queries**: Regex validation, array operators (@>, <@, &&), dynamic query building
- **RESTful Patterns**: Proper HTTP verb usage, status code precision, parameter naming consistency

---

## DEVELOPING - Active Implementation (Working But Learning) 🔧

### 🌐 Frontend-Backend HTTP Integration
- **HTTP Client Architecture**: AuthClient with reqwest for POST requests, proper header configuration
- **Error Type Design**: Custom RegisterUserError and AuthenticateUserError with status code mapping
- **Response Flow Understanding**: HTTP bytes → JSON deserialization → domain types
- **Status Code Handling**: Branching on 200/201/401/422/500 to map server responses to user-facing errors
- **Async Patterns in Dioxus**: spawn() for calling async functions from sync event handlers
- **Signal Move Semantics**: Cloning Signals and client instances for async block capture
- **Custom Deserialize Implementation**: Newtype validation through HTTP boundaries using manual deserialize impls
- **API Endpoint Construction**: URL path building with set_path() for RESTful routes
- **Loading State Implementation**: Spinner activation/deactivation around async operations
- **Error Recovery UX**: Automatic error clearing on successful retry
- **OnceLock Configuration**: One-time config loading cached across AuthClient instances
- *Note: Working end-to-end but token storage and global auth state not yet implemented*

### 🎮 Swipe-Based Navigation & Gestures
- **Position Tracking**: BasicPoint (i32 x/y) for screen coordinates independent of swipe detection
- **Multi-Screen Architecture**: Always-render pattern with CSS transforms controlling visibility
- **Swipe-to-Submit Pattern**: Detecting submission_swipe direction in SwipeConfig to trigger form submissions (implementation pending)
- **Transform Calculations**: CSS calc() combining xpx/ypx (finger delta) with xvw/yvh (screen_displacement offsets)
- **Axis Locking**: traversing_axis (X/Y) set on first movement, locks screen to horizontal or vertical based on SwipeConfig
- **Smart Displacement Updates**: update_position() only called for directions in navigation_swipes, not submission_swipe
- **Direction Resolution**: Separate tracking of detected swipe vs allowed navigation for dual-purpose gestures
- **Event Handler Closures**: Extracting validation logic into reusable closures callable from multiple handlers
- **Coordinate System**: Browser coordinates (positive Y down) with proper delta calculations
- **State Management**: Decoupled position updates from swipe detection for flexible interaction patterns
- **Abstraction Patterns**: Consolidated onswipestart/move/end for identical touch/mouse behavior
- **Modular Swipe Architecture**: Split into axis, config, direction, onmouse, ontouch, state, time_point modules (7 files)
- **Swipeable Component**: Reusable wrapper requiring external Signal<SwipeState> for shared state across screens
- **SwipeConfig Structure**: navigation_swipes (Vec), submission_swipe (Option), from_main_screen (Option) for positioning
- **Shared State Pattern**: Parent component creates Signal<SwipeState>, passes to all child Swipeable components
- **Smooth Animation System**: is_swiping flag controls return_animation_seconds (0.0 during swipe, non-zero after)
- **VH_GAP/VW_GAP Constants**: 75vh/75vw spacing between screens for natural swipe distances
- *Note: Navigation complete, submission detection next - need use_effect or custom handlers*

### 🏗️ Service Architecture & Dependency Injection
- **Generic Service Patterns**: Service<R> and Service<DR, CR> implementations across domains
- **Repository Abstraction**: Understanding why services depend on repository traits vs concrete types
- **Cross-Domain Orchestration**: DeckService and AuthService coordinating between multiple repositories
- **Dual-Generic Pattern**: AuthService<AR: AuthRepository, UR: UserRepository> for cross-domain data composition
- **Atomicity Understanding**: Services orchestrate business logic, repositories handle atomic database operations
- **Transaction Helper Patterns**: Helper functions taking PgTransaction for reusable atomic operations
- **Tuple Returns for Atomicity**: Using (User, RefreshToken) returns to maintain atomicity across operations
- **Service Layer Separation**: Services handle orchestration and business logic, never direct database transactions
- *Note: Solid understanding of orchestration patterns and when atomicity matters*

### 🎯 Feature Flag Architecture & Shared Models
- **Cargo Feature Flags**: Optional dependencies, feature-gated compilation, granular module control across entire library
- **Architectural Decision Making**: Evaluated multiple approaches for frontend-backend code sharing, chose pragmatic solution
- **Pragmatic Architecture**: Choosing workflow efficiency over theoretical purity when appropriate
- **Granular Feature Gating**: `#[cfg(feature = "zerver")]` patterns for fine-grained access control across ALL domains
- **Library Design**: Understanding when to separate concerns vs when to keep related code together
- **Frontend Validation Refinement**: Complete mastery of separating frontend-accessible validation types from server-only business logic
- **Handler Import Separation**: Sophisticated understanding of what frontend needs vs what should remain server-only
- **Compilation Boundary Management**: Expert-level control of frontend/backend compilation separation

### 🏗️ Advanced Architecture Patterns
- **Configuration Management**: Production-ready config loading at startup vs runtime env reads
- **Performance Optimization**: Resolved repeated file system access inefficiencies
- **OnceLock Pattern**: Thread-safe one-time initialization for expensive config operations
- **Separate Environment Files**: Frontend/backend .env separation for deployment flexibility
- **Environment Variable Strategy**: Backend uses full config, frontend only gets necessary values

### 🧪 Testing & Validation
- **Test Organization**: Clean categorization of test functions by concern
- **Edge Case Testing**: Validation of error conditions and boundary cases
- **Environment Testing**: Understanding environment coupling vs testable design
- **Newtype Testing**: Testing validation at correct levels

### 🌐 External API Integration & Data Processing
- **HTTP Client Setup**: reqwest crate integration with proper headers and error handling
- **JSON Processing**: serde_json parsing and complex JSON deserialization
- **Scryfall API Understanding**: Complete MTG card data structure and API patterns
- **Custom Serde Deserializers**: Flexible type handling for inconsistent API data

### 🔮 Advanced Type Systems
- **Opaque vs Concrete Types**: Understanding `impl UserService` vs `US: UserService` trade-offs
- **Type Inference Patterns**: When type inference works vs explicit generic parameters needed
- **Generic Constraints**: Complex trait bounds and generic programming patterns

---

## LEARNING - Recently Introduced, Needs Guidance 📚

### 🎨 Dioxus Async Patterns & Reactivity
- **spawn() Limitations**: spawn() runs async tasks but doesn't return values - meant for side effects only
- **use_resource Pattern**: Correct tool for async data fetching that returns values (None for loading, Some(Result) for ready)
- **Resource vs Signal Mental Model**: Resources = "fetch this data and render it", Signals = "already have value, validate/update as side effect"
- **use_resource Structure**: Handles `Option<Result<T, E>>` - None while loading, Some(Ok(data)) on success, Some(Err(e)) on failure
- **Resource Rendering**: Match on three states: None (loading spinner), Some(Ok(data)) (render content), Some(Err(e)) (error message)
- **Async Closure Pattern**: `move || async move { }` for creating Futures that can be `.await`ed when called
- **use_future Loops**: Background tasks with infinite loops for periodic operations (session refresh every N seconds)
- **Signal + Send Issues**: mut Signal parameters can't cross async boundaries due to RefCell (not Sync) - pass Signal by value instead
- **Context in Async**: use_context() works in spawn() blocks since they capture component context
- **Infallible Patterns**: ActiveSession trait with infallible_get_active_session collapses Result<Option<T>, E> to Option<T> for simpler error handling
- **Session Validation Flow**: Signal holds session from context, spawn() validates and updates signal, component re-renders on change
- **Error Honesty Pattern**: Returning `Err(SessionExpired)` instead of `Ok(vec![])` when session missing - honest errors better than fake empty states
- *Note: Still building mental model - async closures returning Futures vs spawn() calling them immediately is confusing*

### 🔐 Backend Session & Token Architecture (Complete) ✅
- **Session-Based Authentication**: Session struct containing user + access_token + refresh_token + both expiration timestamps
- **Rotating Token Strategy**: Security model where refresh operation generates new access + new refresh token, invalidating old refresh
- **Access vs Refresh Tokens**: JWTs (self-contained, 24hr) vs opaque hex strings (64-char, 14-day, hashed with SHA-256 for storage)
- **Token Hashing Strategy**: SHA-256 for refresh tokens (fast verification) vs Argon2 for passwords (slow, memory-hard)
- **Refresh Token Database Design**: Separate table with user_id, hashed token value, created_at, expires_at, revoked flag
- **Multi-Device Session Support**: Multiple refresh tokens per user (max 5) enabling concurrent device authentication
- **Token Refresh Flow**: 401 response triggers refresh, not proactive checking (performance + no security benefit)
- **Request Type Simplification**: CreateSession and RevokeSessions wrap Uuid directly, RefreshSession contains user_id + refresh_token string
- **Session Port Architecture**: AuthService and AuthRepository traits for session management operations
- **Token Expiration Strategy**: 14-day refresh tokens, 24-hour access tokens balancing security vs UX
- **RefreshToken Self-Containment**: Struct with value + expires_at fields for tight coupling and consistency
- **Sha256Hash Trait Pattern**: Flexible trait implementation enabling hashing of RefreshToken and String types
- **SQLx Session Operations**: create_user_and_refresh_token, create_refresh_token, use_refresh_token, delete_users_refresh_tokens, delete_expired_refresh_tokens implementations
- **Token Rotation Pattern**: Delete old token, create new token atomically in use_refresh_token method
- **Session Validation Logic**: Expiration checking, ownership verification (user_id match), revocation status checking
- **Session Maximum Enforcement**: SQL window functions (ROW_NUMBER() OVER PARTITION BY) with transaction helpers for automatic oldest-token cleanup
- **Transaction Helper Pattern**: create_refresh_token_with_tx and enforce_refresh_token_max_with_tx reused across operations
- **Atomic Registration**: create_user_and_refresh_token wraps user creation + token creation in single transaction preventing orphaned accounts
- **Service Session Methods**: Complete implementation of create_session, refresh_session, revoke_sessions with proper orchestration
- **Cross-Domain Session Creation**: AuthService orchestrates UserRepository (fetch user data) + AuthRepository (token operations) + JWT generation
- **Expired Token Cleanup**: delete_expired_refresh_tokens method using WHERE expires_at < NOW() for scheduled maintenance via sync binary
- **Middleware Session Integration**: AuthenticatedUser extractor constructs Jwt from bearer token, validates with JwtSecret, includes username
- **Jwt/AccessToken Separation**: Moved validation from AccessToken to Jwt for cleaner type responsibilities
- **HTTP Session Responses**: Register/login/refresh handlers all return Session struct with complete user and token data
- **Refresh Endpoint**: Complete /api/auth/refresh POST endpoint with HttpRefreshSession request/response types and route function
- **Logout Endpoint**: Complete /api/user/logout POST endpoint with revoke_sessions handler using AuthenticatedUser middleware
- **Enhanced Error Logging**: RefreshSessionError variants include user_id for security audit trails (NotFound, Expired, Revoked, Forbidden)
- **Exhaustive Error Mapping**: Removed all catch-all patterns from ApiError implementations ensuring explicit per-variant handling
- **Scheduled Token Cleanup**: CheckSessions trait in zync binary with weekly cleanup using map_or() for Option handling, memory-tracked latest_token_clean_up
- **RESTful Logout Design**: POST verb for logout (not GET) due to side effects and state modification
- *Note: Backend complete and production-ready, frontend integration (storage, auto-login, 401 handling) next phase*

### 📱 Frontend Session Management (Complete) ✅
- **PersistentSession Trait**: Built trait on Session for keyring-based storage (persist, retrieve, clear methods)
- **Keyring Integration**: Using keyring crate for iOS Keychain/Android KeyStore abstraction
- **Session Expiration Checking**: retrieve() checks refresh_token expiration and auto-clears expired sessions
- **In-Memory Sessions Working**: Login/register flows create sessions successfully, navigation works
- **iOS Entitlements Blocker**: Keychain access requires Dioxus.toml bundle config with keychain-access-groups - deferred to deployment
- **Development Strategy**: Sessions work in-memory during development, will persist after entitlements configured for production
- **ActiveSession Wrapper**: Type-safe wrapper ensuring validated sessions for HTTP requests
- **GetActiveSession Trait**: Three-path token handling (valid, refresh needed, re-auth needed) with proper error returns
- **Session Context Integration**: Using use_context() pattern for accessing session state in async functions
- **HTTP Client Session Patterns**: AuthClient methods handle session validation and refresh automatically
- *Note: Complete session management system ready for production use*

### 🔮 Advanced Rust Patterns
- **Advanced Async Patterns**: Complex Future handling, async streaming, async iterators
- **Type-Level Programming**: Advanced trait constraints, generic programming patterns
- **Complex Lifetime Management**: Advanced lifetime parameters and borrowing patterns

### 🚀 Production Deployment & Scaling
- **Containerization**: Docker, Kubernetes deployment strategies
- **Monitoring & Observability**: Metrics collection, logging, distributed tracing
- **Performance Tuning**: Query optimization, connection pool sizing, caching strategies
- **Rate Limiting**: Request throttling, abuse prevention mechanisms

### 🎮 MTG-Specific Business Logic
- **Format Validation**: Standard/Modern legality checking, card legality rules
- **Deck Rules**: 60-card minimums, 4-card limits, sideboard validation
- **Card Interactions**: Rules engine for card interactions and abilities

---

## UNEXPLORED - Future Learning Areas 🔍

### 🧬 Advanced Rust Language Features
- **Procedural Macros**: Deep proc macro implementation (staying away until necessary)
- **Unsafe Rust**: Memory manipulation, FFI, performance optimizations
- **Embedded Rust**: Hardware programming, real-time systems
- **WebAssembly**: Rust to WASM compilation and browser integration

### 🏢 Enterprise Infrastructure
- **Microservices Architecture**: Service discovery, distributed systems patterns
- **Message Queues**: RabbitMQ, Kafka integration for async processing
- **Distributed Databases**: Sharding, replication, consistency patterns
- **Cloud Platforms**: AWS/GCP deployment, serverless architectures

### 🎨 Frontend Integration
- **Dioxus Mobile**: Cross-platform mobile app development
- **WebRTC**: Real-time communication for multiplayer features
- **Progressive Web Apps**: Offline functionality, service workers
- **State Management**: Complex client-side state synchronization

### 🔬 Advanced Performance Engineering
- **Profiling & Benchmarking**: CPU profiling, memory analysis, performance testing
- **Custom Allocators**: Memory management optimization
- **SIMD Programming**: Vectorized operations for data processing
- **Lock-Free Programming**: Concurrent data structures, atomic operations

---

## LEGACY KNOWLEDGE - Previously Used Technologies 📚

### 🗂️ Diesel ORM (Migrated to SQLx)
- **Connection Pooling**: r2d2 integration and connection management
- **Query Building**: .filter(), .select() patterns and query construction
- **Foreign Key Queries**: Table-qualified queries and relationship handling
- **Schema Usage**: Diesel schema files and compile-time query verification
- **Migration System**: Diesel CLI and database migration management

*Note: Migrated to SQLx for direct SQL control and better performance*
