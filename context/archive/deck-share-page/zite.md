# Deck share page — zite changes

The page mirrors the app's deck cards view: pinned command zone, grouped
list, grouping picker, the same filters — running the same zwipe-core code.

## 1. Route — `zite/src/main.rs`

`#[route("/deck/:token")]` → `SharedDeck { token: String }`, following the
`/verify/:token` pattern (SPA routing already works via the 404.html
fallback; no Pages config change).

## 2. Page — `zite/src/pages/shared_deck.rs`

- On mount: `GET {API_BASE}/api/share/deck/{token}` (reqwest, same as the
  verify page). States: loading skeleton → deck → friendly 404 ("This deck
  is no longer shared") → network error with retry.
- **Command zone pinned** above the groups: commander (+ partner /
  background / signature spell when present), card image + name.
- **Grouped list**: `deck_cards.group_by(GroupByOption::CardType)` by
  default — the exact `GroupCards` trait from zwipe-core the app's view.rs
  uses. Group header shows name + count ("Creatures (24)"); rows show
  name, mana cost glyphs (reuse zite's mana-symbol rendering if present,
  else port the `symbol_class` mapping from zwiper's `oracle_text.rs`),
  and quantity when > 1.
- **Grouping picker**: chips for the same `GroupByOption::all()` set the app
  offers (card type / mana value / color / category).
- **Filters**: the in-memory path — a `CardQueryBuilder` + the core
  client-side filtering/sorting (`.matches()` / `.sorted()` — the same calls
  zwiper's maybeboard filter effect uses at `add.rs` ~line 866). v1 filter
  surface: name contains, card type, color, mana value range; skip the full
  filter sheet (this is a reading page, not a building page).
- **Header**: deck name, format chip, card count, price estimate if the
  payload carries prices. Footer CTA: "Built with Zwipe" → zwipe.net home
  (the growth hook, styled prominently but honestly).
- Layout: single column on phones, 2–3 group columns on wide screens
  (Archidekt-style density), terminal aesthetic, `overflow-x` contained.

## 3. SEO / meta

- `noindex` on the route (private-by-link decks shouldn't be crawled;
  revisit if a public browse ever exists).
- Page `<title>`: deck name once loaded. OG link-preview cards are a known
  SPA limitation — out of v1 (see overview "Later").

## 4. Copy rules

Sentence case, no em dashes, "Zwipe" capitalized. Guides: add a short
"Sharing your deck" section to the deck-management guide when the client
button ships (not before — no orphan docs for a button that doesn't exist).
