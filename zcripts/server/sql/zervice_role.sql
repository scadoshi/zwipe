-- Scoped Postgres role for the nightly sync/upkeep binary (zervice).
-- Phase 2 of context/archive/zervice_least_privilege.md + the upkeep grants
-- from context/plans/client-error-reporting.md.
--
-- IDEMPOTENT: first run and every re-run are the same command. Re-run this
-- whenever zervice gains a table (grants are the ledger of what it may touch).
--
--   sudo -u postgres psql -d zwipe < zervice_role.sql
--
-- First run only, set the password interactively (never store it in a file):
--   sudo -u postgres psql -d zwipe -c "\password zervice"
-- Dev parity: dev setup scripts run this same file with a throwaway local
-- password so a local zervice run can prove the grants (see dev-env/).

BEGIN;

DO $$ BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'zervice') THEN
        CREATE ROLE zervice LOGIN;
    END IF;
END $$;

-- current_database(): 'zwipe' on prod, 'zerver' on dev boxes.
DO $$ BEGIN
    EXECUTE format('GRANT CONNECT ON DATABASE %I TO zervice', current_database());
END $$;
GRANT USAGE ON SCHEMA public TO zervice;

-- Sync write surface: card sync (step 1), oracle tags + projection/grouping
-- (step 2), derive categories (step 3), run metrics.
GRANT SELECT, INSERT, UPDATE, DELETE
    ON scryfall_data, card_profiles, oracle_tags, card_oracle_tags, zervice_metrics
    TO zervice;

-- Step 4: REFRESH MATERIALIZED VIEW requires OWNERSHIP, and the backing query
-- runs with the owner's privileges — so zervice also needs SELECT on the
-- signal sources (scryfall_data/card_profiles already covered above).
GRANT SELECT ON commander_card_signal, otag_context_signal TO zervice;

-- Ownership transfer strips the previous owner's implicit read, and zerver
-- serves from these views — so capture each view's owner BEFORE the ALTER and
-- grant SELECT back to it ('zwipe' on prod, the dev user locally). Per-view,
-- so a migration that recreated one view heals on the next re-run.
DO $$
DECLARE v text; prev text;
BEGIN
    FOREACH v IN ARRAY
        ARRAY['latest_cards', 'card_signal_rollup', 'otag_context_signal_rollup']
    LOOP
        SELECT matviewowner INTO prev FROM pg_matviews WHERE matviewname = v;
        EXECUTE format('ALTER MATERIALIZED VIEW %I OWNER TO zervice', v);
        IF prev IS NOT NULL AND prev <> 'zervice' THEN
            EXECUTE format('GRANT SELECT ON %I TO %I', v, prev);
        END IF;
    END LOOP;
END $$;

-- Step 5 (upkeep): diagnostic retention prunes.
GRANT SELECT, DELETE ON client_errors, crash_reports TO zervice;

-- Step 5 (upkeep): global expired-session dusting. Deliberately
-- destruction-only — DELETE plus column-scoped SELECT on exactly the column
-- the WHERE clause reads. zervice can log users out (self-healing nuisance)
-- but can never read value_hash/user_id (zero session-data exposure).
GRANT DELETE ON refresh_tokens TO zervice;
GRANT SELECT (expires_at) ON refresh_tokens TO zervice;

COMMIT;

-- Everything else (users, decks, deck_cards, signal writes, email tokens, ...)
-- is deliberately ungranted: a compromised zervice reads zero user data.
--
-- FOOTGUN (documented in operations/infrastructure/server.md): a migration
-- that drops/recreates one of the three matviews resets ownership to the
-- migration user — re-run this file afterward, or the next nightly run fails
-- loudly via the alert email.
