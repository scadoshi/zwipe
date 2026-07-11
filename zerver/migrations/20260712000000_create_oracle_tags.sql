-- Oracle tags (otags): community-maintained functional card tags from Scryfall's
-- Oracle Tags bulk file (the Tagger project). See context/plans/otags/.
--
-- Two tables: a catalog of every otag (with its human metadata + hierarchy) and a
-- normalized card -> otag correlation keyed by oracle_id. The correlation is the
-- source of truth; a denormalized JSONB serve projection on card_profiles arrives
-- in a later phase (context/plans/otags/sequencing.md).

-- Catalog: one row per otag.
CREATE TABLE otags (
    id UUID PRIMARY KEY,                          -- Scryfall tag id
    slug TEXT NOT NULL UNIQUE,                     -- the key we correlate on, e.g. 'removal'
    label TEXT NOT NULL,
    description TEXT,                              -- present for most tags, null for some
    parent_ids UUID[] NOT NULL DEFAULT '{}',       -- hierarchy (parent tag ids)
    aliases TEXT[] NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Card -> otag correlation (normalized source of truth). oracle_id is intentionally
-- unconstrained (a tag may reference oracle_ids not present locally, and oracle_id is
-- not unique in scryfall_data) -- mirrors the signal tables.
CREATE TABLE card_otags (
    oracle_id UUID NOT NULL,
    otag TEXT NOT NULL,                            -- otags.slug
    source TEXT NOT NULL DEFAULT 'scryfall',       -- 'scryfall' | 'heuristic' (later phase)
    PRIMARY KEY (oracle_id, otag)
);

CREATE INDEX idx_card_otags_otag ON card_otags (otag);
CREATE INDEX idx_card_otags_oracle_id ON card_otags (oracle_id);
