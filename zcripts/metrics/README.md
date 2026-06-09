# Metrics Scripts

Read-only usage reports you run against the production database, then paste the
output back to Claude for analysis. No instrumentation, no dashboards — just a
single SQL file that aggregates everything we already capture about user usage.

## What we capture

| Table | What it holds |
|---|---|
| `users` | signups, email verification, lockouts |
| `decks` (`first_completed_at`) | decks built + whether they reached a valid state |
| `deck_cards` (`board`) | card contents per deck — deck / maybeboard / sideboard |
| `user_lifetime_counters` | per-user totals: swipes (R/L/U/D), searches, decks; `updated_at` = last active |
| `user_daily_activity` | per-user, per-day swipe + search counts (drives DAU/WAU/MAU) |
| `user_events` | sparse log: `signup`, `deck_created`, `deck_completed`, `first_swipe` |
| `user_audit_log` | credential changes: `username_changed`, `email_changed`, `password_changed` |

## `overview.sql` — one-shot usage report

Sixteen labeled sections in one run: snapshot, signups, activation funnel,
DAU/WAU/MAU, recency, daily volume, swipe-direction mix, engagement
distribution, top users, deck counts/completion/size, format popularity, top
commanders, top cards, board usage, and account activity.

It's **read-only** — no writes, no locks. Every section prints a `── header ──`
so the output is self-describing.

### Run it

On the server (simplest):

```bash
sudo -u postgres psql zwipe -f zcripts/metrics/overview.sql
```

Or anywhere with the connection string (the same `DATABASE_URL` from
`zerver/.env`):

```bash
psql "$DATABASE_URL" -f zcripts/metrics/overview.sql
```

To capture to a file you can paste back wholesale:

```bash
sudo -u postgres psql zwipe -f zcripts/metrics/overview.sql > /tmp/zwipe-usage.txt
```

### The loop

1. Run `overview.sql`.
2. Paste the full output back to Claude.
3. Claude reads it section-by-section and tells you what people are actually
   doing — where activation drops off, who the power users are, what's getting
   built, what features (maybeboard, sideboard, search) are being ignored.

Then we decide what to dig into with a follow-up query.

## `decks-truth.sql` — build-independent deck read

Until 1.0.3 telemetry reaches users, the swipe/search/daily-active numbers are
blind (the live App Store build doesn't report them, so they reflect only the
dev device). This script reads **only** the `decks` / `deck_cards` tables, which
are written server-side on every create regardless of app build — so these
numbers are real for *all* users right now. Decks-per-day, builder activation,
deck sizes, formats, top commanders.

```bash
sudo -u postgres psql zwipe -f zcripts/metrics/decks-truth.sql
```

## `backfill-deck-counter.sql` — one-time counter reconcile

The public "Decks created" number on zwipe.net is `SUM(decks_created)` from
`user_lifetime_counters` — a server-side counter bumped on every create/clone,
but only since the metrics handler deployed. Decks built **before** that deploy
never bumped it, so their builders sit at `decks_created = 0` while clearly
owning decks. This script raises each such counter to the user's current live
deck count.

- **Only ever raises** (`WHERE decks_created < live_count`) — never lowers, so
  the monotonic "never drops" marketing property holds and counters already
  above their live count (created-then-deleted) are left untouched.
- Wrapped in a transaction; prints BEFORE total, a per-user preview, and the
  AFTER total, then COMMITs. Change the final `COMMIT` to `ROLLBACK` to dry-run.
- One-time fix: new decks count correctly on their own going forward.

```bash
sudo -u postgres psql zwipe -f zcripts/metrics/backfill-deck-counter.sql
```

## Notes

- All timestamps are `TIMESTAMPTZ` and the pool is pinned to UTC, so every
  "day" / "last N days" window is UTC. Today's DAU is partial (the day isn't
  over yet).
- The activation funnel (section 3) is built from durable state (counters +
  decks), not the event log, so it's correct even for users who signed up
  before event logging existed.
- Swipe directions: **right** = primary action (add on search / remove on deck
  view), **left** = skip/pass, **up** = maybeboard, **down** = undo.
