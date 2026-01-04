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

---

## Bugs

1. **Deck List Nav Bug** - First deck creation navigates back but deck doesn't appear until navigating back again

2. **Set Filter Broken** - Not returning any results

3. **Create Deck Layout** - Commander image pushes "deck name" label into header area
