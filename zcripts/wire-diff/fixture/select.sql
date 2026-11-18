-- The cards the CI fixture holds, one printing each (the one latest_cards shows): the most played
-- Commander-legal English cards, the most played legendary creatures for the seed's commanders,
-- the basic lands and every card the replay names.
-- Ordered, so the same catalog gives the same fixture.
CREATE TEMP TABLE fixture_cards AS
WITH legal AS (
    SELECT id, oracle_id, name, type_line, edhrec_rank
    FROM latest_cards
    WHERE oracle_id IS NOT NULL AND lang = 'en' AND legalities->>'commander' = 'legal'
),
picked AS (
    (SELECT oracle_id FROM legal ORDER BY edhrec_rank NULLS LAST, oracle_id LIMIT 300)
    UNION
    (SELECT oracle_id FROM legal WHERE type_line LIKE 'Legendary Creature%'
     ORDER BY edhrec_rank NULLS LAST, oracle_id LIMIT 100)
    UNION
    SELECT oracle_id FROM legal WHERE type_line LIKE 'Basic Land%'
    UNION
    SELECT oracle_id FROM legal WHERE name = 'Sol Ring' OR name LIKE 'Krenko%'
)
SELECT lc.id, lc.oracle_id
FROM latest_cards lc
JOIN picked p USING (oracle_id);

-- The oracle tags the picked cards carry, with every ancestor, so the tag tree stays whole.
CREATE TEMP TABLE fixture_tags AS
WITH RECURSIVE tags AS (
    SELECT DISTINCT ot.id, ot.parent_ids
    FROM oracle_tags ot
    JOIN card_oracle_tags cot ON cot.oracle_tag = ot.slug
    JOIN fixture_cards fc ON fc.oracle_id = cot.oracle_id
    UNION
    SELECT parent.id, parent.parent_ids
    FROM tags
    JOIN oracle_tags parent ON parent.id = ANY (tags.parent_ids)
)
SELECT DISTINCT id FROM tags;
