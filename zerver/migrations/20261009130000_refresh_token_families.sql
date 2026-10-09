-- Refresh token families. Every rotation carries family_id and login_at from
-- the row it replaces; a login starts a new family. A rotated row stays with
-- replaced_at set so a replay of it is recognizable (the whole family is then
-- deleted); the nightly upkeep removes replaced rows after a grace window.
-- Defaults keep inserts from a binary that does not know the columns valid.
-- Existing rows each become their own family that began at creation.

ALTER TABLE refresh_tokens
    ADD COLUMN family_id UUID NOT NULL DEFAULT gen_random_uuid(),
    ADD COLUMN login_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN replaced_at TIMESTAMPTZ;

UPDATE refresh_tokens SET login_at = created_at;

CREATE INDEX idx_refresh_tokens_family_id ON refresh_tokens(family_id);

-- The upkeep role reads replaced_at in its WHERE clause (see
-- zcripts/server/sql/zervice_role.sql); a test database has no such role.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'zervice') THEN
        GRANT SELECT (replaced_at) ON refresh_tokens TO zervice;
    END IF;
END $$;
