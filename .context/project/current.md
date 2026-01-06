# Currently Working On 🎯

Active development tasks and immediate focus areas.

**Last Updated**: Implemented computed boolean fields (`is_valid_commander`, `is_token`) with full backend/frontend integration.

**Current Focus**: Complete card management workflows (remove cards screen with undo), then polish deck screens.

**Recent Achievements**:
- Computed card fields: `is_valid_commander` and `is_token` columns in `card_profiles` table
- Commander validation (legendary creatures, vehicles with P/T, "can be your commander" text)
- Token detection using Scryfall `layout == "token"`
- CardFilter support for both flags (optional filtering)
- Commander-only search in CreateDeck screen
- Token exclusion in AddDeckCard screen
- Database indexes for fast boolean filtering
- Previous: Order By filter, card info display, sort preservation, NULL filtering, filter accordion

**Current Success**: Add card screen fully functional with commander/token filtering, sorting, card info, pagination, de-duplication, and swipe gestures.

---

## Top 5 Priorities

1. **Remove Cards Screen** - Build UI for removing cards from deck with filtering and swipe gestures

2. **Undo on Down Swipe** - Down swipe undoes last action (go back one card if skipped, remove from deck and re-show if added)

3. **View Deck Screen** - Display deck cards categorized by type with counts and organization

4. **Deck List Screen Redesign** - Better list styling, improved layout with utility bar, visual hierarchy

5. **Filter Duplicate Name Cards** - Hide cards with repeated names separated by // (likely meld/transform variants)
