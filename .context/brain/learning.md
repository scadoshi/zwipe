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
- **EventHandler Limitations**: EventHandler props with complex multi-statement closures fail SuperFrom trait bounds—can't capture mutable references and perform multi-statement logic
- **Function Pointer Workaround**: Pass fn(&mut T) instead of EventHandler closures to components for multi-statement logic—invoke directly without trait constraints
- **TriToggle Function Pointer Pattern**: Component accepts fn getters/setters as props, invokes directly in onclick handlers—avoids EventHandler SuperFrom issues entirely
- **Function Pointer Comparison Warning**: `unpredictable_function_pointer_comparisons` lint warns addresses not guaranteed unique—causes occasional extra re-renders but no correctness issues
- **Suppressing Function Pointer Warning**: Use #[allow(unpredictable_function_pointer_comparisons)] on component when using fn pointers for props
- *Note: Hook selection (effect vs future vs resource) is critical - wrong choice causes infinite loops or missing reactivity*

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
