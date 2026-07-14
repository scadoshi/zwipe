# Oracle tag descriptions + dictionary

**Status: Part 1 mechanism SHIPPED (2026-07-13, `0114cb38`); authoring + Part 2 open.**
Two linked pieces the owner asked for:
1. **Our own descriptions layer** — Scryfall describes only ~29% of tags (1,302 of
   4,494; the biggest tags are often blank); author our own over time until we
   describe **all** of them (fully replacing Scryfall's). **Mechanism done; bulk
   authoring is the ongoing work.**
2. **Oracle tag dictionary** — a browsable page of all ~4,500 tags + descriptions
   to surf. **Not started.**

They're one arc: the dictionary renders the descriptions. Raw slugs (shipped
`ffd52c5e`) make this more valuable: the slug is the stable key, the "?" and the
dictionary carry the meaning.

## One sentence

Give oracle tags human-written descriptions we author gradually, kept durable
against the daily Scryfall resync by **writing them into the `description` column
at sync time** (ours always wins), and surfaced in a searchable dictionary of all
~4,500 tags.

> **Note (2026-07-13):** the shipped design **differs from the original plan below**.
> We did *not* do a serve-time merge; instead `zervice` **overlays** our authored
> descriptions into `oracle_tags.description` inside the sync transaction, right
> after the `DELETE` + reinsert. See the updated Part 1. The rest of the doc
> (authoring priority, Part 2 dictionary) still stands.

---

## Part 1 — Our own descriptions layer (backend)

### The hard constraint (why we can't just fill the column)

`oracle_tags.description` is populated by Scryfall, and the daily sync
(`zerver/.../card/helpers/oracle_tags.rs::sync_oracle_tags`) does
**`DELETE FROM oracle_tags` then re-INSERT** — a full replace. Any description
written into that column out of band is **wiped on the next `zervice` run**. So our
descriptions must live in a store the Scryfall sync never touches, and be
re-applied every sync.

### Design (SHIPPED 2026-07-13): repo const, overlaid into the column at sync time

Our descriptions are keyed by `slug`, **authored in a repo file, compiled into the
binary, and shipped by a normal GitHub push -> deploy** (no DB table, no
admin/live-edit path — deliberately). Concretely: `ORACLE_TAG_DESCRIPTIONS`, a
`&[(&str, &str)]` const in
`zerver/src/lib/outbound/sqlx/card/helpers/oracle_tag_descriptions.rs`.

**How it's applied (this is the change from the original plan):** rather than
merging at serve time, `sync_oracle_tags` runs an **overlay `UPDATE` inside the sync
transaction**, right after the catalog reinsert:

```sql
UPDATE oracle_tags SET description = d.description
FROM unnest($slugs, $descriptions) AS d(slug, description)
WHERE oracle_tags.slug = d.slug
```

Ours always wins: it **replaces** Scryfall's where we have one and **fills** the
blanks otherwise. Because the merged text now lives in the column post-sync, every
reader (`get_oracle_tags`, the picker definition bar, the Part 2 dictionary) picks
it up with **no serve-time merge and no client change**. A non-fatal log warn flags
any authored slug the fresh catalog lacks (a typo would otherwise match nothing).

Why an overlay (not serve-merge): simpler readers (the column is the single source
post-sync), and it matches the owner's ask ("zervice writes it in when done"). It
migrates cleanly to a DB table later if live editing is ever wanted — out of scope.

### Files (shipped)

- **`zerver/.../card/helpers/oracle_tag_descriptions.rs`** — the authored
  `ORACLE_TAG_DESCRIPTIONS` const + `description_pairs()` flattener + tests
  (unique slugs, non-blank). Server-only; `zwipe-core` stays pure.
- **`zerver/.../card/helpers/oracle_tags.rs`** — the overlay `UPDATE` + typo warn
  inside `sync_oracle_tags`'s transaction.
- No `get_oracle_tags` change, no wire change, no client change, no migration,
  no `MIN_CLIENT_VERSION` bump — additive.

### Authoring workflow (the open work)

Add entries to `ORACLE_TAG_DESCRIPTIONS`, push, deploy; the next `zervice` run
writes them in. Priority order: **highest card-population blanks first** (e.g.
`triggered-ability` @ 7,885, `attack-trigger`, `removal-creature`), then the
curated ~48, then the long tail — until coverage is satisfactory, then all of them.
The starter batch (7 tags) shipped with the mechanism. Descriptions are user-facing:
short, plain, em-dash-free; do **not** carry Scryfall's `[label](slug)` cross-link
syntax (we overwrite with our own plain text everywhere). A catalog dump with
per-tag populations to prioritize from lives in the sweep scratchpad
(`otag-sweep/catalog.tsv`) — regenerate with the query in Part 1's authoring notes
if gone.

---

## Part 2 — Oracle tag dictionary (browsable page)

### What

A searchable, browsable reference of all ~4,500 tags: **slug** (mono) +
**description** + **hierarchy** (parent/child from `parent_slugs`). Live search
(filter by slug/description), optional grouping by role or alphabetical. Tags with
no description show "No description yet" (honest, and shows authoring progress).

### Data source

The existing **`GET /api/card/oracle-tags`** (now serving merged descriptions from
Part 1) — one endpoint, no new backend. It returns all ~4,500 as
`OracleTag { slug, label, description, parent_slugs }`. The client trait
**`ClientGetOracleTags` already exists** (`zwiper/.../client/card/get_oracle_tags.rs`),
so the screen reuses it — no new client wiring.

### Where (decided 2026-07-12): in-app (zwiper), read-only

A new **zwiper screen** — a searchable, browsable reference. Read-only: authoring
happens in the repo file (Part 1), not in the app. Reachable from the Oracle tags
picker (a link/button) and/or the Oracle tags "?" hint. Not on zite.

### Files

- **New** `zwiper/src/lib/inbound/screens/.../oracle_tag_dictionary.rs` — the
  screen: `use_resource` over `client.get_oracle_tags()`, a search box, the list,
  and parent/child hierarchy chips.
- Router entry (`zwiper/.../router.rs`) + an entry point: a button in
  `oracle_tag_select.rs` (the picker) and/or a line in the Oracle tags hint copy.
- Model the fetch/loading/skeleton on the picker (`oracle_tag_select.rs` already
  `use_resource`s the same catalog); model list styling on existing list screens.

### UX notes

- Search is client-side over the fetched list (~4,500 rows is fine in memory).
- Render **slugs in mono** (matches the raw-slug direction), descriptions in body
  text; tags with no description show "No description yet."
- Parent/child: tapping a parent chip filters to that subtree (reuse the hierarchy
  already in `parent_slugs`).

---

## Sequencing

1. **Part 1** (descriptions store + serve-merge + tests) — small, additive;
   immediately enriches the picker def-bar.
2. **Author batch 1** (curated ~48 + top tags).
3. **Part 2** (dictionary page) — consumes the merged endpoint.

Part 1 gates Part 2's usefulness (empty descriptions aren't worth surfing), but
Part 2 can be built in parallel against the endpoint and light up as descriptions
land.

## Decisions (resolved 2026-07-12)

- **Descriptions store:** a **repo file compiled into the server**, deployed by a
  normal GitHub push. No DB table, no admin/live-edit path (owner declined it).
- **Dictionary home:** **in-app (zwiper)**, read-only. Not zite.
