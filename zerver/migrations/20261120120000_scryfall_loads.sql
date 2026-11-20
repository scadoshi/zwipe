-- The Scryfall bulk file each sync loaded last, one row per kind ('default_cards',
-- 'oracle_tags'): when Scryfall made it, where it came from and how many items it held.
-- zervice refuses a file older than the one recorded here, and a load only ever raises
-- updated_at.
--
-- Additive only: a new table nothing references, so the running zervice and zerver keep
-- working unchanged until their new binaries ship.
CREATE TABLE IF NOT EXISTS scryfall_loads (
    kind       TEXT        PRIMARY KEY,
    updated_at TIMESTAMPTZ NOT NULL,
    uri        TEXT        NOT NULL,
    items      BIGINT      NOT NULL,
    loaded_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- zervice reads and raises its own rows. zcripts/server/sql/zervice_role.sql holds the same
-- grant. A test database has no such role.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'zervice') THEN
        GRANT SELECT, INSERT, UPDATE ON scryfall_loads TO zervice;
    END IF;
END $$;
