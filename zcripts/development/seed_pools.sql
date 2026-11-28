-- Adds every seeded account's swipes to the pooled signal tables on a LOCAL
-- database, so the card stats (Swipes, As a commander) and Most skipped's
-- baseline read real numbers. seed.sh runs it last, after seed.sql and the
-- busy year seed_activity.sql gives seed_tester. seed_reset.sql takes the
-- same amounts back out before it removes the accounts.
--
-- The pools are sums over every account, as zerver's usage flush keeps them:
-- card_swipe_signal per card and commander_card_signal per commander and card
-- sum the seeded accounts' user_card_signal. commander_select_signal has no
-- per-account rows, so each seeded deck counts as a pick of its commander,
-- shown three times and skipped twice for each pick.

BEGIN;

CREATE TEMP TABLE seed_accounts ON COMMIT DROP AS
SELECT id FROM users WHERE email LIKE '%@seed.zwipe.test';

INSERT INTO card_swipe_signal (card_oracle_id, shown, added, skipped, maybed, removed)
SELECT s.card_oracle_id, sum(s.shown), sum(s.added), sum(s.skipped), sum(s.maybed), sum(s.removed)
FROM user_card_signal s
JOIN seed_accounts a ON a.id = s.user_id
GROUP BY s.card_oracle_id
ON CONFLICT (card_oracle_id) DO UPDATE SET
    shown      = card_swipe_signal.shown   + EXCLUDED.shown,
    added      = card_swipe_signal.added   + EXCLUDED.added,
    skipped    = card_swipe_signal.skipped + EXCLUDED.skipped,
    maybed     = card_swipe_signal.maybed  + EXCLUDED.maybed,
    removed    = card_swipe_signal.removed + EXCLUDED.removed,
    updated_at = now();

INSERT INTO commander_card_signal (commander_oracle_id, card_oracle_id, shown, added, skipped, maybed, removed)
SELECT s.commander_oracle_id, s.card_oracle_id,
       sum(s.shown), sum(s.added), sum(s.skipped), sum(s.maybed), sum(s.removed)
FROM user_card_signal s
JOIN seed_accounts a ON a.id = s.user_id
GROUP BY s.commander_oracle_id, s.card_oracle_id
ON CONFLICT (commander_oracle_id, card_oracle_id) DO UPDATE SET
    shown      = commander_card_signal.shown   + EXCLUDED.shown,
    added      = commander_card_signal.added   + EXCLUDED.added,
    skipped    = commander_card_signal.skipped + EXCLUDED.skipped,
    maybed     = commander_card_signal.maybed  + EXCLUDED.maybed,
    removed    = commander_card_signal.removed + EXCLUDED.removed,
    updated_at = now();

INSERT INTO commander_select_signal (commander_oracle_id, shown, selected, skipped)
SELECT sd.oracle_id, 3 * count(*), count(*), 2 * count(*)
FROM decks d
JOIN seed_accounts a ON a.id = d.user_id
JOIN scryfall_data sd ON sd.id = d.commander_id
WHERE sd.oracle_id IS NOT NULL
GROUP BY sd.oracle_id
ON CONFLICT (commander_oracle_id) DO UPDATE SET
    shown      = commander_select_signal.shown    + EXCLUDED.shown,
    selected   = commander_select_signal.selected + EXCLUDED.selected,
    skipped    = commander_select_signal.skipped  + EXCLUDED.skipped,
    updated_at = now();

SELECT (SELECT count(*) FROM user_card_signal s JOIN seed_accounts a ON a.id = s.user_id) AS seeded_swipe_rows,
       (SELECT coalesce(sum(s.shown), 0) FROM user_card_signal s JOIN seed_accounts a ON a.id = s.user_id) AS seeded_swipes;

COMMIT;
