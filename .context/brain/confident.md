# CONFIDENT - Could Teach Others 🎓

Knowledge areas where you could teach others without hesitation.

---

## 🦀 Core Rust Fundamentals
- **Basic Syntax**: Ownership, borrowing, basic pattern matching, error handling with Result
- **Type System**: Understanding of structs, enums, traits, basic generics
- **Memory Safety**: Conceptual understanding of Rust's ownership model
- **Module System**: Basic use of mod, pub, use statements for code organization
- **Cargo Workflow**: Creating projects, running tests, managing dependencies
- **Debugging**: Using println!, investigating compiler errors, reading documentation

## 🏗️ Hexagonal Architecture & Clean Design
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
- **Pragmatic Newtype Judgment**: Knowing when newtypes add value (validation, type safety, domain methods) vs unnecessary ceremony (Set names are just display strings)
- **Modular File Organization**: Per-operation domain model files, error/models/helpers SQLx modules, separated HTTP handlers
- **Ownership Validation Patterns**: Trait-based ownership checking (OwnsDeck) preventing unauthorized resource access
- **Ownership Bug Patterns**: Recognizing inverted authorization logic from missing negation in ownership checks
- **Full-Stack Debugging**: Tracing issues through frontend → domain → SQLx layers to identify root causes
- **Foreign Key Understanding**: Database primary keys (card_profile.id) vs external IDs (scryfall_data.id) for proper relationships
- **Direct Domain Serialization**: Serialize domain types directly when HTTP shape matches, avoiding unnecessary wrapper boilerplate

## 🔒 Security & Authentication
- **JWT Security Flow**: Complete token generation/validation with proper configuration
- **Password Security**: Argon2 hashing, salts, rainbow table prevention, timing attack mitigation
- **Cryptographic Hashing Strategy**: SHA-256 for tokens (fast verification) vs Argon2 for passwords (slow, memory-hard)
- **Authentication Delays**: Why delays in auth are important for security
- **Information Disclosure Prevention**: Avoiding enumeration attacks through generic error responses
- **Security Spidey Senses**: When to step carefully with sensitive data and implementation details
- **Middleware Security**: Functional type-based authentication through authorization headers

## 🚨 Error Handling Architecture
- **Strategic Error Types**: When to use thiserror (decisive system responses) vs anyhow (console error stacks)
- **Error Flow Design**: Comprehensive error handling patterns throughout application layers
- **HTTP Status Mapping**: Business logic errors to appropriate status codes
- **Two-Tier Error Strategy**: User-facing vs internal error separation

## 🗄️ Database Design & Relationships
- **Relational Modeling**: Foreign keys, composite keys, constraints, and indices
- **Complex Joins**: Multi-table queries and relationship management
- **Database Constraints**: Strategic use for business rule enforcement
- **JSONB Confidence**: Comfortable storing JSON in PostgreSQL tables
- **Composite vs Surrogate Keys**: When to use natural vs artificial primary keys

## 📡 HTTP & RESTful API Design
- **Parameter Extraction**: Body, path, and query parameter handling
- **RESTful Patterns**: Proper HTTP verb usage, nested resource routes
- **Status Code Precision**: Correct HTTP status codes for different operations
- **Parameter Naming Consistency**: Aligned naming across request pipelines

## ⚙️ Basic Implementation Patterns
- **Environment Setup**: .env files, configuration management
- **macro_rules!**: Basic declarative macro creation and usage
- **Build Scripts**: build.rs with cargo directives (rustc-env, warning, rerun-if-changed) for compile-time configuration
- **Compile-Time Environment Variables**: env!() macro for baking config into binaries (required for desktop/mobile apps)
- **Infallible Config Pattern**: Using unwrap() in Config::from_env() when config is deployment requirement (better to crash than run with invalid config)
- **Frontend Logging**: tracing_subscriber::fmt() setup in main() with configurable log levels for development debugging
- **Cargo Workspace Configuration**: Multi-package workspace setup with resolver 2, centralized dependency management via workspace.dependencies
- **Workspace Dependency Inheritance**: Using workspace = true in child Cargo.toml files to inherit versions from root, eliminating drift
- **Build Profile Consolidation**: Centralizing profiles ([profile.wasm-dev], etc.) at workspace root instead of per-package
- **Clippy Workspace Configuration**: workspace.lints.clippy in root Cargo.toml for lint levels, clippy.toml for thresholds, [lints] workspace = true in packages
- **Clippy Fix Workflow**: cargo clippy --fix --allow-dirty --allow-staged for auto-fixes, git diff review, commit cleaned changes
- **Copy Type Semantics**: Understanding when & is unnecessary for Copy types (Uuid, primitives) vs non-Copy (String, Vec)

## 🎨 Dioxus Component Development & State Management
- **Component Architecture**: Function components with RSX macro for HTML-like syntax
- **State Management**: Signal types for reactive state with use_signal() patterns
- **Context API**: use_context_provider() at root, use_context() in components for global reactive state
- **Props vs Context**: Props for parent-child relationships, context for app-wide state (Session, AuthClient, filter, cards)
- **Context for Route-Shared State**: Signals that need to persist across navigation must live in context, not as route parameters
- **Signal Serialization Limits**: Signals can't be reliably passed as route parameters (not serializable), use context for cross-route state
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
- **Guard Route Pattern**: Root route component conditionally redirecting to /home or /login based on session presence
- **Minimal UI Design**: Lowercase text, centered layouts, inline buttons for clean aesthetic consistency
- **Profile Management UI**: Forms for username/email/password changes with inline change buttons
- **Logout Implementation**: Session clearing with Persist trait delete(), signal updates, and navigation to login
- **Authenticated HTTP Requests**: Pattern of session validation → token refresh check → bearer header → API call → conditional session update
- **Success Message System**: Random success feedback with get_random_success_message() for positive UX
- **Profile Change Operations**: Complete change_username, change_email, change_password with backend calls and session updates
- **Form Validation Patterns**: Per-field validation functions, error display after first submit, clearing submission_error on success
- **Swipeable Component Integration**: Wrapping all screens with Swipeable for consistent moveable UI (SwipeConfig::blank() for non-navigating screens)
- **Resource Pattern Matching**: `.value().with(|result| match result {...})` to access Resource data without "temporary value dropped" errors
- **Backend Logout Flow**: POST to /api/auth/logout with bearer token revokes server-side refresh tokens before local session clearing
- **Signal Simplification**: Using signal() syntax instead of signal.read() for cleaner Dioxus code (17+ simplifications across components)
- **Signal Copy vs Non-Copy**: signal() syntax only works for Copy types (primitives, simple structs); non-Copy types (Vec, String, complex structs) require explicit .read()/.write()
- **Direct Mutation vs Reactive Patterns**: Prefer direct filter.write() access when possible; only use local signal + use_effect for read/write conflicts
- **Modular Component Architecture**: Breaking monolithic components into focused sub-components with shared Signal for state coordination
- **Navigator Async Constraints**: Navigator can't be called inside spawn() async blocks - use signal + use_effect bridge (set signal in async, watch in effect)
- **Debounced Search**: Implementing search with 300ms delay using tokio::time::sleep in spawned async tasks
- **use_memo Pattern**: Computed signals deriving state from other signals, only updating when computed value changes (not when source signal content changes)
- **Reusable Component Abstraction**: Extracting repetitive UI patterns (TextInput) into focused components with clear props interface
- **Component Props Design**: Balancing required vs optional props, using Option<String> for optional labels, Signal<String> for two-way binding
- **DRY Component Boundaries**: Knowing when to extract components (repeated 3+ times with identical structure) vs when inline is clearer
- **Signal Initialization from Context**: Pattern of `use_signal(|| { context().field()... })` for initializing local signals from context on mount, ensuring form inputs display persisted state
- **Mount-Time Initialization**: Understanding that signal closures run once on component mount, capturing initial values from context for local editing
- **EventHandler Props Pattern**: Building generic components with EventHandler<()> props for flexible callback integration (on_swipe_left, on_swipe_right, etc.)
- **EventHandler Invocation**: EventHandler.call(()) pattern to invoke callbacks passed as props from parent components
- **Mutable Closure Requirements**: Closures that modify signals or call other closures must be declared with `let mut` keyword to satisfy borrow checker
- **Component Generalization**: Refactoring context-specific components into generic wrappers by replacing hardcoded logic with EventHandler props
- **CSS Class Removal for Positioning**: Understanding when CSS classes cause unintended layout effects (position: fixed creating full-screen overlays)
- **@keyframes vs CSS Transitions**: Transitions require element to exist in DOM with a "before" state; @keyframes animations trigger immediately on DOM insertion—critical for conditional rendering
- **dioxus_primitives Conditional Rendering**: AccordionContent uses `if render_element()` pattern, meaning element only exists in DOM when open—transitions can't animate opening, must use @keyframes
- **CSS Grid Animation Pattern**: grid-template-rows: 0fr → 1fr animates actual content height; combined with @keyframes for open, transitions for close
- **Hybrid Animation Strategy**: Use @keyframes for elements being inserted (open), transitions for elements being removed (close)—solves conditional rendering animation challenges
- **AlertDialog Component System**: Built wrapper components around dioxus_primitives alert_dialog with custom styling for confirmation modals (logout, delete)
- **Modal Dialog Pattern**: AlertDialogRoot, Content, Title, Description, Actions, Cancel, Action components creating consistent confirmation UX
- **Global CSS Loading Requirement**: document::Link inside components doesn't reliably render to <head>—must load component CSS globally in main.rs with Asset constants
- **CSS Loading Architecture**: Component stylesheets (accordion.css, alert-dialog.css, toast.css) loaded at app root ensures styles available on component mount
- **Utility Bar Standardization**: Consolidated navigation buttons to bottom .util-bar divs with .util-btn styling across 20+ screens for consistent mobile-first UX
- **Dioxus Reactivity (.read() vs .peek())**: .read() reads AND subscribes to changes (dependency tracking), .peek() reads WITHOUT subscribing (one-time read), critical for performance in use_effect
- **Explicit Dependency Pattern**: Watch specific signals with `let _ = signal();` in use_effect, peek others to avoid unwanted re-runs
- **Optimistic Updates**: Update local state (HashSet, Vec) immediately on API success without refetching for instant UI feedback
- **LIFO Undo Stack**: Vec<Action> tracking user actions without storing full objects, index manipulation for undo (3000x memory savings vs storing full data)
- **Performance Limits**: Hard caps (MAX constant) and warning thresholds for unbounded data structures, periodic warnings (modulo checks), graceful degradation

## 💾 SQLx Database Operations & Advanced Patterns
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
- **PostgreSQL UPSERT Pattern**: ON CONFLICT (column) DO UPDATE SET for insert-or-update operations preventing constraint violations
- **EXCLUDED Table Reference**: Special PostgreSQL table in UPSERT containing attempted insert values for UPDATE clause
- **Trigger Functions**: PL/pgSQL functions with RETURNS TRIGGER, NEW/OLD variables, BEFORE/AFTER timing for automated database logic
- **Trigger Attachment**: CREATE TRIGGER with FOR EACH ROW executing functions on table events (INSERT/UPDATE/DELETE)
- **ON DELETE RESTRICT**: Foreign key constraint preventing parent deletion when children exist (vs CASCADE which deletes children)
- **Delta Sync Pattern**: Fetch existing records, filter with PartialEq to find changes, upsert only delta—reduces database load
- **Empty Collection Guards**: Check for empty slices before building SQL to prevent invalid queries (INSERT with no VALUES errors)
- **Tuple Return Pattern**: Returning (Vec<T>, usize) from functions to communicate both results and metadata (cards + skip count)
- **QueryBuilder Invalid SQL**: SQLx QueryBuilder with empty VALUES clause creates syntax errors—must guard against empty collections
- **JSONB Operator Types**: @> and <@ work with jsonb on both sides, ?| requires jsonb ?| text[] for "contains any" queries
- **Type Conversion for Operators**: Colors::to_short_name_vec() pattern converting domain types to Vec<String> for proper SQL parameter binding
- **Operator Selection Strategy**: Use @> for exact match (jsonb contains jsonb), ?| for "any of" (jsonb contains any text[])
- **SQLx Custom Enum Serialization**: Manual implementation of Type/Encode/Decode traits for enums, type_info() declares PostgreSQL type, encode_by_ref() converts to database format, decode() parses from database
- **PostgreSQL = ANY() Pattern**: Using = ANY(ARRAY[...]) for efficient multi-value equality checks, SQLx automatically encodes Vec<T> to PostgreSQL arrays
- **Custom Serde for Enums**: Implementing custom Serialize/Deserialize using TryFrom for flexible parsing (case-insensitive, multiple formats), avoiding derive limitations
- **.as_ref() vs .as_deref() Semantics**: .as_ref() converts Option<T> → Option<&T> (keeps wrapper), .as_deref() converts Option<T> → Option<&T::Target> (uses Deref trait), crucial for Option<Newtype> handling
- **VARCHAR vs TEXT in PostgreSQL**: Functionally identical types, TEXT preferred for modern PostgreSQL (no length limits, more flexible), VARCHAR legacy compatibility
- **SQLx Type Compatibility**: Type impl must exactly match database column type (text vs varchar), SQLx strictly enforces type matching for safety

## 🌐 Advanced HTTP & Middleware Patterns
- **Custom Middleware**: AuthenticatedUser extractor with FromRequestParts trait
- **Type-Safe Authentication**: Compile-time route protection through handler parameters
- **Generic Handler Patterns**: Complex generic state types across all handlers
- **Error Architecture**: Sophisticated ApiError with domain error mapping
- **Request/Response Conversion**: Clean TryFrom patterns for HTTP-to-domain conversion
- **Bearer Token Extraction**: JWT validation and claims parsing in middleware
- **Exhaustive Error Mapping**: Pattern of explicit per-variant matching in From<DomainError> for ApiError implementations, avoiding catch-all patterns to ensure compiler catches missing error cases when enums evolve

## 🔄 Async Programming & Trait Constraints
- **Async Function Design**: Building async functions with proper trait bounds
- **Send + Sync Constraints**: Understanding trait requirements for async service patterns
- **Generic Async Patterns**: Handler functions with generic state types and async operations
- **Future Handling**: Async trait implementations across hexagonal boundaries
- *Note: Manual thread synchronization (.lock(), message passing) not yet implemented*

## 📊 Production API Design
- **Nested Resource Routes**: Hierarchical /api/deck/{deck_id}/card/{card_profile_id} patterns
- **Tuple Path Extraction**: Path<(String, String)> for multi-parameter routes
- **Composite Key Architecture**: Natural primary keys eliminating surrogate IDs
- **Comprehensive Search System**: CMC/power/toughness ranges, dual color identity modes, input sanitization
- **PostgreSQL Advanced Queries**: Regex validation, array operators (@>, <@, &&), dynamic query building
- **RESTful Patterns**: Proper HTTP verb usage, status code precision, parameter naming consistency

## 🎨 Frontend Deck CRUD & Complete Flow
- **Deck Creation Screen**: CreateDeck component with name input, searchable commander field, CopyMax selection
- **Searchable Commander Field**: Debounced card search with tokio::time::sleep
- **Dropdown UX**: Search results in normal document flow
- **CopyMax Selection**: Three-option UI (standard/singleton/none)
- **View/Edit Separation**: ViewDeckProfile (read-only) vs EditDeckProfile (editable form)
- **EditDeckProfile Mastery**: Complete 362-line screen with pre-population, change tracking, conditional updates
- **Pre-Population Pattern**: use_effect watching resources, populating form signals
- **Change Tracking**: Separate original_* signals comparing to current values
- **Conditional Update Requests**: Only send changed fields via Option<T>
- **Commander Image Display**: Large Scryfall images with text fallback
- **Image Fallback Pattern**: Qualified match on ImageUris with graceful degradation
- **Comprehensive Error Handling**: Separate load_error, submission_error, delete_error signals
- **Delete Deck Functionality**: Async error handling with navigation on success
- **CSS Overflow Handling**: overflow-y: auto preventing button cutoff
- **Complete Client Method Suite**: All 19 backend endpoints across 5 domains
- **Unified ApiError Architecture**: Single error enum shared between frontend/backend
- **From Trait Patterns**: Automatic (StatusCode, String) → ApiError conversion
- **Status Code Mapping**: Centralized HTTP status code handling
- **Request Convenience Methods**: .json() for serialization, .bearer_auth() for auth
- **HTTP Verb Consistency**: Correct REST verbs across all operations
- **Authenticated Request Pattern**: Session validation → token refresh → bearer header → API call
- **Success/Error State Management**: Mutually exclusive signals

## 🎮 Swipe-Based Navigation & Gestures (Production Ready)
- **Position Tracking**: BasicPoint (i32 x/y) for screen coordinates
- **Multi-Screen Architecture**: Always-render pattern with CSS transforms
- **Swipe-to-Submit Implementation**: use_effect watching latest_swipe signal
- **Gesture Separation Strategy**: Horizontal for submission, vertical for navigation
- **Transform Calculations**: CSS calc() combining deltas with viewport units
- **Extended Axis Locking**: traversing_axis including all gesture directions
- **Smart Displacement Updates**: Only called for navigation directions
- **Direction Resolution**: Separate detection vs allowed navigation tracking
- **Effect-Based Submission**: spawn() wrapping async submission logic
- **Coordinate System**: Browser coordinates with proper delta calculations
- **State Management**: Decoupled position updates from swipe detection
- **Abstraction Patterns**: Consolidated onswipestart/move/end handlers
- **Modular Swipe Architecture**: 7 separate module files
- **Generic Swipeable Component**: EventHandler props for all 4 directions
- **SwipeConfig Simplification**: allowed_directions, distance_threshold, speed_threshold
- **EventHandler Integration**: on_swipe_left/right/up/down callbacks
- **use_effect Clearing Pattern**: Watches signal, calls handler, clears to prevent re-trigger
- **Shared State Pattern**: Parent creates Signal<SwipeState>, passes to component
- **Smooth Animation System**: is_swiping flag controlling return_animation_seconds
- **Transform Simplification**: Pure delta-based translate(xpx, ypx)
- **Axis Locking Preservation**: All 4 directions for future extensibility
- **CSS Class Removal**: Preventing position: fixed overlay issues
- **Component Reusability**: Generic design for any swipeable entity
- **Screen Migration**: Removed from 18 screens, replaced with centered divs
- **ScreenOffset Type System**: Point2D<i32> for flexible multi-dimensional positioning
- **Screen Positioning Trait**: Factory and chaining methods for positioning
- **Multi-Dimensional Layouts**: Arbitrary x/y coordinate positioning
- **Position-Aware Submissions**: Guards check screen_offset before submission

## 🏗️ Service Architecture & Dependency Injection (Mastered)
- **Generic Service Patterns**: Service<R> and Service<DR, CR> implementations
- **Repository Abstraction**: Why services depend on traits vs concrete types
- **Cross-Domain Orchestration**: DeckService and AuthService coordination patterns
- **Dual-Generic Pattern**: AuthService<AR: AuthRepository, UR: UserRepository>
- **Atomicity Understanding**: Services orchestrate, repositories handle atomic operations
- **Transaction Helper Patterns**: Helper functions taking PgTransaction
- **Tuple Returns for Atomicity**: (User, RefreshToken) maintaining atomicity
- **Service Layer Separation**: Business logic orchestration without direct transactions

## 🎯 Feature Flag Architecture & Shared Models (Expert Level)
- **Cargo Feature Flags**: Optional dependencies, feature-gated compilation, granular module control
- **Architectural Decision Making**: Evaluated approaches, chose pragmatic solution
- **Pragmatic Architecture**: Workflow efficiency over theoretical purity when appropriate
- **Granular Feature Gating**: `#[cfg(feature = "zerver")]` for fine-grained control
- **Library Design**: When to separate vs keep related code together
- **Frontend Validation Refinement**: Separating frontend-accessible types from server-only logic
- **Handler Import Separation**: Sophisticated understanding of visibility boundaries
- **Compilation Boundary Management**: Expert-level frontend/backend separation

## 🏗️ Advanced Architecture Patterns (Production Ready)
- **Configuration Management**: Production-ready config loading patterns
- **Performance Optimization**: Resolved file system access inefficiencies
- **OnceLock Pattern**: Thread-safe one-time initialization for expensive operations
- **Separate Environment Files**: Frontend/backend .env separation
- **Environment Variable Strategy**: Backend full config, frontend minimal

## 🧪 Testing & Validation (Confident)
- **Test Organization**: Clean categorization by concern
- **Edge Case Testing**: Validation of error conditions and boundaries
- **Environment Testing**: Understanding environment coupling vs testable design
- **Newtype Testing**: Testing validation at correct levels

## 🌐 External API Integration & Data Processing (Production Ready)
- **HTTP Client Setup**: reqwest with proper headers and error handling
- **JSON Processing**: serde_json parsing and complex deserialization
- **Scryfall API Understanding**: Complete MTG card data structure and patterns
- **Custom Serde Deserializers**: Flexible type handling for inconsistent API data

## 🔮 Advanced Type Systems (Confident)
- **Opaque vs Concrete Types**: `impl UserService` vs `US: UserService` trade-offs
- **Type Inference Patterns**: When inference works vs explicit parameters needed
- **Generic Constraints**: Complex trait bounds and generic programming patterns

## 🔐 Backend Session & Token Architecture
- **Session-Based Authentication**: Session struct containing user + access_token + refresh_token + both expiration timestamps
- **Rotating Token Strategy**: Security model where refresh generates new access + new refresh token, invalidating old refresh
- **Access vs Refresh Tokens**: JWTs (self-contained, 24hr) vs opaque hex strings (64-char, 14-day, hashed with SHA-256)
- **Token Hashing Strategy**: SHA-256 for refresh tokens (fast verification) vs Argon2 for passwords (slow, memory-hard)
- **Multi-Device Session Support**: Multiple refresh tokens per user (max 5) enabling concurrent device authentication
- **Token Refresh Flow**: 401 response triggers refresh, not proactive checking
- **Session Port Architecture**: AuthService and AuthRepository traits for session management operations
- **SQLx Session Operations**: create_user_and_refresh_token, create_refresh_token, use_refresh_token, delete_users_refresh_tokens
- **Token Rotation Pattern**: Delete old token, create new token atomically in use_refresh_token method
- **Session Maximum Enforcement**: SQL window functions (ROW_NUMBER() OVER PARTITION BY) for automatic oldest-token cleanup
- **Atomic Registration**: create_user_and_refresh_token wraps user + token creation in single transaction
- **Cross-Domain Session Creation**: AuthService orchestrates UserRepository + AuthRepository + JWT generation
- **Middleware Session Integration**: AuthenticatedUser extractor constructs Jwt from bearer token, validates with JwtSecret
- **Refresh/Logout Endpoints**: Complete /api/auth/refresh and /api/auth/logout POST endpoints
- **Enhanced Error Logging**: RefreshSessionError variants include user_id for security audit trails

## 📱 Frontend Session Management
- **PersistentSession Trait**: Keyring-based storage (persist, retrieve, clear methods) for iOS Keychain/Android KeyStore
- **Session Expiration Checking**: retrieve() checks refresh_token expiration and auto-clears expired sessions
- **ActiveSession Wrapper**: Type-safe wrapper ensuring validated sessions for HTTP requests
- **GetActiveSession Trait**: Three-path token handling (valid, refresh needed, re-auth needed)
- **Session Context Integration**: use_context() pattern for accessing session state in async functions
- **HTTP Client Session Patterns**: AuthClient methods handle session validation and refresh automatically

## 🃏 Card Filtering System & Modular Architecture
- **Modular Filter Architecture**: Refactored monolithic filter into focused sub-components (text, types, combat, mana, rarity, set)
- **Text Filter Component**: name_contains, oracle_text_contains, flavor_text_contains with direct filter mutation
- **Types Filter Component**: Basic type grid toggles + chip-based other-types multi-select with searchable dropdown
- **Combat Filter Component**: Power/toughness equals + range inputs with stepper controls and FilterMode enum
- **Mana Filter Component**: CMC equals/range + color identity grid (W/U/B/R/G) with equals/contains mode toggle
- **Parsing Pattern**: String signals → onblur triggers parsing closure → validates → updates CardFilterBuilder → displays errors
- **Color Identity UI**: For loop using Color::all() rendering toggle boxes, onclick modifies selected_colors signal
- **Toggle Grid UI**: Always-visible grid of clickable boxes for basic types, onclick toggles card_type_contains_any array
- **Chip-Based Multi-Select**: Selected items shown as removable chips, local signal + use_effect syncing to filter
- **Filtered Dropdown**: Type-to-search input shows top matches from resource, clicking adds to selection
- **Direct vs Reactive Patterns**: Basic types use direct filter mutation (no read/write conflict), complex use local signal
- **JSONB Operator Debugging**: Fixed color_identity_contains_any from && (array operator) to ?| (jsonb contains any text[])
- **Get Sets/Types Endpoints**: Full stack implementation returning Vec<String> for frontend filtering

## 🃏 Deck Card Management & Swipe Navigation
- **AddDeckCard Screen**: Card display, empty state, filter navigation, complete swipe-to-add workflow
- **Card Swipe Integration**: Wrapped card image in Swipeable with left (skip) and right (add) gesture handlers
- **Index-Based Card Iteration**: current_index signal tracking position without mutating list (preserves for undo)
- **Pagination Implementation**: current_offset tracking database offset, is_loading_more preventing duplicate loads
- **Load-More Threshold**: Triggers pagination when user within 10 cards of end, seamless infinite scroll
- **De-duplication with HashSet**: Tracks existing card IDs preventing duplicate cards across pagination batches
- **Pagination Reset on Filter Change**: use_effect watching filter_builder resets offset and index
- **Refresh Trigger Pattern**: Bool signal toggle forcing use_effect re-run for manual refresh
- **Pagination Exhaustion Tracking**: Signal tracking when API returns no new cards, triggers end-of-results toast
- **Toast Feedback**: toast.info/success/warning with ToastOptions for user feedback on swipe actions

## 🔀 Sort & Order Patterns
- **Sleeve Order Preservation**: HashMap loses insertion order; iterate over source data (scryfall_data) not lookup target (card_profiles) to preserve DB sort
- **Query Builder NULL Filtering**: Add WHERE conditions to exclude NULL values when sorting by nullable fields (prices, power, toughness)
- **ORDER BY Construction**: sqlx QueryBuilder with Separated for WHERE clauses, direct push for ORDER BY/LIMIT/OFFSET
- **Display Trait for Enums**: impl std::fmt::Display for user-facing enum text, call .to_string().to_lowercase() at render site
- **is_empty() Validation**: Exclude config fields (limit, offset, order_by, ascending) when checking if filter has actual conditions
