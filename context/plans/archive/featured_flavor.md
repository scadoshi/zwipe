# Featured flavor — one shared pick, rotating hourly

**Status: BUILT + owner-verified locally (2026-08-05).** Server half
smoke-tested (pick from the 12-month window, cache-hit on repeat calls) and
both surfaces confirmed working by the owner. Remaining: server deploys on
next push, zite surface goes live with it, app half rides 1.7.6 — then
archive this plan. The server introduced the reusable serving-cache
primitive (`TtlSlot` — see `architecture/decisions.md` "Serving Caches" for
the pattern and its determinism-over-persistence rule). Supersedes the
bucketed "flavor of the hour" draft (this file's git history has it).

**One sentence:** replace the client's per-user random flavor fetch with ONE
server-picked card per hour — "Featured flavor" — drawn from a single pool
(flavor-text cards released in the last 12 months), served from zerver's
in-memory cache and surfaced in the app AND on zwipe.net.

## Owner decisions (2026-08-05, final)

- **Name: "Featured flavor"** (not "flavor of the hour") — UI label on both
  surfaces.
- **Hourly rotation**, top-of-UTC-hour aligned — everyone flips together.
- **ONE pool, one rule: `released_at >= now − 12 months`** (plus the
  existing `flavor_text IS NOT NULL` and the standard playability filters
  the current flavor fetch uses). NO buckets, NO weighted schedule — the
  bucketed design (new/any/pre-2000, day-parts, legendaries, etc.) was
  considered and cut for simplicity. If variety ever wants spice, buckets
  can return later; nothing forecloses them.
- **Fallback guard:** if the 12-month pool yields no pick for an hour
  (degenerate filter intersection), fall through to the all-time pool for
  that hour rather than serving nothing.

## Standing decisions carried from the draft (2026-08-04)

- **No CF edge caching** (its 2h TTL floor fights the hourly flip); clients
  fetch zerver every time, zerver holds the pick in memory and re-queries
  once per hour when the UTC hour bucket changes.
- **Deterministic pick:** seed selection with the UTC hour (e.g. order by
  `md5(id::text || to_char(now() at time zone 'utc', 'YYYY-MM-DD-HH24'))`,
  first match) so a zerver restart mid-hour re-derives the same card — the
  cache is a pure optimization, never load-bearing state.
- **Server before client** (additive endpoint; old clients keep using card
  search, which stays).

## Shape

- One unauthed endpoint returning the current pick (same `Card` payload the
  flavor UI already renders); rides the existing public rate-limit bucket
  and the CORS allowances the public-metrics endpoint already cleared.
- **App:** the `FlavorCard` context stays as the client cache; its TTL
  becomes "until the top of the next hour." Tap-name-to-preview and the
  buy-links row unchanged; home label becomes "Featured flavor."
- **Zite:** same endpoint on the zwipe.net home page via the `StatsStrip`
  fetch pattern (`zite/src/components/stats_strip.rs` — unauthed
  `use_resource` + reqwest). Placement near the stats strip; card-name tap
  uses the share-page preview pattern, not the app-only overlay.
