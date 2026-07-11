# Deck color identity on the profile

**Status: PLANNED, conservative scope (updated 2026-07-11).** Part of the
deck-cards-screen revamp (squircle mana pips, price/stat tags, flowing card
rows — those shipped). This slice is the one piece that needs a server half.
Scope is deliberately **derive-only**: pips for decks whose color identity is
derivable from a command zone. A stored/declared field for other formats was
**considered and declined** (see below).

## Goal

Show a deck's color identity as mana pips on the deck **list** row, right after
the commander/zone chips:

```
[deck name] [12 cards] [Commander] [Nteza…] [W][U][B]
```

Derived from the command zone only: the union of the color identities of the
commander, partner, background, and signature spell. Decks with no command-zone
cards (a 60-card Modern deck, etc.) simply show no pips.

## Key decision: derive at read time, do NOT store

The identity comes from at most four card IDs already on the `decks` row
(`commander_id`, `partner_commander_id`, `background_id`, `signature_spell_id`),
so it is a few indexed PK lookups into `scryfall_data` — the same shape as the
existing `commander_name` subquery. Deriving it in the query means **no
migration, no backfill, no sync-on-write, and no second source of truth**.

`color_identity` is added to the `DeckProfile` **DTO** (the per-request response
object), not to any table — the same way `commander_name` lives on that DTO.

**DTO field type: `Vec<String>` (WUBRG short codes), not `Colors`.** The original
draft used the `Colors` newtype, but `Colors` (and `Color`) currently derive only
`Debug, Clone, PartialEq` — **no `Serialize`/`Deserialize`, no `Default`** (verified
2026-07-11, `colors.rs:121`), so putting it on a `#[serde(default)]` DTO field would
mean adding serde + a short-name adapter to a shared type. `Vec<String>` of short
codes is leaner, serializes trivially, matches the DB `text[]` column exactly (the
subquery returns `text[]` straight onto the field), and the client already needs a
WUBRG sort helper either way. No shared-type changes.

## Considered and declined: a stored field for non-derivable formats

Idea (2026-07-11): give **any** format a formal, user-declared `color_identity`
(a stored `decks.color_identity` column) so non-Commander decks also carry one —
useful as a metrics dimension for the otags non-EDH moat (`otags/moat.md` keys
signal on `(format, color_identity, otag_set)`).

**Declined for now — stay conservative.** It reverses the no-store/no-migration
decision, adds a column + a WUBRG picker UI + DTO write path, and the declared
data would be sparse (most users won't set it). Revisit **only if** the non-EDH
signal moat actually needs a color axis; at that point build it in the **same
wave as otags Phase 3** (it would share the `decks` column + create/update
queries + DeckProfile change with `decks.oracle_tags`). Until then: derive-only.

## Sequencing vs otags

This slice overlaps **otags Phase 3 ("deck otag selection")** — both add a field
to `DeckProfile` / `DatabaseDeckProfile` and edit the same deck-profile
constructor queries + `.sqlx/`. They are additive and coexist fine, but **do not
build them on parallel branches** (git + `.sqlx/` conflicts on the same queries).
Land them sequentially. Per the owner's ordering, **otags goes first**; once
Phase 3's DeckProfile/deck-query changes are in, adding `color_identity` next to
`oracle_tags` in the same 4 sites is trivial. (This derive-only slice has **no
migration**, so if it must ship before Phase 3 it can — just re-`sqlx prepare`
after whichever lands second.)

## Approach

### zwipe-core — `domain/deck/models/deck_profile.rs`
- `DeckProfile` (struct ~L12) gains `#[serde(default)] pub color_identity:
  Vec<String>` (empty = colorless / not derivable). No other core changes; do
  **not** touch the `Colors` type.

### zerver — `outbound/sqlx/deck/models.rs`
- `DatabaseDeckProfile` (struct ~L17): add `pub color_identity: Option<Vec<String>>`.
- In `TryFrom<DatabaseDeckProfile> for DeckProfile` (~L78): set
  `color_identity: value.color_identity.unwrap_or_default()`.

### zerver — `outbound/sqlx/deck/mod.rs` — **4 constructor sites** (not 3)
All four build a `DatabaseDeckProfile` and need the subquery. The **3 macros**
require `cargo sqlx prepare --workspace` + committing `.sqlx/`; the QueryBuilder
one is a runtime query (no `.sqlx`):
1. `create_deck_profile` — `query_as!` `RETURNING …` (~L84-90).
2. `get_deck_profile` — `query_as!` `SELECT …` (~L209-227). **Has GROUP BY** — the
   correlated subquery references `d.commander_id` etc.; they're covered by
   `GROUP BY d.id` functional dependency (confirm the GROUP BY leads with `d.id`,
   else the subquery errors).
3. `get_deck_profiles` — `query_as!` `SELECT …` (~L239-257). Same GROUP BY note.
4. `update_deck_profile` — **runtime `QueryBuilder`** `.push(" RETURNING …")` (~L368).
   Runtime, so no `.sqlx` entry, but still returns a `DatabaseDeckProfile`.

Subquery to add to each RETURNING/SELECT list (unaliased for the RETURNING sites,
`d.`-aliased for the two `SELECT` sites):
```sql
(SELECT array_agg(DISTINCT ci)
   FROM scryfall_data sd, unnest(sd.color_identity) AS ci
  WHERE sd.id IN (commander_id, partner_commander_id,
                  background_id, signature_spell_id))
  as "color_identity?: Vec<String>"
```
NULL command-zone IDs simply don't match, so it's null-safe (no command zone →
NULL → `unwrap_or_default()` → empty).

- **Share-link fetch:** the zite `/deck/:token` page renders a shared deck. Find
  the by-share-token fetch (it is **not** `set_share_token`/`clear_share_token`,
  which only rotate the token) — if it constructs a `DeckProfile`, add the
  subquery there too; if it reuses one of the 4 above, it's already covered.
- `cargo sqlx prepare --workspace`, commit `.sqlx/`.

### client — `zwiper/src/lib/inbound/screens/deck/list.rs`
- The zone chips render as `stat-chip stat-chip-zone` for commander / partner /
  background / signature spell (~L149-156). Render the identity pips **right after
  that block**, before the user-tag loops:
  ```rust
  if !profile.color_identity.is_empty() {
      span { class: "deck-list-identity",
          for code in sorted_short_codes(&profile.color_identity) {
              i { key: "{code}", class: "ms ms-{code} ms-cost ms-shadow" }
          }
      }
  }
  ```
- `sorted_short_codes(&[String]) -> Vec<String>`: sort to canonical WUBRG (map
  each short code to `Color` for its `Ord`, then back to short — mirror how
  `card_row.rs` sorts today). Add it where the list can reach it.
- `deck-list-identity` CSS in `zwiper/assets/main.css`: `inline-flex`, small gap.
  Pips inherit the global squircle + border rule from `components.css`
  automatically (they use `ms-cost`).

## Open toggle

Also render the same pips on the deck **view** header (the `[deck name]
[commander]` line in `deck/view.rs`)? Default: **list only** first; add the view
header as a fast follow if it looks good. Owner to confirm.

## Ship

Server and client are the same repo/main, so the server half deploys ahead of
the client on the normal pipeline; the additive `#[serde(default)]` field means
older payloads parse fine and the client render is inert until the server ships.
No migration, no min-version gate.

Verify (pre-push checklist, `context/development/commit_guidelines.md`):
- `cargo +nightly fmt` (the CI gate — stable fmt silently passes but CI fails).
- `cargo clippy -p zwipe-core -p zerver --all-targets -- -D warnings`.
- `set -a; source zerver/.env; set +a; cargo test -p zwipe-core -p zerver` (add a
  `TryFrom` union unit test).
- `cargo sqlx prepare --workspace --check` (touched 3 query macros → must be fresh).
- Manual `dx serve` pass of the deck list on a Commander deck (pips present) and a
  non-command-zone deck (no pips).
