# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: Completed GroupCards trait + Deck Cards View screen. Core deck management feature-complete.

**Current Focus**: Testing coverage, bug fixes, and polish. Main functionality is in place.

**Recent Achievements**:
- **GroupCards Trait**: Domain-layer extension trait on `Vec<Card>` partitioning cards into labelled groups
  - Three grouping modes: CardType (8 buckets by type_line), Cmc (0–6+), Color (WUBRG + multicolor + colorless)
  - Bucket-based O(n) classification with fixed-order labels, empty group filtering
  - Same extension-trait-on-`Vec<Card>` pattern as `FilterCards`
- **Deck Cards View Screen**: Grouped card list browser for deck contents
  - `filter_by` → `group_by` in-memory data pipeline (no server round-trips after mount)
  - Group-by chips (type / cmc / color), column headers, expandable card rows
  - Commander fetched separately via get_deck_profile → get_card, deduplicated into card list
  - Active filter warning toast on mount, per-screen scroll containers for mobile
  - Full filter panel (8 accordion sections) shared with add/remove screens
- **Deck Profile View Rework**: Name + copy rule side-by-side, commander image below with 42vh sizing
- **Button Label Standardization**: Shortened labels across all deck screens (create, save, add, remove, view, delete)
- **Filter Behavior Refinement**: Add screen requires filter (toast warning), remove/view screens allow empty filter

---

## Top 5 Priorities

1. **Testing Coverage** - Integration tests for repository patterns, domain traits, and frontend components

2. **Bug Fixes** - Address layout shift after deck creation and iOS keyboard push issues

3. **CardFilter Enhancements** - Continue refining default filter to exclude non-playable cards

4. **Performance Optimization** - Review patterns for optimization opportunities

5. **Polish & UX** - Tri-toggle labels, language filter refinement, minor UI improvements
