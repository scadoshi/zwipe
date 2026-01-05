# Next Immediate Priorities 🎯

Planned work after completing current tasks.

---

## Enhancements

1. **Remove Card Screen** - Build UI for removing cards from deck with filtering and swipe gestures

2. **Undo on Down Swipe** - Down swipe undoes last action:
    - If last action was skip: go back one card
    - If last action was add: remove card from deck and re-show for decision
    - Track action history for undo capability

3. **Commander Search Validation** - Filter to only return valid commanders:
    - Legendary creatures
    - Legendary vehicles/spacecraft with power/toughness (flagship, starship subtypes)
    - Cards with "can be your commander" text (some planeswalkers, special cards)

4. **Deck Card Search** - Exclude token cards from results

5. **Deck List Screen Redesign** - Better list styling, improved layout with utility bar, visual hierarchy

6. **View Deck Screen** - Display deck cards categorized by type (creatures, spells, lands) with counts

7. **Deck Cards Browser** - Full-screen card viewer for deck contents (MAJOR)
    - Swipeable navigation between cards in deck
    - Stack-like visual format showing card depth
    - Categorized sections (by type, category, mana cost, etc.)
    - Swipe between categories or within category
    - Card count indicators per category
    - Readable card display with proper image sizing

8. **Filter Duplicate Name Cards** - Hide cards with repeated names separated by // (e.g. "Satya Aetherflux Genius // Satya Aetherflux Genius"). Investigate what these cards are (likely meld/transform variants with identical faces).

9. **Refresh Excludes Deck Cards** - When clicking refresh, newly fetched cards should still exclude cards already in the current deck

10. **Cross-Deck Card Ownership Indicator** - Highlight cards that are already in other decks:
    - Visual indicator when browsing cards for one deck (e.g., "In 2 other decks")
    - Show which decks contain the card
    - Helps users avoid over-buying duplicate cards

11. **Toast: Card in Other Decks** - When viewing a card that exists in other decks, show a toast notification:
    - Message options: "You use this card in other decks" or "You seem to like this card"
    - Only show when the card is being viewed for deck-building (add/remove context)

12. **Deck Profile/Grid View** - View all user decks in a grid layout on profile:
    - Visual cards showing deck commander or key art
    - Quick deck stats (card count, mana curve preview)
    - Launch to build/edit deck from grid

13. **EDHREC Integration** - Sort and filter cards by deck synergies:
    - Fetch synergy data from EDHREC API for current commander
    - Sort by "synergy score" or "popularity in decks with this commander"
    - Highlight cards frequently paired with the current commander
    - Helps players discover strong deck synergies

---

## Bugs

1. **Deck List Nav Bug** - First deck creation navigates back but deck doesn't appear until navigating back again

2. **Set Filter Broken** - Not returning any results

3. **Create Deck Layout** - Commander image pushes "deck name" label into header area
