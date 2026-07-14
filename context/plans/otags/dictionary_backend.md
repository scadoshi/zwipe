# Dictionary backend — serving the oracle-tag definitions (build + test plan)

**TL;DR: the backend that serves all the definitions already exists, and it's
already edge-cached.** The dictionary page consumes `GET /api/card/oracle-tags`,
which returns the full ~4,500-tag catalog with our overlaid descriptions. So this is
a **verify + harden + test** job, not a new endpoint. The one genuinely new concern
is **cache freshness** (descriptions change on our deploys, not just nightly syncs).

Part 2 UX/client of the dictionary lives in
[`tag_descriptions_and_dictionary.md`](tag_descriptions_and_dictionary.md); this doc
is only the backend/serving/caching slice.

---

## What already exists (nothing to build for MVP)

| Layer | Where | Notes |
|-------|-------|-------|
| Route | `zerver/.../http/routes.rs` → `GET /api/card/oracle-tags` | **Public** (in the `/card` nest, no auth), governor-limited by `public_card_config`. |
| Handler | `zerver/.../http/handlers/card/get_oracle_tags.rs` | Returns `(200, Json<Vec<OracleTag>>)`. No origin `Cache-Control` (correct — CF owns the TTL). |
| Service | `card_service.get_oracle_tags()` | Thin passthrough to the repo. |
| Repo query | `zerver/.../outbound/sqlx/card/mod.rs::get_oracle_tags` | `query_as!` over `oracle_tags`, `ORDER BY slug`, `parent_slugs` resolved via a correlated subquery over `parent_ids`. **Returns ALL rows** (no WHERE) — undescribed tags included, which the dictionary needs. |
| Wire type | `zwipe-core/.../card/models/oracle_tag.rs::OracleTag` | `{ slug, label, description: Option<String>, parent_slugs: Vec<String> }` — exactly the dictionary shape. Serde-derived, lives in core (shared). |
| Client | `zwiper/.../outbound/client/card/get_oracle_tags.rs` | `ClientGetOracleTags`; `self.client.get(url).send()` — **sends NO `Authorization: Bearer`** → cache-eligible (see caching note). |

The descriptions themselves land in `oracle_tags.description` via the sync-time
overlay of `ORACLE_TAG_DESCRIPTIONS` (`helpers/oracle_tags.rs`); the endpoint just
reads the column. 1,100 authored so far.

---

## Caching — already covered, with one freshness caveat

`/api/card/oracle-tags` is **already** matched by Cloudflare **Rule 1 "Cache card
metadata"** (`starts_with(path, "/api/card/")`, Edge TTL **24h**, ignore origin
Cache-Control) — see [`../../operations/infrastructure/cloudflare.md`](../../operations/infrastructure/cloudflare.md).
**No new cache rule is required.** Origin gets ~one hit per POP per day; cache HITs
skip the tunnel (~5–10ms vs ~125ms).

Two things to know:

1. **No-bearer requirement (already satisfied).** CF bypasses cache for requests
   carrying `Authorization`. The `get_oracle_tags` client method sends none, and the
   route is public — so it caches. **Test guards this** (below): if someone ever makes
   the dictionary call authed, cache HITs silently vanish and origin load 100×'s.

2. **Freshness — the one real new concern.** Rule 1's 24h TTL assumes card metadata
   only changes on the nightly Scryfall sync. But **oracle-tag descriptions change on
   our deploys** (edit the const → push → next `zervice` overlays). With a 24h edge
   TTL, a freshly deployed description batch can take **up to 24h** to appear in the
   dictionary. Options:
   - **Accept the lag** (descriptions aren't urgent) — simplest, recommended default.
   - **Purge on deploy:** after a description deploy + `zervice` run, purge the one URL
     (dashboard: Caching → Purge → Custom → URL `https://api.zwipe.net/api/card/oracle-tags`;
     or the API call in the cloudflare doc). Surgical, evicts just that response.
   - A dedicated `eq(path, "/api/card/oracle-tags")` rule with a shorter TTL is possible
     but the free-plan minimum is 2h and it spends one of the 10 rule slots — not worth
     it; purge-on-deploy is better when immediacy matters.

---

## Perf & payload (fine as-is)

- **Query:** correlated subquery resolves `parent_slugs` per row across 4,494 rows.
  `p.id = ANY(o.parent_ids)` hits the `oracle_tags` PK — a handful of index lookups
  per row. At one origin hit per POP per day this is a non-issue; no optimization
  needed. (If it ever mattered: a single `LEFT JOIN LATERAL` or a recursive CTE would
  collapse it, but don't pre-optimize a daily query.)
- **Payload:** 4,494 rows, ~263 kB of slug+label+description text, ~**0.5–0.7 MB**
  JSON uncompressed, ~**100–150 kB** gzipped (CF compresses). The dictionary fetches
  once per open, holds in memory, searches client-side. Fine.
- **`query_as!` is a MACRO** → it uses `.sqlx/` offline data. We are **not** changing
  the query, so no `cargo sqlx prepare` is needed. (If a future change does touch it,
  run `cargo sqlx prepare --workspace` from the root and commit `.sqlx/`.)

---

## Test plan (the actual deliverable)

Existing coverage: `zerver/tests/repo_oracle_tags.rs::get_oracle_tags_returns_catalog_with_parent_slugs`
(repo layer: ordering, parent-slug resolution, NULL description preserved). Keep it.

**Add (mirroring `zerver/tests/changelog.rs`, which uses the `common::TestApp` harness):**

1. **HTTP contract + public/no-auth (guards cacheability).** New
   `zerver/tests/http_oracle_tags.rs`: seed a couple of `oracle_tags` rows, then
   `TestApp::new(pool).get("/api/card/oracle-tags", None)` (note: `None` auth):
   - assert `200`,
   - body is a JSON array ordered by slug,
   - each entry has `slug` / `label` / `description` / `parent_slugs`,
   - a parent/child pair resolves `parent_slugs` correctly,
   - a row with no description serializes `description: null` (dictionary shows
     "No description yet").
   The **no-auth** assertion is the important one — it's the regression guard for the
   CF cache contract.

2. **Overlay → serve, end-to-end (currently untested).** In `repo_oracle_tags.rs`:
   call `sync_oracle_tags` with a tag whose slug is a real entry in
   `ORACLE_TAG_DESCRIPTIONS` (e.g. `spot-removal`) carrying a **different** Scryfall
   description, then `get_oracle_tags` and assert the served description is **ours**,
   not Scryfall's. Proves the full pipeline (const → overlay → column → endpoint). Pair
   it with a slug NOT in the const to assert Scryfall's text (or NULL) passes through
   untouched.

3. **(Optional) Full-catalog completeness.** Assert the endpoint returns undescribed
   tags too (already implied by "no WHERE"); a small explicit test stops a future
   "only serve described tags" optimization from silently breaking the dictionary.

Run: `set -a; source zerver/.env; set +a; cargo test -p zerver --test repo_oracle_tags --test http_oracle_tags`.

---

## Verification runbook (post-deploy)

```bash
# cache working? expect MISS then HIT on a second call to the same POP
curl -sI https://api.zwipe.net/api/card/oracle-tags | grep -i cf-cache-status
# shape sanity
curl -s https://api.zwipe.net/api/card/oracle-tags | jq 'length, .[0]'
```

After a description deploy, if you want it live before the 24h TTL: purge the one URL
(see Caching §2).

---

## Summary of work

- **Build:** nothing required for MVP — endpoint, wire type, client, and CF cache all
  exist. (Optional later: query micro-opt, only if a cold origin hit ever shows up slow.)
- **Test:** add the HTTP no-auth/contract test (#1) and the overlay→serve integration
  test (#2). ~1–2 hours.
- **Ops:** decide freshness policy (accept 24h lag vs purge-on-deploy). Recommend
  accept-the-lag now, purge manually after a big batch if you want it visible.
