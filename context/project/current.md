# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: Completed Remove Card screen + card_profiles schema cleanup.

**Current Focus**: Deck card browsing and remaining deck management workflows.

**Recent Achievements**:
- **Remove Card Screen**: Full swipe-based card removal screen implemented
  - Two-signal display model (`deck_cards` source of truth + `displayed_cards` filtered view)
  - Two-effect reactivity split preventing filter re-triggers on card removal
  - Local `RemoveAction` enum (Skip / Removed(Box<Card>)) for undo stack
  - Undo remove: `create_deck_card` backend restore + vec re-insertion at current index
  - Undo skip: decrement `current_index` only
  - Filter panel (all 8 accordion sections) with in-memory `FilterCards::filter_by`
  - `is_empty()` guard preventing config defaults from silently filtering deck cards
  - Animation direction branching in `onanimationend` (right = remove, left = advance index)
- **FilterCards Trait**: In-memory `Vec<Card>` filtering mirroring SQL adapter logic, enabling filter without server round-trip
- **card_profiles Shared PK**: Removed surrogate `id` column, promoted `scryfall_data_id` to PRIMARY KEY
  - `CardProfile.scryfall_data_id` is now the single identifier (no more dual-UUID ambiguity)
  - Fixed misleading `card_profile_id` param names on 3 frontend client traits
  - Resolved the "deck card not found" class of bug at the schema level

---

## Top 5 Priorities

1. **Deck Cards Browser** - Full-screen card viewer for deck contents (MAJOR)

2. **Bug Fixes** - Address layout shift after deck creation and iOS keyboard push issues

3. **Testing Coverage** - Integration tests for repository patterns

4. **Performance Optimization** - Review patterns for optimization opportunities

5. **API Documentation** - Consider OpenAPI/Swagger generation from documented handlers
