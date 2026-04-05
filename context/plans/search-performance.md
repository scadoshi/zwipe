# Search Query Performance — Plan

User-facing card search queries are hitting SQLx slow query warnings after switching from `oracle_cards` (~35k) to `default_cards` (~110k+). This is a real performance issue affecting every search.

## Root Cause

The search query in `zerver/src/lib/outbound/sqlx/card/mod.rs:175-618` has structural bottlenecks that scale poorly with 3x more data:

### Problem 1: ILIKE with leading wildcards
`name ILIKE '%sol%'` forces a sequential scan on 110k rows. No B-tree index can help with leading `%`. Affects name, type_line, oracle_text, and flavor_text searches.

### Problem 2: color_identity_within power-set explosion
For a 3-color commander, generates 7 OR clauses. For WUBRG, 31 OR clauses:
```sql
(color_identity <@ ARRAY['W'] OR color_identity <@ ARRAY['U'] OR color_identity <@ ARRAY['W','U'] OR ...)
```
Should be a single `color_identity <@ ARRAY['W','U','B']::text[]` operator.

### Problem 3: Double table read
The CTE scans `scryfall_data JOIN card_profiles`, applies `ROW_NUMBER() OVER (PARTITION BY oracle_id)`, then the outer query re-joins `scryfall_data` to get `SELECT *`. Two passes over a wide 88-column table.

### Problem 4: Window function on all matching rows
`ROW_NUMBER()` runs across every row that passes the WHERE filters before `LIMIT` can take effect.

## Fixes (ordered by impact)

### Fix 1: pg_trgm indexes for ILIKE queries (highest impact, easiest)
```sql
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_scryfall_data_name_trgm ON scryfall_data USING GIN (name gin_trgm_ops);
CREATE INDEX idx_scryfall_data_oracle_text_trgm ON scryfall_data USING GIN (oracle_text gin_trgm_ops);
CREATE INDEX idx_scryfall_data_type_line_trgm ON scryfall_data USING GIN (type_line gin_trgm_ops);
```
Turns `ILIKE '%sol%'` from sequential scan → index scan. Single migration, no code changes.

### Fix 2: Replace color_identity_within power-set with `<@` operator
Current (lines 310-338): generates all 2^N-1 subsets as OR clauses.
Replace with:
```sql
color_identity <@ ARRAY['W','U','B']::text[]
```
Single operator, same semantics ("color identity is a subset of these colors"). May need a GIN index on `color_identity`:
```sql
CREATE INDEX idx_scryfall_data_color_identity ON scryfall_data USING GIN (color_identity);
```

### Fix 3: Eliminate double table read
Instead of CTE → re-join, select all needed columns inside the CTE and filter `rn = 1` directly:
```sql
SELECT * FROM (
  SELECT scryfall_data.*, card_profiles.is_token, card_profiles.mechanical_categories,
         ROW_NUMBER() OVER (PARTITION BY COALESCE(oracle_id, id) ORDER BY released_at DESC) as rn
  FROM scryfall_data
  JOIN card_profiles ON scryfall_data.id = card_profiles.scryfall_data_id
  WHERE <filters>
) sub
WHERE rn = 1
ORDER BY ... LIMIT 100 OFFSET 0
```
Eliminates the second scryfall_data scan.

### Fix 4: Consider materialized view for deduplication (future)
If performance is still an issue, a materialized view of "latest printing per oracle_id" refreshed after each sync would eliminate the window function entirely from search queries. The sync already runs nightly, so the refresh cost is amortized.

## Files to Change

| File | Change |
|------|--------|
| New migration | `pg_trgm` extension + trigram indexes + color_identity GIN index |
| `zerver/src/lib/outbound/sqlx/card/mod.rs:310-338` | Replace power-set generation with single `<@` operator |
| `zerver/src/lib/outbound/sqlx/card/mod.rs:178-581` | Restructure CTE to avoid double table read |

## Verification

- Run `EXPLAIN ANALYZE` on a typical search query before and after to measure improvement
- Check that SQLx slow query warnings stop for normal search operations
- Confirm sync still works (indexes add overhead to writes, but sync is nightly and tolerant)
