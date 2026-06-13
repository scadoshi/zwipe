# Recommander integration — live deck-aware synergy for mature decks

**Status: branch `recommander-integration`, started 2026-06-12.** Server-side
only (zerver). Not a client update — the app already calls the deck-card search
endpoint; this changes only where the ranking signal comes from for larger
decks.

This plan is the contract for the zwipe side. The companion decision (deck-size
split, fallback, why) lives in the synergy worker's private repo; what follows
is everything needed to build it here.

---

## The rule, in one table

| Deck state | Synergy signal | Source |
|---|---|---|
| Commander only / **0–24** cards | cached commander-level signal | the **synergy data layer** (existing read path — `commander_synergy`) |
| **25+** cards | **Recommander**, called LIVE, deck-aware | `api.recommander.cards` over HTTP |
| Recommander slow / down / rate-limited (any deck) | fall back to the cached signal | the synergy data layer |

Three load-bearing constraints, all confirmed with Michael (Recommander) on
2026-06-12:

1. **25-card threshold is Michael's own guidance, not arbitrary.** His model
   needs real deck context to be good; below ~25 cards (and commander-only) the
   cached per-commander signal is the right — and only — call. Don't hit his
   API below the threshold.
2. **Graceful degradation, never block the request path.** If Recommander is
   slow, rate-limited (`429`), cold-booting, or errors, drop to the cached
   signal rather than failing or hanging the search. A tight client timeout is
   the "it's killing our perf → fall back" mechanism. Same spirit as the
   existing read path: a missing signal degrades to the filter's own ordering,
   never to a failed search.
3. **Respect rate limits and attribution terms.** Commercial use is sanctioned
   for zwipe specifically (Michael, 2026-06-12). Attribution to Recommander is
   required by their terms.

## Naming / privacy boundary — important

- **Recommander may be named freely** in this public repo — code, docs,
  commits, and user-facing attribution. It is a sanctioned public API; their
  terms *require* attribution. This is the opposite of the synergy worker's
  source.
- **The synergy worker's upstream source must still never be named here.** In
  this repo it is only ever "the synergy worker" / "synergy data layer" / "the
  cached synergy signal." The neutrality rule applies to that source, not to
  Recommander.

---

## Recommander API contract

Base URL: `https://api.recommander.cards/public-release`
All paths below are relative to it.

### `POST /api/decks/recommend/top`

Request body (`RecommendQuery`):

| Field | Type | Req | Null | Notes |
|---|---|---|---|---|
| `card_format` | enum | no | no | `oracle_id` (default), `scryfall_id`, or `name`. **We send `oracle_id`** — we have oracle ids natively, so no name-resolution dance. |
| `commander` | string | yes | no | main commander identifier (oracle_id) |
| `partner` | string | no | yes | optional partner commander (oracle_id) |
| `deck` | string[] | no | no | current deck contents, encoded in `card_format` |

Response: `ApiResult<RecommendResult>` envelope:

```json
{
  "result_code": "success",
  "data": { "recommendations": [
    { "oracle_id": "795b096a-…", "name": "Enchantress's Presence", "score": 0.9998 }
  ] },
  "error": null
}
```

`CardRecommendation`: `oracle_id` (uuid), `name` (string), `score` (double).

`result_code` (`ApiResultCode`) — one of:
`success`, `error_unknown`, `error_not_found`, `error_invalid_deck`,
`error_invalid_cards`, `error_invalid_backend`, `error_backend_downstream`,
`error_rate_limited`, `error_booting`, `error_model_loading`.

On failure, `data` is null and `error.messages` is a `string[]`.

Behavior notes:
- **Only cards with `score > 0.7` are returned.** An empty list is a legitimate
  "no strong picks," not an error — treat it as "no live signal," fall back.
- `error_booting` / `error_model_loading` mean the model is cold-starting —
  transient, fall back this request, fine next time.
- `error_rate_limited` (and HTTP `429`) — back off, fall back. Honor any
  `Retry-After` if present.

---

## Architecture mapping (hexagonal — mirror the Resend adapter, to the letter)

Concerns go where they belong, following the existing `email` precedent exactly:
a vendor-neutral **domain port** implemented by a vendor-specific **outbound
adapter** (`EmailSender` ← `Resend`). Here: `SynergyRecommender` ← `Recommander`.

**Purity / placement:** these models + port are **server-only** — the client
never calls Recommander; the live call is server-side. So per zwipe-core's
purity rules ("if only the server needs it, it stays in zerver") they live in
**zerver**, not zwipe-core. New self-contained domain module
`zerver/src/lib/domain/synergy/` (`models/` + `ports.rs`), mirroring
`domain/email/`. The port name is the concept (`SynergyRecommender`); the
vendor name (`Recommander`) appears only in the outbound adapter.

**New domain port** `domain/synergy/ports.rs`: a `SynergyRecommender` with a
single `recommend(query) -> Result<Vec<CardRecommendation>, RecommendError>`.
Pure trait; the HTTP details live in the adapter.

**New domain models** `domain/synergy/models/`: `RecommendQuery` (commander
oracle_id, optional partner, deck oracle_ids), `CardRecommendation` (oracle_id,
name, score), a result-code enum, and `RecommendError` (network / api(status,
code) / parse). Lenient parsing, same as `card::models::synergy` — upstream
shape drift degrades to "no signal," never to a hard failure.

**New outbound adapter:** `zerver/src/lib/outbound/recommander/` — `reqwest`
POST to the endpoint above, **tight per-request timeout** (the perf guardrail),
maps any non-success / timeout / `429` / boot code to `RecommendError`.

**Read-path change — the only domain-logic edit** — in
`deck/services.rs::search_deck_cards` (around the existing `synergy_scores`
block, ~line 296). Today it unconditionally pulls `commander_synergy_payload`
when there's a commander and no explicit `order_by`. New shape:

```
if order_by.is_none() && commander present:
    if deck_size >= 25:
        try Recommander(commander, partner, deck oracle_ids)
            ok & non-empty  -> use as the synergy signal
            err / empty     -> fall back to cached commander payload (today's path)
    else:
        cached commander payload   (today's path, unchanged)
```

- **Threshold = unique non-land mainboard cards ≥ 25** *(decided 2026-06-12)*.
  This only applies to Commander, so it's effectively "the 99": count distinct
  non-land cards in the mainboard only — lands excluded, sideboard/maybeboard
  excluded, duplicates collapsed. The commander/profile slots are already
  excluded. `deck_cards` is fetched just above for exclusion (`get_deck_cards`);
  the count needs board membership + is-land, so reduce over that set.
- **Score-map shape = oracle_id → score for the Recommander path** *(decided
  2026-06-12)*. Both sides carry oracle_id (it's why we send
  `card_format=oracle_id`), so match exactly and sidestep name-collision edge
  cases (double-faced, tokens, punctuation). This means an oracle_id-keyed
  deck-aware ordering variant alongside the existing name-keyed one. The cached
  path stays name-keyed (`SynergyPayload::into_scores`) — unchanged.

**Config (`config.rs`) — new env, mirror the Resend keys:**
- `RECOMMANDER_BASE_URL` — default `https://api.recommander.cards/public-release`.
- `RECOMMANDER_TIMEOUT_MS` — tight (start ~1000ms); the fall-back trigger.
- `RECOMMANDER_ENABLED` — optional kill switch (default on) so we can disable
  the live call from `.env` + restart, no deploy (same pattern as the version
  gate).
- No API key in the public release. Higher limits / fuller model = a follow-up
  conversation with Michael, not code.

## Out of scope here
- The synergy worker keeps drip-feeding the cache exactly as today; the cache is
  now the **floor (0–24)** and the **universal fallback**, so its coverage is
  load-bearing — but that work lives in its own repo.
- No client/app change. No new endpoint — the existing deck-card search carries
  the better ranking transparently.

## Build checklist
- [x] Domain models + `RecommendError` (lenient parse) — `domain/recommendation/models.rs`
- [x] `CardRecommender` port — `domain/recommendation/ports.rs`
- [x] `outbound/recommander/` adapter (reqwest, tight timeout, error mapping, parse-once-regardless-of-status, kill switch)
- [x] Config: `RECOMMANDER_BASE_URL` / `_TIMEOUT_MS` / `_ENABLED` (+ `.env.example`); wired into `zerver.rs` app state
- [x] Read-path gate in `search_deck_cards` (deck-size split + fallback) — `Service::recommend_scores`
- [x] oracle_id-keyed score map — `SynergyOrder`/`SynergyKey`, oracle_id ordering branch in the deck-aware SQL
- [x] Tests: envelope parse (success/error/missing fields), disabled kill switch short-circuits
- [x] No schema change → no `cargo sqlx prepare` needed (runtime QueryBuilder, no new `query!` macros)
- [x] User-facing Recommander attribution — dimmed `HintCredit` line ("Recommendations powered by Recommander") in the add-card swipe hint dialog (`zwiper`), at the point of use
- [ ] **Server-side result cache** (see "Rate limits" below) — the next build
- [ ] Live verification against the real API once a 25+ deck exists in a dev DB

## Rate limits + the per-page multiplier — why we cache

`search_deck_cards` fires on **every page** of the add-card stack
(`load_more_cards` prefetches when the user is within 5 of the end —
`add.rs:240`), so paging through cards re-calls Recommander repeatedly **with
the same deck** — identical input, identical scores, wasted calls. Worse: the
public release has **no API key**, so Recommander rate-limits by **IP** — our
one server IP — meaning *every user shares a single bucket*. Call volume today
scales with (active 25+ builders) × (pages swiped) × (filter changes), all
funnelled through that one bucket.

Rough math (`L` = his limit req/min, shared; `r` ≈ 2–4 calls/min per active
builder without caching): `U_max = L / r`. If `L = 60/min`, ~20 concurrent
big-deck builders saturate it — very reachable. **Caching by deck-state turns
paging/skips/re-searches into cache hits**, dropping `r` toward ~1/min (a call
only when the deck actually changes), ~3× the headroom, and makes pagination
instant instead of paying the timeout per page.

**Decision: add a short-TTL, in-memory, server-side result cache**, agreed
2026-06-12. Design:
- A **caching decorator** implementing `CardRecommender` that wraps the
  `Recommander` adapter — keeps the deck service unchanged (hex-arch clean,
  same decorator shape as a middleware). Lives in `outbound`.
- **Key** = hash of `(commander, partner, sorted deck oracle_ids)`. **Value** =
  the `Vec<CardRecommendation>` (or reduced score map). **TTL** ~30–60s.
- In-memory via `dashmap` (already a zerver dep). No schema, no migration.
- **Adding a card changes deck-state → cache miss → one fresh call** — inherent
  and accepted; it's exactly when a new recommendation is warranted. Paging,
  skips, maybeboard swipes, and re-opening the same deck are all hits.
- Failures/empties are **not** cached (so a transient Recommander blip doesn't
  pin us to the fallback for the whole TTL).

Still need from Michael: actual **requests/min (or /hour)** and **burst/bucket
depth**, and whether a partnership unlocks a **key + higher limit** (and the
fuller model). That number sizes the TTL/throttle. Also: **meter 429s + the
fallback rate** so saturation is visible before users feel it (graceful, but
silent — a 429 quietly serves the cached commander floor).

## Future opportunity — deck-fit analysis from Recommander's stats

Beyond *ordering* the add-card search, Recommander serves richer per-deck stats
(seen on his web app's **DECK STATS**): **Commander Synergy %**, **Synergy
Range**, a **Lift Distribution** histogram, and per-card classification —
**Definitive / Staple / Unique / Other** with a per-card **lift %**.

Idea (2026-06-12): use these to **flag cards that don't belong vs. ones that
really do** — e.g. strongly negative-lift cards as "might not fit," and
high-lift / Definitive cards as "core to this deck." A deck-review/analysis
surface, distinct from the swipe-to-add ordering. Natural home: the deck view
screen.

Open question before scoping: the current integration only uses
`/api/decks/recommend/top` (which returns `{oracle_id, name, score}`). The
lift/classification stats may be a **different endpoint** (or only in his web
UI). **Confirm with Michael whether these per-card stats are API-accessible**
before designing the feature.
