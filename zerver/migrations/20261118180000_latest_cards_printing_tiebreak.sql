-- no-transaction
--
-- latest_cards picks one printing per card with DISTINCT ON, and its ORDER BY
-- ended at released_at, so printings tied on every key (the same release
-- date, language, frame and layout) were picked in no fixed order: a refresh
-- or a restored copy of the database could show a different printing of the
-- same card; on the full catalog, 6,633 cards have two or more printings
-- tied on every key. The ORDER BY now ends at the collector number, its
-- leading digits compared as a number so 99 comes before 100, and the
-- printing id, the tail the wanted-printing swap in the card search uses, so
-- every pick is fixed. A card whose pick was not tied keeps its printing.
--
-- The view is built under a temporary name with every index while searches
-- keep reading the old one, then swapped in one short transaction that takes
-- over the old view's owner and grants and moves deck references off a pick
-- that changed, so no search ever finds the view missing. Running it again
-- after a failure starts from a clean temporary view.

DROP MATERIALIZED VIEW IF EXISTS latest_cards_next;

CREATE MATERIALIZED VIEW latest_cards_next AS
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
         (sd.layout = 'reversible_card') ASC,
         sd.released_at DESC,
         -- Only ties reach these: they make the pick the same on every
         -- refresh and every copy of the database. The collector number's
         -- leading digits compare as a number, so 99 comes before 100.
         substring(sd.collector_number FROM '^[0-9]+')::numeric ASC NULLS LAST,
         sd.collector_number ASC,
         sd.id ASC
WITH DATA;

CREATE UNIQUE INDEX idx_latest_cards_next_id ON latest_cards_next(id);
CREATE UNIQUE INDEX idx_latest_cards_next_oracle_id ON latest_cards_next (oracle_id);
CREATE INDEX idx_latest_cards_next_name_trgm ON latest_cards_next USING GIN (name gin_trgm_ops);
CREATE INDEX idx_latest_cards_next_oracle_text_trgm ON latest_cards_next USING GIN (oracle_text gin_trgm_ops);
CREATE INDEX idx_latest_cards_next_type_line_trgm ON latest_cards_next USING GIN (type_line gin_trgm_ops);
CREATE INDEX idx_latest_cards_next_color_identity ON latest_cards_next USING GIN (color_identity);
CREATE INDEX idx_latest_cards_next_name ON latest_cards_next(name);
CREATE INDEX idx_latest_cards_next_cmc ON latest_cards_next(cmc);
CREATE INDEX idx_latest_cards_next_rarity ON latest_cards_next(rarity);
CREATE INDEX idx_latest_cards_next_set_name ON latest_cards_next(set_name);
CREATE INDEX idx_latest_cards_next_lang ON latest_cards_next(lang);
CREATE INDEX idx_latest_cards_next_edhrec_rank ON latest_cards_next (edhrec_rank, name);
CREATE INDEX idx_latest_cards_next_printing_set_names ON latest_cards_next USING GIN (printing_set_names);

ANALYZE latest_cards_next;

BEGIN;

-- The outgoing picks, so deck references can follow a pick that changed.
CREATE TEMP TABLE old_picks ON COMMIT DROP AS
SELECT COALESCE(oracle_id, id) AS okey, id
FROM latest_cards;

-- The new view takes over the old one's owner (zervice on the server, which
-- the nightly refresh needs) and every grant it had.
DO $$
DECLARE
    old_owner regrole;
    grant_row record;
BEGIN
    SELECT relowner::regrole INTO old_owner
    FROM pg_class
    WHERE oid = 'public.latest_cards'::regclass;
    IF old_owner::oid <> (SELECT oid FROM pg_roles WHERE rolname = current_user) THEN
        EXECUTE format('ALTER MATERIALIZED VIEW latest_cards_next OWNER TO %s', old_owner);
    END IF;
    FOR grant_row IN
        SELECT acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_class c, aclexplode(c.relacl) acl
        WHERE c.oid = 'public.latest_cards'::regclass
          AND acl.grantee <> c.relowner
    LOOP
        EXECUTE format(
            'GRANT %s ON latest_cards_next TO %s%s',
            grant_row.privilege_type,
            CASE WHEN grant_row.grantee = 0 THEN 'PUBLIC'
                 ELSE quote_ident(pg_get_userbyid(grant_row.grantee)) END,
            CASE WHEN grant_row.is_grantable THEN ' WITH GRANT OPTION' ELSE '' END
        );
    END LOOP;
END $$;

DROP MATERIALIZED VIEW latest_cards;
ALTER MATERIALIZED VIEW latest_cards_next RENAME TO latest_cards;
ALTER INDEX idx_latest_cards_next_id RENAME TO idx_latest_cards_id;
ALTER INDEX idx_latest_cards_next_oracle_id RENAME TO idx_latest_cards_oracle_id;
ALTER INDEX idx_latest_cards_next_name_trgm RENAME TO idx_latest_cards_name_trgm;
ALTER INDEX idx_latest_cards_next_oracle_text_trgm RENAME TO idx_latest_cards_oracle_text_trgm;
ALTER INDEX idx_latest_cards_next_type_line_trgm RENAME TO idx_latest_cards_type_line_trgm;
ALTER INDEX idx_latest_cards_next_color_identity RENAME TO idx_latest_cards_color_identity;
ALTER INDEX idx_latest_cards_next_name RENAME TO idx_latest_cards_name;
ALTER INDEX idx_latest_cards_next_cmc RENAME TO idx_latest_cards_cmc;
ALTER INDEX idx_latest_cards_next_rarity RENAME TO idx_latest_cards_rarity;
ALTER INDEX idx_latest_cards_next_set_name RENAME TO idx_latest_cards_set_name;
ALTER INDEX idx_latest_cards_next_lang RENAME TO idx_latest_cards_lang;
ALTER INDEX idx_latest_cards_next_edhrec_rank RENAME TO idx_latest_cards_edhrec_rank;
ALTER INDEX idx_latest_cards_next_printing_set_names RENAME TO idx_latest_cards_printing_set_names;

-- Move deck references off a pick that changed. A row on any other printing
-- was chosen deliberately (printing sheet) and stays put. commander_maybeboard
-- keys by oracle_id, so it needs no remap.
CREATE TEMP TABLE remap ON COMMIT DROP AS
SELECT op.id AS old_id, lc.id AS new_id
FROM old_picks op
JOIN latest_cards lc ON COALESCE(lc.oracle_id, lc.id) = op.okey
WHERE op.id <> lc.id;

UPDATE deck_cards dc SET scryfall_data_id = r.new_id FROM remap r WHERE dc.scryfall_data_id = r.old_id;
UPDATE decks d SET commander_id = r.new_id FROM remap r WHERE d.commander_id = r.old_id;
UPDATE decks d SET partner_commander_id = r.new_id FROM remap r WHERE d.partner_commander_id = r.old_id;
UPDATE decks d SET background_id = r.new_id FROM remap r WHERE d.background_id = r.old_id;
UPDATE decks d SET signature_spell_id = r.new_id FROM remap r WHERE d.signature_spell_id = r.old_id;

COMMIT;
