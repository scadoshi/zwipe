-- no-transaction
--
-- Fills name_slug and front_slug on the rows already stored, 5,000 at a time
-- in id order, each batch committed on its own so no lock lasts longer than
-- one batch. The fold is zwipe-core's card_slug in SQL: lower case, accents
-- off through f_unaccent, every other run of characters one dash, no dash at
-- either end, NULL when nothing is left. A name the two fold differently
-- stays wrong only until the next sync writes the Rust value. Running it again
-- recomputes every row and writes only the ones that differ.
DO $$
DECLARE
    after uuid := '00000000-0000-0000-0000-000000000000';
    upto uuid;
BEGIN
    LOOP
        SELECT id INTO upto
        FROM (SELECT id FROM scryfall_data WHERE id > after ORDER BY id LIMIT 5000) batch
        ORDER BY id DESC
        LIMIT 1;
        EXIT WHEN upto IS NULL;
        UPDATE scryfall_data sd
        SET name_slug = folded.name_slug, front_slug = folded.front_slug
        FROM (
            SELECT id,
                   NULLIF(btrim(regexp_replace(f_unaccent(lower(name)), '[^a-z0-9]+', '-', 'g'), '-'), '') AS name_slug,
                   CASE WHEN strpos(name, ' // ') > 0 THEN
                       NULLIF(btrim(regexp_replace(f_unaccent(lower(split_part(name, ' // ', 1))), '[^a-z0-9]+', '-', 'g'), '-'), '')
                   END AS front_slug
            FROM scryfall_data
            WHERE id > after AND id <= upto
        ) folded
        WHERE sd.id = folded.id
          AND (sd.name_slug, sd.front_slug) IS DISTINCT FROM (folded.name_slug, folded.front_slug);
        after := upto;
        COMMIT;
    END LOOP;
END
$$;
