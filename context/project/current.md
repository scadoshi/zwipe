# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: Deck metrics, ViewDeck stats, UI polish across screens.

**Current Focus**: Deck composition correctness (CopyMax enforcement) then deck import.

**Recent Achievements**:
- **Deck Metrics**: `DeckMetrics` struct + `ComputeMetrics` trait in deck domain — generic over `IntoIterator<Item = &Card>`, single-pass CMC histogram, type/color distribution, avg CMC, land counts (7 tests)
- **ViewDeck Stats**: Fetches full deck, renders stats section, ASCII mana curve (8-row scaled bars), type/color distributions; metrics section hidden when empty
- **ViewDeckCard Commander Pinning**: Commander isolated in its own always-visible "commander" group — immune to filter/group operations, stripped from groupable deck_cards
- **ViewDeckCard Detail Polish**: Type line first + greyed, equal 0.5rem spacing, rarity | set, mana cost removed; card-row-detail entry animation (180ms ease-out fade + 4px slide)
- **Push Flow Fix**: Commander excluded from add results via `deck_profile.commander_id` in exclusion HashSet
- **Commander Search Fix**: EditDeck commander search now filters `is_valid_commander: true`
- **Filter UX**: "filter cleared" (info) / "filter already cleared" (warning) toasts on all three filter screens; "filters" → "filter" label
- **card-shape**: Sized to real card ratio (42vh, 5/7 aspect-ratio, proportional border-radius)
- **UI Polish**: Profile button labels shortened, logout at end; home button order; ViewDeck button order and add/remove labels

---

## Top 5 Priorities

1. **CopyMax Enforcement** - Backend defends, frontend asserts; deck limits currently unenforced (prerequisite for deck import)

2. **Deck Import** - Paste text list or Archidekt/Moxfield URL to bulk-add cards (needs CopyMax enforcement first)

3. **Multi-Copy Add Flow** - Quantity picker on swipe-right for standard decks

4. **Deck List Redesign** - Better list styling, improved layout with utility bar

5. **Integration Tests** - Repository tests with real PostgreSQL (longer-term)
