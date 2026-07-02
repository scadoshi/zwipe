# CardFilter split — DB query vs in-memory collection operation

**Status: READY.** The old blocker ("frontend not testable") is cleared — the iOS
sim boots via `zcripts/ios/sim.sh`, so the zwiper call sites can be exercised end
to end. This is an **internal de-duplication of an over-loaded type, not a security
fix**: the DoS it once touched was already closed by the server SQL clamp
(commit `fe5324ac`: `MAX_SEARCH_LIMIT = 250` in
`zerver/src/lib/outbound/sqlx/card/mod.rs`, plus an OFFSET cast guard). That clamp
is the correct control in every design and **stays put** as defense-in-depth.

---

## Problem

`CardFilter` (zwipe-core, `domain/card/models/search_card/card_filter/`) is **one
type doing two unrelated jobs**, glued together only because they share the same
~50 predicate fields:

1. **A database query** — "describe what to SELECT from rows *not yet loaded*."
   Predicate + `limit`/`offset` (pagination) + ordering + synergy. The server turns
   it into `WHERE … ORDER BY … LIMIT … OFFSET`. Here `limit` is **untrusted input
   that must be bounded** (DB cost).
2. **An in-memory operation on a `Vec<Card>` already in hand** — the deck
   remove/view/add screens run a predicate (+ sort) over cards **already served and
   loaded** on the client. No database, no pagination. Here `limit` is **meaningless**.

**Smoking gun — the `limit` field.** The DB path needs it capped (≤250). The
in-memory path wants it gone, so every client filter site sets `set_limit(10_000)`
as a "don't truncate" sentinel and then sorts separately. One field can't be both
bounded and unbounded — proof these are two types wearing one coat.

**Finding that de-risks the frontend bulk.** `filter_by` today does
predicate → sort → `skip/take`. But every client call site already (a) sets the
`10_000` sentinel and (b) calls `sort_by_filter(&builder)` *separately afterward*.
So `filter_by`'s internal sort+truncate is dead weight at all sites, and the 4th
site (`view.rs:310`) is a per-card membership test that only wants the predicate.
Making the in-memory op predicate-only **drops behavior nobody relies on**, and the
three sentinels become straight deletions.

---

## Target design (locked)

Composition, not one struct. Locked names/vocabulary:

```rust
// shared core — "what matches a card". ~50 predicate fields. No pagination, no order.
struct CardCriteria { power_range, cmc_range, name_contains, type_line_contains_any,
                      is_commander_in_format, legalities_contains_any, /* … ~50 */ }
impl CardCriteria { fn matches(&self, &Card) -> bool; }

// the sort KEY only (rename of OrderByOption; variants unchanged).
// direction stays separate in `ascending: bool` — hence "Key", not "Ordering".
enum CardSortKey { Name, Cmc, Rarity, EdhrecRank, Random, /* … */ }

// clamping newtype for the bounded DB limit.
struct Limit(u32);            // Limit::new(n) -> min(n, 250); transparent serde

// path 1 — the DATABASE query.
struct CardQuery {
    #[serde(flatten)] criteria: CardCriteria,          // wire stays byte-identical
    limit: Limit,
    offset: u32,
    #[serde(rename = "order_by")] sort: Option<CardSortKey>,   // keep JSON key
    ascending: bool,
    synergy: bool,                                     // deck-aware search only
}

// path 2 — the IN-MEMORY collection. operations ONLY take CardCriteria.
struct Cards(Vec<Card>);
impl Cards {
    fn matching(self, &CardCriteria) -> Cards;         // predicate
    fn sorted(self, CardSortKey, bool) -> Cards;       // ordering (key, ascending)
    fn any_match(&self, &CardCriteria) -> bool;        // membership (view.rs:310)
}
impl Deref for Cards { type Target = [Card]; }         // reads like a slice
// no `limit`/`offset` method exists — the in-memory path can't truncate, by construction
```

Two explicit steps for the in-memory path: `cards.matching(&criteria).sorted(key, ascending)`.

**Why `CardSortKey` carries no direction:** it's the key, and `ascending` lives
separately (already `#[serde(default = true)]`). Bundling would (1) contradict the
name, (2) be flattened back to two wire fields anyway, and (3) carry a meaningless
`ascending` for `CardSortKey::Random`. If the dangling bool ever bugs us, wrapping
`CardSort { key, ascending }` for the *in-memory API only* is a trivial, non-wire
follow-up — not worth doing up front.

---

## Wire: keep it flat (Option A)

`#[serde(flatten)]` the `criteria` inside `CardQuery` → the POST body stays
byte-identical to today's `CardFilter` (criteria fields at top level alongside
`limit`/`offset`/`order_by`/`ascending`/`synergy`). **No wire break ⇒ no min-version
gate, no transition window;** client and server ship independently. Renaming the
field `sort` keeps the `order_by` JSON key via `#[serde(rename)]`.

**Correctness gate:** a round-trip test asserting today's `CardFilter` JSON ⇄
`CardQuery` both directions (`serde(flatten)` × `skip_serializing_none` ×
`serde(default)` interplay). Prove this **first** — everything rides on no wire break.

**Rejected — Option B (restructure the wire behind the gate).** The wire is a
private client↔server contract; `flatten` already gives clean *internal* types, so B
buys only a cosmetic JSON envelope at the cost of a dual-shape server + a burned
force-update cycle during a growth phase. No lock-in: Option A doesn't block a
future Option B if a genuinely breaking change ever needs one.

---

## Frontend builder decision

The filter UI keeps **one `CardQueryBuilder`** as its state (single source of truth
for the filter form) and exposes both:

- `.build()` → full `CardQuery` (search path)
- `.build_criteria()` → bare `CardCriteria` (in-memory path)

Preferred over a parallel `CardCriteriaBuilder`, which would split the UI state.

---

## Plan

**Phase 1 — zwipe-core (the bulk)**
1. Rename `OrderByOption` → `CardSortKey` (`order_by_option.rs` → `card_sort_key.rs`);
   variants unchanged. Standalone mechanical commit.
2. Extract `CardCriteria` — move the ~50 predicate fields + getters out of
   `CardFilter`; predicate logic → `CardCriteria::matches(&Card)`.
3. `Limit` newtype — clamp ≤250, transparent serde.
4. `CardQuery` — flatten criteria + `limit`/`offset`/`sort`/`ascending`/`synergy`;
   pagination/sort/synergy getters live here.
5. `Cards(Vec<Card>)` — `matching`/`sorted`/`any_match`/`Deref`/`From`; retire the
   `FilterCards`/`SortCards` traits.
6. Port the ~15 `filter_cards.rs` unit tests to `Cards`.
7. **Round-trip serde test** (the Option A gate) — prove first.

**Phase 2 — zerver (mechanical)**
8. Handlers deserialize `Json<CardQuery>` (`search_card.rs`, `search_deck_cards.rs`).
9. sqlx binding: WHERE from `query.criteria`, `LIMIT query.limit()` (already bounded);
   **keep the `MAX_SEARCH_LIMIT` clamp** as defense-in-depth. Same SQL text → no
   `sqlx prepare` churn expected.

**Phase 3 — zwiper (mechanical, now testable)**
10. Search sites (`home`, `deck_fields`, `add`, `swipe_select`, `view`) build
    `CardQuery`; `set_limit`/`set_offset`/sort move onto its builder.
11. In-memory sites (`add:814`, `remove:268`, `view:321`) →
    `Cards::from(v).matching(&criteria).sorted(key, asc)`; **delete the three
    `set_limit(10_000)` sentinels**; fold the separate `sort_by_filter` into `.sorted`.
12. `view:310` membership → `criteria.matches(card)` (kills the per-card `vec![c]` alloc).

**Phase 4 — verify**
13. `cargo test --workspace` + clippy `-D warnings`; sim pass: search + pagination,
    deck add/remove/view filter + sort, synergy toggle.

**Commits:** rename → core split + tests → zerver → zwiper. Option A means no wire
dependency, so order is habit (server before client), not necessity.

---

## Files touched

- **zwipe-core**: `card_filter/` (new `CardCriteria`, `CardQuery`, `Limit`;
  `CardSortKey` rename; split the builder to expose `build_criteria()`),
  `filter_cards.rs` (new `Cards` type; `matching`/`sorted`/`any_match`;
  retire `FilterCards`/`SortCards`), getters/setters.
- **zerver**: `inbound/http/handlers/card/search_card.rs` +
  `deck/search_deck_cards.rs` (deserialize `CardQuery`),
  `outbound/sqlx/card/mod.rs` (bind from `CardQuery.criteria`/`limit`; keep the
  `MAX_SEARCH_LIMIT` clamp).
- **zwiper**: `screens/home.rs`, `deck/card/remove.rs`, `deck/card/view.rs`,
  `deck/card/add.rs`, `deck/components/swipe_select.rs`, `deck/components/deck_fields.rs`.

---

## Done when

The in-memory filter path can't see a `limit`; the server query carries a bounded
`Limit`; the old-JSON round-trip test passes (wire unchanged); the frontend is
exercised end to end on the sim.
