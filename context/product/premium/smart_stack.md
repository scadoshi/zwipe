# Smart stack ordering + taste profile

**Tier: premium.** The single most premium-feeling feature because it upgrades
the thing users already do — the swipe stack — rather than adding a screen
they have to find.

## Smart stack ordering

Instead of filter-order, rank the stack by "fits this deck":

- **Tag match** — boost cards whose mechanical categories match the deck's
  tags (`deck_tags.md`). Cheapest signal, available first.
- **Co-occurrence stats** — "decks with your commander also run X," computed
  from our own deck corpus as it grows.
- **Recommander integration** — if the recommander.cards integration lands
  (see `progress/backlog.md`), its recommendations become a ranking signal,
  cacheable per commander.
- **Taste profile** (below) — deprioritize what this user always rejects.

## Taste profile

"You left-swipe counterspells 90% of the time" → auto-deprioritize. Left-swipe
data is something Moxfield structurally cannot have — it only exists because
the core loop is swiping.

### Storage design — counters, not history (worked out 2026-06-10)

The profile needs **running aggregates, not raw swipe history**. Never run a
big aggregation job; maintain counters incrementally:

- `(user_id, mechanical_category, direction) → count` — incremented per swipe,
  O(1) write, a few hundred rows per user. The taste profile is a direct read.
  The ~50-category vocabulary is the aggregation bucket (vs 35k cards).
- `(user_id, oracle_id) → counts` — for card-level "stop showing me Sol Ring,
  I've rejected it 14 times." Bounded by cards the user has *seen* —
  thousands of rows per active user, not millions.
- **Global per-card aggregates** — one row per card, rejection/acceptance
  counts across all users. Trivially small, and valuable independent of the
  taste profile (recommendation-partner feedback signal, own ranking stats).

Raw event log: keep it but cap it — append-only, partitioned by month, rolled
into the counters, partitions dropped after 6–12 months. ~100 bytes/row, so
1M swipes/month ≈ 100MB before pruning. It exists only as backfill insurance
(re-derive counters if the taxonomy changes); option to skip it entirely if
that flexibility isn't worth it.

### Client batching

zwiper accumulates swipes locally and flushes a summary every N swipes or on
session end — not one HTTP write per flick. Matters for battery and server
load regardless of the taste profile.

## Depends on

Deck tags + card categories for the tag-match signal; swipe-event ingestion
(new — nothing records left swipes today); IAP entitlements to gate ranking.
