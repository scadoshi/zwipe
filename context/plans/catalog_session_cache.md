# Session catalog cache — prefetch filter metadata at app load

**Status: PLANNED 2026-07-13. Client-only (zwiper). No new backend endpoints.**  
**Related:** [`server_driven_catalogs.md`](server_driven_catalogs.md) (what catalogs
exist), [`otags/dictionary_client.md`](otags/dictionary_client.md) (first consumer
of oracle-tags cache), [`otags/dictionary_backend.md`](otags/dictionary_backend.md)
(CF Rule 1 already edge-caches `/api/card/*` at 24h).

**One sentence:** fetch slow-changing card/deck **catalog lists once at app
start** (with a **1-day TTL**), hold them above the router, and have filter
screens / pickers / the dictionary **read the cache** instead of each firing
`use_resource` on open.

---

## Why

Today, opening a filter subsection or picker typically does:

```text
open Artist filter → GET /api/card/artists
open Keywords     → GET /api/card/keywords
open Oracle words → GET /api/card/oracle-words
open Sets         → GET /api/card/sets
open Types        → GET /api/card/types
open Card roles   → GET /api/card/roles
open Oracle tags  → GET /api/card/oracle-tags   (picker + filter + soon dictionary)
open Deck tags    → GET /api/deck/tags          (create/edit; authed)
```

Those lists change when **Scryfall / zervice** refreshes (daily) or when **we
deploy** catalog consts — not when the user opens the filter sheet. Re-fetching
on every sheet open is wasteful (latency on mobile, redundant origin/CF work,
janky skeletons mid-flow).

**CF already treats card metadata as 24h-stable** (Rule 1: `/api/card/*`, Edge
TTL 24h). Client-side TTL of **1 day** matches that product reality and the
nightly sync cadence.

Existing precedents in `session_upkeep.rs`:

| Pattern | Scope | TTL |
|---------|--------|-----|
| `ChangelogCache` | session memory, fetch once per launch | none (one-shot) |
| `FlavorCard` | session memory | 1 hour |
| *(this plan)* | session memory (+ optional disk later) | **1 day** |

---

## Scope — what to cache

### In scope (public card catalogs — no auth)

All are `GET /api/card/…`, public routes, **must not** send `Authorization`
(so CF cache HITs keep working).

| Catalog | Client trait (today) | Typical consumer |
|---------|----------------------|------------------|
| Artists | `ClientGetArtists` | filter `artist.rs` |
| Keywords | `ClientGetKeywords` | filter keywords (add path) |
| Oracle words | `ClientGetOracleWords` | filter oracle words (add path) |
| Sets | `ClientGetSets` | filter `set.rs` |
| Card types | `ClientGetCardTypes` | filter other types |
| Card roles | `ClientGetCardRoles` | filter `category.rs` |
| Oracle tags | `ClientGetOracleTags` | picker, filter, **dictionary** |
| Languages | `ClientGetLanguages` | if/when a filter uses it; include if endpoint is live |

Payload sizes are list-shaped (strings or small view structs). Oracle tags is
the largest (~4.5k rows, ~0.5–0.7 MB JSON uncompressed / ~100–150 kB gzip) —
already acceptable in memory for the dictionary plan.

### In scope (authed, smaller)

| Catalog | Client trait | Notes |
|---------|--------------|--------|
| Deck tags | `ClientGetDeckTags` | Requires session; used on create/edit. Prefetch **after** session is available (or on first login / ensure_fresh), not as a blind cold-start public call. |

### Out of scope

| Data | Why not |
|------|---------|
| Card search results / swipe stacks | Per-deck, high churn; already has `add_stack_cache` |
| Individual cards / printings | Huge, per-id; CF edge is enough |
| Deck profiles / deck lists | User-specific, mutates constantly |
| Min client version | Polled on the 60s upkeep interval by design |
| Changelog | Already has `ChangelogCache` (one-shot; can later join the same *machinery* but not the 1d card-metadata TTL story) |
| Remove-screen deck-derived keyword/oracle-word lists | Those intentionally extract from **deck cards**, not the global catalog — keep that branch |

---

## Design

### 1. One app-scoped store above the router

In `spawn_upkeeper` (same place as `ChangelogCache` / filter store):

```text
CatalogCache (or per-kind signals)
  for each catalog kind:
    state: Loading | Loaded(T) | Failed
    fetched_at: Option<DateTime<Utc>>   // for TTL
```

**API surface for consumers** (sketch — names flexible):

```rust
// Pseudocode — exact shape TBD at implement time
fn catalog_artists(cache) -> CatalogSlice<&[String]>
// Loading | Ready(&[…]) | Failed
```

Filters stop owning `use_resource(|| client.get_artists())` and instead:

1. Read the session cache.
2. If `Loading` → existing skeleton / "Loading…" chip empty state.
3. If `Loaded` and not expired → use data.
4. If `Failed` or **expired** → trigger refresh (see below); show last good data
   if any (stale-while-revalidate) or empty + toast.

### 2. TTL = 1 day

```text
const CATALOG_TTL: Duration = Duration::from_secs(24 * 60 * 60);
```

- On read: if `now - fetched_at >= CATALOG_TTL`, schedule a background re-fetch
  for that kind (or all kinds).
- **Stale-while-revalidate preferred:** keep showing `Loaded` data while a
  refresh is in flight so filters never blank mid-session after 24h of continuous
  use (rare for mobile, but cheap).
- Cold start with empty store: fetch immediately (do not wait for first filter
  open).

**Why 1 day (not "once per launch" only):** a user can leave the app process
alive for days on iOS/Android; launch-once is not enough. Aligns with CF 24h and
zervice cadence. **Disk persistence across process death is optional Phase 2**
(see below) — Phase 1 is in-memory + refetch when expired or process restarts.

### 3. When to fetch

| Moment | Action |
|--------|--------|
| App start (`use_future` in upkeeper) | Prefetch **all public** catalogs (parallel `join!` / separate futures). |
| Session becomes available | Prefetch **deck tags** (authed). |
| Consumer finds expired / Failed | Single-flight re-fetch for that kind (dedupe concurrent callers). |
| Pull-to-refresh | **Not required** for MVP. |
| Manual "refresh cache" in Profile | **No.** Cold start (app fully quit / process killed) clears
  session memory and re-prefetches on next launch — enough for users and for
  owner testing. No Profile system-row; no "cache" language in the UI. Per-screen
  toast + retry only if a fetch **Failed**, not for "force freshness." |

Do **not** block first paint of Home on catalog completion — fire-and-forget
background, same as changelog.

### 4. Single-flight / no stampede

Multiple filters must not each start a full artists fetch if the cache is empty.
Pattern: one `fetching: bool` or shared future per kind (mirror
`EnsureFresh` single-flight spirit). Startup already serializes via one
`use_future` (or one task with `join_all`).

### 5. Auth & CF

- Public catalog client methods **must stay bearer-free** (document in client
  modules + this plan). Regression = silent CF MISS storm.
- Deck tags: only with session; failures do not poison public catalogs.

### 6. Errors

- Startup failure: log + `Failed`; **toast when a screen that needs the catalog
  first observes Failed** (ToastProvider sits under the upkeeper — same note as
  dictionary). Do not toast six times for six failed catalogs; prefer one
  "Couldn't load card filters" or per-screen first use.
- Partial success: independent per kind (`artists` Loaded, `sets` Failed is fine).

### 7. Remove-screen special case

Keywords / oracle words on **remove** (and similar) use `DeckCards` context to
extract terms from the current deck. Keep that path. Only the **global**
`client.get_*` branch moves to the session cache (add-card filter path).

---

## Implementation sketch (files)

| Area | Change |
|------|--------|
| `zwiper/.../session_upkeep.rs` | Provide `CatalogCache` (or family of caches); startup + TTL refresh |
| New module e.g. `zwiper/.../outbound/catalog_cache.rs` or under `inbound/components` | Types, TTL helpers, single-flight fetch, `ensure_fresh(kind)` |
| Filter screens | `artist`, `set`, `types`, `category`, keywords, oracle words, oracle tags — read cache |
| `oracle_tag_select`, create/edit deck tags | Read cache |
| Dictionary (when built) | Read oracle-tags slice |
| Tests | Unit tests for TTL expiry / stale-while-revalidate; no need for full UI |

No `zerver` / migration / `MIN_CLIENT_VERSION` changes.

---

## Phasing

### Phase 0 — Oracle tags only (can ship with dictionary)

Dedicated `OracleTagCache` mirroring `ChangelogCache` (Loading / Loaded /
Failed), fetch once at startup. Unblocks
[`otags/dictionary_client.md`](otags/dictionary_client.md) without boiling the
ocean. **No multi-kind TTL machinery yet** if we want the smallest dictionary
PR — but prefer designing types so Phase 1 extends rather than replaces.

### Phase 1 — All public card catalogs + 1-day TTL

- Unified store, parallel prefetch, consumers migrated.
- Deck tags when session present.
- Drop per-open `use_resource` for global catalogs.

### Phase 2 — Optional durability (later)

- Persist catalogs to local storage (platform keyring/fs/app data) with
  `fetched_at`, hydrate on launch → fewer cold-start network hits.
- Still revalidate if TTL expired.
- Only worth it if cold-start payload + flaky network become real pain; memory
  + CF HIT is already strong.

### Phase 3 — Nice-to-haves (not blocking)

- Join changelog into the same "session remote data" module (different TTL /
  fallback rules).
- Metrics: time-to-first-filter-open with warm vs cold cache.
- Explicit purge hook after rare mid-day catalog deploys (usually wait for TTL;
  oracle-tag **descriptions** can lag CF 24h already — see dictionary_backend).

---

## Sequencing vs other work

```text
dictionary client  ──needs──►  at least OracleTagCache (Phase 0)
catalog cache Phase 1        ──extends──►  same machinery to all filter lists
server_driven_catalogs       ──already shipped──►  roles + deck tags endpoints exist
```

Recommended order:

1. Plan docs (this + dictionary client) — **now**.
2. Phase 0 + dictionary UI (letter-first).
3. Phase 1 migrate remaining filter catalogs.
4. Phase 2 only if measured need.

---

## Decisions

| Topic | Decision |
|-------|----------|
| When to load | App load (background), not first filter open |
| TTL | **1 day**, aligned with daily sync + CF Rule 1 |
| Storage (MVP) | In-memory session (above router) |
| User force-refresh | **None** — app close / cold start re-prefetches; no Profile button |
| Stale behavior | Stale-while-revalidate when refreshing (long-lived process >24h) |
| Auth catalogs | Deck tags after session; public catalogs always |
| Bearer on public routes | Never |
| Backend | None — reuse existing GETs |
| Remove-screen deck extracts | Unchanged |

---

## Success criteria

- Opening Artist / Keywords / Sets / Roles / Oracle tags after warm start does
  **not** show a network-bound skeleton (data already `Loaded`).
- A long-lived process older than 24h revalidates without blanking the UI.
- CF cache status for public catalog GETs remains HIT-capable (no auth header).
- Dictionary and picker share one oracle-tag copy in memory.
