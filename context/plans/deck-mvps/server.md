# Deck MVPs — server changes

File-by-file. Steps 1–7 ship with 1.4.0 (phase 1); 8–9 are server-only
follow-ups. After any query change: `cargo sqlx prepare --workspace` **from
the workspace root** (never from `zerver/` — see `operations/infrastructure/cicd.md`).

## 1. Migration — `zerver/migrations/<ts>_add_deck_card_mvp.sql`

```sql
-- MVP podium: up to 3 starred cards per deck. The timestamp is the vesting
-- clock (global signal counts it after 3 days); NULL = not an MVP. Cap and
-- board rules enforced in the update handler, not the schema.
ALTER TABLE deck_cards ADD COLUMN mvp_at TIMESTAMPTZ;
```

## 2. Core type — `zwipe-core/src/domain/deck/models/deck_card.rs`

`DeckCard` gains `pub mvp_at: Option<DateTime<Utc>>` with `#[serde(default)]`
(new client ↔ old server parses; old client ignores the new field). chrono is
an allowed core dependency.

## 3. Contract — `zwipe-core/src/http/contracts/deck_card.rs`

`HttpUpdateDeckCard` gains:

```rust
/// Star (true) or unstar (false) this card as a deck MVP. Absent = untouched.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub mvp: Option<bool>,
```

## 4. Domain request — `zerver/src/lib/domain/deck/models/deck_card/update_deck_card.rs`

Thread `mvp: Option<bool>` through the request type + constructor, mirroring
how `board` rides today.

## 5. Handler — `zerver/src/lib/inbound/http/handlers/deck_card/update_deck_card.rs`

Map `body.mvp` into the domain request. New error variant maps to 422 with
the exact copy **"This deck already has 3 MVPs"** (sentence case, no em
dashes — client shows it verbatim).

## 6. Repository — `zerver/src/lib/outbound/sqlx/deck/mod.rs`

In the update fn's tx:

- **Cap check** when `mvp == Some(true)`: count mainboard MVPs on the deck
  excluding this row; `>= 3` → the cap error.
- **Stamp**: `SET mvp_at = CASE WHEN $mvp IS NULL THEN mvp_at
  WHEN $mvp THEN COALESCE(mvp_at, now()) ELSE NULL END`
  (`COALESCE` keeps the original vesting clock if a client re-sends true).
- **Board rule**: MVPs are mainboard-only. Moving a card off `deck` board
  clears `mvp_at` in the same UPDATE.

## 7. Every `deck_cards` read/copy site

Grep `scryfall_data_id, oracle_id, quantity, board` in
`zerver/src/lib/outbound/sqlx/deck/mod.rs` and add `mvp_at` to each column
list — known sites: create RETURNING (~line 128), import insert (~597),
**clone bulk-copy (~675: both the INSERT columns and the SELECT — clone
inherits MVPs)**, plus the get-deck SELECTs. `DatabaseDeckCard` in
`outbound/sqlx/deck/models.rs` + its `TryFrom` gain the field.

## 8. Signal weight (phase 2, server-only)

New matview + refresh (mirror `card_signal_rollup` end to end — migration,
`refresh_deck_mvp_rollup` through the card ports, zervice call):

```sql
CREATE MATERIALIZED VIEW deck_mvp_rollup AS
SELECT lc.oracle_id AS card_oracle_id, COUNT(*)::float8 AS vested
FROM deck_cards dc
JOIN decks d ON d.id = dc.deck_id
JOIN latest_cards lc ON lc.id = d.commander_id  -- resolve commander printing → oracle
WHERE dc.mvp_at IS NOT NULL AND dc.mvp_at < now() - interval '3 days'
  AND dc.board = 'deck'
GROUP BY lc.oracle_id;
```

(Actually group by the *card's* oracle: `dc.oracle_id` — the commander join
is for the future pair-level term; v1 pools per card:
`SELECT dc.oracle_id AS card_oracle_id, COUNT(*) ... GROUP BY dc.oracle_id`.)

Serving (`outbound/sqlx/card/mod.rs`): second LEFT JOIN in the
signal-ordering FROM, numerator term `+ W_MVP * COALESCE(mvp.vested, 0)`
inside the shrunk rate's numerator; `const W_MVP: f64 = 3.0` next to the
other dials.

## 9. Deck steering (phase 3, server-only)

For the serving deck, boost cards sharing the deck's MVPs' mechanical
categories: a scalar subquery collects the MVP cards'
`card_profiles.mechanical_categories` for `deck_id`, and the score gains
`+ W_STEER * (overlap count with that set)`. Score shifts happen *before*
banding, so steering reads as band migration. Design detail at build; keep
`W_STEER` a dial with 0.0 revert.
