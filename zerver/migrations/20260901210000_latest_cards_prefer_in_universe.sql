-- Rebuild latest_cards twice over, both halves feeding the in-universe work
-- (context/plans/in_universe_filter.md):
--
-- 1. New ORDER BY tiebreaker between language and recency: in-universe
--    printings win the pick over Universes Beyond ones (triangle stamp OR a
--    code in oou_sets, the table zervice overlays from zwipe-core's
--    OUT_OF_UNIVERSE_SETS). Third verse of the same shadowing bug fixed by
--    the 2026-06-06 (promo/digital) and 2026-08-12 (foreign-language)
--    rebuilds: as of 2026-09-01 Sol Ring's pick was a Secret Lair Drop row,
--    so excluding that set hid Sol Ring from search entirely. UB-only cards
--    tie on the new term and keep behaving as before. Consequence worth
--    keeping: after this, a pick that is still OOU proves the card has no
--    in-universe printing, so the pick's own stamp+set is a card-level truth
--    the coming exclude-UB filter can read directly.
--
-- 2. New column printing_set_names text[]: every set the card has EVER been
--    printed in, aggregated per oracle identity, GIN-indexed. Set filters
--    will move onto it ("has a printing in X" / "every printing excluded")
--    so that no exclusion list can hide a card that also exists elsewhere;
--    today they consult only the picked printing's set_name. Deliberately
--    aggregates ALL printings, digital-only ones included; refine only if
--    that ever proves confusing in practice.
--
-- FOOTGUN (operations/infrastructure/server.md): this recreate resets the
-- matview's owner to the migration user; re-run zcripts/server/sql/
-- zervice_role.sql afterward or the nightly refresh fails loudly.

-- Snapshot the outgoing picks first: unlike the 2026-06-06/08-12 remaps
-- (which repointed EVERY row not on the new pick, deliberate printing-sheet
-- choices included), this remap moves only rows sitting on a pick that
-- changed. A hand-chosen printing is left alone.
CREATE TEMP TABLE old_picks AS
SELECT COALESCE(oracle_id, id) AS okey, id
FROM latest_cards;

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
-- Composite (edhrec_rank, name) matching 20260630010000 — the DROP above
-- destroys it, so the rebuild recreates it with the same shape.
CREATE INDEX idx_latest_cards_edhrec_rank ON latest_cards (edhrec_rank, name);
-- New: array-semantics set filters.
CREATE INDEX idx_latest_cards_printing_set_names ON latest_cards USING GIN (printing_set_names);

REFRESH MATERIALIZED VIEW latest_cards;

-- Remap deck and deck_card references off picks that changed: a row on the
-- OLD pick for its card moves to the NEW pick (Sol Ring rows on the Secret
-- Lair default move to the in-universe one). Rows on any other printing were
-- chosen on purpose (printing sheet) and stay put. commander_maybeboard keys
-- by oracle_id, so it needs no remap.

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
