# Wildcard slot — server implementation

One file: `zerver/src/lib/outbound/sqlx/card/mod.rs`, all inside
`search_scryfall_data_deck_aware`'s signal-ordering branch (the only branch
that changes; explicit sorts, synergy-ON membership, plain search untouched).

## Consts (next to the other dials)

```rust
/// Wildcard slots per page: cards drawn from beyond the reachable horizon
/// (rank > MAX_CARDS_IN_STACK) so the deep pool accrues impressions at all.
/// 0 reverts to pure band serving.
const WILDCARD_SLOTS: i64 = 1;
```

The horizon reuses `MAX_CARDS_IN_STACK`'s value (500) — define it here as
`DEEP_POOL_FLOOR: i64 = 500` with a comment tying it to the client cap
(zwiper `action_history.rs`), since zerver can't import zwiper's const.

## Query restructure — CTE with two slices

The current branch bakes the band ordering into the outer ORDER BY. The
wildcard needs the same ranked pool twice (band slice + deep slice), so the
query becomes a CTE when (and only when) the branch is active AND
`WILDCARD_SLOTS > 0` AND `deck_id` is Some:

```sql
WITH pool AS (
    SELECT latest_cards.*,
           row_number() OVER (ORDER BY <score expr> DESC, name ASC, latest_cards.id) AS rn,
           hashtext(COALESCE(latest_cards.oracle_id::text, '') || $seed) AS shuffle,
           COALESCE(sig.shown, 0) AS shown
    FROM latest_cards
    JOIN card_profiles ...
    LEFT JOIN card_signal_rollup sig ...
    CROSS JOIN (...) g
    WHERE <existing predicate pushes>
)
(SELECT * FROM pool
 ORDER BY (rn - 1) / {BAND_SIZE}, shuffle, name
 LIMIT {limit - WILDCARD_SLOTS} OFFSET {offset})
UNION ALL
(SELECT * FROM pool
 WHERE rn > {DEEP_POOL_FLOOR}
 ORDER BY shown ASC, shuffle
 LIMIT {WILDCARD_SLOTS} OFFSET {page_index * WILDCARD_SLOTS})
```

- `<score expr>` is the existing base + signal push (shared closure, as the
  band branch already does).
- `page_index = offset / limit` — each page draws the *next* deep cards, so
  paging never repeats a wildcard within a serve day.
- The extra SELECT columns (`rn`, `shuffle`, `shown`) ride along;
  `DatabaseScryfallData` is built with `query_as` on `latest_cards.*` — the
  extra columns are simply ignored by the struct mapping (verify at build;
  if the mapper is strict, wrap the outer slices in
  `SELECT <latest_cards columns> FROM (...)`).

## Splice (Rust side)

The UNION returns band cards then wildcards. Interleave in Rust after the
fetch: insert each wildcard at a fixed in-page position (proposed: index 17
of 25 — deep enough to not lead the hand, early enough to be seen). Keep it
one line: `cards.insert(17.min(cards.len()), wildcard)` per slot.

## Edge behavior

- **Deep pool empty** (pool smaller than 500 after filters — common with
  tight filters or synergy ON): the second slice returns nothing; serve the
  band slice as-is. This is automatic (no special casing) and correct —
  small pools have no unreachable tail to probe.
- **deck_id None**: no seed → no wildcard (branch already falls back to pure
  score ordering).
- **Duplicates**: the two slices are disjoint by construction
  (`rn <= horizon` band pagination can only overlap `rn > horizon` slice —
  it can't; band slice pages within ranked order and only reaches horizon at
  offset ≈ 475+, at which point the client's 500-cap has ended the session).

## Verification checklist

- Wildcard respects an active filter (e.g. type=Creature → wildcard is a
  creature).
- Skipped wildcard suppresses; next serve draws a different deep card.
- Same deck same day → same wildcards per page; tomorrow rotates.
- `WILDCARD_SLOTS = 0` → byte-identical to band serving.
- Perf: the CTE materializes the ranked pool once (~the current window-sort
  cost); confirm EXPLAIN stays in the current ballpark on the firehose pool.
