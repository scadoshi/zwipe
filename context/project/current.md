# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: Unified screen layout — all 14 screens use `.screen` fixed-frame pattern.

**Current Focus**: Enhancements and UI polish.

**Recent Achievements**:
- **Unified Screen Layout**: Replaced 5 different layout patterns across 14 screens with a single `.screen` fixed-frame layout (`position: fixed; inset: 0` + flexbox). Fixes layout shift after deck creation and iOS keyboard push issues.
  - `.page-header` and `.util-bar` changed from `position: sticky` to `flex-shrink: 0` flex items
  - `.screen-content` fills remaining space with `flex: 1; overflow-y: auto; min-height: 0`
  - Safe-area insets moved from `body` to `.screen`
  - `.card-info` changed from fixed positioning to normal flow
- **ViewDeck Simplified**: Removed commander image, simplified to Profile-style label/value rows (deck name, copy rule, commander name)
- **Generics Sweep**: All domain validator constructors accept `impl AsRef<str>` (or `impl Into<String>` for `DeckName`)
- **Deref Refactor (zerver)**: All domain newtypes implement `Deref` instead of bespoke getters
- **Unit Tests**: Full coverage on all pure domain logic (filter_cards 24, group_cards 15, copy_max 9, quantity, SwipeState 32)

---

## Top 5 Priorities

1. **CardFilter Enhancements** - Refine default filter to exclude non-playable cards

2. **Polish & UX** - Tri-toggle labels ("any" instead of "neither"), language filter refinement

3. **Deck List Redesign** - Better list styling, improved layout with utility bar

4. **Deck Composition** - Multi-copy add, CopyMax enforcement, quantity editing (see next.md)

5. **Integration Tests** - Repository tests with real PostgreSQL (longer-term)
