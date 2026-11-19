-- Seeds users, decks and deck cards on a LOCAL database for testing a branch
-- before it merges. seed.sh runs it and refuses any database that isn't on
-- this machine; never run it against prod.
--
-- It reads two session settings, so plain psql can run it:
--   zwipe.seed_size  small (the test accounts and 40 users) or load (100,000 users)
--   zwipe.seed       a whole number for a different but repeatable world (default 42)
--
-- Every seeded account's email ends in @seed.zwipe.test, and reset.sql
-- removes exactly those accounts and everything they own. It refuses to run
-- while seeded accounts exist, so a second run never doubles them.
--
-- Cards come from the local catalog (latest_cards), so the card sync must have
-- run first. Each random choice is a hash of the seed and the row's number,
-- not random(), so the same seed gives the same rows whatever plan Postgres
-- picks.

BEGIN;

DO $$
BEGIN
    IF current_setting('zwipe.seed_size', true) IS NULL
       OR current_setting('zwipe.seed_size') NOT IN ('small', 'load') THEN
        RAISE EXCEPTION 'zwipe.seed_size must be small or load';
    END IF;
    IF EXISTS (SELECT 1 FROM users WHERE email LIKE '%@seed.zwipe.test') THEN
        RAISE EXCEPTION 'seeded accounts already exist; run with --reset to replace them';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM latest_cards) THEN
        RAISE EXCEPTION 'latest_cards is empty: run the card sync (zervice) first';
    END IF;
END $$;

-- A number in [0, 1) from the seed, a stream (one per kind of choice) and a key.
CREATE FUNCTION pg_temp.rnd(stream int, key bigint) RETURNS float8
LANGUAGE sql STABLE AS $$
    SELECT (hashint8extended(key,
                stream::bigint * 1000003
                + COALESCE(NULLIF(current_setting('zwipe.seed', true), ''), '42')::bigint)
            & 2147483647)::float8 / 2147483648.0
$$;

-- A whole number from lo to hi.
CREATE FUNCTION pg_temp.pick(stream int, key bigint, lo int, hi int) RETURNS int
LANGUAGE sql STABLE AS $$
    SELECT lo + floor(pg_temp.rnd(stream, key) * (hi - lo + 1))::int
$$;

-- The five colors as bits, for matching a card's identity to a commander's.
CREATE FUNCTION pg_temp.mask(identity text[]) RETURNS int
LANGUAGE sql IMMUTABLE AS $$
    SELECT (CASE WHEN 'W' = ANY(identity) THEN 1 ELSE 0 END)
         + (CASE WHEN 'U' = ANY(identity) THEN 2 ELSE 0 END)
         + (CASE WHEN 'B' = ANY(identity) THEN 4 ELSE 0 END)
         + (CASE WHEN 'R' = ANY(identity) THEN 8 ELSE 0 END)
         + (CASE WHEN 'G' = ANY(identity) THEN 16 ELSE 0 END)
$$;

-- One English printing per card, legal in Commander, most played first.
CREATE TEMP TABLE s_catalog ON COMMIT DROP AS
SELECT DISTINCT ON (oracle_id)
    id, oracle_id, name, type_line, pg_temp.mask(color_identity) AS mask, edhrec_rank
FROM latest_cards
WHERE oracle_id IS NOT NULL
  AND lang = 'en'
  AND legalities->>'commander' = 'legal'
ORDER BY oracle_id, released_at DESC, id;

-- The commanders decks are built around: the 1,500 most played legendary creatures.
CREATE TEMP TABLE s_commanders ON COMMIT DROP AS
SELECT row_number() OVER (ORDER BY edhrec_rank NULLS LAST, oracle_id) - 1 AS pos,
       id, oracle_id, name, mask
FROM s_catalog
WHERE type_line LIKE 'Legendary Creature%'
ORDER BY edhrec_rank NULLS LAST, oracle_id
LIMIT 1500;

-- For each color identity, the 2,500 most played cards a commander of that
-- identity can run, numbered from 0. Basic lands come separately.
CREATE TEMP TABLE s_pool ON COMMIT DROP AS
SELECT m.mask, ranked.pos, ranked.id, ranked.oracle_id
FROM generate_series(0, 31) AS m(mask)
CROSS JOIN LATERAL (
    SELECT row_number() OVER (ORDER BY c.edhrec_rank NULLS LAST, c.oracle_id) - 1 AS pos,
           c.id, c.oracle_id
    FROM s_catalog c
    WHERE c.mask & ~m.mask = 0
      AND c.type_line NOT LIKE 'Basic Land%'
    ORDER BY c.edhrec_rank NULLS LAST, c.oracle_id
    LIMIT 2500
) ranked;
CREATE INDEX ON s_pool (mask, pos);

CREATE TEMP TABLE s_pool_size ON COMMIT DROP AS
SELECT mask, count(*)::int AS size FROM s_pool GROUP BY mask;

-- The five basic lands, one printing each, with their color's bit.
CREATE TEMP TABLE s_basics ON COMMIT DROP AS
SELECT DISTINCT ON (c.name) c.id, c.oracle_id, b.bit
FROM s_catalog c
JOIN (VALUES ('Plains', 1), ('Island', 2), ('Swamp', 4), ('Mountain', 8), ('Forest', 16)) AS b(name, bit)
  ON b.name = c.name
WHERE c.type_line LIKE 'Basic Land%'
ORDER BY c.name, c.id;

-- The accounts: three named test accounts, then the numbered ones.
CREATE TEMP TABLE s_users ON COMMIT DROP AS
SELECT n, username, decks, verified,
       md5('zwipe-seed-user-' || username)::uuid AS id,
       now() - make_interval(days => pg_temp.pick(1, n, 0, 365)) AS created_at
FROM (
    VALUES (-1, 'seed_tester', 12, true),
           (-2, 'seed_newbie', 0, true),
           (-3, 'seed_unverified', 2, false)
    UNION ALL
    SELECT n,
           format('seed_%s', lpad(n::text, 6, '0')),
           CASE
               WHEN pg_temp.rnd(2, n) < 0.80 THEN pg_temp.pick(3, n, 2, 3)
               WHEN pg_temp.rnd(2, n) < 0.95 THEN pg_temp.pick(3, n, 4, 7)
               ELSE pg_temp.pick(3, n, 8, 20)
           END,
           true
    FROM generate_series(1, CASE current_setting('zwipe.seed_size') WHEN 'load' THEN 100000 ELSE 40 END) AS n
) AS u(n, username, decks, verified);

INSERT INTO users (id, email, username, password_hash, created_at, updated_at,
                   email_verified_at, last_active_at)
SELECT id,
       username || '@seed.zwipe.test',
       username,
       -- argon2id of Seedling.Local.42, a password that only exists on seeded local databases.
       '$argon2id$v=19$m=19456,t=2,p=1$endpcGVzZWVkc2FsdDAx$fS0BqT/mDLu2pMO1RIZzStoveRnOLDWcrpRsAbOigys',
       created_at,
       created_at,
       CASE WHEN verified THEN created_at + interval '5 minutes' END,
       created_at + make_interval(days => pg_temp.pick(4, n, 0, 30))
FROM s_users;

-- The decks: a commander from the top 1,500, a spread of sizes, some shared.
CREATE TEMP TABLE s_decks ON COMMIT DROP AS
SELECT d.*, c.id AS commander_id, c.oracle_id AS commander_oracle, c.name AS commander_name,
       c.mask
FROM (
    SELECT row_number() OVER (ORDER BY u.n, j) AS deck_no, u.n AS user_n, u.id AS user_id,
           u.verified, u.created_at AS user_created_at, j
    FROM s_users u
    CROSS JOIN LATERAL generate_series(1, u.decks) AS j
) d
JOIN s_commanders c
  ON c.pos = pg_temp.pick(5, d.deck_no, 0, (SELECT count(*)::int - 1 FROM s_commanders));

ALTER TABLE s_decks
    ADD COLUMN id uuid, ADD COLUMN main int, ADD COLUMN maybe int, ADD COLUMN side int,
    ADD COLUMN stars int, ADD COLUMN start int, ADD COLUMN step int,
    ADD COLUMN created_at timestamptz, ADD COLUMN shared bool;
UPDATE s_decks SET
    id = md5('zwipe-seed-deck-' || user_n || '-' || j)::uuid,
    -- An unverified account stays under its lower card limit.
    main = CASE WHEN verified THEN pg_temp.pick(6, deck_no, 30, 70) ELSE 30 END,
    maybe = CASE WHEN verified THEN pg_temp.pick(7, deck_no, 0, 15) ELSE 5 END,
    side = CASE WHEN verified THEN pg_temp.pick(8, deck_no, 0, 10) ELSE 0 END,
    stars = pg_temp.pick(9, deck_no, 0, 3),
    start = pg_temp.pick(10, deck_no, 0, 2499),
    -- A prime past every pool's size, so start + i * step visits distinct cards.
    step = (ARRAY[2503, 2521, 2531, 2539, 2543])[pg_temp.pick(11, deck_no, 1, 5)],
    created_at = user_created_at + make_interval(days => pg_temp.pick(12, deck_no, 0, 60)),
    shared = pg_temp.rnd(13, deck_no) < 0.10;

INSERT INTO decks (id, name, commander_id, format, user_id, created_at, updated_at, share_token)
SELECT id,
       left(commander_name, 56) || ' ' || j,
       commander_id,
       'commander',
       user_id,
       created_at,
       created_at,
       CASE WHEN shared THEN md5('zwipe-seed-share-' || deck_no)::uuid END
FROM s_decks;

-- Each deck's cards from its commander's pool: the mainboard first, then the
-- maybeboard, then the sideboard. The first few mainboard cards are starred,
-- and about one card in 25 is flagged.
INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board,
                        created_at, updated_at, mvp_at, flagged)
SELECT d.id, p.id, p.oracle_id, 1,
       CASE WHEN i < d.main THEN 'deck'
            WHEN i < d.main + d.maybe THEN 'maybeboard'
            ELSE 'sideboard' END,
       d.created_at, d.created_at,
       CASE WHEN i < d.stars THEN d.created_at + make_interval(hours => i + 1) END,
       pg_temp.rnd(14, d.deck_no * 256 + i) < 0.04
FROM s_decks d
JOIN s_pool_size ps ON ps.mask = d.mask
CROSS JOIN LATERAL generate_series(0, LEAST(d.main + d.maybe + d.side, ps.size) - 1) AS i
JOIN s_pool p ON p.mask = d.mask AND p.pos = (d.start + i::bigint * d.step) % ps.size
WHERE p.oracle_id <> d.commander_oracle;

-- Basic lands for each of the commander's colors, several copies each.
INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board,
                        created_at, updated_at)
SELECT d.id, b.id, b.oracle_id,
       CASE WHEN d.verified THEN pg_temp.pick(15, d.deck_no * 32 + b.bit, 3, 12)
            ELSE pg_temp.pick(15, d.deck_no * 32 + b.bit, 2, 6) END,
       'deck', d.created_at, d.created_at
FROM s_decks d
JOIN s_basics b ON d.mask & b.bit <> 0;

-- Commanders each account saved for later: none to five, all distinct.
INSERT INTO commander_maybeboard (user_id, oracle_id, created_at)
SELECT u.id, c.oracle_id, u.created_at + make_interval(days => k)
FROM s_users u
CROSS JOIN LATERAL generate_series(0, pg_temp.pick(16, u.n, 0, 5) - 1) AS k
JOIN s_commanders c
  ON c.pos = (pg_temp.pick(17, u.n, 0, 1499) + k * 1511) % (SELECT count(*) FROM s_commanders);

SELECT (SELECT count(*) FROM s_users) AS users,
       (SELECT count(*) FROM s_decks) AS decks,
       (SELECT count(*) FROM deck_cards dc JOIN s_decks d ON d.id = dc.deck_id) AS deck_cards,
       (SELECT count(*) FROM s_decks WHERE shared) AS shared_decks;

COMMIT;
