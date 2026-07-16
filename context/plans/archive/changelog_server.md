# Changelog: serve from the server (rough plan)

**Status: DONE 2026-07-15 (shipped, archived).** The changelog was **hard-coded in the app
binary** (`zwipe-components/src/changelog.rs`, `RELEASES`/`UPCOMING` consts). That
means showing new entries — including "what's in the pipeline" (`UPCOMING`) —
requires an App Store / Play resubmission. Silly. Fix: let the **mobile client
fetch** the changelog from a server endpoint (Cloudflare-cached), so we can update
it with a plain server deploy, no app rebuild.

## Shape (decided)

- **Server hard-codes the changelog too** and serves it from an additive endpoint.
  Updating = edit the data + deploy `zerver` (push to main auto-deploys). No app
  resubmit.
- **zite keeps hard-coding it** (renders the compiled-in data directly, no fetch).
- **Only the mobile client fetches.** Existing app clients keep rendering their
  in-binary copy; new clients hit the endpoint. **Additive only — no
  `MIN_CLIENT_VERSION` bump.**

## Endpoint (decided — do NOT over-build)

**One** public endpoint: `GET /api/changelog`, unauthenticated, **IP-rate-limited
via the existing governor**, with `Cache-Control` so Cloudflare edge-caches it.

Rationale (owner asked one-vs-two): the changelog is public, identical for every
user, wanted pre-login too, fetched once per launch, and tiny. A second auth'd
by-user endpoint doubles surface for zero benefit; auth would actively hurt
(pre-login can't see it). Cloudflare absorbs virtually all traffic; the governor is
cheap abuse insurance behind it. **Two endpoints = overengineering.**

## Where the data lives (the one real refactor)

Today the data sits in `zwipe-components` (a UI crate, has `dioxus`). `zerver` must
not depend on a UI crate, so the **data must move to `zwipe-core`** (pure domain,
already imported by zerver + zite + zwiper + zwipe-components).

- **`zwipe-core::domain::changelog`** (new): serde-serializable `Release { version,
  date, entries }` + `RELEASES` + `UPCOMING` consts (moved verbatim from
  `changelog.rs`), and a wire type (e.g. `ChangelogResponse { upcoming, releases }`).
- **`zwipe-components::Changelog`**: keep the rendering component, but feed it the
  data (import from `zwipe-core`, or take it as a prop). `major_minor` filter logic
  stays in the component.

## Steps

1. **Move data to `zwipe-core`** — `domain::changelog` with the `Release` type
   (derive `Serialize`/`Deserialize`) + the two consts. Keep it pure (no dioxus).
2. **Point the component at it** — `zwipe-components::Changelog` renders from the
   core data (zite path: compiled-in, unchanged UX).
3. **Server endpoint** — `GET /api/changelog` in zerver returns the core consts as
   JSON. Public route, governor IP limit, `Cache-Control: public, max-age=...` (pick
   a TTL; 5–15 min is fine, or longer + purge on deploy). No auth, no DB.
4. **Client fetch (zwiper)** — fetch `/api/changelog` **at app startup**, hold in
   memory for the session (drop on app close). The Changelog view renders the
   fetched data; **fall back to the compiled-in `zwipe-core` copy** if the fetch
   fails or hasn't landed yet (so it always shows *something* and degrades to
   today's behavior offline).
5. **Later (only if it grows):** persist the last-fetched changelog on device
   between sessions. Not needed now — the payload is small.

## Notes / decisions

- **Fallback is mandatory:** zwiper compiles in the same `zwipe-core` data, so a
  failed fetch just shows the shipped-with-the-binary list. No blank states.
- **`UPCOMING` is the payoff:** once cut over, we can tease pipeline items to new
  clients via a server deploy alone.
- **No wire/versioning drama:** the JSON is a flat list; additive fields only.
- **Cache invalidation:** simplest is a short TTL; if we want instant, purge the
  Cloudflare cache key on deploy (optional).
- Backward-compat is free: old clients never call the endpoint.
