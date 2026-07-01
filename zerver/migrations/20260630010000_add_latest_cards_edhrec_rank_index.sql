-- Serve "sort by popularity" browsing (ORDER BY edhrec_rank ASC NULLS LAST,
-- name ASC — the EdhrecRank sort used by commander Zwipe-select and card search)
-- with an index instead of a full, disk-spilling sort of every matching creature.
-- Composite (edhrec_rank, name) so the name tiebreak that keeps pagination stable
-- is also index-ordered, letting Postgres walk in rank order and stop at LIMIT.
-- (edhrec_rank/name both ASC → NULLS LAST by default, matching the query.)
CREATE INDEX idx_latest_cards_edhrec_rank ON latest_cards (edhrec_rank, name);
