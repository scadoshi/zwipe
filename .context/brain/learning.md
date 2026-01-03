# LEARNING - Recently Introduced, Needs Guidance 📚

Recently encountered concepts requiring guidance and practice.

---

## 🎨 iOS-Specific CSS & Modern Layout Properties (Recently Introduced)
- **overflow: clip vs overflow: hidden**: Vague understanding - knows overflow: hidden allows programmatic scrolling, overflow: clip prevents all scrolling. Needs hands-on practice to solidify differences and use cases.
- **overflow-clip-margin**: Not yet understood - controls where clipping begins relative to element boundary, can be combined with env() variables for precise control
- **env(safe-area-inset-*) Variables**: Conceptual awareness - knows these are CSS environment variables that query device for safe zone dimensions (top/bottom/left/right), but not confident implementing without reference
- **iOS CSS Debugging Techniques**: Not yet familiar - transform: translateZ(0) for GPU acceleration fixing keyboard bugs, sticky vs fixed positioning behavior on iOS, notch overlap troubleshooting patterns
- *Note: Honest self-assessment shows learning mindset - understands existence of these tools, will gain confidence through repeated application and experimentation*

## 🔧 Clippy Linting & Code Quality
- **Workspace Clippy Configuration**: Established 26 workspace-level lints across quality, safety, performance, and code quality categories
- **Clippy Lint Categories**: Basic quality (redundant_clone, needless_borrow), unwrap/panic prevention (unwrap_used, expect_used, panic, indexing_slicing), performance (needless_collect, clone_on_ref_ptr, or_fun_call), code quality (too_many_arguments, unused_async, dbg_macro)
- **Clippy Fix Workflow**: cargo clippy identifies issues → --fix for auto-fixable → git diff review → commit → iterate on manual fixes
- **Easy vs Complex Warnings**: Auto-fixable (single_char_pattern, needless_borrow, or_fun_call) vs requires refactoring (too_many_arguments, unwrap_used, panic)
- **Builder Pattern Requirements**: SearchCards (17 params) and SyncMetrics (10 params) trigger too_many_arguments lint requiring architectural refactoring
- **Copy Type Optimization**: Removed 100+ unnecessary & references for Uuid leveraging Copy trait for cleaner signatures
- **Structured Logging**: Migrated println! to tracing::info! for proper log level control and structured output
- *Note: Easy warnings resolved (single_char, or_fun_call, needless_borrow), complex refactoring (builders, unwrap elimination, panic removal) up next*

## 🎨 Dioxus Reactivity & Async Patterns (MAJOR BREAKTHROUGH)
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
- **Signal Navigation Bug**: Signals passed as route parameters don't persist across navigation - Dioxus may create new instances or fail to track
- **Context Solution**: App-level context (use_context_provider in spawn_upkeeper) solves cross-route Signal persistence
- **Router Signal Limitations**: Signals aren't serializable, can't be reliably used as route parameters despite compiling
- **Debugging Signal Reactivity**: Check if Signal updates in one component, navigation completes, but reading component shows stale data = context issue
- **CRITICAL: resource() vs resource.read()** — `resource()` clones the value (safe across await), `resource.read()` returns borrow guard (UNSAFE across await). Clippy's await_holding_lock only triggers on `.read()` guards, not on cloned values.
- **Direct Resource Chaining**: Resources can read other resources directly with `resource()` - no intermediate signals needed. Pattern: `let Some(Ok(Data { field, .. })) = other_resource() else { return Ok(None) };`
- **When to Use Effects**: Edit screens need effects to populate form signals, but view screens can render directly from resources with pattern matching in RSX.
- **Resource Separation of Concerns**: Keep resources pure (just fetch), use effects for side effects (populate signals), or render directly from resources for read-only display.
- *Note: Hook selection (effect vs future vs resource) is critical - wrong choice causes infinite loops or missing reactivity*

## 🔐 Backend Session & Token Architecture (Complete) ✅
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

## 📱 Frontend Session Management (Complete) ✅
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

## 🃏 Card Filtering System & Modular Architecture (Phase 1 Complete)
- **CardType Enum**: Basic MTG card types (Instant, Sorcery, Creature, Enchantment, Artifact, Planeswalker, Land) with Display trait
- **WithCardTypes Trait**: Trait on Vec<CardType> providing with_all_card_types() factory method for UI population
- **SearchCards Domain Integration**: Frontend uses backend SearchCards struct directly with type_line_contains_any and card_type_contains_any fields
- **GET→POST Refactor**: Changed search_cards endpoint from GET with query params to POST with JSON body solving complex type serialization
- **Backend SQL Filtering**: Built OR conditions for type_line_contains_any and card_type_contains_any in dynamic QueryBuilder
- **Get Card Types Endpoint**: GET /api/card/types extracts ~670 distinct subtypes using STRING_TO_ARRAY, UNNEST, TRIM, DISTINCT
- **Modular Filter Architecture**: Refactored 239-line monolithic filter into 5 focused sub-components (text, types, printing, mana, stats)
- **Text Filter Component**: Simple name_contains input with direct filter mutation pattern
- **Types Filter Component**: Basic type grid toggles + chip-based other-types multi-select with searchable dropdown
- **Combat Filter Component**: Power/toughness equals + range inputs (8 fields total) with i32 parsing closures and error handling
- **Mana Filter Component**: CMC equals/range + color identity grid (W/U/B/R/G) with equals/contains mode toggle button
- **Parsing Pattern**: String signals → onblur triggers parsing closure → validates i32/f64 → updates CardFilterBuilder → displays errors
- **Color Identity UI**: For loop using Color::all() rendering toggle boxes, onclick modifies selected_colors signal
- **Color Mode Toggle**: Single button cycling between "equals" (exact match) and "contains" (any of) with labeled interface
- **use_effect Color Sync**: Watches selected_colors and color_mode, converts to Colors type, sets appropriate CardFilterBuilder field
- **Color Domain Enhancement**: long_name() returns "White"/"Blue"/etc for UI, short_name() returns "W"/"U"/etc for API
- **Color Display Impl**: Uses long_name() so "{color}" in RSX shows full names automatically
- **Colors Collection Methods**: to_short_name_vec() for SQL queries, to_long_name_vec() for potential UI needs
- **Toggle Grid UI**: Always-visible grid of clickable boxes for basic types, onclick toggles card_type_contains_any array
- **Chip-Based Multi-Select**: Selected other types shown as removable chips, local signal + use_effect syncing to filter.type_line_contains_any
- **Filtered Dropdown**: Type-to-search input shows top 5 matching subtypes from resource, clicking adds to selected_other_types
- **Resource Integration**: use_resource calling get_card_types on mount, providing Vec<String> for frontend filtering
- **Direct vs Reactive Patterns**: Basic types use direct filter mutation (no read/write conflict), other types use local signal (avoid conflict in chip rendering)
- **Filter Naming Consistency**: Renamed card_filter → filter throughout codebase for cleaner, more consistent naming
- **JSONB Operator Debugging**: Fixed color_identity_contains_any from && (array operator) to ?| (jsonb contains any text[])
- **SQL Type Conversion**: Created to_short_name_vec() to convert Colors → Vec<String> for proper text[] parameter binding
- **CSS .mana-box Styling**: Matches .type-box pattern with selected state, hover effects, flex-wrap for responsive layout
- **Get Sets Endpoint**: Full stack implementation (domain error, ports, service, SQLx with SELECT DISTINCT set_name, frontend client) returning Vec<String>
- **Pragmatic Newtype Decisions**: Recognized when newtypes add value (validation, type safety) vs unnecessary ceremony (Set names are just display strings)
- *Note: Phase 1 complete (Text, Types, Combat, Mana), Set endpoint done, Rarity newtype next*

## 🃏 Deck Card Management & Swipe Navigation (Complete)
- **AddDeckCard Screen Foundation**: Built with card display, empty state ("no cards yet"), filter navigation, and complete swipe-to-add workflow
- **Empty State UI**: Created `.card-shape` CSS class (25vh × 35vh, centered with `margin: 0 auto`, flex-centered content)
- **Modular Filter Refactoring**: Broke 239-line monolithic filter into parent orchestrator + 5 sub-components (text, types, printing, mana, stats)
- **Router Structure**: Separate routes for AddDeckCard and AddDeckCardFilter with deck_id, filter Signal, and cards Signal
- **Filter Naming**: Renamed card_filter → filter throughout for consistency and clarity
- **Text Filter**: Name search input with direct filter.write().name_contains mutation
- **Types Filter**: Basic type grid + chip-based other-types multi-select, combining direct mutation with local signal patterns
- **Signal Pattern Discovery**: signal() only works for Copy types; SearchCards requires .read()/.write() due to Vec/String fields
- **Direct vs Reactive Balance**: Use direct filter access when possible; local signal + use_effect only for read/write conflicts
- **Image Validation**: Search results filtered to exclude cards without large images for display
- **Helper Methods**: HttpSearchCards::blank(), is_blank(), PartialEq derive; Optdate::is_changed()
- **Architecture Pattern**: Sub-components receive shared filter Signal, mutate directly, parent orchestrates search execution
- **Card Swipe Integration**: Wrapped card image in Swipeable component with left (skip) and right (add to deck) gesture handlers
- **Index-Based Card Iteration**: current_index signal tracking position in cards list without mutating the list (preserves for undo)
- **Pagination Implementation**: current_offset tracking database offset, is_loading_more preventing duplicate loads, pagination_limit=100 matching backend
- **Load-More Threshold**: Triggers pagination when user within 10 cards of end, seamless infinite scroll experience
- **De-duplication with HashSet**: Tracks existing card IDs using std::collections::HashSet preventing duplicate cards across pagination batches
- **Filter-Based De-duplication**: Filters cards missing large images AND cards already in HashSet before adding to display list
- **Pagination Reset on Filter Change**: use_effect watching filter_builder resets current_offset and current_index ensuring fresh results
- **Non-Mutating Iteration Benefits**: Keeping full card list intact enables potential undo, rewind, or history features in future
- **RemoveDeckCard Planning**: Will need separate filter implementation filtering deck's cards instead of all cards
- **Three-Layer Vision**: Filter (separate screen), card display (center), metrics (down) with navigation between
- *Note: Complete swipe-to-add workflow functional, exit animations and deck card filtering planned next*

## 🔮 Advanced Rust Patterns
- **Advanced Async Patterns**: Complex Future handling, async streaming, async iterators
- **Type-Level Programming**: Advanced trait constraints, generic programming patterns
- **Complex Lifetime Management**: Advanced lifetime parameters and borrowing patterns

## 🚀 Production Deployment & Scaling
- **Containerization**: Docker, Kubernetes deployment strategies
- **Monitoring & Observability**: Metrics collection, logging, distributed tracing
- **Performance Tuning**: Query optimization, connection pool sizing, caching strategies
- **Rate Limiting**: Request throttling, abuse prevention mechanisms

## 🎮 MTG-Specific Business Logic
- **Format Validation**: Standard/Modern legality checking, card legality rules
- **Deck Rules**: 60-card minimums, 4-card limits, sideboard validation
- **Card Interactions**: Rules engine for card interactions and abilities
