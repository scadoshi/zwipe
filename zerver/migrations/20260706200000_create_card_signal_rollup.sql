-- Pooled per-card suggestion signal, refreshed nightly by zervice (see
-- refresh_card_signal_rollup). Feeds the default synergy ordering's signal
-- term (context/plans/suggestion_signal.md, Phase 3a+3b).
--
-- net encodes what each gesture means: an add is full credit, a maybe is
-- expressed interest (half credit; impression-only would have made it
-- arithmetically identical to a skip), a deliberate removal is a full
-- take-back. Skips need no term here: shown = added + skipped + maybed, so
-- every skip is denominator drag. net can go negative — that's demotion.
CREATE MATERIALIZED VIEW card_signal_rollup AS
SELECT card_oracle_id,
       SUM(added + 0.5 * maybed - removed)::float8 AS net,
       SUM(shown)::float8                          AS shown
FROM commander_card_signal
GROUP BY card_oracle_id;

CREATE UNIQUE INDEX idx_card_signal_rollup_card
    ON card_signal_rollup (card_oracle_id);
