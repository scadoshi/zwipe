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

## `retention.sql` — repeat deck-builders

"Retention" = did a builder come back on a *later day* to build again. Derived
purely from `decks.created_at` (distinct UTC build days per user), so it's real
for every user regardless of app build. **Re-run weekly** and watch the
`repeat_builders` / `repeat_pct` numbers move. Shows a per-builder timeline and
a return-gap distribution.

```bash
sudo -u postgres psql zwipe -f zcripts/metrics/retention.sql
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
