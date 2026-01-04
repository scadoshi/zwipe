# Currently Working On 🎯

Active development tasks and immediate focus areas.

**Last Updated**: Added refresh button and end-of-results toast to add card screen. Fixed filter bugs, improved toast UX, refactored filter components to use direct writes.

**Current Focus**: Complete card management workflows (add/remove cards), then polish deck screens and implement sorting.

**Recent Achievements**:
- Refresh button for card search (bool toggle forcing use_effect re-run)
- End-of-results toast with pagination exhaustion tracking
- Filter bug fixes (set filter, session refresh loop, chip selection race condition)
- Toast UX improvements (stacking limits, z-index, shorter durations)
- Stepper controls for power/toughness filters

**Current Success**: Add card screen fully functional with pagination, de-duplication, swipe gestures, refresh, and end-of-results feedback.

---

## Top 5 Priorities

1. **Remove Cards Screen** - Build UI for removing cards from deck with filtering and swipe gestures

2. **Order By Filter** - Add sorting option to filters pane (Name, Cmc, Power, Toughness, Rarity, ReleasedAt, Price, Random)

3. **Commander Search Validation** - Filter to only return valid commanders (legendary creatures, vehicles with P/T, "can be your commander" text)

4. **Deck Card Search** - Exclude token cards from results

5. **View Deck Screen** - Display deck cards categorized by type with counts and organization
