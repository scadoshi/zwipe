# Search Query Performance — Complete

User-facing card search queries were hitting SQLx slow query warnings after switching from `oracle_cards` (~35k) to `default_cards` (~110k+). All four fixes have been implemented.

## Root Cause

The search query in `zerver/src/lib/outbound/sqlx/card/mod.rs` had structural bottlenecks that scaled poorly with 3x more data: leading-wildcard ILIKE without trigram indexes, exponential power-set OR clauses for color identity subset queries, a CTE that scanned the table twice, and ROW_NUMBER() computed on every request.

## Fixes Applied

### Fix 1: pg_trgm GIN indexes — Complete

`CREATE EXTENSION IF NOT EXISTS pg_trgm` in `20250810194450_create_scryfall_data.sql`. Trigram GIN indexes on `name`, `oracle_text`, `type_line` created on the `latest_cards` materialized view (see Fix 4). Turns `ILIKE '%sol%'` from sequential scan → index scan.

### Fix 2: Single `<@` operator for color_identity_within — Complete

Replaced 29-line power-set bit manipulation (generating up to 31 OR clauses for WUBRG) with:
```rust
sep.push("color_identity <@ ");
sep.push_bind_unseparated(colors.to_short_names());
```
GIN index on `color_identity` supports the `<@` operator.

### Fix 3: Eliminate double table read — Complete (subsumed by Fix 4)

The CTE selected only `id` + `rn`, then the outer query re-joined `scryfall_data` for `SELECT *`. With the materialized view, there is no CTE — queries read directly from the pre-deduplicated view.

### Fix 4: Materialized view for deduplication — Complete

`latest_cards` materialized view pre-computes "latest printing per oracle_id" using `DISTINCT ON`. Created in `20250810194452_create_latest_cards_view.sql` with `WITH NO DATA` (populated on first refresh). Refreshed by zervice after sync + classification.

Both `search_scryfall_data` (dynamic QueryBuilder) and `find_cards_by_exact_names` (static query) now query `latest_cards` (~35k rows) instead of computing ROW_NUMBER() over `scryfall_data` (~110k rows).

## Files Changed

| File | Change |
|------|--------|
| `zerver/migrations/20250810194450_create_scryfall_data.sql` | Added `pg_trgm` extension |
| `zerver/migrations/20250810194452_create_latest_cards_view.sql` | New — materialized view + all search indexes |
| `zerver/src/lib/domain/card/ports.rs` | Added `refresh_latest_cards` to both traits |
| `zerver/src/lib/domain/card/services.rs` | Added delegating implementation |
| `zerver/src/lib/outbound/sqlx/card/mod.rs` | Rewrote both queries to use `latest_cards`, replaced power-set with `<@`, added refresh impl |
| `zerver/src/bin/zervice.rs` | Calls `refresh_latest_cards` after sync + classification |
