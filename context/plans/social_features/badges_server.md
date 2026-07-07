# Weekly badges — server

## 1. Migration — `zerver/migrations/<ts>_create_user_week_badges.sql`

```sql
-- Computed week-close artifacts. One row per (user, closed week) with any
-- activity; badges are stable once written (recompute = delete + rerun).
CREATE TABLE user_week_badges (
    user_id     UUID   NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start  DATE   NOT NULL,
    badges      TEXT[] NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, week_start)
);
```

`badges` holds the serde keys of the shared enum (below), priority-ordered,
length 1–3.

## 2. Shared types — `zwipe-core/src/http/contracts/badges.rs`

Pure enum + contracts (same pattern as `AnonymousEventKind`):

```rust
pub enum WeekBadge {            // snake_case serde + as_str parity test
    SwipeKing,                  // volume: total directional swipes, top tier
    Grinder,                    // volume: mid tier
    TheController,              // dominant facet category = removal/counter
    Groundskeeper,              // dominant facet category = ramp/lands
    Devoted(…) or per-color set // ≥60% of adds one color: MonoWhite… MonoGreen
    UltimateIndecision,         // maybes ≥ adds, both nonzero
    TheCurator,                 // removals high vs adds (pruning week)
    TheSeeker,                  // searches high vs swipes
    DeckMachine,                // ≥2 decks created this week (join decks.created_at)
    ShowedUp,                   // fallback: any activity, nothing else earned
}
impl WeekBadge { pub fn title(&self) -> &str; pub fn blurb(&self) -> &str; }
```

Exact thresholds tune at build time against real `user_week_signal`
distributions (read prod read-only first — thresholds should make badges
scarce enough to mean something; aim for the top badge hitting <10% of
active users). Copy: sentence case, no em dashes, playful but crisp.

`HttpWeeklyRecap`: `week_start`, `badges: Vec<WeekBadge>`, the week's
counters (swipes by direction, added/skipped/maybed/removed, searches), top
category, top color, and `history: Vec<(week_start, Vec<WeekBadge>)>`
(capped 12).

## 3. Badge job — zervice step

After the existing refresh steps in `zerver/src/bin/zervice.rs` (~line 87
where `refresh_card_signal_rollup` sits): compute badges for **every closed
ISO week that has `user_week_signal` rows but no `user_week_badges` rows**.
Idempotent backfill, not a Monday check — zervice runs daily at 4am via
cron, so a missed run self-heals next morning, and the first deploy
backfills all history since 2026-07-02.

Rust rule evaluation over one SQL read per week (signal + facet + deck
count joined per user), then batch insert. Rules: evaluate all, sort by
priority, take 3, `ShowedUp` if empty. Port surface: new methods on the
metrics domain (`compute_week_badges(week_start)`,
`closed_weeks_missing_badges()`) — Repository/Service/Erased/blanket,
following the existing pattern.

## 4. Recap endpoint

`GET /api/user/weekly-recap` (private routes, JWT): latest **closed** week's
recap + history, from `user_week_badges` + `user_week_signal`. Users with no
closed-week activity get 200 with `badges: []` (client shows nothing). No
new rate-limit config needed (private routes already governed per-user).

`.sqlx`: new queries → `cargo sqlx prepare --workspace` from the workspace
root, commit.

## 5. Tests

Unit: badge rules (given counter rows → expected badges, priority, cap,
fallback) live next to the rules. Integration (once
[`integration_tests`](../integration_tests/overview.md) harness exists):
seed two users' week signal → run job → assert rows; idempotency (second
run adds nothing); recap endpoint shape.

## Compatibility

Additive only: old clients never call the endpoint. Server deploys first,
accrues badge history silently until a client renders it.
