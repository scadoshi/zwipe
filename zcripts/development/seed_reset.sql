-- Removes every seeded account from a LOCAL database, and with it everything
-- the account owns and its swipes in the pooled signal tables. Accounts that
-- aren't seeded are never touched. seed.sh
-- --reset runs it before seeding again.
--
-- The deck cards and decks go first, each in one set-based delete: left to
-- the foreign keys' ON DELETE CASCADE, a load-sized reset deletes millions of
-- rows one at a time. The account's other rows (saved commanders, settings,
-- activity) still go through their cascades.

BEGIN;

CREATE TEMP TABLE seed_users ON COMMIT DROP AS
SELECT id FROM users WHERE email LIKE '%@seed.zwipe.test';

CREATE TEMP TABLE seed_decks ON COMMIT DROP AS
SELECT d.id FROM decks d JOIN seed_users u ON u.id = d.user_id;

-- The seeded accounts' swipes come back out of the pooled tables first, the
-- same amounts seed_pools.sql added: their per-card signal from the two card
-- pools, and three shows, a pick and two skips per seeded deck from its
-- commander's. A pooled row left at zero goes.
CREATE TEMP TABLE seed_card ON COMMIT DROP AS
SELECT s.card_oracle_id, sum(s.shown) AS shown, sum(s.added) AS added, sum(s.skipped) AS skipped,
       sum(s.maybed) AS maybed, sum(s.removed) AS removed
FROM user_card_signal s JOIN seed_users u ON u.id = s.user_id
GROUP BY s.card_oracle_id;

UPDATE card_swipe_signal p SET
    shown   = greatest(0, p.shown   - c.shown),
    added   = greatest(0, p.added   - c.added),
    skipped = greatest(0, p.skipped - c.skipped),
    maybed  = greatest(0, p.maybed  - c.maybed),
    removed = greatest(0, p.removed - c.removed)
FROM seed_card c WHERE c.card_oracle_id = p.card_oracle_id;

CREATE TEMP TABLE seed_pair ON COMMIT DROP AS
SELECT s.commander_oracle_id, s.card_oracle_id, sum(s.shown) AS shown, sum(s.added) AS added,
       sum(s.skipped) AS skipped, sum(s.maybed) AS maybed, sum(s.removed) AS removed
FROM user_card_signal s JOIN seed_users u ON u.id = s.user_id
GROUP BY s.commander_oracle_id, s.card_oracle_id;

UPDATE commander_card_signal p SET
    shown   = greatest(0, p.shown   - c.shown),
    added   = greatest(0, p.added   - c.added),
    skipped = greatest(0, p.skipped - c.skipped),
    maybed  = greatest(0, p.maybed  - c.maybed),
    removed = greatest(0, p.removed - c.removed)
FROM seed_pair c
WHERE c.commander_oracle_id = p.commander_oracle_id AND c.card_oracle_id = p.card_oracle_id;

CREATE TEMP TABLE seed_picks ON COMMIT DROP AS
SELECT sd.oracle_id, count(*) AS picks
FROM decks d
JOIN seed_decks s ON s.id = d.id
JOIN scryfall_data sd ON sd.id = d.commander_id
WHERE sd.oracle_id IS NOT NULL
GROUP BY sd.oracle_id;

UPDATE commander_select_signal p SET
    shown    = greatest(0, p.shown    - 3 * c.picks),
    selected = greatest(0, p.selected - c.picks),
    skipped  = greatest(0, p.skipped  - 2 * c.picks)
FROM seed_picks c WHERE c.oracle_id = p.commander_oracle_id;

DELETE FROM card_swipe_signal p USING seed_card c
WHERE c.card_oracle_id = p.card_oracle_id
  AND p.shown = 0 AND p.added = 0 AND p.skipped = 0 AND p.maybed = 0 AND p.removed = 0;
DELETE FROM commander_card_signal p USING seed_pair c
WHERE c.commander_oracle_id = p.commander_oracle_id AND c.card_oracle_id = p.card_oracle_id
  AND p.shown = 0 AND p.added = 0 AND p.skipped = 0 AND p.maybed = 0 AND p.removed = 0;
DELETE FROM commander_select_signal p USING seed_picks c
WHERE c.oracle_id = p.commander_oracle_id
  AND p.shown = 0 AND p.selected = 0 AND p.skipped = 0;

DELETE FROM deck_cards dc USING seed_decks d WHERE dc.deck_id = d.id;
DELETE FROM decks dk USING seed_decks d WHERE dk.id = d.id;
DELETE FROM users us USING seed_users u WHERE us.id = u.id;

COMMIT;
