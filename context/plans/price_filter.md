# Price Filter + Deck Price Limit

## Goal

Two budget-building features requested across the launch thread (Tenellum and
others build budget decks):

1. **Price range filter** — min/max price on the card swipe stack, in USD, EUR, or
   Tix. Range is the target; a single max is the floor of acceptable.
2. **Deck price limit** — a per-deck total budget; notify the user when the deck
   approaches its limit while building (mirrors `land_signals.md`).

Today there's only a price *sort* as a stopgap; this adds the actual budget tools.

---

## Part 1 — Price range filter

### Data

Cards carry prices on `scryfall_data.prices` (`usd`, `eur`, `tix`), each optional.
The filter applies against the **selected currency's** price.

### Layers (mirror the existing mana-value range filter)

- **Core** (`zwipe-core` `CardFilter` / `CardFilterBuilder`): add a price range
  (`price_min`, `price_max`) + a **currency selector** (USD / EUR / Tix). Model on
  the existing MV-range filter so the in-memory predicate (maybeboard client-side
  filtering) and the server query spec share one shape.
- **Server** (`outbound/sqlx` card search QueryBuilder): `WHERE` on the chosen
  currency's price column, with the usual `MAX_SEARCH_LIMIT` clamp untouched.
- **Client** (`screens/deck/card/filter/`): a new price filter screen (mirror the
  MV range screen) — min/max inputs + a currency toggle. Add it to the filter
  sheet.

### Edge cases / decisions

- **Null price in the chosen currency** (card has USD but no EUR, or no price at
  all): include, exclude, or treat as "unknown and filtered out"? Default:
  exclude from results when a price bound is active (you asked for a budget, a
  priceless card can't be confirmed under it).
- **Currency scope:** is the chosen currency a per-filter pick, or a single
  app/deck-wide preference also used by the deck limit (Part 2)? Leaning a
  **single shared currency preference** so the filter and the budget agree.
- Price is a decimal — confirm the type used in core/SQL to avoid float drift.

---

## Part 2 — Deck price limit + approaching-limit notification

### Behavior

- A **per-deck total price limit** the user sets (a number + currency).
- While building (Add / Remove), when the deck total **approaches** the limit
  (e.g. ≥ 90%) or **crosses** it, fire a toast ("Deck is nearing your $X budget"
  / "Over your $X budget"). Advisory, never blocks adding.
- Reuse the **existing deck price estimate** (decks already show price estimates)
  rather than recomputing.

### Key decision — where the limit is stored

1. **Server-side deck field** (`price_limit` + currency on the deck): syncs across
   devices, survives reinstall — needs `zwipe-core` field + migration + server
   deploy (server-first rule).
2. **Client-local**: cheaper, no deploy, but per-device and lost on reinstall.

**Recommendation:** server-side deck field — a budget you set should follow the
deck. Plan it as a server-touching change (batch with other backend work), while
Part 1's filter can ship client-first if the filter currency stays client-side.

### Trigger logic

- Recompute deck total on add/remove; toast on **crossing** the warn threshold and
  on **crossing** the limit (debounce — don't re-toast every card once over).
- Mirror the crossing-not-count debounce from `land_signals.md`.

### Open questions

- Warn threshold percentage (90%? user-configurable?).
- Which price source for the deck total when a card lacks the chosen currency —
  skip it, or fall back to another currency?
- Currency mismatch: filter in USD but limit in EUR — enforce one shared currency
  preference to avoid confusion (ties to Part 1's decision).

---

## Relationship to other plans

- The deck-limit notification shares the **crossing-toast pattern** with
  `land_signals.md` — build them on the same small "deck threshold signal" helper
  if it's clean to do so.
- Currency preference may also touch how `CardInfoDisplay` shows prices (it shows
  all three today).

## Effort & deploy

- **Part 1 (filter):** cross-layer but each layer mirrors the MV-range filter —
  **M**. Can ship client-first if currency stays client-side; if the filter needs
  the server price `WHERE`, that's a server deploy (server-first).
- **Part 2 (deck limit):** **M**, server-side if the limit persists on the deck
  (recommended). After any query change: `cargo sqlx prepare --workspace`, commit
  `.sqlx/`.
