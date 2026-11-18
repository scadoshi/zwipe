# Wire diff

Checks a candidate commit's zerver against main's by what they answer, not by reading the diff. Both servers get the same requests the shipped clients send, each on its own copy of one seeded local database. Every difference in their replies has to be written down in `approved.json`, or the run fails. It's the wire-compat audit from `context/development/api_evolution.md` made mechanical.

Run it by hand before a wire change merges:

```bash
zcripts/wire-diff/run.sh [candidate-ref] [main-ref]     # defaults: HEAD, origin/main
```

It exits 0 when every difference is approved, and 1 otherwise, printing each unapproved one with its route, path and both values.

## What it does, and what it costs

| Step | Time |
|---|---|
| Exports each ref with `git archive` into `/tmp/wire-diff/{main,cand}` (no worktrees) | ~1s |
| Builds each zerver in its own target dir | ~1 min cold, seconds warm |
| Copies the dev database, migrates it with main's migrations, seeds it (`zcripts/seed/seed.sh small --reset`) | ~15s |
| Copies that twice and migrates each copy with its own ref's migrations, so a migration change shows as a wire change | ~10s |
| Starts both zervers (:3201 and :3202) and replays `requests.json` (87 steps) | ~20s |
| Stops both and drops the three databases (`KEEP=1` keeps them) | ~1s |

It uses the verify skill's template lookup and the seed's local-only guard, so it refuses any database that isn't on this machine. The template is `VERIFY_TEMPLATE_URL`, else `zerver/.env`'s `DATABASE_URL`. Like the verify skill, it can't copy the dev database while something is connected to it, and it says so.

## The files

- `requests.json`: the steps, in order. Each step is one request, sent to main and then to the candidate. It covers every call `zwipe-client` makes, written the way the shipped clients send them, signed in as the seeded accounts or signed out. A step can capture a value from its reply (`{"deck": "/0/id"}`) for later steps to use as `{{deck}}`. Each server keeps its own captures, since rows a step creates get different ids on each side.
- `normalize.json`: values that change from run to run (tokens, the health timestamp), masked on every step, and arrays whose order isn't fixed, sorted before comparing. A step's own `mask` adds the values it creates (a new deck's id and times). A masked value keeps its JSON kind, so a field turning `null` still shows.
- `approved.json`: the differences a change means to make. One entry per difference: `route` (the template from requests.json), `path` (a JSON pointer; `*` stands for any one segment), `kind` (`added`, `removed`, `type`, `value` or `status`), `reason` and `approved_by`. An entry missing a field doesn't count. An entry nothing matches any more is reported as stale; delete it.
- `lib.mjs` and `lib.test.mjs`: the comparison, tested with `node --test zcripts/wire-diff/lib.test.mjs`.

Added response fields need an entry too, so every wire change is written down once, even the safe kind.

## Not replayed

- Archidekt import: it fetches from archidekt.com, which neither server should depend on in a check.
- The share preview image: a PNG, not JSON.

Steps that send email answer 500 on both servers, since a local run has no Resend key; they still compare. The bad-token, wrong-password and missing-metrics steps answer 4xx on purpose.

## Controls

- main against itself must come out clean: `run.sh origin/main origin/main`. If it doesn't, something in a reply changes run to run and belongs in `normalize.json` (with a reason in the commit), not in `approved.json`.
- A planted break must fail: a scratch commit that renames `min_version` on the wire fails with `/min_version` removed and `/minimum_version` added.

## Later

Run it in CI on pull requests that touch `zerver/`, `zwipe-core/` or `zwipe-client/`, with the dev database replaced by a fixture the job can build.
