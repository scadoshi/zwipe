# Deck color identity on the profile

**Status: SHIPPED (local, unpushed) 2026-07-11.** Client render committed in
`c46ee448` (with pip color tinting); server derivation + DTO field + `.sqlx` in
`4ae63435`. Not yet pushed to `main` (push auto-deploys prod). Derive-only: **no
migration, no stored column.**

Part of the deck-cards-screen revamp (squircle mana pips, price/stat tags, flowing
card rows — those shipped separately). This slice added the server half.

## What it does

Mana pips on the deck **list** row showing the deck's color identity:

```
[deck name] [W][U][B] [12 cards] [Commander] [Nteza…]
```

(Pips render right after the deck name — the owner moved them there from the
originally-planned position after the zone chips.)

**Derivation (the key design point): union of the command zone AND every mainboard
card.** For a given deck the identity is the DISTINCT union of `color_identity`
across:
- the command-zone cards (`commander_id`, `partner_commander_id`, `background_id`,
  `signature_spell_id`), and
- every `deck_cards` row with `board = 'deck'` (mainboard only; maybeboard and
  sideboard excluded, matching how card_count is computed).

Consequences, all intended:
- **Commander decks** → the commander's legal identity. Cards are always within the
  commander's identity, so the union equals it, and the command-zone half covers it
  even for a brand-new deck with zero cards.
- **Non-Commander decks** (Modern, etc., no command zone) → the colors the deck
  **actually plays**, from the mainboard.
- **Empty decks** (no zone, no cards) → colorless → no pips.

This replaced the original **command-zone-only** scope (which left every non-EDH
deck blank). Verified against real dev decks: an 83-card non-Commander deck went
from blank → RUW, while Commander decks were unchanged (union == command-zone
identity).

## Key decision: derive at read time, do NOT store

Identity is computed in the deck-profile queries — a correlated subquery over the
command-zone IDs plus the deck's mainboard rows. No migration, no backfill, no
sync-on-write, no second source of truth. `color_identity` lives on the
`DeckProfile` **DTO** only (like `commander_name`), never on a table.

### Considered and declined: a stored, user-declared field

Idea: a `decks.color_identity` column so any format carries a formal, user-set
identity (a metrics axis for the otags non-EDH moat, `otags/moat.md` keying signal
on `(format, color_identity, otag_set)`). **Declined** — the read-time union now
gives non-EDH decks an identity for free, so the stored column buys only a
*declared-vs-actual* distinction that isn't worth a column + WUBRG picker UI + write
path + sparse data. Revisit only if the moat needs declared identity specifically;
if so, build it in the same wave as otags Phase 3 (shares the `decks` column +
create/update queries + DeckProfile change with `decks.oracle_tags`).

### DTO field type: `Vec<String>`, not `Colors`

`Vec<String>` of WUBRG short codes: matches the DB `text[]` the subquery returns,
serializes trivially, `#[serde(default)]`-friendly. `Colors` was rejected because
it has no `Default` (`colors.rs:121` derives only `Debug, Clone, PartialEq`) and
adding one to a shared type just for the DTO is needless churn. (`Color`/`Colors`
*do* have serde impls now, `colors.rs:57-84,143-168` — the old "no serde" note was
stale.) Client sorting iterates `Color::all()` (already WUBRG) filtered to members;
no sort helper needed.

## What was built

### zwipe-core — `domain/deck/models/deck_profile.rs`
`DeckProfile` gained `#[serde(default)] pub color_identity: Vec<String>` (empty =
colorless / empty deck). No `Colors` changes.

### zerver — `outbound/sqlx/deck/models.rs`
`DatabaseDeckProfile` gained `pub color_identity: Option<Vec<String>>`; `TryFrom`
sets `color_identity: value.color_identity.unwrap_or_default()`. Two unit tests
cover the present-passes-through and null-becomes-empty mappings.

### zerver — `outbound/sqlx/deck/mod.rs` — 4 constructor sites
All four build a `DatabaseDeckProfile`; the 3 macros needed `sqlx prepare`, the 4th
is runtime:
1. `create_deck_profile` — `query_as!` RETURNING (unaliased; mainboard clause keys
   on the inserted row's `id`, which has no cards yet, so on create only the command
   zone contributes).
2. `get_deck_profile` — `query_as!` SELECT, `d.`-aliased. GROUP BY leads with
   `d.id`, so the correlated subquery is legal. **Also covers the shared-deck page**
   (`get_shared_deck` → `get_deck` → `get_deck_profile`).
3. `get_deck_profiles` — `query_as!` SELECT, `d.`-aliased.
4. `update_deck_profile` — runtime `QueryBuilder` RETURNING, `decks.`-prefixed, no
   `.sqlx` entry.

The subquery (shape shown `d.`-aliased; RETURNING sites use bare / `decks.` forms).
Note the inner alias is **`sci`** — the SELECT sites already use `sd` for the
commander name join, so a distinct alias is required:
```sql
(SELECT array_agg(DISTINCT ci)
   FROM scryfall_data sci, unnest(sci.color_identity) AS ci
  WHERE sci.id IN (d.commander_id, d.partner_commander_id,
                   d.background_id, d.signature_spell_id)
     OR sci.id IN (SELECT dc2.scryfall_data_id FROM deck_cards dc2
                    WHERE dc2.deck_id = d.id AND dc2.board = 'deck'))
  as "color_identity?: Vec<String>"
```
No matching rows (empty deck) → NULL → `unwrap_or_default()` → empty. Cost is
negligible: `deck_cards(deck_id)` is indexed (leading col of the unique
constraint), and the list query already joins `deck_cards` for card_count.

### client — `zwiper/.../deck/list.rs` + `zwiper/assets/main.css`
Pips parse the DTO codes into a `HashSet<Color>`, then iterate `Color::all()`
filtered to members, rendering `i.ms.ms-{code_lower}.ms-cost` (lowercase mana-font
classes). `.deck-list-identity` is `inline-flex` with a small gap; pips inherit the
global squircle/border from `components.css` via `ms-cost`. `c46ee448` also tinted
each pip's outline + glyph to its own mana color.

## Still open

- **Deck view header pips.** The same pips on the `deck/view.rs` header (`[deck
  name] [commander]` line) are **not** done — list-only for now. Fast-follow if the
  list version looks good. Owner to confirm.
- **Push / deploy.** Both commits are local. Pushing to `main` auto-deploys prod;
  server derivation ships ahead of the client, and the `#[serde(default)]` field
  keeps older payloads valid. No migration, no min-version gate. Owner triggers the
  push.

## Verification (all green before commit)

- `cargo +nightly fmt` (CI gate — stable fmt silently passes, nightly is required).
- `cargo clippy -p zwipe-core -p zerver -p zwiper --all-targets -- -D warnings`.
- `cargo test -p zwipe-core -p zerver` — 117 zerver (incl. the 2 `color_identity`
  tests) + all zwipe-core, 0 failures.
- `cargo sqlx prepare --workspace --check` — cache fresh (3 macro queries changed).
- Functional check on dev DB: non-Commander deck derives its mainboard colors;
  Commander decks unchanged; empty decks blank.
- Pending: manual `dx serve` eyeball pass (owner).
