# Next Immediate Priorities 🎯

Planned work after completing current tasks.

---

---

## Enhancements

1. **Remove Card Screen** - Build UI for removing cards from deck with filtering and swipe gestures

2. **Deck List Screen Redesign** - Better list styling, improved layout with utility bar, visual hierarchy

3. **Deck Cards Browser** - Full-screen card viewer for deck contents (MAJOR)
    - Display deck cards categorized by type (creatures, spells, lands) with counts
    - Swipeable navigation between cards in deck
    - Stack-like visual format showing card depth
    - Categorized sections (by type, category, mana cost, etc.)
    - Swipe between categories or within category
    - Card count indicators per category
    - Readable card display with proper image sizing

4. **CardFilter Enhancements (Serve Only Playable Cards)** - Continue refining default CardFilter to exclude non-playable/non-standard cards.

### Pending Improvements
   - **tri-toggle labels** - Improve clarity of boolean filter options
     - Current: "show / hide / neither"
     - Proposed: "show / hide / any" (or "no filter")
     - Applies to: playable, digital, oversized, promo, content_warning filters

   - **Language filter refinement** - Hide language selector when using OracleCards
     - Backend infrastructure complete and ready
     - Frontend: Remove language chip UI from config.rs when OracleCards enabled
     - Keeps all backend support for future language needs

### Set Type Filter (Phase 2)
   - **set_type filter** - Filter by set classification
     - Domain: `SetType` enum or string filter
     - Default: hide `funny`, `memorabilia`, `token` set types
     - Not exposed on frontend initially

### Legality Filter (Phase 3 - Complex)
   - **legality/format filter** - Filter by format legality
     - Uses existing `Legality` and `LegalityKind` enums
     - Requires special UI handling (format + legal status)
     - Deferred - needs design work

5. **Cross-Deck Card Ownership Indicator** - Highlight cards that are already in other decks:
    - Visual indicator when browsing cards for one deck (e.g., "In 2 other decks")
    - Show which decks contain the card
    - Helps users avoid over-buying duplicate cards

6. **Toast: Card in Other Decks** - When viewing a card that exists in other decks, show a toast notification:
    - Message options: "You use this card in other decks" or "You seem to like this card"
    - Only show when the card is being viewed for deck-building (add/remove context)

7. **EDHREC Integration** - Sort and filter cards by deck synergies:
    - Fetch synergy data from EDHREC API for current commander
    - Sort by "synergy score" or "popularity in decks with this commander"
    - Highlight cards frequently paired with the current commander
    - Helps players discover strong deck synergies

---

## Bugs

1. **Deck List Nav Bug** - First deck creation navigates back but deck doesn't appear until navigating back again

2. **Create Deck Layout** - Commander image pushes "deck name" label into header area
