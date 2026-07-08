-- First-party commander-select signal: which command-zone candidates Zwipe
-- users are shown vs. select on the Zwipe-select screen. Pure aggregate keyed
-- by the candidate card's oracle_id — NO user_id, no per-swipe rows (same
-- posture as commander_card_signal). `shown` is the impression denominator,
-- client-derived as selected + skipped; `selected` is a right-swipe pick into
-- the command zone; `skipped` is a left-swipe pass. Read by the select serve's
-- wildcard deep slice (least-shown weighting) and, later, an optional
-- popularity term. See context/plans/commander_select_signal.md.
CREATE TABLE commander_select_signal (
    commander_oracle_id UUID        NOT NULL PRIMARY KEY,
    shown               BIGINT      NOT NULL DEFAULT 0,
    selected            BIGINT      NOT NULL DEFAULT 0,
    skipped             BIGINT      NOT NULL DEFAULT 0,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
