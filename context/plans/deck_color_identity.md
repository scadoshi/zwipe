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
draft used the `Colors` newtype. `Color` and `Colors` now **do** carry manual
`Serialize`/`Deserialize` impls (verified 2026-07-11, `colors.rs:57-84,143-168` —
`Color` serializes to its short code), so the earlier "no serde" rationale is
stale. But `Colors` still has **no `Default`** (`colors.rs:121` derives only
`Debug, Clone, PartialEq`), which a `#[serde(default)]` field requires, and adding
`#[derive(Default)]` to a shared type just to satisfy the DTO is churn we don't
need. `Vec<String>` is leaner, serializes trivially, and matches the DB `text[]`
column exactly (the subquery returns `text[]` straight onto the field). No
shared-type changes. Sorting is handled client-side by iterating `Color::all()`
(already WUBRG) and filtering to members, so no separate sort helper is needed.

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

otags **Phase 2 (card-level oracle tags) is DONE and committed** (`f11cc1e3`,
2026-07-11); main is clean and compiles. This slice does **not** collide with
anything landed so far — Phase 2 touched `card_profiles`, not `decks`/`DeckProfile`.

The collision is only with **otags Phase 3 ("deck otag selection")**, which is
**not started yet** — both it and this slice add a field to `DeckProfile` /
`DatabaseDeckProfile` and edit the same 4 deck-profile constructor queries +
`.sqlx/`. They are additive and coexist fine, but **do not build them on parallel
branches** (git + `.sqlx/` conflicts on the same queries). Land them sequentially.
Per the owner's ordering, **otags goes first**; once Phase 3's DeckProfile/deck-query
changes are in, adding `color_identity` next to `oracle_tags` in the same 4 sites is
trivial. (This derive-only slice has **no migration**, so if it must ship before
Phase 3 it can — just re-`sqlx prepare` after whichever lands second.) **Green-light
question for the owner:** does this go now (Phase 3 not started) or wait for Phase 3?

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
1. `create_deck_profile` — `query_as!` `RETURNING …` (**L83-92**; existing
   `commander_name?` subqueries at L89-92 are the template).
2. `get_deck_profile` — `query_as!` `SELECT …` (**L209-227**). **Has GROUP BY**
   (L226-227) — the correlated subquery references `d.commander_id` etc. Confirmed:
   the GROUP BY **leads with `d.id`**, so the command-zone id columns are covered by
   functional dependency and a `d.`-aliased subquery is legal. This is the site that
   also covers the shared-deck page (see share-link note above).
3. `get_deck_profiles` — `query_as!` `SELECT …` (**L239-257**). Same GROUP BY,
   also leads with `d.id` (L256-257).
4. `update_deck_profile` — **runtime `QueryBuilder`** `.push(" RETURNING …")`
   (**L368-371**). Runtime, so no `.sqlx` entry, but still returns a
   `DatabaseDeckProfile`; use the **unaliased** subquery form (bare `commander_id`).

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

- **Share-link fetch — RESOLVED, no extra work.** `get_shared_deck`
  (`services.rs:900`) resolves the token to `(deck_id, owner_id)` via
  `get_deck_id_by_share_token` (which selects only `id, user_id` — no profile),
  then calls `get_deck` → `get_deck_profile` (site #2). So fixing the 4 sites above
  covers the shared-deck page automatically; no separate query.
- `cargo sqlx prepare --workspace`, commit `.sqlx/`.

### client — `zwiper/src/lib/inbound/screens/deck/list.rs`
- The zone chips render as `stat-chip stat-chip-zone` for commander / partner /
  background / signature spell (confirmed **L149-160**, signature spell ends L159-160).
  Render the identity pips **right after that block**, before the tag loops at L161.
- **Pip class must be lowercase, no `ms-shadow`** — mirror the live pattern at
  `card/filter/mana/color_identity.rs:131`: `i { class: "ms ms-{code_lower} ms-cost" }`
  (`ms-w`/`ms-u`/…; `ms-shadow` is not used anywhere and would be a dead class).
- **No sort helper needed** — `Color::all()` already returns WUBRG order, so filter
  it by membership. Parse the DTO codes into a set of `Color` (via `Color::try_from`,
  case-insensitive), then iterate `Color::all()`:
  ```rust
  {
      let present: std::collections::HashSet<Color> = profile
          .color_identity
          .iter()
          .filter_map(|s| Color::try_from(s.as_str()).ok())
          .collect();
      (!present.is_empty()).then(|| rsx! {
          span { class: "deck-list-identity",
              for color in Color::all().into_iter().filter(|c| present.contains(c)) {
                  i {
                      key: "{color.to_short_name()}",
                      class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                  }
              }
          }
      })
  }
  ```
  (`use zwipe_core::domain::card::models::scryfall_data::colors::Color;` at file scope.)
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
