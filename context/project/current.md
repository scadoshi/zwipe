# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: Basic land singleton exemption, ViewDeck histogram charts, layout improvements.

**Current Focus**: Deck import, multi-copy add flow.

**Recent Achievements**:
- **Basic Land Singleton Exemption**: Basic lands can increment qty in singleton decks; `change_quantity` takes `is_basic_land` flag to skip copy-max checks; non-basic singleton cards show "remove" button instead of -/+; EditDeck `max_entry_quantity` ignores basic lands for truncation warning
- **ViewDeck Histogram Charts**: Types and colors rendered as histogram bar charts (same pattern as mana curve) with 5-letter abbreviated labels, counts above bars, rounded containers, pipe separators between labels
- **Layout Improvements**: ViewDeck and Profile screens widened from `container-sm` to `max-width: 40rem` inline style; spacing reduced from `mb-4` to `mb-2`
- **Qty Column Always Visible**: Removed `.no-qty` CSS class; qty column always shows for all deck types
- **CopyMax Truncation on UpdateDeckProfile**: Backend `truncate_deck_card_quantities` clamps card quantities when copy_max becomes more restrictive, atomically within the same transaction
- **Deck Metrics**: `DeckMetrics` struct + `ComputeMetrics` trait in deck domain — generic over `IntoIterator<Item = &Card>`, single-pass CMC histogram, type/color distribution, avg CMC, land counts (7 tests)

---

## Top 5 Priorities

1. **Deck Import** - Paste text list or Archidekt/Moxfield URL to bulk-add cards (CopyMax enforcement now complete)

2. **Multi-Copy Add Flow** - Quantity picker on swipe-right for standard decks

3. **Deck List Redesign** - Better list styling, improved layout with utility bar

4. **Mana Pip Balance** - Pips consumed vs produced per color for mana base balancing

5. **Integration Tests** - Repository tests with real PostgreSQL (longer-term)
