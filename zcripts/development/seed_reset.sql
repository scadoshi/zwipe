-- Removes every seeded account from a LOCAL database, and with it everything
-- the account owns. Accounts that aren't seeded are never touched. seed.sh
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

DELETE FROM deck_cards dc USING seed_decks d WHERE dc.deck_id = d.id;
DELETE FROM decks dk USING seed_decks d WHERE dk.id = d.id;
DELETE FROM users us USING seed_users u WHERE us.id = u.id;

COMMIT;
