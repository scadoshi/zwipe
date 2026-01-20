# Complete - Frontend ✅

Production-ready frontend implementations.

---

## 🎨 Dioxus Component System

### Core Architecture
- **Component Foundation**: Function components with RSX macro
- **State Management**: Signal types with reactive patterns
- **Context API**: Global state with use_context_provider/use_context
- **Props vs Context**: Clear separation - props for parent-child, context for app-wide
- **Router Architecture**: Single HomeGuard route with conditional rendering
- **SessionProvider Pattern**: Centralized session/auth_client context

### Reactive Patterns
- **Signal Patterns**: signal() for Copy types, .read()/.write() for complex types
- **use_effect**: Dependency tracking for side effects
- **use_future**: One-time execution for background tasks
- **use_resource**: Async data fetching with three-state rendering
- **use_memo**: Computed signals deriving from other signals
- **Conditional Updates**: `if new != old` pattern preventing infinite loops

### Navigation & Routing
- **Guard Route Pattern**: Conditional redirect based on session
- **use_navigator**: Programmatic routing between screens
- **Navigator Async Constraints**: Signal + use_effect bridge for async navigation
- **Context for Route State**: Signals in context persisting across navigation

## 🔐 Authentication & Session

### Session Management
- **PersistentSession Trait**: Keyring-based storage (iOS Keychain/Android KeyStore)
- **Session Expiration**: Auto-clearing expired sessions on retrieve
- **ActiveSession Wrapper**: Type-safe validated sessions
- **GetActiveSession Trait**: Three-path token handling (valid/refresh/re-auth)
- **In-Memory Development**: Sessions work without keychain entitlements

### Auth Screens
- **Login Screen**: Generic error messaging for security
- **Register Screen**: Real-time validation with user-friendly feedback
- **Password Change**: Separate current/new password handling
- **Username/Email Change**: Inline forms with success messages
- **Logout**: Session clearing with backend token revocation
- **Utility Bar Navigation**: Consistent bottom navigation across auth screens

## 🃏 Deck Management

### Deck CRUD
- **Create Deck**: Name input, searchable commander field, CopyMax selection
- **Deck List**: Resource-based fetching with loading/success/error states
- **View Deck**: Read-only display with commander image and deck profile
- **Edit Deck**: Pre-populated form with change tracking and conditional updates
- **Delete Deck**: AlertDialog confirmation with async error handling

### Deck Composition
- **AddDeckCard Screen**: Card display with swipe-to-add workflow
- **Card Swipe Integration**: Left (skip) and right (add) gestures
- **Index-Based Iteration**: Non-mutating card navigation preserving full list
- **Pagination**: Automatic loading when within 10 cards of end
- **De-duplication**: HashSet tracking preventing duplicate displays
- **Empty State UI**: .card-shape CSS with "no cards yet" messaging
- **RemoveDeckCard Planning**: Separate screen for card removal (pending)

### Commander Search
- **Debounced Search**: 300ms delay using tokio::time::sleep
- **Dropdown UX**: Results in normal flow pushing other elements
- **Image Display**: Large Scryfall images with text fallback
- **Image Fallback Pattern**: Qualified match on ImageUris with graceful degradation
- **Commander Filter Integration**: CreateDeck search filters with `set_is_valid_commander(true)` for valid commanders only
- **Token Exclusion Integration**: AddDeckCard search excludes tokens with `set_is_token(false)` for deck building

## 🔍 Card Filtering System

### Filter Architecture
- **Modular Components**: Parent orchestrator + 7 sub-components
- **Context-Based State**: Filter Signal in app-level context
- **Direct Mutation**: Prefer filter.write() over local signal + effect
- **Accordion Organization**: Collapsible sections matching Scryfall order (text, types, mana, combat, rarity, set, sort)

### Filter Components
- **Text Filter**: Name/oracle text search with direct mutation
- **Types Filter**: Basic type grid + chip-based other-types multi-select
- **Combat Filter**: Power/toughness equals + range inputs (8 fields)
- **Mana Filter**: CMC equals/range + color identity toggles (W/U/B/R/G)
- **Rarity Filter**: Multi-select chips for all rarities
- **Set Filter**: Multi-select chips for sets
- **Artist Filter**: Chip-based multi-select with searchable dropdown (top 5 results)
- **Sort Filter**: Single-select chips for order_by + ascending/descending toggle
- **Config Filter**: Dynamic language selector (use_resource loading from API) + TriToggle for boolean config fields (playable, digital, oversized, promo, content_warning)

### Filter Patterns
- **Parsing Pattern**: String signals → onblur → validate → update builder
- **Color Mode Toggle**: Equals (exact) vs contains (any of)
- **use_effect Sync**: Watches multiple signals, builds complex filter state
- **Chip-Based Selection**: Removable chips for multi-select values
- **Filtered Dropdown**: Type-to-search showing top 5 results with duplicate prevention
- **Resource Integration**: use_resource loading card types/sets/artists/languages from API
- **LanguageCodeToFullName Trait**: Domain-layer trait converting language codes to full display names (18 languages)
- **Empty Filter Validation**: Toast warning when applying empty filter
- **Case-Insensitive Search**: Frontend .to_lowercase() for artist/set/type filtering

## 🎮 Swipe Detection & Gestures

### Core Implementation
- **Swipeable Component**: Generic wrapper with EventHandler props
- **Direction Detection**: Left/right/up/down with threshold-based resolution
- **Velocity Calculation**: Distance and speed thresholds for reliable detection
- **Axis Locking**: Prevents diagonal swiping, set on first movement
- **Touch/Mouse Events**: Cross-platform input with unified handling

### Configuration
- **SwipeConfig**: allowed_directions, distance_threshold, speed_threshold
- **EventHandler Integration**: on_swipe_left/right/up/down callbacks
- **Shared State Pattern**: Parent creates Signal<SwipeState>, passes to component
- **use_effect Clearing**: Watches latest_swipe, calls handler, clears signal

### Animation
- **Smooth Transitions**: is_swiping flag controlling return_animation_seconds
- **Transform Simplification**: Pure delta-based translate(xpx, ypx)
- **CSS Integration**: Dynamic inline styles with signal-driven values

## 🌐 HTTP Client Layer

### Client Architecture
- **Unified ApiError**: Single error enum shared with backend
- **From Trait Patterns**: Automatic (StatusCode, String) → ApiError conversion
- **Request Methods**: .json() for serialization, .bearer_auth() for auth headers
- **HTTP Verb Consistency**: Correct REST verbs across all 19 endpoints

### Domain Organization
- **Auth Client**: login, register, logout, refresh (4 methods)
- **User Client**: get_user, change_username, change_email, change_password, delete_user (5 methods)
- **Deck Client**: create_deck_profile, get_deck_profiles, get_deck, update_deck_profile, delete_deck (5 methods)
- **Deck Card Client**: create_deck_card, update_deck_card, delete_deck_card (3 methods)
- **Card Client**: get_card, search_cards, get_card_types, get_sets, get_artists, get_languages (6 methods)

### Request Patterns
- **Authenticated Requests**: Session validation → token refresh → bearer header → API call
- **Success/Error Management**: Mutually exclusive success_message and submission_error signals
- **Loading States**: is_loading Signal with spinner animation
- **Conditional Updates**: Only update session signal when token actually changed

## 🎨 UI Components & Patterns

### Reusable Components
- **TextInput**: Consolidated label, input, binding, common attributes
- **TriToggle**: 3-state Option<bool> chip selector using function pointers for flexible labels
- **AlertDialog**: Modal confirmation with styled buttons and backdrop
- **Toast**: Notification system wrapper (CSS loaded, implementation pending)
- **Accordion**: Expandable sections with @keyframes animations

### Form Patterns
- **Input Binding**: Two-way binding with Signal<String>
- **Validation Display**: Errors shown after first submit attempt
- **Error Timing**: Per-field validation with clear submission_error on success
- **Loading Feedback**: Spinning card animation during async operations
- **Success Messages**: Random positive feedback with get_random_success_message()

### Styling
- **Minimal UI Design**: Lowercase text, centered layouts, clean spacing
- **Utility Bar Standardization**: Bottom .util-bar with .util-btn across 20+ screens
- **CSS Class Patterns**: .card-shape, .type-box, .mana-box for consistent components
- **Global CSS Loading**: Component stylesheets loaded in main.rs for reliability

## 🔧 Advanced Patterns

### Resource Handling
- **Lifetime Pattern**: .value().with(|val| ...) avoiding temporary value errors
- **Three-State Rendering**: None/Some(Ok)/Some(Err) for loading/success/error
- **Direct Resource Chaining**: Resources reading other resources directly
- **Owned Data Extraction**: Clone inside closure, use outside for rendering

### Form Pre-Population
- **use_effect Pattern**: Watch resource, extract data, populate form signals
- **Change Tracking**: Separate original_* signals comparing to current values
- **Conditional Updates**: Only send changed fields to backend

### Signal Management
- **Copy vs Non-Copy**: Understanding .read()/.write() requirements
- **Signal Cloning**: When to clone Signals vs values for async blocks
- **Context Initialization**: use_signal(|| context().field()) pattern
- **Mount-Time Capture**: Signal closures run once capturing initial values

### Event Handling
- **Async Event Patterns**: spawn() calling async functions from sync handlers
- **Debounced Search**: tokio::time::sleep in spawned tasks
- **Mutable Closures**: let mut for closures modifying signals
- **EventHandler Props**: Building generic components with callback props

## 🎭 Animation & CSS

### Animation Strategies
- **@keyframes vs Transitions**: @keyframes for DOM insertion, transitions for removal
- **Conditional Rendering**: dioxus_primitives pattern requiring hybrid animation
- **Grid Animation**: grid-template-rows: 0fr → 1fr for content height
- **Transform Integration**: CSS calc() combining deltas with viewport units

### CSS Patterns
- **Global Loading Requirement**: document::Link doesn't work inside components
- **Component Stylesheets**: accordion.css, alert-dialog.css, toast.css in main.rs
- **Responsive Patterns**: max-height with overflow-y for tall content
- **Positioning**: CSS class removal preventing unintended fixed overlays

## 🔄 Async & Reactivity

### Hook Selection
- **use_effect**: Side effects with automatic dependency tracking
- **use_future**: One-time background tasks without dependency tracking
- **use_resource**: Async data fetching that refetches on dependency changes
- **spawn()**: Fire-and-forget async tasks breaking dependency tracking

### Debugging Patterns
- **Infinite Loop Recognition**: Read + write same signal = exponential explosion
- **Conditional Updates**: Preventing unnecessary re-runs
- **Strategic Logging**: Track effect re-runs and exponential growth
- **Signal Reactivity**: Context solves cross-route persistence issues

## 📱 Platform Integration

### Configuration
- **Build Scripts**: build.rs with cargo:rustc-env directives
- **Compile-Time Variables**: env!() macro for BACKEND_URL
- **Frontend Logging**: tracing_subscriber with RUST_LOG configuration
- **Infallible Config**: Panic on invalid environment (better than run with bad config)

### Development Patterns
- **Mock Data**: Spoof trait for development without backend dependencies
- **Environment Files**: Separate .env for frontend configuration
- **Hot Reload**: dx serve with automatic rebuilds

## 🗂️ Code Organization

### Architecture
- **Hexagonal Frontend**: inbound/ui/, outbound/client/, domain/ layers
- **Screen Hierarchies**: Separate auth/ and app/ directories
- **Component Modularization**: Focused files for reusable components
- **Domain Integration**: Shared backend types (Username, EmailAddress, Password)

### File Structure
- **Modular Filters**: filter/ directory with sub-components
- **Screen Organization**: One file per screen with clear naming
- **Shared Models**: Feature-flagged types from zwipe library
- **Client Organization**: Domain-based folders for HTTP methods
