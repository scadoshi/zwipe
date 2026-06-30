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

## Decisions (settled 2026-06-30)

- **Default state — ON when a commander is set** (and the synergy cache is warm);
  OFF when there's no commander or no synergy data. Sells the "it just works"
  story; flip OFF to reach the full legal pool.
- **Toggle placement — a chip next to the Search/Maybeboard source selector**
  (`add.rs`), shown only for the **Search** source (synergy is meaningless in
  Maybeboard mode).
- **Cold cache — fall back to OFF + a subtle "synergy warming up" note.** A
  brand-new commander's synergy row fills lazily (~seconds); rather than an empty
  stack (which reads as broken), serve the full pool and hint that synergy is
  warming.
- **Membership predicate — the jsonb `(<scores> ->> LOWER(name)) IS NOT NULL`
  probe; no new index.** `idx_latest_cards_name` is on `name`, not `LOWER(name)`,
  so a name-list `ANY` wouldn't use it — and today's synergy `ORDER BY` *already*
  does the identical per-row jsonb probe over the full pool, so the membership
  `WHERE` is **perf-parity** with the existing synergy path. Future optimization
  if ever needed: add `CREATE INDEX … (LOWER(name))` and switch to a name-list
  `ANY`.
- **Out of scope (deferred):** #6 MV-aware adaptive weighting and the
  synergy-quality / embeddings work. This change is purely membership + the flag.

## Build scope (server-first)

1. **`zwipe-core` — the flag.** Add `synergy: bool` (`#[serde(default)]`) to
   `CardFilter` + a builder getter/setter. Old clients omit it → `false` →
   today's behavior. (The deck-aware search body is the shared `CardFilter`.)
2. **`zerver` — membership.**
   - Service (`services.rs:304`): fetch synergy scores when
     `commander_id.is_some() && (synergy_on || order_by().is_none())`; pass a new
     `synergy_only: bool` to the repo.
   - SQL (`search_scryfall_data_deck_aware`, `card/mod.rs`): add a `synergy_only`
     param; when true and scores are present, push
     `AND (<scores> ->> LOWER(name)) IS NOT NULL` into the existing `WHERE`
     (seeded `TRUE`, `AND`-chained ~line 215). The `ORDER BY` block (708–759) is
     **untouched** — it already applies the user's sort when set, else the synergy
     default.
   - `cargo sqlx prepare --workspace`.
3. **`zwiper` — toggle + states.**
   - A "Synergy" ON/OFF chip beside the source selector (Search source only),
     wired into `CardFilterBuilder.synergy`. Default ON when a commander is set.
   - Re-query on toggle (reuse the existing search effect); works under any sort.
   - Cold / no-commander: when ON returns nothing because the cache is cold, fall
     back to OFF + the "warming up" hint; disable/hide the toggle when there's no
     commander.

### Risks / edge cases
- **Name-key membership.** Keys are lowercased `name` (matching today's ordering).
  Confirm score-map keys line up with `latest_cards.name` lowercased (watch
  DFC/split name forms).
- **Pagination under ON + non-synergy sort.** `LIMIT` must scan to fill 25 within
  the membership set — fine for a few-hundred-card pool; watch a sparse one.
- **Non-commander / no synergy signal → ON unavailable** (no set to constrain to).
- **Plain (non-deck) search ignores the flag** — confirm it's never read there.

### Testing
- Unit: `CardFilter` serde default (old client omits `synergy` → `false`).
- Integration (real PG): ON constrains to the scored set; ON + price sort =
  cheapest-that-work; OFF = full pool; no-commander → OFF.
- Manual: toggle in app, change sort within ON, cold-commander fallback.

## Status

**Scoped + decided 2026-06-30 — ready to build.** Model: server-side membership +
user sort, paginated. Sequence: server slice (1–2, small) first — it unblocks the
price-sort expectation (`price_filter.md`) and #6; client (3) is the bulk. Synergy
quality improves later via embeddings/learned ranking over collected data
(`suggestion_signal.md`, #7) with no further architecture change.
