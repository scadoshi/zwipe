-- Reversible cards arrive from Scryfall with no top-level oracle_id (it sits on
-- each face instead), and until now nothing filled it in. Two consequences:
--
-- 1. Adding one to a deck failed. The client sends oracle_id as a string, so an
--    absent id went over the wire as "" and the server rejected it with
--    "invalid oracle id: failed to parse a uuid". Six reports since 2026-08-06,
--    all from real installs, on cards people actually want: Blood Crypt,
--    Hallowed Fountain, Anointed Procession. Diagnosis in
--    context/plans/invalid_oracle_id.md.
--
-- 2. With no oracle_id they grouped by their own id in latest_cards, so each
--    appeared as a separate search result with a doubled name
--    ("Blood Crypt // Blood Crypt") alongside the normal printing.
--
-- This migration fixes both, and the ingest side stops it regressing
-- (ScryfallData::backfill_oracle_id_from_faces, applied in the Scryfall
-- adapter before the delta comparison sees the card).
--
-- The ORDER BY gains a reversible term, placed last so it only breaks ties the
-- language and in-universe keys left unresolved. Without it the backfill trades
-- one bug for another: measured against the catalog on 2026-09-22, 33 groups
-- would have been won by the reversible printing and shown the doubled name.
-- With it, all 71 affected groups pick a normal printing and none stay
-- reversible. Every affected card has a normal alternative, so nothing leaves
-- the catalog; the reversible printings simply stop being the face shown.
--
-- FOOTGUN (operations/infrastructure/server.md): this recreate resets the
-- matview's owner to the migration user; re-run zcripts/server/sql/
-- zervice_role.sql afterward or the nightly refresh fails loudly.

-- Snapshot the outgoing picks so deck references can follow a changed pick,
-- exactly as the 2026-09-01 rebuild does.
CREATE TEMP TABLE old_picks AS
SELECT COALESCE(oracle_id, id) AS okey, id
FROM latest_cards;

-- Both faces of a reversible card carry the same oracle_id (verified: 82 of 82
-- agree, none null), so either serves.
UPDATE scryfall_data
   SET oracle_id = (card_faces->0->>'oracle_id')::uuid
 WHERE oracle_id IS NULL
   AND card_faces->0->>'oracle_id' IS NOT NULL;

DROP MATERIALIZED VIEW IF EXISTS latest_cards;

CREATE MATERIALIZED VIEW latest_cards AS
SELECT DISTINCT ON (COALESCE(sd.oracle_id, sd.id))
       sd.*,
       agg.printing_set_names
FROM scryfall_data sd
JOIN card_profiles cp ON sd.id = cp.scryfall_data_id
JOIN (
    SELECT COALESCE(oracle_id, id) AS okey,
           array_agg(DISTINCT set_name) AS printing_set_names
    FROM scryfall_data
    GROUP BY 1
) agg ON agg.okey = COALESCE(sd.oracle_id, sd.id)
ORDER BY COALESCE(sd.oracle_id, sd.id),
         (sd.digital) ASC,
         (sd.promo) ASC,
         (sd.oversized) ASC,
         (COALESCE(sd.content_warning, false)) ASC,
         (sd.lang IS DISTINCT FROM 'en') ASC,
         -- COALESCE keeps a NULL stamp from poisoning the OR into NULL,
         -- which ASC would sort LAST, i.e. worse than an actual UB printing.
         (COALESCE(sd.security_stamp, '') = 'triangle'
          OR sd.set IN (SELECT code FROM oou_sets)) ASC,
         -- New: a normal printing beats a reversible one. Last of the
         -- preference keys so it never overrides language or universe.
         (sd.layout = 'reversible_card') ASC,
         sd.released_at DESC
WITH NO DATA;

CREATE UNIQUE INDEX idx_latest_cards_id ON latest_cards(id);
CREATE INDEX idx_latest_cards_name_trgm ON latest_cards USING GIN (name gin_trgm_ops);
CREATE INDEX idx_latest_cards_oracle_text_trgm ON latest_cards USING GIN (oracle_text gin_trgm_ops);
CREATE INDEX idx_latest_cards_type_line_trgm ON latest_cards USING GIN (type_line gin_trgm_ops);
CREATE INDEX idx_latest_cards_color_identity ON latest_cards USING GIN (color_identity);
CREATE INDEX idx_latest_cards_name ON latest_cards(name);
CREATE INDEX idx_latest_cards_cmc ON latest_cards(cmc);
CREATE INDEX idx_latest_cards_rarity ON latest_cards(rarity);
CREATE INDEX idx_latest_cards_set_name ON latest_cards(set_name);
CREATE INDEX idx_latest_cards_lang ON latest_cards(lang);
CREATE INDEX idx_latest_cards_edhrec_rank ON latest_cards (edhrec_rank, name);
CREATE INDEX idx_latest_cards_printing_set_names ON latest_cards USING GIN (printing_set_names);

REFRESH MATERIALIZED VIEW latest_cards;

-- Move deck references off a pick that changed. A row on any other printing
-- was chosen deliberately (printing sheet) and stays put. commander_maybeboard
-- keys by oracle_id, so it needs no remap.

WITH remap AS (
    SELECT op.id AS old_id, lc.id AS new_id
    FROM old_picks op
    JOIN latest_cards lc ON COALESCE(lc.oracle_id, lc.id) = op.okey
    WHERE op.id <> lc.id
)
UPDATE deck_cards dc
SET scryfall_data_id = r.new_id
FROM remap r
WHERE dc.scryfall_data_id = r.old_id;

WITH remap AS (
    SELECT op.id AS old_id, lc.id AS new_id
    FROM old_picks op
    JOIN latest_cards lc ON COALESCE(lc.oracle_id, lc.id) = op.okey
    WHERE op.id <> lc.id
)
UPDATE decks d
SET commander_id = r.new_id
FROM remap r
WHERE d.commander_id = r.old_id;

WITH remap AS (
    SELECT op.id AS old_id, lc.id AS new_id
    FROM old_picks op
    JOIN latest_cards lc ON COALESCE(lc.oracle_id, lc.id) = op.okey
    WHERE op.id <> lc.id
)
UPDATE decks d
SET partner_commander_id = r.new_id
FROM remap r
WHERE d.partner_commander_id = r.old_id;

WITH remap AS (
    SELECT op.id AS old_id, lc.id AS new_id
    FROM old_picks op
    JOIN latest_cards lc ON COALESCE(lc.oracle_id, lc.id) = op.okey
    WHERE op.id <> lc.id
)
UPDATE decks d
SET background_id = r.new_id
FROM remap r
WHERE d.background_id = r.old_id;

WITH remap AS (
    SELECT op.id AS old_id, lc.id AS new_id
    FROM old_picks op
    JOIN latest_cards lc ON COALESCE(lc.oracle_id, lc.id) = op.okey
    WHERE op.id <> lc.id
)
UPDATE decks d
SET signature_spell_id = r.new_id
FROM remap r
WHERE d.signature_spell_id = r.old_id;

DROP TABLE old_picks;
