# Currently Working On 🎯

Active development tasks and immediate focus areas.

**Last Updated**: Implemented Language filter and Configuration filters UI with TriStateFilter component.

**Current Focus**: Complete card management workflows (remove cards screen with undo), then polish deck screens.

**Recent Achievements**:
- Language enum with 11 languages (English, Spanish, French, German, Italian, Portuguese, Japanese, Korean, Russian, Simplified/Traditional Chinese)
- Language filter in CardFilter (default: English only)
- Configuration filter accordion in frontend with TriStateFilter component
- TriStateFilter reusable component for Option<bool> (yes/no/any chips)
- Frontend exposure of config filters: is_playable, digital, oversized, promo, content_warning
- CardFilterBuilder getters for all config/flag fields
- Previous: is_valid_commander, is_token computed fields, Order By filter, card info display

**Current Success**: Add card screen fully functional with language filtering, config filter toggles, commander/token filtering, sorting, card info, pagination, de-duplication, and swipe gestures.

---

## Top 5 Priorities

1. **Remove Cards Screen** - Build UI for removing cards from deck with filtering and swipe gestures

2. **Undo on Down Swipe** - Down swipe undoes last action (go back one card if skipped, remove from deck and re-show if added)

3. **View Deck Screen** - Display deck cards categorized by type with counts and organization

4. **Deck List Screen Redesign** - Better list styling, improved layout with utility bar, visual hierarchy

5. **Filter Duplicate Name Cards** - Hide cards with repeated names separated by // (likely meld/transform variants)
