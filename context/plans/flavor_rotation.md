# Flavor rotation — a flavor of the hour, cycling era buckets

**Status: QUEUED (2026-08-04, owner). Not specced beyond this doc.**

**One sentence:** replace the client's random flavor-card fetch with ONE
server-picked flavor of the hour — the era bucket cycles every hour (new sets →
any card → pre-2000, repeat), served from zerver's in-memory cache so the DB is
touched once per hour, not per request.

## Today

`home.rs` fetches a random card with `set_has_flavor_text(true)` through card
search, cached app-wide in the `FlavorCard` context signal with a TTL. Every user
sees a different card; there is no shared "card of the hour."

## The rotation (owner calls 2026-08-04: hourly, weighted ratio — not day parts)

One pick at a time, a new card every hour. The bucket for each hour comes from a
**weighted repeating schedule** (ratio TBD by owner), not a flat 1:1:1 cycle —
e.g. a fixed 24-slot array indexed by UTC hour-of-day, which keeps the pick
deterministic while letting the ratio be anything (say newest-heavy:
`new, any, new, old, any, new, ...`). Core buckets:

| Bucket | Filter |
|--------|--------|
| Newest set(s) | recent `released_at` (window TBD — e.g. last 6–12 months, or the latest N sets) |
| Any card | no era filter |
| Old cards | `released_at < 2000-01-01` (before the turn of the century) |

No day-part / timezone mapping (an earlier draft tied eras to morning/noon/night
by local time; owner chose the simpler hourly rotation — one global pick,
everyone sees the same card at the same moment). All buckets keep the existing
`flavor_text IS NOT NULL` requirement.

### Candidate extra buckets (owner asked; pick a few, all filterable today)

- **Legendaries** — legendary creatures (or anything commander-legal at the
  helm). The most on-brand bucket for a commander-first app.
- **Funny** — `set_type = 'funny'` (Un-sets etc.); the densest source of great
  flavor text. Needs a legality caveat check (these cards aren't playable in
  most decks — fine for flavor, but worth a conscious yes).
- **Color of the day** — any-era bucket constrained to a color identity that
  cycles W→U→B→R→G by day; five-day rhythm layered on the hourly ratio.
- **Anniversary** — released this calendar week N years ago (`released_at`
  month/day window). Nostalgia hook, easy query, always in season.

Whatever the final set, keep the schedule a single const array — adding a
bucket later is one filter + one slot edit, no wire change.

## Caching decision (owner + AI, 2026-08-04)

- **No CF edge caching.** The plan's minimum edge TTL is 2 hours; the rotation is
  hourly. Rather than fight it, clients fetch from zerver every time.
- **zerver holds the pick in memory**, `CatalogCache`-style, refreshed when the
  UTC hour bucket changes (top-of-hour aligned, NOT a rolling 60min from first
  request — everyone flips together). One query per hour; the per-request cost is
  serving a few KB from memory. At current traffic this is negligible; revisit
  edge caching only if flavor traffic ever actually hurts.
- **Deterministic pick (recommended):** seed the selection with the hour bucket
  (e.g. order by `md5(id::text || '<UTC hour>')`, take the first match for that
  hour's era filter). Then a zerver restart mid-hour re-derives the same card and
  the cache is a pure optimization, not load-bearing state.

## Shape

- One endpoint returning the current pick (one wire type in core — the same
  `Card` payload the flavor UI already renders).
- Client: `FlavorCard` context stays as the app-side cache; its TTL becomes
  "until the top of the next hour." Tap-name-to-preview and the buy-links row
  are unchanged.
- Server before client as always (dual-serve isn't needed: old clients keep
  using card search, which stays; new endpoint is additive).
- **Zite too (owner, 2026-08-04):** surface the flavor of the hour on
  zwipe.net as well — same endpoint, same hourly flip, giving the site a
  living element and visitors a taste of the in-app flavor feature.
  The pattern already exists: the home page's `StatsStrip`
  (`zite/src/components/stats_strip.rs`) does an unauthed client-side
  `use_resource` + reqwest fetch against `api.zwipe.net` — the flavor
  element is that same shape with the flavor payload. Requirements: the
  endpoint is unauthed + CORS-allowed for the site origin (public-metrics
  already cleared both hurdles). Placement TBD (home page next to the stats
  strip is the natural spot). Card-name tap should use the share-page card
  preview pattern (zite already renders card images/previews) rather than
  the app-only overlay.

## Open bits

- The ratio itself (how newest-heavy?) and which extra buckets make the cut.
- "New sets" window (6 months? 12? by `released_at` vs latest N sets).
- Whether the pre-2000 pool's flavor-text coverage is thin enough to need a
  wider era (check counts before committing to the cutoff).
