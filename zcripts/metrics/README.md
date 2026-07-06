# Metrics Scripts

Read-only usage reports you run against the production database, then paste the
output back to Claude for analysis. No instrumentation, no dashboards — one
big overview plus small single-topic scripts for quick checks.

## What we capture

| Table | What it holds |
|---|---|
| `users` | registrations, email verification, lockouts, `last_active_at`, `first_swiped_at` |
| `decks` (`first_completed_at`) | decks built + whether they reached a valid state |
| `deck_cards` (`board`) | card contents per deck — deck / maybeboard / sideboard |
| `user_lifetime_counters` | per-user totals: swipes (R/L/U/D), searches, decks; `updated_at` = last counter write (use `users.last_active_at` for last-active) |
| `user_daily_activity` | per-user, per-day swipe + search counts (drives DAU/WAU/MAU) |
| `user_events` | sparse log: `register`, `login`, `refresh`, `logout`, `deck_created`, `deck_completed`, `first_swipe` |
| `user_audit_log` | auth timeline: `username_changed`, `email_changed`, `password_changed`, `login`, `refresh`, `logout` |
| `anonymous_events` | pre-auth funnel: `app_opened`, `register_viewed`, `register_submitted` per random session UUID (no identity) |
| `commander_card_signal` | global (commander, card) tallies: shown / added / skipped / maybed / removed |
| `user_card_signal` | per-user mirror of the commander signal |
| `user_week_signal` / `user_week_facet_signal` | per-user weekly counters + adds by category / color (Monday UTC weeks) |
| `deck_card_suppressions` | durable per-deck skips (`skip` / `removal` provenance, 5,000 cap per deck) |

## The scripts

| Script | Question it answers | Cadence |
|---|---|---|
| `pulse.sql` | What happened in the last day or two? | Daily-ish |
| `overview.sql` | Full state of the world, 16 sections | Weekly / before decisions |
| `retention.sql` | Do deck-builders come back on a later day? | Weekly |
| `swipes.sql` | How deep, how picky, how habitual is swiping? | Weekly |
| `funnel.sql` | Where do people drop off before registering? | After a build with funnel events ships |
| `signal.sql` | Is the suggestion-signal substrate filling in? What does it say? | Before Phase 3 (ranking) work |
| `swipe-memory.sql` | Are durable skips used? Any deck near the 5k cap? | Occasional |

All of them are **read-only** — no writes, no locks — and print `── header ──`
sections so the output is self-describing. Run any of them the same way:

```bash
# On the server (simplest):
sudo -u postgres psql zwipe -f zcripts/metrics/<script>.sql

# Or anywhere with the connection string (same DATABASE_URL as zerver/.env):
psql "$DATABASE_URL" -f zcripts/metrics/<script>.sql

# Capture to a file you can paste back wholesale:
sudo -u postgres psql zwipe -f zcripts/metrics/pulse.sql > /tmp/zwipe-pulse.txt
```

### The loop

1. Run the script that matches your question (`pulse.sql` if you just want a
   heartbeat).
2. Paste the full output back to Claude.
3. Claude reads it section-by-section and tells you what people are actually
   doing — then we decide what to dig into with a follow-up query.

## Script details

### `pulse.sql` — the heartbeat

Today vs yesterday, the last 7 days (registrations, actives, volume), the last
10 registrations with what each account did since, the event log over 48h, and
a 48h glance at the pre-auth funnel. Small enough to run every morning.

### `overview.sql` — one-shot usage report

Sixteen labeled sections in one run: snapshot, registrations, activation
funnel, DAU/WAU/MAU, recency, daily volume, swipe-direction mix, engagement
distribution, top users, deck counts/completion/size, format popularity, top
commanders, top cards, board usage, and account activity.

### `retention.sql` — repeat deck-builders

"Retention" = did a builder come back on a *later day* to build again. Derived
purely from `decks.created_at` (distinct UTC build days per user), so it's real
for every user regardless of app build. **Re-run weekly** and watch the
`repeat_builders` / `repeat_pct` numbers move. Shows a per-builder timeline and
a return-gap distribution.

### `swipes.sql` — the shape of swipe activity

Session depth (swipes per active user-day, median/p90 + buckets), weekly
add-rate trend (right / decisive swipes — falling means pickier users or a
serve running dry), undo rate per day vs the lifetime baseline (spikes after
a release = accidental-swipe regression), per-user selectivity distribution
(collectors vs curators, min 20 decisive swipes), consecutive-day streaks
(longest-ever distribution + live streaks), and swipes per completed deck
(effort-to-value; falling across cohorts = suggestion quality improving).

### `funnel.sql` — pre-auth registration funnel

App opened → register viewed → register submitted (distinct anonymous
sessions), with registrations from `users` as the final leg — sessions carry
no identity, so the last step is counted alongside, never joined. Also:
open-to-submit time (median / p90) and submit-retry distribution (form
friction). Empty until the `anonymous_events` migration and an instrumented
client build are both live.

### `signal.sql` — suggestion-signal health

Substrate size (pairs, commanders, impression totals), commanders with the
most signal, strongest keeps and strongest passes (min 5 impressions), weekly
signal volume, and facet adds by category / color. Run before starting
suggestion-signal **Phase 3 (ranking)** to judge whether there's enough data.

### `swipe-memory.sql` — durable skips

Suppression totals with the skip/removal split, adoption among decks active in
the last 30 days, per-deck depth distribution (flags anyone near the 5,000
cap), the 10 largest skip lists, and daily write volume.

## Notes

- All timestamps are `TIMESTAMPTZ` and the pool is pinned to UTC, so every
  "day" / "last N days" window is UTC. Today's DAU is partial (the day isn't
  over yet).
- The activation funnel (overview section 3) is built from durable state
  (counters + decks), not the event log, so it's correct even for users who
  registered before event logging existed.
- Swipe directions: **right** = primary action (add on search / remove on deck
  view), **left** = skip/pass, **up** = maybeboard, **down** = undo.
- `anonymous_events` sessions are random per-launch UUIDs: one human can be
  many sessions, and a relaunch restarts the funnel. Read trends, not
  absolutes.
