# First-Party Suggestion Signal

**Status: Phases 1 + 2 SHIPPED (2026-06-30, collecting in prod). Phases
3a + 3b SHIPPED 2026-07-06 (server 1.3.2) — then REVISED the same day:
score-jitter (`W_JITTER` 0.01 → 0.04 → 0.08) was replaced by a BAND SHUFFLE.
Live Krenko tests proved jitter permutes positions (77 of top-100 differed
between decks) but never rotates the visible *cast* — the same fifteen
goblins in lightly different order reads as "the same" to a human, including
the owner. Band shuffle: cards ranked by (base + signal) are cut into
`BAND_SIZE = 25` hands (matches the client page size: each page is one shuffled hand); bands serve in strict order, position within a band
is purely the (card, deck, day) hash. A different opening hand per deck per
day; a band-2 card can never lead band 1; `BAND_SIZE = 1` + `W_SIGNAL = 0`
reverts to pure score order. Signal now reads as band *migration* (a proven
card breaks into the opening hand). Worst-case perf (31.7k-row firehose
pool, window sort): ~150 ms vs 35 ms — real serves filter to far smaller
pools; acceptable. Phase 3c (pair-level term) remains, gated on pair-depth.** Two refinements landed
beyond the original plan: the right-swipe column is named **`added`** (not
`kept`) to pair with **`removed`** — a new column capturing a *deliberate
removal* from a deck (Remove-screen right-swipe + deck-cards `[-]`-to-zero), a
stronger delayed-negative than a skip. Also added a **flush-on-background**
trigger (JS `visibilitychange: hidden` / `pagehide`) so a swipe-to-close no
longer loses the last unflushed window — the whole telemetry buffer (swipes,
searches, signals) shares it. Server-first deploy still pending.

> **Privacy posture superseded (2026-07-02).** This plan's "aggregate counts,
> no per-user rows" stance described the app at the time. The swipe-memory
> batch (`plans/swipe_memory.md`) deliberately added **per-user** collection —
> `user_card_signal` (user × commander × card) plus weekly tables — and the
> privacy policy was updated to disclose per-account activity. The
> `commander_card_signal` aggregate described below still exists and Phase 3
> can rank from it unchanged; per-user personalization is a later consumer.

## Goal

Refine the order of the add-card swipe stack using **real in-app behavior** —
which suggested cards players actually keep vs. skip for a given commander —
while preserving the app's privacy-first stance (aggregate counts, no per-user
behavioral log).

Today the add-stack is ordered by a synergy signal. This plan adds a second,
self-improving signal sourced from how Zwipe players actually build, so ordering
gets better the more the app is used.

## Current state (why this is needed)

Swipes are recorded as **aggregate counters only**:

- The client buffers swipe/search counts in memory and flushes `HttpUsageBatch`
  (`swipes_right/left/up/down` + `searches`, all integers) every ~30s and on
  backgrounding.
- The server increments `user_lifetime_counters` and `user_daily_activity`.
- The per-event log (`user_events`) stores rare events only (register, login,
  deck created/completed, first-swipe marker).

So **which** card was swiped **which** direction for **which** commander is not
retained anywhere — it's discarded at the moment of the swipe. That means the
accept/skip signal that could improve suggestion ordering is currently lost.
Every day the app runs without this, that signal is gone for good.

## Design — a privacy-preserving aggregate

Mirror the existing "client buffers, flushes periodically, server increments a
counter" pattern, one level richer. No per-user rows, no new PII.

### Schema (zerver migration)

```sql
CREATE TABLE commander_card_signal (
    commander_oracle_id UUID        NOT NULL,
    card_oracle_id      UUID        NOT NULL,
    shown               BIGINT      NOT NULL DEFAULT 0,
    kept                BIGINT      NOT NULL DEFAULT 0,  -- right swipe
    skipped             BIGINT      NOT NULL DEFAULT 0,  -- left swipe
    maybed              BIGINT      NOT NULL DEFAULT 0,  -- up swipe
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (commander_oracle_id, card_oracle_id)
);
```

The primary key is `(commander, card)` — **there is no `user_id`**. The table is
a pure aggregate, the same privacy posture as `user_lifetime_counters`. The
derived ranking input is the accept rate `kept / (kept + skipped)` per commander,
with `shown` available for impression weighting.

### Client → server contract

Extend the existing usage flush rather than adding a high-frequency endpoint.
The client already knows the deck's commander and the card at swipe time, so it
maintains a small in-session map keyed by `(commander_oracle_id, card_oracle_id)`
accumulating `{shown, kept, skipped, maybed}`, and flushes it alongside the
counters:

- Add an **optional** `signals: Vec<CardSignalDelta>` field to the usage batch
  contract (backward compatible — older clients omit it).
- Each delta: `{ commander_oracle_id, card_oracle_id, shown, kept, skipped, maybed }`.
- Clamp per flush (reuse the existing `MAX_PER_FLUSH` clamp idea) so a client
  can't inflate the aggregate.
- Server upserts with `ON CONFLICT (...) DO UPDATE SET col = col + EXCLUDED.col`,
  exactly like `user_daily_activity`.

Only captured when the deck has a commander/command-zone card set; decks with no
command zone contribute nothing (no key to attribute the signal to).

### Ranking integration — Phase 3 design (settled 2026-07-06)

**Supersedes the original "confidence threshold" sketch.** A hard threshold is
an on/off switch; the settled design is a dial that starts data-poor by
construction and converts itself into signal-driven ranking as data accrues —
no flag day, no revamp later. It also fixes a live UX complaint (Reddit DM,
2026-07-06): the default synergy order is identical every serve, every deck,
every user.

```
serve_score(card) =
    base                                 // today's order: synergy score / edhrec rank
  + w_s · shrunk_rate(card)              // pooled add-rate, Bayesian-shrunk
  + w_j · noise(seed) / sqrt(shown + 1)  // seeded jitter, loudest where data is thinnest
```

- **Base is the control.** Both coefficients at zero reproduce today's
  ordering byte for byte — that's also the revert lever.
- **Signal term**: per-card pooled add-rate across all commanders,
  `(added + k·g) / (shown + k)` where `g` is the global mean add-rate —
  small samples hug the prior, big samples earn their raw rate. Re-ranks
  within the pool only; never adds/removes cards (filters own membership).
  Because skips accrue ~2x faster than adds, the first visible effect is
  **demotion** of consistently-declined cards — the negative signal matures
  first and that's fine.
- **Jitter term**: deterministic noise seeded by
  `hash(oracle_id, deck_id, date)` — different decks get different orders
  (the exact complaint), the same deck stays stable within a day (plays nice
  with parked stacks and undo), and tomorrow drifts. Scaled by
  `1/sqrt(shown+1)`, so unknown cards explore and well-measured cards settle.
  The jitter is the exploration engine that feeds the signal term: today's
  deterministic head-only serving starves the tail of impressions.
- **Bounds**: jitter moves cards within quality bands, not across them (a
  rank-500 card must not leapfrog a top staple); explicit user sorts
  (price, name, MV, the existing Random) stay exact — this shapes only the
  default synergy/rank ordering.
- **Hierarchy (the "adjusts as signal grows" mechanism)**: the same formula
  one level deeper — pair-rate shrinks toward card-rate shrinks toward
  global — activates commander-specific ranking automatically once pair
  denominators exist. Nothing to redesign later; the join just goes one
  level down.

**Where it runs / perf (measured on dev, 2026-07-06):** insertion point is the
synergy-default ORDER BY branch in `search_scryfall_data_deck_aware`
(`outbound/sqlx/card/mod.rs` ~line 787) — explicit-sort branch untouched.
zynergy is unchanged (its cached payload stays the `base` term). The signal
rate is precomputed (card-level rollup; matview on the zervice cron once
volume warrants), so only the jitter hash is runtime. Worst-case pool
(31.7k commander-legal rows, full formula with an *inline* rollup join):
29 ms → 35 ms; jitter alone is unmeasurable; matview takes the join to ~0.
Caveat: Zwipe-select sends an explicit `EdhrecRank` sort from the client, so
server-only jitter (3a) varies the 99 but not commander-select — that needs a
small client change (omit the sort when it's the untouched default) in a
future release.

**Units (measured on prod, 2026-07-06):** synergy scores (max-per-name over
`payload->lists[].cards[].synergy`, floor −10 for scoreless) span −0.62..1.00,
median 0.064, ~267 cards/commander → typical neighbor gap ≈ 0.002. Sizing:
`W_JITTER = 0.04` (±0.02 swing: the stack *head* visibly varies per deck —
head gaps run 0.005–0.044, so most adjacent pairs can swap while standouts
hold; raised from 0.01 after the 2026-07-06 live test left the top 7 pinned,
defeating the point of the variety fix), `W_SIGNAL ≈ 0.15–0.2` on the
*centered* rate (`shrunk_rate − global_rate`, so a neutral card moves zero),
`SHRINK_K = 10`.

**Skips/removes/maybes:** skips need no extra term — `shown =
added+skipped+maybed`, so every skip is already denominator drag (the measured
0.03-rate pairs are skip-driven). The rollup numerator is
`SUM(added + 0.5·maybed − removed)`: a maybe is expressed interest (half an
add, owner call 2026-07-06 — impression-only would have made it
arithmetically identical to a skip, backwards), a removal is a full
take-back. The 0.5 maybe-credit is a named constant; later it can be
calibrated from the measurable maybeboard→mainboard graduation rate.

**Unscored cards (synergy OFF only):** synergy ON is a membership fence and is
untouched. With synergy OFF the unscored tail anchors at `UNSCORED_ANCHOR`
(−10.5, below the −10 scoreless-list floor) plus signal + jitter — so it
shuffles internally but stays below every scored card, and zero dials
reproduce the old `NULLS LAST` order exactly. (The original "let strong
signal lift tail cards into the scored region" idea was **corrected during
build 2026-07-06**: anchoring at 0 would have jumped the tail above the ~40%
of synergy lists with negative scores even at zero dials, breaking
base-is-the-control. Lifting the tail is a future retune: raise the anchor
and/or W_SIGNAL deliberately.)

**Data-readiness baseline (prod, 2026-07-06):** 22,101 pairs / 191
commanders / 27,370 shown / 8,968 added. Pair-level: 0 pairs with ≥20
impressions, 15 with ≥10 — commander-specific term not viable yet.
Card-level pooled: 108 cards ≥20 impressions, 449 ≥10; add-rates on the ≥20
set discriminate strongly (mean 0.36, stddev 0.20, range 0.04–0.90). High-
impression pairs skew to ~0.03 add-rate — the demotion signal is already
real. Compare against these numbers at the next readiness check.

`updated_at` leaves room for a future recency-decay job if needed.

## Phase 3a+3b implementation (code plan, 2026-07-06)

Ships as one server deploy. No client change, no contract change, no `.sqlx`
change (the search is QueryBuilder-built and the refresh is a raw
`sqlx::query`, mirroring `refresh_latest_cards`).

1. **Migration — `create_card_signal_rollup.sql`.** Materialized view:
   ```sql
   CREATE MATERIALIZED VIEW card_signal_rollup AS
   SELECT card_oracle_id,
          SUM(added + 0.5 * maybed - removed)::float8 AS net,
          SUM(shown)::float8                          AS shown
   FROM commander_card_signal
   GROUP BY card_oracle_id;
   CREATE UNIQUE INDEX idx_card_signal_rollup_card
       ON card_signal_rollup (card_oracle_id);
   ```
   `net` can go negative (removals) — shrinkage handles it; that's demotion.

2. **Card ports — `refresh_card_signal_rollup`.** Mirror
   `refresh_latest_cards` exactly: `CardRepository` + `CardService` methods,
   `ErasedCardService` twin + blanket forward, sqlx impl
   (`REFRESH MATERIALIZED VIEW card_signal_rollup`, raw query). Called from
   `zervice.rs` right after the `refresh_latest_cards` call (nightly is
   plenty; signal moves slowly).

3. **`outbound/sqlx/card/mod.rs` — the ordering.** Consts next to
   `MAX_SEARCH_LIMIT`, each doc-commented as a dial:
   `W_SIGNAL: f64 = 0.15`, `W_JITTER: f64 = 0.01`, `SHRINK_K: f64 = 10.0`.
   In `search_scryfall_data_deck_aware`, when (and only when)
   `request.sort().is_none() && synergy_scores.is_some()`:
   - Bake into the initial FROM string:
     `LEFT JOIN card_signal_rollup sig ON sig.card_oracle_id = latest_cards.oracle_id
      CROSS JOIN (SELECT COALESCE(SUM(net)/NULLIF(SUM(shown),0), 0) AS rate
                  FROM card_signal_rollup) g`
     (uncorrelated scalar — computed once per query; ~9.5k rows today).
   - Replace the synergy ORDER BY with:
     ```sql
     ORDER BY (
       COALESCE(($scores ->> LOWER(name))::float8, 0)
       + W_SIGNAL * ((COALESCE(sig.net,0) + SHRINK_K * g.rate)
                     / (COALESCE(sig.shown,0) + SHRINK_K) - g.rate)
       + W_JITTER * (((hashtext(latest_cards.oracle_id::text || $seed)::bigint
                       % 1000 + 1000) % 1000)::float8 / 1000.0 - 0.5)
                  / sqrt(COALESCE(sig.shown,0) + 1.0)
     ) DESC, name ASC
     ```
     Details that matter: the hash is normalized to [0,1) then **centered**
     (−0.5) so jitter can't systematically inflate; the signal term is
     **centered** on `g.rate` so a no-data card contributes exactly 0;
     `NULLS LAST` is replaced by the COALESCE(base, 0) (the deliberate
     unscored-tail behavior above). `$seed` is bound from Rust:
     `format!("{deck_id}:{}", Utc::now().date_naive())`; if `deck_id` is
     None (plain web search) skip the jitter term entirely.

4. **Every other query path is untouched** — explicit sorts, synergy-ON
   membership, plain search, Zwipe-select. The join is only baked when the
   new branch is active.

5. **Verification (manual, dev):**
   - consts at 0 → ordering identical to today (diff the first 250 for a
     real commander deck)
   - same deck same day → stable; different deck / next day → different
   - a high-`net` card visibly climbs; a skip-heavy card sinks
   - `EXPLAIN ANALYZE` stays ~30 ms on the full pool (measured 35 ms with
     the *inline* rollup; the matview only improves that)
   - revert lever: zero the consts, deploy.

## Phasing (collect first, rank later)

1. **Server — start collecting. ✅ BUILT.** Migration `commander_card_signal`
   (`added/skipped/maybed/removed/shown`, PK `(commander, card)`, no user_id) +
   `signals: Vec<CardSignalDelta>` on `HttpUsageBatch` (`#[serde(default)]`,
   clamped: ≤1000 deltas, each field ≤`MAX_PER_FLUSH`) + an additive upsert in
   the existing metrics tx. `.sqlx` regenerated.
2. **Client — emit signals. ✅ BUILT.** `UsageBuffer` holds a
   `(commander, card) → tally` map (`record_signal` for add/skip/maybe on the
   add stack; `record_removal` on deliberate deck removal), drained into the
   batch. Keyed by the **primary** commander oracle id; `shown` derived as
   `added + skipped + maybed`. Down-swipe (undo) is excluded.
3. **Ranking — apply the signal. ⬜ TODO (design settled 2026-07-06, see
   "Ranking integration" above).** Sub-phases, each server-only /
   independently shippable / revertable by zeroing a coefficient:
   3a. **Jitter only** (`w_s = 0`): seeded, uncertainty-scaled noise on the
       existing order in `search_scryfall_data_deck_aware`. Fixes the
       same-stack-every-time complaint outright.
   3b. **Signal blend**: card-level rollup (`SUM(added), SUM(shown)` from
       `commander_card_signal`, inline or a zervice-refreshed matview) joined
       into the ordering with shrinkage.
   3c. **Pair-level term**: hierarchical shrinkage (pair → card → global);
       activates itself as pair denominators mature — re-run the readiness
       queries first.

## Open questions / decisions

- **Multi-command-zone decks** (partner / background / signature spell): key the
  signal on the primary commander, or skip multi-commander decks initially?
- **`shown` accounting**: track true impressions (every card surfaced) or derive
  "shown" as `kept + skipped + maybed`? True impressions need the client to
  count displays, not just actions.
- **Blend weight (`w_s`) and jitter weight (`w_j`)** vs. the existing synergy
  signal — tune against right-swipe-rate once live. (The original hard
  confidence threshold is superseded by shrinkage + uncertainty scaling; see
  the settled Phase 3 design above.)
- **Recency decay**: flat lifetime counts, or decay older signal? Defer; the
  schema supports adding it later.

## Privacy

The signal table holds **no user identity and no per-swipe rows** — only
`(commander, card)` totals. It is the same aggregate-only posture as the existing
usage counters: nothing here is personal data, so it adds no new privacy-policy
or data-subject obligations.

## Verification

- Unit tests on the clamp and the upsert delta math (mirror `HttpUsageBatch`).
- Apply the migration locally, run `cargo sqlx prepare --workspace`, commit
  `.sqlx/`.
- End-to-end in `dx serve`: swipe in a deck with a commander set → confirm
  `commander_card_signal` rows increment with the right kept/skipped tallies;
  confirm a deck with no command zone contributes nothing.
- Confirm old clients (omitting `signals`) still flush usage normally.

## Deploy order

Server-first, per the project rule (same as deck tags / synergy): the migration
and the endpoint that accepts `signals` must be live **before** any client build
that emits them. The ranking-integration phase is a server-only read-path change
and ships independently.
