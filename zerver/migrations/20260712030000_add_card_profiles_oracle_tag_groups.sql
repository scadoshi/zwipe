-- Server-computed oracle-tag grouping for the card display: each card's
-- functional oracle tags bucketed under the coarse role they fall beneath (via
-- the tag hierarchy + the role root map), plus an "other" bucket for functional
-- tags under no role. Computed server-side so the role<->tag mapping and the
-- noise filter update on deploy, without waiting on mobile client releases.
-- See context/plans/otags/.

ALTER TABLE card_profiles
    ADD COLUMN oracle_tags_by_role JSONB NOT NULL DEFAULT '{}',
    ADD COLUMN other_oracle_tags JSONB NOT NULL DEFAULT '[]';
