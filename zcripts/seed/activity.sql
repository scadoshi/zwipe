-- A busy year of Activity for one account on a LOCAL database, for recording
-- the Activity clip. Never run it against prod: the swipes and searches
-- would land in the public counters on zwipe.net. activity.sh runs it and
-- refuses any database that isn't local.
--
-- It reads two session settings, so plain SQL can run it (psql, or a test):
--   zwipe.seed_user  the account's username or email (required)
--   zwipe.seed       a number from -1 to 1 for the random generator (optional)
--
-- It replaces the account's whole activity: its daily activity, weekly
-- signal and per-card signal rows are deleted, then a year is written in
-- their place, and its lifetime swipe and search counters are set to the
-- year's totals. The pooled tables every player shares are left alone. So a
-- second run gives the same shape of year, not twice as much.
--
-- The year is built as one row per swipe, then summed into each table the
-- way zerver's usage flush sums them, so every panel agrees with every other:
-- the heatmap, the totals, card roles and Most skipped.

BEGIN;

DO $$
BEGIN
    PERFORM setseed(COALESCE(NULLIF(current_setting('zwipe.seed', true), ''), '0.42')::float8);
END $$;

-- The account.
CREATE TEMP TABLE seed_user ON COMMIT DROP AS
SELECT id FROM users
WHERE username = current_setting('zwipe.seed_user')
   OR email = current_setting('zwipe.seed_user');

DO $$
BEGIN
    IF (SELECT count(*) FROM seed_user) <> 1 THEN
        RAISE EXCEPTION 'no account named %', current_setting('zwipe.seed_user');
    END IF;
END $$;

-- The commanders the swipes are filed under: the account's own commander
-- decks, or else the four most played legendary creatures in the catalog.
CREATE TEMP TABLE seed_commander_ids ON COMMIT DROP AS
SELECT DISTINCT sd.oracle_id
FROM decks d
JOIN scryfall_data sd ON sd.id = d.commander_id
WHERE d.user_id = (SELECT id FROM seed_user) AND sd.oracle_id IS NOT NULL;

INSERT INTO seed_commander_ids
SELECT lc.oracle_id
FROM latest_cards lc
WHERE NOT EXISTS (SELECT 1 FROM seed_commander_ids)
  AND lc.oracle_id IS NOT NULL
  AND lc.type_line LIKE 'Legendary Creature%'
ORDER BY lc.edhrec_rank NULLS LAST, lc.oracle_id
LIMIT 4;

CREATE TEMP TABLE seed_commanders ON COMMIT DROP AS
SELECT oracle_id, row_number() OVER (ORDER BY oracle_id) AS n
FROM seed_commander_ids;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM seed_commanders) THEN
        RAISE EXCEPTION 'no commander to file swipes under: the catalog has no legendary creature';
    END IF;
END $$;

-- The cards dealt: the most played cards that have a card role, so the Card
-- roles panel fills. Every 23rd is one the account passes on and never keeps,
-- which is what fills Most skipped.
CREATE TEMP TABLE seed_pool ON COMMIT DROP AS
SELECT lc.oracle_id, row_number() OVER (ORDER BY lc.edhrec_rank NULLS LAST, lc.oracle_id) AS n
FROM latest_cards lc
JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
WHERE lc.oracle_id IS NOT NULL
  AND NOT cp.is_token
  AND jsonb_array_length(COALESCE(cp.card_roles, '[]'::jsonb)) > 0
  AND lc.oracle_id NOT IN (SELECT oracle_id FROM seed_commanders);

DELETE FROM seed_pool WHERE n > 800;

CREATE TEMP TABLE seed_kept ON COMMIT DROP AS
SELECT oracle_id, row_number() OVER (ORDER BY n) AS n FROM seed_pool WHERE n % 23 <> 0;

CREATE TEMP TABLE seed_passed ON COMMIT DROP AS
SELECT oracle_id, row_number() OVER (ORDER BY n) AS n FROM seed_pool WHERE n % 23 = 0;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM seed_kept) OR NOT EXISTS (SELECT 1 FROM seed_passed) THEN
        RAISE EXCEPTION 'the catalog needs at least 23 cards with a card role';
    END IF;
END $$;

-- The year, day by day. Four weeks in five are active, so the heatmap shows
-- runs and gaps; an active week still skips about a day in five. Weekends
-- swipe more than weekdays, each week has its own mood, and about one day in
-- thirty is a binge at four times the usual.
CREATE TEMP TABLE seed_days ON COMMIT DROP AS
WITH days AS (
    SELECT gs::date AS day
    FROM generate_series((now() AT TIME ZONE 'UTC')::date - 364,
                         (now() AT TIME ZONE 'UTC')::date, interval '1 day') AS gs
),
weeks AS (
    SELECT week, random() < 0.8 AS active, 0.6 + random() * 0.9 AS mood
    FROM (SELECT DISTINCT date_trunc('week', day)::date AS week FROM days) w
)
SELECT d.day,
       CASE
           WHEN NOT w.active AND random() < 0.85 THEN 0
           WHEN random() < 0.2 THEN 0
           ELSE greatest(1, round((8 + random() * 30) * w.mood
                * CASE extract(isodow FROM d.day) WHEN 6 THEN 1.8 WHEN 7 THEN 1.6 WHEN 5 THEN 1.2 ELSE 1 END
                * CASE WHEN random() < 0.035 THEN 4 ELSE 1 END))::int
       END AS swipes
FROM days d
JOIN weeks w ON w.week = date_trunc('week', d.day)::date;

-- About one search per six swipes, a few more on quiet days.
ALTER TABLE seed_days ADD COLUMN searches int;
UPDATE seed_days
SET searches = CASE WHEN swipes = 0 THEN 0 ELSE round(swipes / 6.0 + random() * 3)::int END;

-- One row per swipe: what it did, under which commander, on which card.
-- About a third are adds, half skips, then saves to the maybeboard and
-- removals. Popular cards come up more often than the long tail.
CREATE TEMP TABLE seed_swipes ON COMMIT DROP AS
SELECT d.day,
       CASE WHEN e.r1 < 0.32 THEN 'added'
            WHEN e.r1 < 0.82 THEN 'skipped'
            WHEN e.r1 < 0.90 THEN 'maybed'
            ELSE 'removed' END AS action,
       e.r2, e.r3, e.r4
FROM seed_days d
CROSS JOIN LATERAL (
    SELECT random() AS r1, random() AS r2, random() AS r3, random() AS r4
    FROM generate_series(1, d.swipes)
) e;

ALTER TABLE seed_swipes ADD COLUMN commander uuid, ADD COLUMN card uuid;

UPDATE seed_swipes s
SET commander = c.oracle_id
FROM seed_commanders c
WHERE c.n = 1 + floor(s.r4 * (SELECT count(*) FROM seed_commanders))::int;

-- One skip in eight goes to a card the account always passes on.
UPDATE seed_swipes s
SET card = p.oracle_id
FROM seed_passed p
WHERE s.action = 'skipped' AND s.r2 < 0.125
  AND p.n = 1 + floor(s.r3 * (SELECT count(*) FROM seed_passed))::int;

UPDATE seed_swipes s
SET card = k.oracle_id
FROM seed_kept k
WHERE s.card IS NULL
  AND k.n = 1 + floor(s.r3 * s.r3 * (SELECT count(*) FROM seed_kept))::int;

-- Out with the account's old activity.
DELETE FROM user_daily_activity WHERE user_id = (SELECT id FROM seed_user);
DELETE FROM user_week_signal WHERE user_id = (SELECT id FROM seed_user);
DELETE FROM user_card_signal WHERE user_id = (SELECT id FROM seed_user);

-- A day's swipes by direction: right adds, left skips and removes, up saves.
CREATE TEMP TABLE seed_daily ON COMMIT DROP AS
SELECT d.day,
       count(*) FILTER (WHERE s.action = 'added')::int AS right_,
       count(*) FILTER (WHERE s.action IN ('skipped', 'removed'))::int AS left_,
       count(*) FILTER (WHERE s.action = 'maybed')::int AS up_,
       d.searches,
       count(*) FILTER (WHERE s.action = 'added')::int AS added,
       count(*) FILTER (WHERE s.action = 'skipped')::int AS skipped,
       count(*) FILTER (WHERE s.action = 'maybed')::int AS maybed,
       count(*) FILTER (WHERE s.action = 'removed')::int AS removed
FROM seed_days d
LEFT JOIN seed_swipes s ON s.day = d.day
WHERE d.swipes > 0 OR d.searches > 0
GROUP BY d.day, d.searches;

INSERT INTO user_daily_activity (user_id, day, swipes_right, swipes_left, swipes_up, swipes_down, searches)
SELECT (SELECT id FROM seed_user), day, right_, left_, up_, 0, searches
FROM seed_daily;

INSERT INTO user_week_signal
    (user_id, week_start, swipes_right, swipes_left, swipes_up, swipes_down, searches,
     added, skipped, maybed, removed)
SELECT (SELECT id FROM seed_user), date_trunc('week', day)::date,
       sum(right_), sum(left_), sum(up_), 0, sum(searches),
       sum(added), sum(skipped), sum(maybed), sum(removed)
FROM seed_daily
GROUP BY date_trunc('week', day)::date;

INSERT INTO user_card_signal
    (user_id, commander_oracle_id, card_oracle_id, shown, added, skipped, maybed, removed)
SELECT (SELECT id FROM seed_user), commander, card,
       count(*) FILTER (WHERE action <> 'removed'),
       count(*) FILTER (WHERE action = 'added'),
       count(*) FILTER (WHERE action = 'skipped'),
       count(*) FILTER (WHERE action = 'maybed'),
       count(*) FILTER (WHERE action = 'removed')
FROM seed_swipes
GROUP BY commander, card;

-- The lifetime counters hold the year's totals; decks created stay as they are.
INSERT INTO user_lifetime_counters (user_id, swipes_right, swipes_left, swipes_up, swipes_down, searches)
SELECT (SELECT id FROM seed_user),
       COALESCE(sum(right_), 0), COALESCE(sum(left_), 0), COALESCE(sum(up_), 0), 0,
       COALESCE(sum(searches), 0)
FROM seed_daily
ON CONFLICT (user_id) DO UPDATE SET
    swipes_right = EXCLUDED.swipes_right,
    swipes_left  = EXCLUDED.swipes_left,
    swipes_up    = EXCLUDED.swipes_up,
    swipes_down  = EXCLUDED.swipes_down,
    searches     = EXCLUDED.searches,
    updated_at   = now();

SELECT count(*) FILTER (WHERE right_ + left_ + up_ > 0) AS active_days,
       sum(right_ + left_ + up_) AS swipes,
       sum(searches) AS searches,
       sum(added) AS added,
       sum(skipped) AS skipped,
       sum(maybed) AS maybed,
       sum(removed) AS removed
FROM seed_daily;

COMMIT;
