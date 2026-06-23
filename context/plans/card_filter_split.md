# CardFilter split — server query spec vs client-side predicate

**Status: PLANNED (deferred). Blocked on the frontend being testable.**

The immediate DoS this addresses was already closed by a server-side SQL clamp
(commit `fe5324ac`: `MAX_SEARCH_LIMIT = 250` in
`zerver/src/lib/outbound/sqlx/card/mod.rs`, plus an OFFSET cast guard). **This
plan is the proper de-duplication of an over-loaded type, not a security fix** —
the clamp is the correct control regardless and stays put.

---

## Problem

`CardFilter` (zwipe-core, `domain/card/models/search_card/card_filter/`) does two
unrelated jobs with one type:

1. **Server query spec** — serialized by zwiper, POSTed to `/api/card/search`
   and the deck-aware search, deserialized by zerver, turned into
   `WHERE … LIMIT … OFFSET`. Here `limit` is pagination and **must be capped**
   (untrusted input → DB cost).
2. **Client-side in-memory predicate** — `Vec<Card>::filter_by(&CardFilter)` /
   `sort_by_filter` over already-loaded lists (deck remove/view screens). Here
   `limit` truncates the local list, and the frontend sets it to `10_000` as a
   "don't truncate" sentinel — `filter_by` honors it
   (`skip(offset).take(limit)`, `filter_cards.rs:628`; there's a test
   `test_limit_caps_results` asserting that).

The two uses want **opposite** things from `limit`: bounded (≤250) vs effectively
unbounded. One type can't enforce both — a clamping cap breaks client filtering;
no cap leaves the server exposed (hence the separate SQL clamp). Swiss-army knife.

---

## Target design

Composition, not one struct:

- **`CardCriteria`** — the ~50 predicate fields (text/mana/combat/flags/legality).
  No pagination, no ordering. This is all an in-memory filter needs.
- **`CardQuery { criteria: CardCriteria, limit: Limit, offset, order_by, ascending }`**
  — the server request. `Limit` is a clamping newtype (≤250), now safe because
  this type is *only ever* a server query, never a client predicate.
- `filter_by` / `sort_by_filter` take `&CardCriteria` (+ ordering for sort).
  Pagination leaves the client filter path entirely; any UI truncation becomes a
  view concern, not a filter concern.

Net: the cap lives in the type for the server path, and the client path can't
even express a limit.

---

## The wire question (decides how hard this is)

`CardFilter`'s serialized JSON is the POST-body contract between zwiper and
zerver. Two ways to split:

### Option A — preserve the wire (recommended; likely NO staged rollout)

`#[serde(flatten)]` the `criteria` inside `CardQuery`. The on-the-wire JSON stays
byte-identical to today's `CardFilter` (criteria fields at top level alongside
`limit`/`offset`/`order_by`/`ascending`).

- Already-shipped clients send the current JSON → new backend deserializes into
  `CardQuery` via flatten → works.
- New frontend serializes `CardQuery` → same JSON → works on old and new backend.
- **No wire break ⇒ no min-version gate, no transition window.** Backend and
  frontend ship independently. Pure internal refactor.
- Must verify flatten round-trips exactly — `serde(flatten)` interacts with
  `#[serde(default)]` and `skip_serializing_none`; add a test asserting today's
  JSON ⇄ the new types both directions.

### Option B — change the wire (only if we ever restructure the JSON itself)

E.g. nesting `criteria` as a sub-object. Breaking contract change → needs the
staged rollout below. **Not required for the split** — documented for completeness.

---

## Staged rollout (only under Option B / any future wire-breaking change)

Per `context/development/api_evolution.md` (server deploys first) + the min-version gate:

1. **Backend flexible** — accept BOTH old and new shapes (`#[serde(untagged)]`
   enum or custom deserializer). Deploy backend first; old clients unaffected.
2. **Ship new frontend** — sends the new shape. Submit to App Store.
3. **Transition window** — both shapes in the wild; backend serves both. Wait for
   adoption (the normal release-propagation wait).
4. **Allow upgrades** — users update naturally.
5. **Gate old versions** — bump `MIN_CLIENT_VERSION` to the first build that sends
   the new shape; older clients hit the force-update screen.
6. **Update server** — drop old-shape support once no traffic uses it.
7. **Fixed** — one clean shape end to end.

---

## Files touched

- **zwipe-core**: `card_filter/` (new `CardCriteria`, `CardQuery`, `Limit`; split
  the builder or add a criteria builder), `filter_cards.rs` (`filter_by` /
  `sort_by_filter` take `CardCriteria`), getters/setters.
- **zerver**: `inbound/http/handlers/card/search_card.rs` +
  `deck/search_deck_cards.rs` (deserialize `CardQuery`),
  `outbound/sqlx/card/mod.rs` (bind from `CardQuery`; keep the `MAX_SEARCH_LIMIT`
  SQL clamp as defense-in-depth even with the `Limit` newtype).
- **zwiper** (the bulk, and why this waits for a testable frontend): every
  `set_limit` / `set_offset` / `filter_by` / `sort_by_filter` / builder call site
  — `screens/home.rs`, `deck/card/remove.rs`, `deck/card/view.rs`,
  `deck/card/add.rs`, `deck/components/deck_fields.rs`.

---

## Why deferred

- Large surface across all three crates, concentrated in **untestable frontend
  call sites** (chat origin: "can't test my frontend rn").
- Zero security urgency — the DoS is already closed by the SQL clamp, which is
  the right control in every design and doesn't move.

## Done when

`filter_by` can't see a `limit`; the server query carries a bounded `Limit`;
old-JSON compat test passes (Option A) or the gate has retired old clients
(Option B); frontend exercised end to end.
