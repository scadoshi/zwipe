# Server-driven catalogs — roles & deck tags without client releases

**Status: Part 0 + B + C DONE 2026-07-12 (committed, UNPUSHED).** Goal met: adding a role, a deck
tag, or a tag correlation is now a server const edit + deploy — no app-store turnaround. Everything
is **additive and existing-client-safe** (the wire is byte-identical slug arrays either way; the
type changes are internal to `zwipe-core` and only affect the next client build). No
`MIN_CLIENT_VERSION` bump. Full workspace green (clippy `--workspace -D warnings`, tests, build).

### DONE (commits, in order)
- **Part 0 — lossy deserialization** (`d036b86b`): `serde_helpers::lossy_vec` on the served enum-vecs
  (`mechanical_categories`, deck `other_tags`) so an unknown slug never crashes an old client.
- **Catalog endpoints + fetchers** (`e16c0ba2`): `GET /api/card/roles` (public, `CardRoleView`) +
  `GET /api/deck/tags` (authed, `DeckTagView` incl. `seed_otags`); `ClientGetCardRoles` /
  `ClientGetDeckTags`.
- **Part B — roles fully server-driven:** the filter fetches the role catalog (`3948829a`); and
  **`CardProfile.card_roles: Vec<CardRole> → Vec<String>` slugs** (`3c3b83f2`) so a new server role
  renders on cards/lists/chips without a release. `card_role::role_label(slug)` resolves labels
  (curated for known, prettified for new). `group_by_category` buckets dynamically by slug.
- **Part C — deck tags fully server-driven:** `DeckProfile.tags` / `HttpSharedDeck.tags →
  Vec<String>` slugs (`fe2e1896`); `deck_tag::deck_tag_label(slug)` resolver; display sites (deck
  view, deck list, zite shared deck) + `seed_oracle_tags` take slugs; **the picker (`TagSelect`)
  now renders options from the fetched `GET /api/deck/tags` catalog** (`818af097`), so a new deck
  tag is selectable — create/edit fetch it and pass it as a `catalog` prop.

### Remaining follow-ups (small, non-gating)
- **Catalog-based seeding:** `seed_oracle_tags` still uses the compiled `DeckTag::oracle_tag_slugs`,
  so a *new* tag's otag seeds don't apply until a release. Finish: seed from the catalog's
  `seed_otags` (a `seed_oracle_tags_from_catalog(slugs, &[DeckTagView])` helper, used in the
  create/edit reconcile which already has the catalog loaded).
- **Deck role-distribution chart** (`deck_metrics.rs`) stays keyed on the known `CardRole` set
  (fixed compact-label axes) — a new role isn't charted there (it still shows in grouped lists +
  on cards). Making it dynamic would change its output type + the chart consumer; deferred.
- **Ship it:** deploy + the next client build (the payoff only reaches users on the client that
  carries these changes; deployed clients are untouched).

> **NOTE:** `MechanicalCategory` is now `CardRole` and its module dir is `card_role/` (both renames
> shipped). The wire/DB field `mechanical_categories` still exists (moves at the Phase M sunset).
> Read `MechanicalCategory` below as `CardRole`. The original plan text is kept for rationale.

## The problem

The per-card role↔otag *relationships are already 100% server-driven*: which roles a card has
(`card_profiles.mechanical_categories`), the tags grouped under each
(`oracle_tags_by_role`), and the "Other" bucket (`other_oracle_tags`) are all computed in
`zervice` from `CATEGORY_ROOTS` + `ROLE_TAG_OVERRIDES` and written to the DB. That is why
Protection grouping, the override pipeline, Card advantage, Energy, and Aggression all reach
every client on a `zervice` run with **zero client release**.

Two things are still **compiled into the client** and therefore gated on releases:

1. **Role display metadata.** The client turns a role slug into a label via the
   `MechanicalCategory` enum's `display_name` / `to_short_name`. A brand-new role slug the
   installed enum doesn't know is `filter_map`-dropped on read (`card_profile.rs` conversion)
   → the role is **invisible** until the app ships the new variant. The grouping is
   server-driven but the *label* isn't.
2. **Deck-tag catalog + the deck-tag→otag seed map.** `DeckTag` (enum + `display_name` +
   `describe`) and `DeckTag::oracle_tag_slugs` (the seed map) are hardcoded, and seeding runs
   **client-side** (`seed_oracle_tags` in `create.rs:287` / `edit.rs:638`). New deck tags or
   new seed relationships need a release.

## What we're building (B + C)

Serve the enums + their metadata + their otag relationships as **catalogs the client fetches
and renders from**, instead of reading its own compiled enums.

### Source of truth — decision: Rust consts served over HTTP (deploy to change)

Keep the `MechanicalCategory` / `DeckTag` enums and the `CATEGORY_ROOTS` / `oracle_tag_slugs`
consts as the source of truth (they still drive server-side derivation, filtering criteria,
and type safety) and **expose them over HTTP**. Changing a catalog = edit the const + deploy;
**no client release**. This matches the otags philosophy ("updates on deploy") and adds
minimal machinery.

> **Future option (out of scope now):** move the mappings into DB tables so a catalog can
> change with a **DB write and no deploy at all**. Bigger change (admin tooling, derivation
> reads from DB). Note it; don't build it yet.

## Part 0 — the compatibility bridge: lossy slug deserialization — ✅ DONE (`d036b86b`)

**Built 2026-07-12 (unpushed).** `zwipe-core/src/serde_helpers.rs::lossy_vec` — reads `Vec<String>`,
`filter_map`s into the enum, drops unknowns — applied via
`#[serde(default, deserialize_with = "crate::serde_helpers::lossy_vec")]` to the served enum-vecs:
`CardProfile.{mechanical_categories, card_roles}` (`Vec<CardRole>`), `DeckProfile.{tags, other_tags}`,
`HttpSharedDeck.{tags, other_tags}`. Deserialize-only (serialization + wire unchanged → deployed
clients unaffected); ships in the not-yet-deployed client so that release and every later one
survives catalog growth. Requests (create/update deck) left strict — the server is always newest.
Tests: `unknown_role_slug_is_dropped_not_errored` (card_profile.rs) + `lossy_vec` unit tests.

**Reminder (operational):** Part 0 protects clients *from this release forward*, not the
already-deployed strict ones. Until a `MIN_CLIENT_VERSION` floor guarantees everyone is on a
Part-0 client, **do not actually add a new role/tag slug** that reaches older clients — it still
crashes them. Part 0 is the enabler; the hot-patch payoff is safe only once old clients age out.

Original rationale below (still accurate):

This is what makes the whole thing "work forever **and** work with existing clients until
they switch." It is a permanent foundation, **not** a stopgap.

**The problem:** the wire carries role/deck-tag **slugs**, but the shared types deserialize
them into **strict enums** — `CardProfile.mechanical_categories: Vec<MechanicalCategory>` and
`DeckProfile.tags: Vec<DeckTag>` (`zwipe-core`) use the derived `Deserialize`. An unknown slug
(a role/tag a newer server knows but this binary doesn't) makes serde **error the entire
card/deck**. So the moment the server grows a catalog, any client whose enum predates it would
break — not just miss the new value.

**The fix:** deserialize these vecs **lossily** — read `Vec<String>`, `filter_map` into the
enum, **drop unknowns** (exactly the user's rule: "if in a vec, remove that entry"; and the
same thing the server's DB adapter already does in `card_profile.rs` / `deck/models.rs`). Add
a `deserialize_with` helper in `zwipe-core` and apply it to `mechanical_categories`,
`DeckProfile.tags`, and any other enum-vec on a served type (`other_tags`, etc.). No stored
`Unknown` variant; unknowns simply vanish.

**Why this delivers the requirement:**
- An **un-switched enum client** (renders from its compiled enum) receiving a new role/tag
  slug: drops it, renders what it knows, **never crashes**. It keeps working with today's
  server and every future server — it just doesn't *show* the new value until it adopts the
  catalog.
- A **catalog-driven client** (Parts B/C) renders **everything** the server sends, because it
  labels by slug from the catalog, not the enum.
- The **server** keeps emitting all slugs on the same fields — no wire change.

This must ship in the **first role/deck-tag-carrying client release** (which, per the owner,
isn't out yet — so the timing is free). Ship it, and every later catalog growth is safe for
that client whether or not it has switched to catalog rendering.

## Part B — role catalog (all in our lane)

**DTO (zwipe-core, mirrors `OracleTag`):**
```rust
pub struct CardRoleView { pub slug: String, pub display_name: String, pub short_name: String }
```
The client needs only slug→label(+short for charts); it already receives the pre-grouped
`oracle_tags_by_role` per card, so the role→otag mapping does **not** ship in this catalog.

**Endpoint:** `GET /api/card/roles` → `Vec<CardRoleView>` (built from `MechanicalCategory::all()`).
Copy the `GET /api/card/oracle-tags` chain exactly:
- `zwipe-core/src/http/paths.rs` — add `api/card/roles`.
- `zerver/.../http/routes.rs` — `.route("/roles", get(get_card_roles))` under `/api/card`.
- `zerver/.../handlers/card/get_card_roles.rs` + `handlers/card/mod.rs`.
- `domain/card/{ports,services}.rs` + `outbound/sqlx/card/mod.rs` (or build straight from the
  enum — no DB read needed; simpler than oracle-tags).
- `zwiper/.../client/card/get_card_roles.rs` — `ClientGetCardRoles`.

**Client consumption (the real work):**
- `zwipe-components/src/card_role_chips.rs` — `CardRoleChips` currently takes
  `roles: Vec<MechanicalCategory>` and calls `display_name`. Change it to take **role slugs +
  a resolver** (or pre-resolved `{slug, label}` views) so it renders any role the server
  sends. ⚠ Shared component — also consumed by zite's shared deck (and the portfolio via the
  git dep); keep a graceful fallback (prettify the slug) when a label is missing.
- `zwiper/.../deck/card/filter/category.rs` — the "Card roles" filter iterates
  `MechanicalCategory::all()`; drive it from the fetched catalog instead.
- **Role distribution chart** (uses `to_short_name`) — drive labels from the catalog.
- **Fallback:** if the catalog fetch fails (offline), fall back to the compiled enum so the
  UI still works. Additive, resilient.

## Part C — deck-tag catalog + seed map (⚠ overlaps the otags agent's live deck-tag picker)

**DTO (zwipe-core):**
```rust
pub struct DeckTagView {
    pub slug: String,
    pub display_name: String,
    pub description: String,
    pub seed_otags: Vec<String>,   // from DeckTag::oracle_tag_slugs
}
```

**Endpoint:** `GET /api/deck/tags` → `Vec<DeckTagView>` (from `DeckTag::all()` + `display_name`
+ `describe` + `oracle_tag_slugs`). Same chain shape, under the deck routes.

**Client consumption:**
- `zwiper/.../deck/components/tag_select.rs` — the deck-tag picker iterates `DeckTag::all()`;
  drive from the catalog.
- `create.rs:287` / `edit.rs:638` — seeding calls `seed_oracle_tags(&tags)` (compiled map).
  Seed from the catalog's `seed_otags` instead, so new correlations apply without a release.
- `deck_fields.rs` — hosts deck-tag selection; wire it to the catalog.

**Why the wire already tolerates this:** deck tags are stored as `decks.tags` JSONB strings
and read back through a `filter_map(DeckTag::try_from … .ok())` that **drops unknowns**
(same tolerant pattern as card roles, `deck/models.rs`). So a server can serve a **new** deck
tag the client's enum doesn't know; the client shows it (from the catalog), the user selects
it, it's stored as a slug and re-served — end to end, no client enum change.

## Wire & compatibility

- **All additive.** New `GET` endpoints; no existing route/field changes. **No
  `MIN_CLIENT_VERSION` bump.**
- **Part 0 is the bridge.** Once a client ships lossy deserialization, it survives every
  future catalog growth — un-switched clients drop unknown slugs and keep working; switched
  clients render everything from the catalog. That is the "works with existing clients until
  they switch" guarantee.
- **New (catalog-driven) clients** fetch on load, render from the catalog, and once shipped
  pick up **future** roles/tags/relationships on the next fetch — no further release, forever.
- **Filter/selection round-trips stay string-slug based** (`mechanical_categories_*`
  criteria, `decks.tags`), and the server already drops unknown slugs — so a catalog-driven
  client sending a new slug is safe against an older server too.

## Sequencing

0. **Part 0 — lossy slug deserialization** (`zwipe-core`, foundational): ✅ **DONE** (`d036b86b`,
   unpushed). Must ship in the first role/deck-tag-carrying client release — it's baked into the
   pending (un-deployed) client, so that's satisfied on the next deploy + client ship.
1. **B — role catalog** (self-contained, our lane): DTO + `GET /api/card/roles` + make
   `CardRoleChips` / category filter / chart catalog-driven, with the compiled enum as an
   offline fallback.
2. **C — deck-tag catalog + seed** (coordinate with the otags agent — it owns `tag_select.rs`,
   `deck_fields.rs`, `create/edit.rs` seeding right now): DTO + `GET /api/deck/tags` + drive
   the picker and seeding from it.

## Open decisions

- **B first, alone** (no agent overlap) — recommended start after Part 0. C waits until the
  otags agent's Slice C settles, to avoid clobbering the live deck-tag picker.
- **Charts & filters scope:** confirm every client surface that reads `MechanicalCategory` /
  `DeckTag` (category filter, role chart, deck-tag picker) migrates in the same pass, or the
  compiled enum lingers as a half-source.
