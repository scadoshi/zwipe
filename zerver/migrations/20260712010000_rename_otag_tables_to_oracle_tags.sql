-- Canonicalize the oracle-tag naming: the concept is `oracle_tag` throughout
-- (DB / Rust / wire). Renames the Phase-1 tables + `otag` column off the `otag`
-- shorthand. Forward-only rename; preserves existing rows. See context/plans/otags/.

ALTER TABLE card_otags RENAME COLUMN otag TO oracle_tag;
ALTER TABLE card_otags RENAME TO card_oracle_tags;
ALTER TABLE otags RENAME TO oracle_tags;

ALTER INDEX idx_card_otags_otag RENAME TO idx_card_oracle_tags_oracle_tag;
ALTER INDEX idx_card_otags_oracle_id RENAME TO idx_card_oracle_tags_oracle_id;
