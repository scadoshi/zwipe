# Currently Working On 🎯

Active development tasks and immediate focus areas.

**Last Updated**: Implemented Order By filter with full backend/frontend integration, card info display, and fixed sort order preservation bug.

**Current Focus**: Complete card management workflows (add/remove cards with undo), then polish deck screens.

**Recent Achievements**:
- Order By filter (sort by name, cmc, power, toughness, rarity, release date, prices, random)
- Card info display (prices, release date, artist) on add card screen
- Sort order preservation fix (sleeve function now preserves DB order)
- NULL filtering for sorted fields (excludes cards without price/power when sorting by those)
- Empty filter toast warning ("try adding a filter")
- Filter accordion reordered to match Scryfall (text, types, mana, combat, rarity, set, sort)

**Current Success**: Add card screen fully functional with sorting, card info, pagination, de-duplication, and swipe gestures.

---

## Top 5 Priorities

1. **Remove Cards Screen** - Build UI for removing cards from deck with filtering and swipe gestures

2. **Undo on Down Swipe** - Down swipe undoes last action (go back one card if skipped, remove from deck and re-show if added)

3. **Commander Search Validation** - Filter to only return valid commanders (legendary creatures, vehicles with P/T, "can be your commander" text)

4. **Deck Card Search** - Exclude token cards from results

5. **View Deck Screen** - Display deck cards categorized by type with counts and organization
