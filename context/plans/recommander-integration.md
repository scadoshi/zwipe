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

## Architecture mapping (hexagonal — mirror the Resend adapter)

The existing `outbound/resend/` adapter is the template: a `reqwest::Client`
wrapper implementing a domain port, constructed once from config and cloned
into services.

**New domain port** (card or deck domain): a `SynergyRecommender` with a single
`recommend(query) -> Result<Vec<CardRecommendation>, RecommendError>`. Pure
trait; the HTTP details live in the adapter.

**New domain models:** `RecommendQuery` (commander oracle_id, optional partner,
deck oracle_ids), `CardRecommendation` (oracle_id, name, score), a result-code
enum, and `RecommendError` (network / api(status,code) / parse). Lenient
parsing, same as `card::models::synergy` — upstream shape drift degrades to "no
signal," never to a hard failure.

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

- `deck_size` is already computable — `deck_cards` is fetched just above for
  exclusion (`get_deck_cards`). **Confirm the exact count definition during
  build** (recommend: total cards across deck boards, commander/profile slots
  excluded — i.e. `deck_cards.len()`).
- Both branches must reduce to the **same score-map shape** the read path
  already consumes so `search_cards_deck_aware` is untouched. The cached path
  produces lowercased-name → score (`SynergyPayload::into_scores`). Recommander
  returns `oracle_id` + `name` + `score`. **Open decision (resolve in build):**
  reuse the name→score map (zero downstream change, but name-keyed matching has
  edge cases) **vs.** add an oracle_id→score ordering variant for the
  Recommander path (exact, since both sides have oracle_id — recommended for
  correctness). Lean oracle_id-based; it's why we send `card_format=oracle_id`.

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
- [ ] Domain models + `RecommendError` + result-code enum (lenient parse)
- [ ] `SynergyRecommender` port
- [ ] `outbound/recommander/` adapter (reqwest, tight timeout, error mapping)
- [ ] Config: `RECOMMANDER_BASE_URL` / `_TIMEOUT_MS` / `_ENABLED`; wire into app state
- [ ] Read-path gate in `search_deck_cards` (deck-size split + fallback)
- [ ] Decide & implement oracle_id-vs-name score-map shape
- [ ] Tests: parse (success/empty/each error code), fallback on error, threshold gating
- [ ] `cargo sqlx prepare` if any query changes (likely none — no schema change)
- [ ] User-facing Recommander attribution (terms require it) — confirm placement
