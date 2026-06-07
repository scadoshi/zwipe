# Latency Optimization Roadmap

**Status: shipped 2026-06-07.** All five planned items addressed in iOS 1.0.2
build 23 + zerver redeploy. Final measurements at the bottom. Kept around as
a reference for the decision process and as a template for future optimization
rounds.

---

## Measurements as of 2026-06-07

Probed via `zcripts/latency/probe.sh` running on the home server (Ubuntu box
behind Cloudflare Tunnel at `api.zwipe.net`). Test user has a non-empty deck.

| Endpoint | Server-local (loopback) | Public (CF Tunnel) | Delta |
|---|---|---|---|
| `GET /health/server` | ~1ms | ~125ms | 124ms |
| `GET /health/database` | ~1ms | ~135ms | 134ms |
| `POST /api/card/search` (39KB) | **~52ms** | ~250ms | ~200ms |
| `GET /api/deck` (4KB) | ~2ms | ~129ms | ~127ms |

One outlier in 40+ samples: a 570ms `/health/database` public — single
transient. Not actionable, just noted.

### Verdict

- **Backend is healthy.** 52ms for a real `name_contains` search across
  110k+ cards is the materialized view + GIN trigram index doing their job.
- **Tunnel floor is ~125-130ms.** Every public request pays this, regardless
  of payload size.
- **Redis would buy almost nothing.** Best-case cache hit takes a 250ms
  public request down to ~200ms — 20% improvement at the cost of cache
  invalidation complexity. Not worth it.
- **Real leverage is at the network layer**, not the DB layer.

---

## Prioritized action items

Ordered by realistic latency impact for the home-server + CF-Tunnel setup.

### 1. Cloudflare edge caching for immutable card data — USER CONFIGURES

The biggest possible win. Card data is immutable between Scryfall syncs (run
nightly via zervice). Cache `/api/card/{id}` at the CF edge with a long TTL —
cache-hit requests skip the tunnel entirely and respond from the nearest
POP in ~5-10ms instead of ~125ms+.

**Scope**: this is configured in the Cloudflare dashboard (Caching → Cache
Rules), not in app code. Scotty handles this part. See
`context/ops/cloudflare-edge-caching.md` (to be written during setup) for
the actual rule shape, TTL chosen, and verification commands.

**Target endpoints for edge caching**:
- `GET /api/card/{scryfall_data_id}` — single card lookup, immutable
- `GET /api/card/{oracle_id}/printings` — printings list, immutable until next Scryfall sync
- `GET /api/card/sets`, `/api/card/types`, `/api/card/keywords`,
  `/api/card/oracle-words`, `/api/card/artists`, `/api/card/languages` —
  enumeration endpoints, very static

**NOT for edge caching** (user-specific or mutable):
- `POST /api/card/search` — request body varies; not idempotent cacheable
  via standard rules
- `/api/deck/*`, `/api/user/*`, `/api/auth/*` — user-specific or stateful

**Cache invalidation strategy**: TTL-based, set to ~24h. Scryfall sync runs
nightly via zervice; if a card's data changes, the edge cache will refresh
within 24h naturally. For urgent invalidation, CF Purge Cache (manual or via
API call from zervice post-sync).

**Verification**: after rule is live, re-run `probe.sh public` on the laptop
hitting `GET /api/card/{some_id}` — cold should be ~125ms, warm should be
under 30ms. CF response will include `cf-cache-status: HIT` header.

### 2. HTTP response compression (Axum middleware)

Add `tower-http`'s `CompressionLayer` to the Axum router. The 39KB card
search response gzips to ~5-8KB. Transfer time savings: ~30-60ms on the
public timing for any large response.

**Implementation**:
- Add `tower-http = { version = "...", features = ["compression-gzip", "compression-br"] }`
  to `zerver/Cargo.toml` (probably already in transitively, just need the features)
- Add `.layer(CompressionLayer::new())` to the public router stack in
  `zerver/src/lib/inbound/http/routes.rs`
- Verify CF Tunnel passes through `Accept-Encoding` and `Content-Encoding`
  headers (it should by default; worth confirming)
- Re-run probe after deploy; `/api/card/search` body size should drop from
  39690b to ~5000-8000b and timing should improve proportionally

### 3. Lower default search page size

`CardFilter::default_limit()` was just set to 100 (in zwipe-core). For the
typical swipe UX, users see 20-30 cards before swiping or loading more.

**Implementation**: change `default_limit()` to return 25, update the iOS
client's swipe stack to request more when it gets close to running out
(implements pagination via existing `offset` field).

**Trade-off**: more requests, but each is smaller. Combined with compression
this is probably net-positive for perceived latency since the first
response arrives faster.

### 4. HTTP/2 keepalive verification

`curl` re-establishes TCP+TLS on each request, which inflates our public
timings by ~30-50ms of handshake overhead per call. The iOS app uses
`reqwest` which *should* reuse connections by default.

**Implementation**: no code change needed if reqwest defaults are good.
Verify by instrumenting one request in zwiper with a tracing span and check
whether subsequent requests show lower TLS-handshake time.

If reqwest isn't pooling: set `reqwest::Client::builder().pool_max_idle_per_host(...).tcp_keepalive(...)`.

### 5. Pagination via cursor for "load more"

Pairs with #3. The iOS swipe UX naturally has a "next batch" trigger as
users approach the end of the current stack. Implement infinite-scroll
pagination using `offset` (already in CardFilter). The iOS app fetches the
next 25 ~2s before the user runs out.

### Items NOT worth pursuing

- **Server-side Redis caching** — saves at most ~50ms on a 250ms public
  request; complexity cost of cache invalidation isn't justified.
- **PG tuning** — backend is 1-52ms; no room to grow.
- **Moving off Postgres** — no signal it's the problem.

### If 125ms tunnel floor becomes unacceptable

The only architectural fix is moving the origin closer to users via a CDN
edge provider (Fly.io, Cloudflare Workers, or a small VPS in a major POP
city). That's a "later" path, not a "now" path. Worth keeping in mind for
when the app has real user traction and someone complains.

---

## Probe re-measurement

After each change ships, re-run to verify the prediction:

```bash
# On server
cp zcripts/latency/.env.example zcripts/latency/.env  # if not done
# edit .env to set ZWIPE_TEST_PASS
bash zcripts/latency/probe.sh
```

Compare to the baseline table above. The diagnostic flow is documented in
`zcripts/latency/README.md`.

---

## Where this fits in the broader plan

This is one of the "Post-Launch Polish → Infrastructure (Reactive)" items in
`context/status/todo.md`. The original todo item said:

> Home server may struggle under marketing load — current host is a single
> Ubuntu box behind Cloudflare Tunnel. If real users surface latency or 5xx
> spikes, evaluate: Redis for session/rate-limit/search cache, or migrate
> zerver to a cheap VPS (Hetzner/Fly). Don't pre-optimize — only act on
> observed pain.

The probe data told us "Redis is not the right move; CF caching + compression
are." That played out: ~10× speedup on backend search work, ~40% on public
search wall-clock, with no infrastructure changes beyond a CF dashboard rule
and a router-layer middleware.

---

## Final measurements (after deploy, 2026-06-07)

Same probe, same endpoints, after all five items landed. `probe.sh` now uses
`--compressed` so `size_download` reflects over-the-wire bytes:

| Endpoint | LOCAL before | LOCAL after | PUBLIC before | PUBLIC after |
|---|---|---|---|---|
| `GET /health/server` | ~1ms / 88b | ~1ms / 74b | ~125ms / 88b | ~120ms / 74b |
| `GET /health/database` | ~1ms / 88b | ~1ms / 74b | ~135ms / 88b | ~123ms / 74b |
| `POST /api/card/search` | **~52ms / 39690b** | **~5ms / 16444b** | **~250ms / 39690b** | **~150ms / 16444b** |
| `GET /api/deck` (4KB) | ~2ms / 3996b | ~2ms / 727b | ~129ms / 3996b | ~129ms / 727b |

Backend search work is now ~5ms — sub-frame. The remaining ~145ms on the
public path is the Cloudflare Tunnel hop and is structural; no further
software change can shrink it without moving the origin to an edge POP.

Card metadata endpoints (sets/types/keywords/etc) verified hitting at the CF
edge via `cf_cache_verify.sh` — POPs converge to 100% HIT within a few warmup
requests.
