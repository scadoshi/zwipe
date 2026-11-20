# Wire diff

Checks a candidate commit's zerver against main's by what they answer, not by reading the diff. Both servers get the same requests the shipped clients send, each on its own copy of one seeded local database. Every difference in their replies has to be written down in `approved.json`, or the run fails. It's the wire-compat audit from `context/development/api_evolution.md` made mechanical.

Run it by hand before a wire change merges:

```bash
zcripts/wire-diff/run.sh [candidate-ref] [main-ref]     # defaults: HEAD, origin/main
```

It builds committed trees, so commit what you want tested: uncommitted changes in the checkout aren't part of the candidate. It exits 0 when every difference is approved, and 1 otherwise, printing each unapproved one with its route, path and both values.

## What it does, and what it costs

| Step | Time |
|---|---|
| Exports each ref with `git archive` into `/tmp/wire-diff/{main,cand}` (no worktrees) | ~1s |
| Builds each zerver in its own target dir | ~1 min cold, seconds warm |
| Copies the dev database, migrates it with main's migrations, seeds it with main's `zcripts/seed/seed.sh small --reset` (the checkout's until main has one) | ~15s |
| Copies that twice and migrates each copy with its own ref's migrations, so a migration change shows as a wire change | ~10s |
| Starts both zervers (:3201 and :3202) and replays `requests.json` (99 steps) | ~20s |
| Stops both and drops the three databases (`KEEP=1` keeps them) | ~1s |

It uses the verify skill's template lookup and the seed's local-only guard, so it refuses any database that isn't on this machine. The template is `VERIFY_TEMPLATE_URL`, else `zerver/.env`'s `DATABASE_URL`. Like the verify skill, it can't copy the dev database while something is connected to it, and it says so. The template has to be main's schema: one that ran another branch's migrations is refused, naming them, with the commands that build a matching template from main's migrations and `fixture/load.sh`, as CI does. A migration that fails shows sqlx's own error.

## The files

- `requests.json`: the steps, in order. Each step is one request, sent to main and then to the candidate. It covers every call `zwipe-client` makes, written the way the shipped clients send them, signed in as the seeded accounts or signed out. A step can capture a value from its reply (`{"deck": "/0/id"}`) for later steps to use as `{{deck}}`. Each server keeps its own captures, since rows a step creates get different ids on each side.
- `normalize.json`: values that change from run to run (tokens, the health timestamp), masked on every step. A step's own `mask` adds the values it creates (a new deck's id and times). A masked value keeps its JSON kind, so a field turning `null` still shows. A step's `sort` sorts an array whose order the server doesn't fix before comparing; it's opt-in per step, since an order that is the contract has to compare as it comes. Only the card search step sorts today: the search breaks no ties, so two identical databases list "Sol Ring" printings in different orders. Issue 578 adds a tiebreak; once it lands, drop that step's `sort`.
- The changelog step compares by shape (`"shape": true`): field names and JSON types, without values or how many lines or releases there are, since nearly every commit adds a line there. An empty list matches any list, so a cut that leaves a release with no lines passes. A release field renamed or retyped still shows.
- `approved.json`: the differences a change means to make. One entry per difference: `route` (the template from requests.json), `path` (a JSON pointer; `*` stands for any one segment, and `""` is the reply's root, where a new route's `status` and body `type` differ from main's 404), `kind` (`added`, `removed`, `type`, `value` or `status`), `reason` and `approved_by`. An entry missing a field doesn't count. An entry nothing matches any more is reported as stale; delete it.
- A data change goes in `approved.json` too, for a candidate whose database answers differently while the contract stays put. Today there is one kind, `"data": "printings"`: another printing of the same card, as when 578's tiebreak changed which printing `latest_cards` picks. It takes `oracle_ids` (a list, or `"*"` for every card), `reason`, `approved_by` and, to keep it to one route, `route`. It approves what another printing can change inside a card whose oracle id it lists, on any route and at any depth, the row's own fields beside the card included, so long as both servers hold the same card there: a value, a field null on one side only, an item more or fewer in a list, or a link more or fewer in `related_uris` or `purchase_uris`, which zwipe passes through from Scryfall untyped. A key added, dropped or renamed, or any other type change, is the card's shape changing and still shows, as does a different card in the same place (a reorder). A value outside any card that names a printing of the same listed card on each side (an id, or a URL holding one, like a deck's `commander_id` and `commander_art_url`; the first UUID in the string) counts too. While one is approved, a card list holding the same cards on both sides compares in oracle id order, since another printing can reorder a list sorted by printing; a list that gained or lost a card keeps its order, so that still shows. Catalogs that follow the picked printings, like the artist and set lists, still need their own entries.

  ```json
  { "data": "printings", "oracle_ids": "*", "reason": "tied printings pick by collector number (578)", "approved_by": "composer" }
  ```
- `lib.mjs` and `lib.test.mjs`: the comparison, tested with `node --test zcripts/wire-diff/lib.test.mjs`.

Added response fields need an entry too, so every wire change is written down once, even the safe kind.

## Not replayed

- Archidekt import: it fetches from archidekt.com, which neither server should depend on in a check.
- The share preview image: a PNG, not JSON.

Steps that send email answer 500 on both servers, since a local run has no Resend key; they still compare. The bad-token, wrong-password and missing-metrics steps answer 4xx on purpose.

## Controls

- main against itself must come out clean: `run.sh origin/main origin/main`. If it doesn't, something in a reply changes run to run and belongs in `normalize.json` (with a reason in the commit), not in `approved.json`.
- A planted break must fail: a scratch commit that renames `min_version` on the wire fails with `/min_version` removed and `/minimum_version` added.

## In CI

`.github/workflows/wire-diff.yml` runs it on pushes to `release` and on pull requests into `main` that touch `zerver/`, `zwipe-core/`, `zwipe-client/`, the lockfile, `.sqlx/`, the seed or this folder. It runs on GitHub's runner against a throwaway `postgres:18.6` service: no secrets, no production database. The template is main's schema loaded with the card fixture, the candidate is the pushed commit (a pull request's merge commit), and the report and both zervers' logs upload as the `wire-diff-report` artifact. An unapproved difference fails the job.

The runner has no synced catalog, so `fixture/cards.csv.gz` stands in for one: 406 cards, one printing each, with their profiles, oracle tags and the out-of-universe sets. `fixture/load.sh` puts it into a migrated database and refreshes the card views. Regenerate it from a local database with the full catalog after a migration changes a card table:

```bash
DATABASE_URL=postgres://you@localhost:5432/zwipe zcripts/wire-diff/fixture/generate.sh
```

The card data is Scryfall's bulk data, as the nightly sync stores it, used under Scryfall's data guidelines; images are links to Scryfall, never the images themselves. It leaves out the Commander tables the synergy worker fills, so commander suggestions and the commander ordering run without them on both sides. No user data is in it.
