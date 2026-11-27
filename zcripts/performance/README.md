# Performance

Measuring speed: the API's latency from the server or a laptop, and the deck page's Stacks in a throttled browser.

## Latency

Quick scripts for measuring and verifying API latency without setting up proper instrumentation. Use these to decide whether real instrumentation (per-request tracing middleware, `pg_stat_statements`) is worth investing in, and to confirm that infrastructure changes (CF caching, compression) actually deliver.

### Setup (one-time)

Both scripts share the same credentials file. Copy the template and fill in the real test account password:

```bash
cp zcripts/performance/.env.example zcripts/performance/.env
# Edit and set ZWIPE_TEST_PASS
```

`.env` is gitignored; credentials never leave your machine. Already-exported shell vars override `.env` if you prefer one-shot usage.

### `latency_probe.sh`: measure latency

Times representative endpoints (`/health/server`, `/health/database`, `/api/card/search`, `/api/deck`) against localhost (on the server) or the public hostname (anywhere). Five samples per endpoint.

```bash
bash zcripts/performance/latency_probe.sh           # both LOCAL and PUBLIC (default)
bash zcripts/performance/latency_probe.sh local     # localhost only; run on server
bash zcripts/performance/latency_probe.sh public    # public only; run from laptop
```

**Reading the output**: each line is `<status> <body-size> <time>`:

```
── PUBLIC  POST /api/card/search ──
  200  39690b  0.251s
```

**Diagnostic table**:

| LOCAL search slow? | PUBLIC search slow? | Verdict |
|---|---|---|
| No | No | App is fine. Slowness perception is elsewhere. |
| No | Yes | Tunnel is the tax. Redis won't help; CDN/edge caching will. |
| Yes | Yes | DB is the bottleneck. Layer in `pg_stat_statements` next. |

### `cf_cache_verify.sh`: confirm CF edge caching works

Runs each immutable card endpoint twice and asserts the second hit returns `cf-cache-status: HIT`. Use after configuring a Cloudflare Cache Rule (see the Cache Rules section of `zwipe-context operations/infrastructure/cloudflare.md`) to confirm it's actually taking effect.

```bash
bash zcripts/performance/cf_cache_verify.sh   # run from laptop, hits api.zwipe.net
```

**Reading the output**:

```
── /api/card/sets ──
  request 1: cf-cache-status: MISS
  request 2: cf-cache-status: HIT
  PASS — cached

─── summary ───
  passed: 6 / 6
  failed: 0 / 6
```

| You see | Meaning |
|---|---|
| `MISS` → `HIT` | Cache Rule is working. |
| `HIT` → `HIT` | Cache Rule working; already populated from a prior run. |
| `DYNAMIC` → `DYNAMIC` | Rule isn't matching this path. Check expression in CF dashboard. |
| `MISS` → `MISS` | Rule matches but CF isn't storing. Rare; check origin `Cache-Control` headers. |
| `EXPIRED` → `MISS` | TTL expired between calls. Lengthen TTL or accept it. |

Exits non-zero if any endpoint fails; chain into CI or git hooks if useful.

### Why throwaway scripts, not real instrumentation?

These answer "is the problem even in the backend?" in 60 seconds with no code changes. If the answer is yes, *then* invest in proper per-request tracing middleware in Axum and `pg_stat_statements` for query-level analysis. The verify script likewise saves you from clicking into the CF dashboard to read analytics; it just tells you yes/no, fast.

See `zwipe-context archive/latency_optimization.md` for the broader plan these scripts supported, and the Cache Rules section of `zwipe-context operations/infrastructure/cloudflare.md` for the CF rule shape that `cf_cache_verify.sh` is validating.

## Stacks probe

`stacks_probe.py` measures the deck page's Stacks on zwipe web at a phone's width (390), with the CPU throttled 4x as a stand-in for a phone, on a mocked 100-card deck in 7 stacks. Run it before and after any change to the stacks, the settle animator or the card faces.

```bash
cd zwiper-web && dx serve --platform web --port 8097     # in one terminal
python3 zcripts/performance/stacks_probe.py http://localhost:8097 regroup base
```

The page is signed in with a fake session. Every API answer is mocked from `fixtures/` except public reads; the deck's card search is answered locally; every other write is failed and printed. Nothing reaches a server.

### Actions

- `regroup`: Stacks, then Group by Color. A regroup moves every card to a new stack.
- `switch`: List, then Stacks, the stacks dealing in.
- `raise`: a tap raises a card in the largest stack, a second tap raises another, then the card stays up for 3 seconds.
- `flip`: the two-faced card's details open, then its Flip.
- `frames` and `rframes`: unthrottled screenshots of a regroup or a raise, saved to `$FRAMES`, for a by-eye check.

### Variants

`base` is the build as served. The others change it in the page to isolate a cost:
- `nosettle` turns the settle animator off.
- `replace` runs its moves without `composite: 'add'`.
- `noghost` skips the copies a removed group leaves.
- `timed` times the animator's snapshot, moves and copies.
- `floatoff` stops a raised card's drift.
- `marginoff` drops the tap stacks' `margin-top` ease.

### What it prints

- frame times from `requestAnimationFrame`: the worst three, how many ran over 50ms, and the time over 16.7ms summed
- long tasks
- Chrome's layout and style counts and durations, around a raise
- render counts, but only from a build that logs them

The budget: a regroup's longest task under 400ms at 4x. Release `a9ccce71f` took 1,096ms; with the settle fixes it takes about 335ms.

`cdplib.py` drives headless Chrome over its debugging pipe. It looks for Playwright's headless shell first, then `chromium`.
