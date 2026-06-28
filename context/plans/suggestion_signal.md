# First-Party Suggestion Signal

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

### Ranking integration (later phase)

Blend the first-party accept rate into the existing add-stack ordering as a
secondary input, applied **only above a confidence threshold** (a minimum
`kept + skipped` sample per pair) so low-sample noise doesn't move ordering.
`updated_at` leaves room for a future recency-decay job if needed.

## Phasing (collect first, rank later)

1. **Server — start collecting.** Migration + extend the usage endpoint to
   accept the optional `signals` array and upsert into `commander_card_signal`.
   No ranking change yet. Ship this first so old clients are unaffected and the
   server is ready before any client sends signals.
2. **Client — emit signals.** Buffer the per-`(commander, card, direction)`
   tallies during a swipe session and flush them with the usage batch. Data
   begins accumulating. Still no ranking change.
3. **Ranking — apply the signal.** Once enough data has accrued, fold the
   accept rate into the add-stack read path behind a confidence threshold.
   Invisible re-ranking through the existing stack endpoint — no new client
   release required for the ranking change itself.

## Open questions / decisions

- **Multi-command-zone decks** (partner / background / signature spell): key the
  signal on the primary commander, or skip multi-commander decks initially?
- **`shown` accounting**: track true impressions (every card surfaced) or derive
  "shown" as `kept + skipped + maybed`? True impressions need the client to
  count displays, not just actions.
- **Confidence threshold** and **blend weight** vs. the existing synergy signal —
  tune against right-swipe-rate once data exists.
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
