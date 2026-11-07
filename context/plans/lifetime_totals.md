# Lifetime totals that never drop

**Status: parked, 2026-11-07, waiting on Scotty. Not built.** It changes what the public numbers mean, so it needs his call first. Nit 158 shipped the other half: the "Live" tag now tells the truth about an edge-cached copy.

## What happened

The public counters (decks created, searches run, cards swiped), on zwipe.net, the app's Home and the login page, come from one query: `public_metrics` in `zerver/src/lib/outbound/sqlx/metrics/mod.rs`, a `SUM` over `user_lifetime_counters`. That table is keyed by user and declared `REFERENCES users(id) ON DELETE CASCADE` (`20260607211058_user_metrics.sql`). Deleting an account deletes its row, so everything that user ever did leaves "everybody's" totals.

On 2026-11-06 Surfer saw 3,125 decks / 43.8k searches / 364.5k swiped on the login page and 2,269 / 31.8k / 264.6k elsewhere (Surfer #53, #56). The higher copy was an edge-cached answer from before the totals fell by about 856 decks and 100k swipes. That is consistent with one or more heavy accounts (test accounts, most likely) being deleted. Production was not checked to confirm which.

## The decision

What should "decks created" mean?

1. **Everything ever done on Zwipe (recommended).** The counters only go up. A deleted account's activity stays in the totals as an anonymous contribution, and only the per-user row goes. This is what a "lifetime" counter reads as, it is what a marketing number wants, and it never shows a visitor a number lower than the one they saw yesterday.
2. **What today's users have done (as now).** The totals fall when someone leaves. Defensible, but a public number that drops reads as a bug, as it did here.

Option 1 cannot recover what is already gone. The totals would stay at today's numbers and only stop falling from now on.

## Migration sketch (option 1)

One new table holding the retired users' sums, filled by a trigger as their rows are deleted, and the public query adding it in.

```sql
-- 2026xxxx_retired_lifetime_counters.sql
CREATE TABLE retired_lifetime_counters (
    id               BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),  -- one row
    swipes_right     BIGINT NOT NULL DEFAULT 0,
    swipes_left      BIGINT NOT NULL DEFAULT 0,
    swipes_up        BIGINT NOT NULL DEFAULT 0,
    swipes_down      BIGINT NOT NULL DEFAULT 0,
    searches         BIGINT NOT NULL DEFAULT 0,
    decks_created    BIGINT NOT NULL DEFAULT 0,
    decks_completed  BIGINT NOT NULL DEFAULT 0
);
INSERT INTO retired_lifetime_counters DEFAULT VALUES;

CREATE FUNCTION retire_lifetime_counters() RETURNS trigger AS $$
BEGIN
    UPDATE retired_lifetime_counters SET
        swipes_right    = swipes_right    + OLD.swipes_right,
        swipes_left     = swipes_left     + OLD.swipes_left,
        swipes_up       = swipes_up       + OLD.swipes_up,
        swipes_down     = swipes_down     + OLD.swipes_down,
        searches        = searches        + OLD.searches,
        decks_created   = decks_created   + OLD.decks_created,
        decks_completed = decks_completed + OLD.decks_completed;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_lifetime_counters_retire
    BEFORE DELETE ON user_lifetime_counters
    FOR EACH ROW EXECUTE FUNCTION retire_lifetime_counters();
```

The trigger runs for the cascade from `users` too, so account deletion needs no code change. `public_metrics` becomes the live sum plus the retired row:

```sql
SELECT
    COALESCE(SUM(u.swipes_right + u.swipes_left + u.swipes_up + u.swipes_down), 0)
        + r.swipes_right + r.swipes_left + r.swipes_up + r.swipes_down AS "cards_swiped!",
    COALESCE(SUM(u.searches), 0) + r.searches AS "searches!",
    COALESCE(SUM(u.decks_created), 0) + r.decks_created AS "decks_created!"
FROM retired_lifetime_counters r
LEFT JOIN user_lifetime_counters u ON TRUE
GROUP BY r.id
```

No wire change: `HttpPublicMetrics` keeps its three fields and types. Per-user counters (`/api/user/metrics`, Activity) are untouched.

## If it is built

- A zerver test: create two users with counts, delete one, and check that `/api/marketing/stats` still sums both.
- `cargo sqlx prepare --workspace` for the changed query, from the workspace root.
- The totals stop falling from the deploy on. Nothing backfills the past.
