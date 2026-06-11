# Collection tracking

**Tier: FREE — deliberately (decided in brainstorm 2026-06-10).** This is a
moat play, not a revenue play. Its job is retention and feeding premium
features, not being one.

## Why free

Once someone's 2,000-card collection lives in Zwipe, the switching cost to
Moxfield/Archidekt is enormous — data gravity. Charging for it keeps the moat
empty. The feature is "basic" as a paid offering anyway; its value is what it
unlocks:

- **Shopping list** = deck minus collection → the price-alert engine's input
  (`price-intelligence.md`). This is where collection data converts to
  premium value and affiliate revenue.
- **"Cards I own" swipe filter** — build only from what I own. **DECIDED
  2026-06-10: free** — it's a filter, filters are free.
- **Owned-percentage on decks** — "you own 78% of this deck ($63 to finish)"
  blends collection + prices into one motivating number.

## Implementation notes

- Schema: `(user_id, oracle_id, quantity)` (+ optional printing/finish detail
  later — start oracle-level, printings are a power-user refinement).
- Entry UX is the hard part on mobile: search-and-add is table stakes; bulk
  paste import reuses the text importer's parser; camera scanning is a big
  separate project (backlog, do not scope in).
- Storage is cheap; this is free-tier load we accept on purpose.

## Sequencing

Build after the price snapshot table exists, so owned-percentage and shopping
list land with dollar amounts attached from day one — the motivating version
of both numbers.
