# Latency Scripts

Quick scripts for measuring and verifying API latency without setting up proper
instrumentation. Use these to decide whether real instrumentation (per-request
tracing middleware, `pg_stat_statements`) is worth investing in, and to confirm
that infrastructure changes (CF caching, compression) actually deliver.

## Setup (one-time)

Both scripts share the same credentials file. Copy the template and fill in
the real test account password:

```bash
cp zcripts/latency/.env.example zcripts/latency/.env
# Edit and set ZWIPE_TEST_PASS
```

`.env` is gitignored — credentials never leave your machine. Already-exported
shell vars override `.env` if you prefer one-shot usage.

---

## Scripts

### `probe.sh` — measure latency

Times representative endpoints (`/health/server`, `/health/database`,
`/api/card/search`, `/api/deck`) against localhost (on the server) or the
public hostname (anywhere). Five samples per endpoint.

```bash
bash zcripts/latency/probe.sh           # both LOCAL and PUBLIC (default)
bash zcripts/latency/probe.sh local     # localhost only — run on server
bash zcripts/latency/probe.sh public    # public only — run from laptop
```

**Reading the output** — each line is `<status> <body-size> <time>`:

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

### `cf_cache_verify.sh` — confirm CF edge caching works

Runs each immutable card endpoint twice and asserts the second hit returns
`cf-cache-status: HIT`. Use after configuring a Cloudflare Cache Rule (see
`context/ops/cloudflare-edge-caching.md`) to confirm it's actually taking
effect.

```bash
bash zcripts/latency/cf_cache_verify.sh   # run from laptop, hits api.zwipe.net
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
| `MISS` → `MISS` | Rule matches but CF isn't storing. Rare — check origin `Cache-Control` headers. |
| `EXPIRED` → `MISS` | TTL expired between calls. Lengthen TTL or accept it. |

Exits non-zero if any endpoint fails — chain into CI or git hooks if useful.

---

## Why throwaway scripts, not real instrumentation?

These answer "is the problem even in the backend?" in 60 seconds with no code
changes. If the answer is yes, *then* invest in proper per-request tracing
middleware in Axum and `pg_stat_statements` for query-level analysis. The
verify script likewise saves you from clicking into the CF dashboard to read
analytics — it just tells you yes/no, fast.

See `context/ops/latency-optimization.md` for the broader plan these scripts
support, and `context/ops/cloudflare-edge-caching.md` for the CF rule shape
that `cf_cache_verify.sh` is validating.
