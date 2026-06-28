# Synergy ON/OFF — Membership-Constrained Card Pool

> Note: filename says "client_sort" for history; the agreed model is **server-side**
> membership + sort (see below), not a client-held pool.

## The problem (a perception trap, not a bug)

Users assume the swipe stack is **"the cards that work best in my deck."** When
they change the sort (e.g. to price low→high), they expect *the same relevant
cards, reordered* — cheapest **synergistic** cards first.

What likely happens today instead: synergy is an **ordering** applied over the
full legal/in-color-identity pool. Pick a different sort (price) and the synergy
ordering is replaced — so the stack now surfaces cheap-but-irrelevant legal cards
first. Nothing is broken, but it **reads as broken**: "I sorted by price and now
it's showing me junk that has nothing to do with my commander."

## The model — a "Synergy ON / OFF" toggle (synergy as *membership*, server still sorts)

A single visible control switching two **server-side** modes. The key realization:
synergy is **membership**, not order — so the server keeps paginating and just
constrains the result set, then sorts by whatever the user picked.

- **Synergy ON (curated mode):** the server **constrains results to the
  synergy-relevant set** for the deck (cards that have a synergy entry for this
  commander), already filtered by format legality + color identity + exclusion of
  cards already in the deck, then `ORDER BY <the user's chosen sort>` (synergy by
  default, else price / mana value / name). **Still paginated 25 at a time.**
  Sorting by price now means "cheapest cards **that work**," because the price sort
  is applied *within* the synergistic set.
- **Synergy OFF (raw mode):** drop the membership constraint — today's behavior,
  the full legal pool paginated with server-side sort. This is the escape hatch
  for legal cards EDHREC hasn't ranked: nothing is unreachable, just turn synergy
  off.

In short: **ON → membership constraint + user sort, paginated; OFF → today's plain
paginated search.** No big payloads, no client-held pool — the server does the
work and the client stays thin.

### Why server-side membership (not a client-held pool)

An earlier version of this plan had ON mode ship the *whole* synergy pool to the
client for client-side sorting. **Rejected** — it means big payloads, slower first
card, and client memory/perf cost, to buy only instant re-sort. The server-side
model is a far smaller change and faster to first card:

- **Tiny query change.** The deck-aware search already joins the synergy cache for
  *ordering*. ON mode makes that a **membership join** (INNER instead of LEFT) and
  lets the existing `ORDER BY` apply the user's sort within the synergistic set.
- **Small payloads, pagination untouched.** Keeps the existing 25-at-a-time
  `spawn`-per-page flow in `add.rs`. No coroutine / streaming needed — the
  big-payload scenario that might have justified one is gone.
- **Explicit mental model + coverage hatch.** "Synergy ON" = every card works, any
  sort stays inside that set; OFF reaches everything legal. No silent surprise.

The only thing the client-held-pool approach uniquely enabled was **deck-state
adaptive re-ranking (#6)** — see the wrinkle below.

## Enhancing SYNERGY over time

"Synergy" is a signal, not a fixed thing. It starts as the current synergy data
and **improves over time** by building **embeddings and learned re-ranking over
the data we already collect** — built decks (`deck_cards` co-occurrence) and, once
the `suggestion_signal` rollup exists, accept/skip rates per commander. The client
already owning the pool in ON mode is exactly where that improved ranking gets
applied — no architecture change needed when the model gets smarter, just a better
score on the same pool. (See `suggestion_signal.md` and feature-request #7.)

## Why this is the right shape

- Matches the user mental model (sort changes order, never relevance).
- Makes the **MV-aware weighting (req #6)** a natural client-side concern — the
  client already holds the pool, so "down-weight high-MV as my curve fills" is a
  local re-rank, no server round-trip.
- Plays well with the `suggestion_signal` plan — client-side re-ranking is where a
  first-party accept-rate signal would also be applied.

## How synergy works today (confirmed in code)

- `commander_synergy` is **one jsonb row per commander** (PK = commander
  `oracle_id`), filled by the zynergy worker. zerver reads it into a **score map**:
  jsonb `{LOWER(name) -> score}`.
- The deck-aware search (`outbound/sqlx/card/mod.rs`) applies that map as the
  **default** `ORDER BY` when no explicit sort is set:
  `ORDER BY (<scores> ->> LOWER(name))::float8 DESC NULLS LAST, name ASC`.
  Unscored cards sort **last but are not removed**.
- When the user picks an explicit sort, the code takes the `order_by` branch and
  **skips synergy entirely** → the whole legal set is sorted by that column → the
  "looks broken" perception. The score "pool" is exactly the cards present in the
  map.

## Implications / tradeoffs (the real work)

- **Server query change is tiny — one `WHERE` predicate.** The score map is
  already passed into the query. Synergy ON adds
  `WHERE (<scores> ->> LOWER(name)) IS NOT NULL` (card is in the pool) and lets the
  user's `ORDER BY` apply within it; OFF drops the predicate (today's behavior).
  No new table, no join, no big payload. Both modes paginate as now.
- **Membership is by lowercased name** (that's how the existing ordering keys it),
  not oracle_id — consistent with today, just be aware of the name-key semantics.
- **Contract change.** Add a `synergy: bool` (or mode) flag to `search_deck_cards`.
  Server-first; old clients default to OFF/today's behavior and are unaffected.
- **Sort change = re-query.** Changing sort in ON mode re-fetches page 1 with the
  new `ORDER BY` (one round trip), exactly like changing a filter today. Acceptable
  — sort changes are infrequent. (This is the cost vs. a client-held pool's instant
  re-sort, and it's a fine trade.)
- **No-commander / non-synergy contexts** (no commander, or a format without a
  synergy signal): Synergy ON is unavailable / disabled — there's no membership set
  to constrain to, so fall back to OFF.
- **`#6` adaptive weighting is the one wrinkle.** "Down-weight high-MV cards *as my
  curve fills*" depends on the deck's live state, which a stateless server sort
  doesn't see. Two options when we build #6: (a) pass the current curve to the
  server as a ranking hint, or (b) do a light client re-rank on just the served
  25-card page. Decide when #6 is scoped; it does **not** affect the membership +
  basic-sort model here.

## Open questions

- **Default state** of the toggle — ON or OFF on first entry? (ON sells the
  "it just works" story; probably ON when a commander is set, OFF otherwise.)
- **Toggle placement** — util-bar button (next to Filter/Refresh) vs a chip like
  the existing Search/Maybeboard source selector.
- **Membership join cost** — confirm the INNER-join variant paginates efficiently
  (index on the synergy table's commander + card keys).
- **Cold cache** — first request for a brand-new commander may have no synergy row
  yet (the worker fills lazily, ~seconds). ON mode on a cold commander = empty
  until warm; decide whether to fall back to OFF or show a "warming up" state.

## Status

**Design direction captured — not yet scoped to build.** The Synergy ON/OFF toggle
(server-side membership + user sort, paginated) is the agreed model. It underpins
the price-sort expectation (`price_filter.md`) and MV-aware weighting (#6), so
settle it before building those sorts, or they ship the "sort looks broken"
perception. Sequence: add the membership flag to the search query + the toggle UI →
then price filter's sort and #6 slot in. Synergy quality itself improves later via
embeddings/learned ranking over collected data (`suggestion_signal.md`, #7) with no
further architecture change.
