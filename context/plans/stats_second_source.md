# A second source behind the public stats

**Status: open, for Scotty.** Nothing here is changed by code. The edge-cache and bake fixes (`fix/stats-from-origin`) shorten how long a bad copy lives; they do not explain where it came from.

## What was seen

- 2026-11-07, Surfer Part 12: right after a hard reload of zwipe.net, the hero showed **4,076 decks / 57.2k searches / 475.5k swiped, "as of 09:16"** (Pacific, so 17:16Z), then the next load showed **2,270 / 31.8k / 264.7k, "Live"**, matching the app.
- 2026-11-06, Surfer #53 and #56 (`lifetime_totals.md`): the login page showed **3,125 / 43.8k / 364.5k** while everything else showed 2,269 / 31.8k / 264.6k.

## What it is not

- **Not zite's baked snapshot.** The last three zite deploys logged what they baked: 2,269 (16:09Z), 2,269 (17:51Z) and 2,270 (18:36Z). A baked copy's chip reads "as of <date>"; "as of 09:16" is the label for a live `/api/marketing/stats` answer whose edge copy was made at 17:16Z (from its `Age` or `Last-Modified`).
- **Not the #293 query.** At 17:16Z production ran `48e5f81b9`, whose `public_metrics` is a plain `SUM` over `user_lifetime_counters`; #293 (the retired row) deployed at 18:30Z. Its query joins one retired row and groups by it, so it cannot count a user twice, and the retired row starts at zero and grows only when an account is deleted.
- **Not the origin now.** Twelve uncached requests (a unique query string each, so every one went to the origin) all answered 2,270 / 31,859 / 264,775.
- **Not a local dev server.** The local zervers on :3000 and :3001 answer 106 decks.

## Why it looks like another database

The inflated copies are uniform multiples of the true totals, in all three counters at once:

| Copy | Decks | Searches | Swipes |
|---|---|---|---|
| 2026-11-07, 17:16Z | 4,076 / 2,270 = 1.796 | 57.2k / 31.8k = 1.80 | 475.5k / 264.7k = 1.796 |
| 2026-11-06 | 3,125 / 2,269 = 1.377 | 43.8k / 31.8k = 1.377 | 364.5k / 264.6k = 1.378 |

Deleted accounts would take arbitrary amounts out of each counter, not the same fraction of all three. A uniform factor reads as a whole other set of totals: a second zerver with its own database answering some of `api.zwipe.net`'s requests, and an edge location caching its answer.

The production box runs one `cloudflared` connector (`/etc/cloudflared/config.yml`) and one zerver. If the same tunnel has another connector (an old home box, a dev machine, scotland-server), Cloudflare spreads requests across them. I could not look: ssh to the other hosts was refused for this session.

## What to check

1. In Cloudflare Zero Trust, Networks > Tunnels: the tunnel serving `api.zwipe.net` and how many connectors it has, with their origins. Or `cloudflared tunnel info <name>` with the account's cert.
2. On any other machine with `cloudflared` or a zerver: whether it is connected to that tunnel and what its `/api/marketing/stats` answers.
3. If a stray connector is found: stop it, and check whether anything else (sign-ups, decks, swipes) reached its database instead of production's during the windows above.

Production does not log successful GETs, so the origin's own log cannot say which machine answered at 17:16Z.

## Also for Scotty

- **Browser cache TTL.** zerver sends `max-age=3600` (and now `max-age=60` for the stats), but browsers get `max-age=14400`: the zone's Browser Cache TTL (4 hours) raises a shorter origin value. Set it to "Respect Existing Headers", or add a Cache Rule for `api.zwipe.net/api/*`, so a browser does not hold a counters answer for four hours.
- **Purge on deploy.** The only Cloudflare token in Actions (`CLOUDFLARE_API_TOKEN`) is scoped to Pages. A purge of the stats URL after a zerver deploy needs a token with Zone > Cache Purge and the zone id as secrets. With a five-minute edge copy it is optional.
