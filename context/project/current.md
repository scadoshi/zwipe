# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: Completed Deref refactor across all newtypes + unit test coverage for zerver domain and zwiper SwipeState.

**Current Focus**: Bug fixes, refactoring, and polish. Unit testing phase complete.

**Recent Achievements**:
- **Deref Refactor (zerver)**: All domain newtypes now implement `Deref` instead of bespoke getters (`as_str()`, `value()`, `id()`, `max()`, `quantity()`). Updated all call sites across zerver outbound layer and zwiper HTTP client (19 files).
- **Unit Tests — zerver domain**: Full coverage on all pure domain logic:
  - `filter_cards.rs` — 24 tests (text, CMC, color identity, combat stats, flags, rarity, set, sort, pagination)
  - `group_cards.rs` — 15 tests (CardType, Cmc, Color grouping)
  - `copy_max.rs` — 9 tests (implemented previously-stubbed `#[ignore]` tests)
  - `quantity.rs` — tests for both `Quantity` and `UpdateQuantity` Deref impls
- **Unit Tests — zwiper SwipeState**: 32 tests covering all pure math in `swipe/state.rs`:
  - `distance_from_start_point`, `delta_from_start_point`, `distance_from_previous_point`
  - `milliseconds_from_previous_point`, `speed`, `calculate_return_animation_seconds`
  - `set_traversing_axis`, `set_latest_swipe`, `reset`
- **GroupCards Trait**: Domain-layer extension trait on `Vec<Card>` with 3 grouping modes
- **Deck Cards View Screen**: Grouped card list browser with in-memory filter → group pipeline

---

## Top 5 Priorities

1. **Bug Fixes** - Layout shift after deck creation, iOS keyboard push issues

2. **Generics Sweep** - Replace `&str`-only validators with `impl AsRef<str>` across domain entry points

3. **CardFilter Enhancements** - Refine default filter to exclude non-playable cards

4. **Polish & UX** - Tri-toggle labels, language filter refinement, minor UI improvements

5. **Integration Tests** - Repository tests with real PostgreSQL (longer-term)
