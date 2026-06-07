# Latency Probes

Quick scripts for measuring backend / tunnel / DB latency without setting up
proper instrumentation. Use these to decide whether real instrumentation
(per-request tracing middleware, `pg_stat_statements`) is worth investing in.

## probe.sh

Measures wall-clock latency for representative endpoints (`/api/health`,
`/api/card/search`, `/api/deck`) against either localhost (on the server) or
the public hostname (from any machine).

### Setup (one-time)

Copy the template and fill in the real test account credentials:

```bash
cp zcripts/latency/.env.example zcripts/latency/.env
# Edit zcripts/latency/.env and set ZWIPE_TEST_PASS
```

The `.env` file is gitignored — it never leaves your machine.

### Run

```bash
# Both sides (default)
bash zcripts/latency/probe.sh

# Localhost only — run on the server
bash zcripts/latency/probe.sh local

# Public only — run from laptop
bash zcripts/latency/probe.sh public
```

The script reads `ZWIPE_TEST_USER` (default `test`) and `ZWIPE_TEST_PASS` from
the `.env` next to itself. Already-exported shell vars override `.env`, so you
can also do a one-shot:

```bash
ZWIPE_TEST_PASS='<your-test-password>' bash zcripts/latency/probe.sh public
```

### Reading the output

```
── LOCAL  GET /api/health ──
  200  18b  0.002s
  200  18b  0.002s
  ...
── PUBLIC  GET /api/health ──
  200  18b  0.118s
  200  18b  0.121s
  ...
```

- `LOCAL /api/health` time = Axum routing + no DB. Baseline for "how fast can
  the app respond at all."
- `PUBLIC /api/health` time minus LOCAL = pure Cloudflare Tunnel hop tax.
- `LOCAL /api/card/search` time = full DB query + serialization. If this is
  large, the DB is the bottleneck.
- `PUBLIC /api/card/search` time = real user experience.

### Diagnostic flow

| LOCAL search slow? | PUBLIC search slow? | Verdict |
|---|---|---|
| No | No | App is fine. Slowness perception is elsewhere. |
| No | Yes | Tunnel is the tax. Redis won't help; consider CDN / edge caching. |
| Yes | Yes | DB is the bottleneck. Layer in `pg_stat_statements` next. |

## Why these scripts, not real instrumentation?

These are throwaway-grade. They answer "is the problem even in the backend?"
in 60 seconds with no code changes and no PG restart. If the answer is yes,
*then* invest in proper per-request tracing middleware in Axum and / or
`pg_stat_statements` for query-level analysis.
