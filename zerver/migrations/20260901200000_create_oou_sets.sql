-- Wholly Universes Beyond set codes, the DB half of zwipe-core's
-- OUT_OF_UNIVERSE_SETS const (domain/card/models/scryfall_data/universe.rs).
-- zervice overlays the const into this table before every latest_cards
-- refresh, so list updates ride a normal zerver deploy + nightly run with no
-- migration. The upcoming latest_cards rebuild reads it in its ORDER BY to
-- prefer in-universe printings; a card whose picked printing is still OOU
-- provably has no in-universe printing at all.
--
-- Seeded here with the const's codes as of 2026-09-01 so a same-deploy view
-- rebuild picks correctly before the first zervice run; the overlay owns the
-- contents from then on. Grants for the zervice role live in
-- zcripts/server/sql/zervice_role.sql (re-run it after this deploys).

CREATE TABLE oou_sets (
    code text PRIMARY KEY
);

INSERT INTO oou_sets (code) VALUES
    -- The Lord of the Rings: Tales of Middle-earth
    ('ltr'), ('ltc'), ('pltr'), ('tltr'), ('tltc'), ('pltc'),
    -- Doctor Who
    ('who'), ('twho'),
    -- Fallout
    ('pip'), ('tpip'),
    -- Warhammer 40,000
    ('40k'), ('t40k'),
    -- Final Fantasy
    ('fin'), ('fic'), ('fca'), ('pfin'), ('tfin'), ('tfic'), ('wfin'), ('rfin'), ('pss5'),
    -- Assassin's Creed
    ('acr'), ('tacr'),
    -- Jurassic World Collection
    ('rex'), ('trex'),
    -- Transformers
    ('bot'), ('tbot'),
    -- Marvel's Spider-Man
    ('spm'), ('spe'), ('pspm'), ('mar'), ('lmar'), ('aspm'), ('tspm'),
    -- Avatar: The Last Airbender
    ('tla'), ('tle'), ('ptla'), ('ttla'), ('ttle'), ('atla'), ('atle'), ('ftla'), ('jtla'),
    -- Marvel Super Heroes
    ('msh'), ('msc'), ('tmsh'), ('tmsc'), ('amsh'), ('fmsc'),
    -- Teenage Mutant Ninja Turtles
    ('tmt'), ('tmc'), ('pza'), ('ttmt'), ('ttmc'), ('ftmc'), ('atmt'),
    -- The Hobbit
    ('hob'), ('hoc'), ('thob'),
    -- Star Trek
    ('trk'), ('trc'), ('ttrk'), ('sds');
