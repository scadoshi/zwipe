# Deck color identity on the profile

**Status: PLANNED (2026-07-11).** Part of the deck-cards-screen revamp
(squircle mana pips, green price tag, flowing card rows: those shipped to the
working tree the same day). This slice is the one piece that needs a server
half.

## Goal

Show a deck's color identity as mana pips on the deck **list** row, right after
the commander/zone chips:

```
[deck name] [12 cards] [Commander] [Nteza…] [W][U][B]
```

Derived from the command zone only: the union of the color identities of the
commander, partner, background, and signature spell. Decks with no command-zone
cards (e.g. a 60-card Modern deck) simply show no pips.

## Key decision: derive at read time, do NOT store

The identity comes from at most four card IDs already on the `decks` row
(`commander_id`, `partner_commander_id`, `background_id`, `signature_spell_id`),
so it is a few indexed PK lookups into `scryfall_data` — the same shape as the
existing `commander_name` subquery. Deriving it in the query means **no
migration, no backfill, no sync-on-write, and no second source of truth**.

Storing a denormalized column was rejected: it would only pay off if derivation
were expensive (e.g. a union over all 99 cards), which this is not.

`color_identity` is added to the `DeckProfile` **DTO** (the per-request response
object), not to any table. That is still "compute at runtime" — `commander_name`
lives on the same DTO the same way.

## Approach

### zwipe-core — `domain/deck/models/deck_profile.rs`
- Add `#[serde(default)] pub color_identity: Colors` to `DeckProfile`.
- `Colors` (`domain/card/models/scryfall_data/colors.rs`) needs a `Default`
  derive (empty = colorless) for `#[serde(default)]`. It already has
  Serialize/Deserialize and `from_short_names`/`to_short_names`.
- `Colors` derefs to `[Color]` and `Color: Ord`, so the client can sort to
  canonical WUBRG the same way `card_row.rs` does today.

### zerver — `outbound/sqlx/deck/`
- `models.rs`: add `pub color_identity: Option<Vec<String>>` to
  `DatabaseDeckProfile`; in `TryFrom<DatabaseDeckProfile> for DeckProfile`,
  `Colors::from_short_names(value.color_identity.unwrap_or_default())?`.
- `mod.rs`: add one aggregating subquery to each of the **3** `query_as!` sites
  that build `DatabaseDeckProfile` — `create_deck_profile` (RETURNING, ~L84),
  `get_deck_profile` (~L214), `get_deck_profiles` (~L244). `color_identity` on
  `scryfall_data` is `text[]`, so union the distinct letters across the command
  zone in SQL:

  ```sql
  (SELECT array_agg(DISTINCT ci)
     FROM scryfall_data sd, unnest(sd.color_identity) AS ci
    WHERE sd.id IN (commander_id, partner_commander_id,
                    background_id, signature_spell_id))
    as "color_identity?: Vec<String>"
  ```
  NULL command-zone IDs simply don't match, so this is null-safe.
- Confirm the **share-link fetch** path (zite's shared deck) — if it returns a
  `DeckProfile` it goes through one of these 3 constructors and is covered; if
  it has its own query, add the subquery there too.
- `cargo sqlx prepare --workspace` and commit `.sqlx/`.

### client — `zwiper/src/lib/inbound/screens/deck/list.rs`
- After the zone chips (background/signature-spell block, before the user-tag
  loops), render sorted pips from `profile.color_identity`:
  ```rust
  if !profile.color_identity.is_empty() {
      span { class: "deck-list-identity",
          for code in sorted_short_codes(&profile.color_identity) {
              i { key: "{code}", class: "ms ms-{code} ms-cost ms-shadow" }
          }
      }
  }
  ```
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
No migration, no min-version gate. Verify: `cargo test --workspace` (add a
`TryFrom` union unit test), `cargo clippy --workspace`, `cargo sqlx prepare
--workspace`, manual `dx serve` pass of the deck list.
