# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: CopyMax truncation enforcement, qty column, truncation warning dialog.

**Current Focus**: Deck import (CopyMax enforcement complete), multi-copy add flow.

**Recent Achievements**:
- **CopyMax Truncation on UpdateDeckProfile**: Backend `truncate_deck_card_quantities` clamps card quantities when copy_max becomes more restrictive, atomically within the same transaction
- **Truncation Warning Dialog**: EditDeck fetches full deck via `get_deck`, computes `would_truncate` memo comparing actual max entry quantity against new limit, shows AlertDialog only when cards actually exceed the new copy_max
- **Qty Column in ViewDeckCard**: Added qty column to card row grid; omitted for singleton decks via `.no-qty` CSS modifier toggling grid-template-columns
- **+/- Quantity Controls**: ViewDeckCard expanded rows have quantity controls with optimistic updates, CopyMax enforcement, and delete-on-zero
- **Deck Metrics**: `DeckMetrics` struct + `ComputeMetrics` trait in deck domain — generic over `IntoIterator<Item = &Card>`, single-pass CMC histogram, type/color distribution, avg CMC, land counts (7 tests)
- **ViewDeck Stats**: Fetches full deck, renders stats section, ASCII mana curve (8-row scaled bars), type/color distributions; metrics section hidden when empty

---

## Top 5 Priorities

1. **Deck Import** - Paste text list or Archidekt/Moxfield URL to bulk-add cards (CopyMax enforcement now complete)

2. **Multi-Copy Add Flow** - Quantity picker on swipe-right for standard decks

3. **Deck List Redesign** - Better list styling, improved layout with utility bar

4. **Mana Pip Balance** - Pips consumed vs produced per color for mana base balancing

5. **Integration Tests** - Repository tests with real PostgreSQL (longer-term)
