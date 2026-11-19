-- Pooled per-card swipe signal across every deck: the Add stack's impressions, adds, skips and
-- maybes and the deck's removals, keyed by the card alone. The ingest writes it for every deck the
-- user owns, with or without a commander, under the same daily budget as commander_card_signal.
-- Pure aggregate: no user_id, the same privacy posture as commander_card_signal.
--
-- A new table, so no lock falls on a live one: the backfill below only reads
-- commander_card_signal, which the ingest's upserts keep writing meanwhile. zerver owns and
-- writes it; zervice reads nothing here.
CREATE TABLE card_swipe_signal (
    card_oracle_id UUID      PRIMARY KEY,
    shown          BIGINT    NOT NULL DEFAULT 0,
    added          BIGINT    NOT NULL DEFAULT 0,
    skipped        BIGINT    NOT NULL DEFAULT 0,
    maybed         BIGINT    NOT NULL DEFAULT 0,
    removed        BIGINT    NOT NULL DEFAULT 0,
    updated_at     TIMESTAMP NOT NULL DEFAULT NOW()
);

-- The Commander history, summed per card. Swipes in decks with no commander were never stored
-- per card, so for those the counts start here.
INSERT INTO card_swipe_signal (card_oracle_id, shown, added, skipped, maybed, removed, updated_at)
SELECT card_oracle_id,
       SUM(shown),
       SUM(added),
       SUM(skipped),
       SUM(maybed),
       SUM(removed),
       MAX(updated_at)
FROM commander_card_signal
GROUP BY card_oracle_id;
