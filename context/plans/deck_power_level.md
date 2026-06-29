# Deck Power Level & Other Tags

## Why / discovery

While building the full-screen **tag** picker and reworking the **format**
selector, we realized decks also want to express *how strong / what kind of pod*
they're for — a **power level** (Casual → cEDH) and a few secondary,
non-gameplay labels (Budget, Jank, Meme, Precon, …).

The existing `DeckTag`s are about **what the deck does / how it plays** — that's
the primary, gameplay axis, and plain `tags` is the right name for it (no
rename). What we're adding is a **second, secondary axis**: tags that are *not*
about gameplay. Rather than overthink the name, call that bucket **`other_tags`**
— a deliberately generic catch-all for secondary descriptors, easy to extend
later.

Two additions to the **deck profile** (no rename of anything existing):

1. **Power level** — a single-select **simple enum** (`PowerLevel`). Not a tag.
2. **Other tags** — a multi-select curated enum (`DeckOtherTag`): the secondary,
   non-gameplay bucket (Budget, Jank, Meme, Precon, Upgraded Precon, …).

## Naming note — keep `tags`, don't rename the wire/column

We keep the primary axis as `DeckTag` / `tags` everywhere (type, wire field, DB
column). Renaming the wire attribute or column buys nothing and the wire rename
specifically would **break the server-first guarantee** (old clients send/read
`tags`; a server that renamed it would silently drop tags for them). The serde
field name is the public contract; leave it alone. New axis is just `other_tags`
alongside it.

## Hard requirement: additive, server-first, no client breakage

The new server must store/serve power level and other tags **before** any mobile
client knows about them, and older installed clients must keep working unchanged.
The deploy pipeline already ships server before client (same repo, shared
`main`), so this is the natural order — but the contracts must be additive so a
new server's responses don't break old apps, and old apps' requests don't require
the new fields. **No min-version gate.**

This mirrors exactly how `tags` was added (see `http/contracts/deck.rs`: the
`#[serde(default)]` note on `HttpUpdateDeckProfile.tags`, and the
forward-compatible "drop unknown strings" parse in
`outbound/sqlx/deck/models.rs`).

### Compatibility rules (the whole point)

- **Response (`DeckProfile`)** — add `power_level: Option<PowerLevel>` and
  `other_tags: Vec<DeckOtherTag>`, both `#[serde(default)]`. `DeckProfile` has no
  `deny_unknown_fields`, so **old clients ignore the new response fields**; new
  clients reading an older payload default to `None` / empty.
- **Create (`HttpCreateDeckProfile`)** — add `power_level: Option<String>` and
  `other_tags: Option<Vec<String>>`, both `#[serde(default, skip_serializing_if = "Option::is_none")]`.
  Old clients omit them → server treats as none.
- **Update (`HttpUpdateDeckProfile`)** — add `power_level: Opdate<String>` and
  `other_tags: Opdate<Vec<String>>`, both `#[serde(default)]` (absent =
  `Unchanged`, `null` = clear). Same pattern as `tags`.
- **DB migration** (additive, defaulted so old INSERTs still work):
  ```sql
  ALTER TABLE decks ADD COLUMN power_level TEXT;                           -- nullable
  ALTER TABLE decks ADD COLUMN other_tags JSONB NOT NULL DEFAULT '[]'::jsonb;
  CREATE INDEX idx_decks_other_tags ON decks USING GIN(other_tags);
  ```
- **SQLx (`DatabaseDeckProfile`)** — add `power_level: Option<String>` and
  `other_tags: Option<serde_json::Value>`; in `TryFrom`, parse power level via
  `PowerLevel::try_from` (drop/clear if unrecognized) and other tags by filtering
  through `DeckOtherTag::try_from` (**drop unknown strings**, exactly like tags),
  so a newer DB written by a future client never breaks an older server reading
  it.

## Scope by layer

### zwipe-core (domain — ships in the server build first)
- `deck/models/power_level.rs` — `PowerLevel` enum + `display_name()`,
  `serde(rename_all = "snake_case")`, `TryFrom<&str>` / `Display`.
- `deck/models/deck_other_tag.rs` — `DeckOtherTag` enum (Budget, Jank, Meme,
  Precon, UpgradedPrecon, …) + `display_name()`, `all()`, `TryFrom`,
  `parse_other_tags()` (drop dupes, like `parse_tags`). **Variants only ever
  added, never removed/renamed** — same rule as `DeckTag`. This is the bucket;
  add to it freely over time.
- `DeckProfile` — add `power_level` and `other_tags` fields.
- Request types (`create_deck_profile.rs`, `update_deck_profile.rs`) +
  validation, mirroring how `format` (single enum string) and `tags` (vec) are
  validated.

### zerver (server — deploy step 1)
- Migration above; `cargo sqlx prepare --workspace` after query changes.
- `DatabaseDeckProfile` + `TryFrom` mapping (forward-compatible parse).
- Create/update services persist the new columns; re-export the core types.

### zwiper (client — deploy step 2, after server is live)
- Deck form (`deck_fields.rs`): a **Power level** field — single-select chips
  (or a small picker) for `PowerLevel`; and an **Other tags** field —
  multi-select chips for `DeckOtherTag`. Reuse the chip-box / "None" field
  pattern just built for tags/format.
- Deck profile/view screen (`deck_profile.rs`): show power level + other tags.
- Wire into `HttpCreateDeckProfile` / `HttpUpdateDeckProfile` (Opdate) like tags.

## Open decisions

1. **`PowerLevel` variants** — simple single-select. Either **Casual, Mid, High,
   cEDH** (simple 4-tier) or the official **Commander Brackets** (Exhibition,
   Core, Upgraded, Optimized, cEDH). Plumbing is identical; default to the simple
   4-tier.
2. **RESOLVED — `PowerLevel` stays separate** from `other_tags`: a rating wants
   exactly one value and is worth sorting/filtering on, which a multi-select
   bucket can't enforce.

## Deploy order

1. **Server:** migration + new core types + accept/store/return `power_level` &
   `other_tags`. Old clients unaffected (ignore new response fields, never send
   new request fields).
2. **Client:** add the inputs + profile display. No gate, no client-side
   migration.
</content>
