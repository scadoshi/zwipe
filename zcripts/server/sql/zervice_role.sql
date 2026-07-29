-- Phase 2 of context/plans/zervice_least_privilege.md: scoped Postgres role
-- for the nightly sync. Grants come from the code, not guesswork — the sync
-- touches exactly these objects (enumerated 2026-07-29 from the four zervice
-- steps + metrics recording).
--
-- Run as the postgres superuser (CREATE ROLE + ALTER OWNER need it):
--   sudo -u postgres psql -d zwipe -f zervice_role.sql
-- Then set the password interactively (never store it in a file):
--   sudo -u postgres psql -d zwipe -c "\password zervice"

BEGIN;

CREATE ROLE zervice LOGIN;

GRANT CONNECT ON DATABASE zwipe TO zervice;
GRANT USAGE ON SCHEMA public TO zervice;

-- Write surface: card sync (step 1), oracle tags + projection/grouping
-- (step 2), derive categories (step 3), run metrics.
GRANT SELECT, INSERT, UPDATE, DELETE
    ON scryfall_data, card_profiles, oracle_tags, card_oracle_tags, zervice_metrics
    TO zervice;

-- Step 4: REFRESH MATERIALIZED VIEW requires OWNERSHIP, and the backing query
-- runs with the owner's privileges — so zervice also needs SELECT on the
-- signal sources (scryfall_data/card_profiles already covered above).
GRANT SELECT ON commander_card_signal, otag_context_signal TO zervice;
ALTER MATERIALIZED VIEW latest_cards OWNER TO zervice;
ALTER MATERIALIZED VIEW card_signal_rollup OWNER TO zervice;
ALTER MATERIALIZED VIEW otag_context_signal_rollup OWNER TO zervice;

-- Ownership transfer strips zwipe's implicit rights; grant reads back —
-- zerver serves from latest_cards and both rollups.
GRANT SELECT ON latest_cards, card_signal_rollup, otag_context_signal_rollup TO zwipe;

COMMIT;

-- Everything else (users, refresh_tokens, decks, deck_cards, signal writes,
-- email tokens, ...) is deliberately ungranted: a compromised zervice reads
-- zero user data.
--
-- FOOTGUN (documented in operations/infrastructure/server.md): a future
-- migration that drops/recreates one of the three matviews resets ownership
-- to the migration user — it must re-run the ALTER ... OWNER TO zervice and
-- the GRANT SELECT ... TO zwipe above, or the next nightly run fails loudly.
