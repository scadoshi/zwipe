# Wildcard slot — exposure floor for deep cards

**Status: BUILT 2026-07-07, dev-verified end-to-end, on main pending deploy.
Server-only; no client change, no migration, no `.sqlx` change. Two as-built
deltas from the original sketch (offset math, determinism sort) — see
[`server.md`](server.md) "As-built deltas".**

**What this builds, in one sentence:** each 25-card hand the default ordering
deals (see [`../suggestion_signal.md`](../suggestion_signal.md), band
shuffle — `BAND_SIZE = 25`, one page = one hand) reserves a slot for a card
drawn from far deeper in the ranking, weighted toward cards with few or no
impressions — every session probes a slice of the pool that today can never
be measured.

**Why:** the add stack caps at 500 in-session cards, so cards ranked 501+
are not merely unlikely to be seen — they are *unreachable*. No impressions
can ever accrue for them, so the signal system (add/skip/maybe rates) is
structurally blind past the horizon. The wildcard slot is the only mechanism
by which the deep pool gets measured at all. Users experience it as a spicy
off-list card in the hand; the signal tables experience it as coverage.

## Design

- **Slot count:** `WILDCARD_SLOTS = 1` per 25-card page (a dial; 0 reverts
  to pure band serving). Every page, including the first — the spice IS the
  feature. ≈ 4 wildcards per 100 swiped.
- **Draw pool:** same WHERE as the main serve (filters, legality, identity,
  suppressions all respected) but ranked below the reachable horizon
  (rank > `MAX_CARDS_IN_STACK` = 500), drawn `ORDER BY COALESCE(shown, 0)
  ASC` then the (deck, day) hash — never-shown cards first, stable within a
  day, rotates tomorrow.
- **Signal:** nothing new — wildcard impressions ride the existing
  shown/added/skipped/maybed flow. Skipped wildcards suppress per deck like
  any card, so a probe never repeats within a deck.
- **Follow-on (separate work): surprise scoring.** Once wildcard impressions
  accrue, center the signal term on a rank-bucket expectation instead of the
  global rate: boost = `shrunk_rate − expected_rate(edhrec_rank bucket)`.
  Cards outperforming their reputation rise; famous underperformers sink.
  This is what promotes a validated deep cut from wildcard slot → real band
  over time.

Implementation detail: [`server.md`](server.md).
