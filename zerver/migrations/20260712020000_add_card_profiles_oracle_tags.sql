-- Denormalized serve/filter projection: each card_profile carries its oracle_tag
-- set as a JSONB array (built nightly from card_oracle_tags), mirroring
-- card_profiles.mechanical_categories. GIN-indexed for `?|` / `@>` overlap tests
-- in the hot serve/filter path. See context/plans/otags/ (Phase 2).

ALTER TABLE card_profiles ADD COLUMN oracle_tags JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE INDEX idx_card_profiles_oracle_tags ON card_profiles USING GIN (oracle_tags);
