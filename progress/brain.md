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

**Last Updated**: Started building deck card addition flow with filtering system and planning multi-layer swipe architecture for card browsing.

**Next Learning Focus**: Complete card filtering UI/UX. Implement nested swipe detection distinguishing card-level from screen-level gestures. Build deck metrics visualization screen.

**Recent Achievement**: Refactored EditDeckProfile with use_memo for computed change detection (deck_name_update, commander_id_update, copy_max_update, has_made_changes). Added conditional deck card management buttons (add/remove) appearing only when no profile changes exist. Built AddDeckCard and RemoveDeckCard screen foundations with Filter component implementing name-based card search, image filtering (excludes cards without large images), and success message feedback. Created helper methods: HttpSearchCards::blank(), is_blank(), PartialEq derive; Optdate::is_changed(). Planned three-layer screen architecture: filters (up), card display (center), deck metrics (down). Designing nested swipe detection system to differentiate card element swipes (left/right for browsing results) from screen swipes (up/down for navigation).

### 🤔 Current Uncertainties (Top 5)
1. **Nested Swipe Detection** - Differentiating card element swipes from screen swipes based on touch target
2. **Card Browsing UX** - Implementing smooth left/right swipe navigation through search results
3. **Quantity Management** - UI patterns for selecting card quantities (respecting deck copy_max rules)
4. **Deck Metrics Visualization** - Calculating and displaying mana curve, color distribution, type breakdown
5. **Multi-Layer Swipe Architecture** - Managing separate SwipeState for card-level vs screen-level gestures

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
- **Ownership Bug Patterns**: Recognizing inverted authorization logic from missing negation in ownership checks
- **Full-Stack Debugging**: Tracing issues through frontend → domain → SQLx layers to identify root causes
- **Foreign Key Understanding**: Database primary keys (card_profile.id) vs external IDs (scryfall_data.id) for proper relationships
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
- **SessionProvider Pattern**: Wrapping Router with context providers for global session/auth_client access across all routes
- **Guard Route Pattern**: Root route redirecting based on session state for initial auth flow control
- **Minimal UI Design**: Lowercase text, centered layouts, inline buttons for clean aesthetic consistency
- **Profile Management UI**: Forms for username/email/password changes with inline change buttons
- **Logout Implementation**: Session clearing with Persist::delete(), signal updates, and navigation to login
- **Authenticated HTTP Requests**: Pattern of session validation → token refresh check → bearer header → API call → conditional session update
- **Success Message System**: Random success feedback with get_random_success_message() for positive UX
- **Profile Change Integration**: Complete change_username, change_email, change_password with backend calls and session updates
- **Form Validation Patterns**: Per-field validation functions, error display after first submit, clearing submission_error on success
- **Swipeable Component Integration**: Wrapping all screens with Swipeable for consistent moveable UI (SwipeConfig::blank() for non-navigating screens)
- **Resource Pattern Matching**: `.value().with(|result| match result {...})` to access Resource data without "temporary value dropped" errors
- **Backend Logout Flow**: POST to /api/auth/logout with bearer token revokes server-side refresh tokens before local session clearing
- **Signal Simplification**: Using signal() syntax instead of signal.read() for cleaner Dioxus code (17+ simplifications across components)
- **Debounced Search**: Implementing search with 300ms delay using tokio::time::sleep in spawned async tasks
- **use_memo Pattern**: Computed signals deriving state from other signals, only updating when computed value changes (not when source signal content changes)

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
- **Direct UUID Handling**: SQLx returns Uuid directly from PostgreSQL without string conversion (database types align with Rust types)
- **Option<Option<T>> Pattern**: Domain model pattern distinguishing "no update" from "set to None" for partial update requests
- **Dynamic SQL Building**: QueryBuilder conditional push pattern for building UPDATE queries with only changed fields
- **Conditional Field Updates**: Using if let to selectively build SQL based on which fields are present in update request

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

### 🎨 Frontend Deck CRUD (Complete Flow)
- **Deck Creation Screen**: Built CreateDeck component with deck name input, searchable commander field, and CopyMax selection
- **Searchable Commander Field**: Debounced card search using HttpSearchCards::by_name() with 500ms delay via tokio::time::sleep
- **Dropdown UX**: Search results display as clickable options in normal document flow (pushes other elements down)
- **CopyMax Selection**: Three-option UI (standard/singleton/none) with centered layout and visual feedback
- **View/Edit Separation**: ViewDeckProfile (read-only) and EditDeckProfile (editable form) for clean separation of concerns
- **EditDeckProfile Implementation**: Complete 362-line screen with pre-population, change tracking, conditional updates
- **Pre-Population Pattern**: use_effect watching resources, populating form signals with current values
- **Change Tracking**: Separate original_* signals tracking initial state, comparing to current values before update
- **Conditional Update Requests**: Only send changed fields (Option<T> for each field), reducing unnecessary database writes
- **Commander Image Display**: Shows large Scryfall card image using image_uris (max-height: 40vh), falls back to text name if unavailable
- **Image Fallback Pattern**: Qualified match on `ImageUris { large: Some(url), .. }` with else block for missing images
- **Comprehensive Error Handling**: Separate load_error, submission_error, delete_error signals for granular display
- **Delete Deck Functionality**: Implemented with async error handling, navigation to deck list on success
- **CSS Overflow Handling**: overflow-y: auto on swipeable, max-height constraint on images preventing button cutoff
- **Option<Option<T>> Pattern**: Domain model distinguishing "no update" from "set to None" for nullable fields
- **Dynamic SQL Building**: QueryBuilder conditional push based on which fields are present in update request
- *Note: Complete CRUD flow working, confirmation dialog next*
- **Complete Client Method Suite**: All 19 backend endpoints covered across 5 domains (auth: 4, user: 5, deck: 5, deck_card: 3, card: 2)
- **Unified ApiError Architecture**: Single error enum shared between frontend and backend, moved to zerver/src/lib/inbound/http.rs
- **Frontend/Backend Error Sharing**: Eliminated duplicate frontend error.rs file, both sides use same ApiError with Network variant for client errors
- **Reqwest Version Alignment**: Synchronized frontend (0.12) and backend (0.12) reqwest versions for clean From trait implementations
- **From Trait Patterns**: `From<(StatusCode, String)> for ApiError` enabling automatic conversions via `.into()` at call sites
- **Status Code Mapping**: Centralized mapping of HTTP status codes (Unauthorized, Forbidden, NotFound, UnprocessableEntity, InternalServerError)
- **Request Convenience Methods**: `.json()` for automatic serialization + Content-Type header, `.bearer_auth()` for authentication headers
- **HTTP Verb Consistency**: All client methods audited for correct REST verbs (GET/POST/PUT/DELETE)
- **Client Method Organization**: Refactored into domain folders - auth/, user/, deck/, deck_card/, card/ for clean separation
- **Auth Domain**: login, register, logout, refresh (Session-producing operations)
- **User Domain**: get_user, change_username, change_email, change_password, delete_user (profile management)
- **Deck Domain**: create_deck_profile, get_deck_profiles, get_deck, update_deck_profile, delete_deck (deck CRUD)
- **Deck Card Domain**: create_deck_card, update_deck_card, delete_deck_card (deck composition)
- **Card Domain**: get_card (by UUID), search_cards (complex query parameters with skip_serializing_if)
- **Authenticated Request Pattern**: Session validation → infallible_get_active_session → bearer header → HTTP request
- **Success/Error State Management**: Mutually exclusive success_message and submission_error signals
- **Backend Serialization Alignment**: Added Serialize/Deserialize derives to backend types for frontend consumption
- **Security Enhancement**: delete_user requires password confirmation with HttpDeleteUser request type
- **Code Reduction Victory**: Net reduction of 263 lines (711 deleted, 448 added) through architectural improvements
- *Note: Complete production-ready HTTP client layer with consistent patterns across all operations*

### 🎮 Swipe-Based Navigation & Gestures
- **Position Tracking**: BasicPoint (i32 x/y) for screen coordinates independent of swipe detection
- **Multi-Screen Architecture**: Always-render pattern with CSS transforms controlling visibility
- **Swipe-to-Submit Implementation**: use_effect watching latest_swipe signal triggers form submissions when submission_swipe direction detected
- **Gesture Separation Strategy**: Horizontal swipes (left/right) for submission, vertical swipes (up/down) for navigation eliminates conflicts
- **Transform Calculations**: CSS calc() combining xpx/ypx (finger delta) with xvw/yvh (screen_displacement offsets)
- **Extended Axis Locking**: traversing_axis (X/Y) includes both navigation_swipes and submission_swipe directions for complete gesture control
- **Smart Displacement Updates**: update_position() only called for directions in navigation_swipes, not submission_swipe
- **Direction Resolution**: Separate tracking of detected swipe vs allowed navigation for dual-purpose gestures
- **Effect-Based Submission**: use_effect pattern with spawn() wrapping async submission logic to control signal dependency tracking
- **Coordinate System**: Browser coordinates (positive Y down) with proper delta calculations
- **State Management**: Decoupled position updates from swipe detection for flexible interaction patterns
- **Abstraction Patterns**: Consolidated onswipestart/move/end for identical touch/mouse behavior
- **Modular Swipe Architecture**: Split into axis, config, direction, onmouse, ontouch, state, time_point modules (7 files)
- **Swipeable Component**: Reusable wrapper requiring external Signal<SwipeState> for shared state across screens
- **SwipeConfig Structure**: navigation_swipes (Vec), submission_swipe (Option), from_main_screen (ScreenOffset) for positioning
- **Shared State Pattern**: Parent component creates Signal<SwipeState>, passes to all child Swipeable components
- **Smooth Animation System**: is_swiping flag controls return_animation_seconds (0.0 during swipe, non-zero after)
- **VW_GAP Adjustment**: Increased to 100vw for horizontal submission gestures, VH_GAP remains 75vh for vertical navigation
- **Universal Integration**: All auth (Login, Register, Home) and app (Profile, MainHome, Decks) screens using Swipeable
- **ScreenOffset Type System**: Point2D<i32> coordinates replacing Option<Direction> for flexible multi-dimensional positioning
- **Screen Positioning Trait**: ScreenOffsetMethods trait providing up/down/left/right factory methods and chaining methods (*_again)
- **Multi-Dimensional Layouts**: Can position screens at any x/y coordinate (diagonal, multiple steps in one direction, etc)
- **Position-Aware Submissions**: Form submission guards check current screen_offset to prevent wrong-screen submissions
- **Screen Offset Calculations**: Transform calculations using offset.x and offset.y directly with VW_GAP/VH_GAP multipliers
- *Note: New system enables complex screen hierarchies like settings menus, multi-level navigation, grid-based layouts*

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

### 🎨 Dioxus Reactivity & Async Patterns (MAJOR BREAKTHROUGH)
- **use_effect Dependency Tracking**: Automatically tracks ALL signal reads as dependencies, re-running effect when any tracked signal changes
- **use_future Independence**: Runs once on mount, does NOT track signal dependencies - correct tool for background loops
- **use_resource Refetch Behavior**: Tracks signal reads and refetches when dependencies change - dangerous if updating signals it reads
- **Infinite Loop Pattern Recognition**: Reading + writing same signal in tracked context (use_effect/use_resource) = exponential task explosion
- **Conditional Update Pattern**: `if new_value != old_value { signal.set(new_value) }` prevents infinite loops in reactive contexts
- **spawn() Breaks Tracking**: Signal reads inside spawn() blocks DON'T register as dependencies for parent effect/resource
- **Background Task Spawning**: Use use_future for infinite loops, NOT use_effect (which would spawn new loop on every dependency change)
- **Fire-and-Forget spawn()**: Direct spawn() in component body for one-time background tasks that don't need hook lifecycle
- **Session Refresh Architecture**: Single centralized background loop in home.rs, individual screens use use_resource with conditional updates
- **Resource + Signal Update Pattern**: Check/refresh in use_resource, make API call, conditionally update signal if changed, return result
- **Exponential Task Explosion**: 3 components × use_effect spawning loops × signal updates = 2→4→8→16→32→64+ tasks in milliseconds
- **Debugging Reactivity Loops**: Add strategic logging to track effect re-runs, look for exponential growth patterns in connection attempts
- **use_resource Structure**: Handles `Option<Result<T, E>>` - None while loading, Some(Ok(data)) on success, Some(Err(e)) on failure
- **Async Closure Pattern**: `move || async move { }` for creating Futures that can be `.await`ed when called
- **Signal + Send Issues**: mut Signal parameters can't cross async boundaries due to RefCell (not Sync) - pass Signal by value instead
- **Navigation Effect Deadlock**: navigator.push() in use_effect causes freeze - use conditional rendering instead
- **Centralized Session Management**: Better to have one background refresh loop than duplicate logic in every component
- **Empty State UX**: Always handle empty lists explicitly (e.g., "no decks yet" message) instead of showing nothing
- **Resource Lifetime Pattern**: `.value().with(|val| ...)` closure to access Resource data, extract owned copies to avoid "temporary value dropped" errors
- **Owned Data Extraction**: Inside `.with()` closure, clone primitives and owned types (e.g., `(id, name.clone())`), use outside closure for rendering
- **Resource Match Strategy**: CORRECT: `.value().with(|result| match result {...})` - match inside with() closure, not on resource.read() directly
- **Resource Borrow Checker Fix**: Can't `match &*resource.read()` - creates temporary value that doesn't live long enough. Must use `.value().with()` closure pattern
- **Three-State Rendering**: Match on Some(Ok(data)), Some(Err(e)), None inside `.with()` closure for loading/success/error states
- **Form Pre-Population Pattern**: use_effect watching resources to extract loaded data and populate form signals with current values
- **Change Tracking Pattern**: Separate original_* signals tracking initial state, comparing to current signals before update submission
- **Conditional Update Requests**: Only send changed fields by comparing current vs original values, reducing unnecessary backend calls
- **Multiple Error Signals**: Separate error signals (load_error, submission_error, delete_error) for granular error display in different contexts
- *Note: Hook selection (effect vs future vs resource) is critical - wrong choice causes infinite loops or missing reactivity*

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

### 🃏 Deck Card Management & Multi-Layer Swipe Architecture (Active Development)
- **AddDeckCard Screen Foundation**: Built screen with Filter component separation, card display, and add button placeholder
- **Filter Component Architecture**: Separate screen positioned "up" from main, swipeable navigation between filter and card display
- **Card Search Integration**: HttpSearchCards with name filtering, image presence validation (filter out cards without large images)
- **Success Message Feedback**: Random success messages after successful card searches using get_random_success_message()
- **Helper Method Creation**: Built HttpSearchCards::blank(), is_blank(), PartialEq derive for filter state validation
- **Optdate Convenience**: Added is_changed() helper complementing is_unchanged() for cleaner conditional logic
- **Three-Layer Screen Planning**: Filter (up), card display (center), metrics (down) with vertical swipe navigation
- **Nested Swipe Detection Challenge**: Need to differentiate card element swipes (left/right for browsing) from screen swipes (up/down for navigation)
- **Touch Target Identification**: Planning to detect which element receives touch/mouse event to route to appropriate swipe handler
- **Card Browsing Vision**: Swipe left/right through search results card-by-card, swipe up/down to navigate screens
- **Deck Metrics Planning**: Down screen will show mana curve, color distribution, type breakdown calculated from card data
- *Note: Early foundation stage, core challenge is nested gesture detection differentiating card vs screen swipes*

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
