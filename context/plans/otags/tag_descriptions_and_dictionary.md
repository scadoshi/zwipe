# Oracle tag descriptions + dictionary

**Status: PLAN (2026-07-12).** Two linked pieces the owner asked for:
1. **Our own descriptions layer** — Scryfall describes only ~29% of tags; author
   our own descriptions over time so tags are obvious to users.
2. **Oracle tag dictionary** — a browsable page of all ~4,500 tags + descriptions
   to surf.

They're one arc: the dictionary renders the descriptions. Raw slugs (shipped
`ffd52c5e`) make this more valuable: the slug is the stable key, the "?" and the
dictionary carry the meaning.

## One sentence

Give oracle tags human-written descriptions we author gradually, stored *outside*
the Scryfall-synced catalog so the daily resync can't wipe them, merged at serve
time, and surfaced in a searchable dictionary of all ~4,500 tags.

---

## Part 1 — Our own descriptions layer (backend)

### The hard constraint (why we can't just fill the column)

`oracle_tags.description` is populated by Scryfall, and the daily sync
(`zerver/.../card/helpers/oracle_tags.rs::sync_oracle_tags`) does
**`DELETE FROM oracle_tags` then re-INSERT** — a full replace. Any description we
write into that column is **wiped on the next `zervice` run**. So our descriptions
must live in a store the Scryfall sync never touches, and be merged at read time.

### Design (decided 2026-07-12): repo file, compiled in, deployed by push

Our descriptions are keyed by `slug`, **authored in a repo file, compiled into the
server binary, and shipped by a normal GitHub push -> deploy** (no DB table, no
admin/live-edit path — deliberately). Concretely: a zerver-owned
`slug -> &'static str` source (a Rust module, or a `.json` asset pulled in with
`include_str!` and parsed once at startup), merged over Scryfall's at serve time.

Why this shape: version-controlled and diff-reviewed in PRs, no admin surface to
secure, and "gradually add descriptions" = add entries to the file, push, deploy.
The owner explicitly does **not** want app-side/admin editing.

(If live editing without a deploy is ever wanted later, this migrates cleanly to a
DB table + `LEFT JOIN` — but that's out of scope and not planned.)

### Serving (both options)

`get_oracle_tags` (`zerver/src/lib/outbound/sqlx/card/mod.rs`, the catalog query)
merges: **`description = our_description ?? scryfall_description`** (ours wins;
falls back to Scryfall, then `None`). The `OracleTag` DTO already has
`description: Option<String>` — so **no wire change and no client change**. This
immediately enriches:
- the deck picker's definition bar (`oracle_tag_select.rs` already renders
  `t.description`),
- anywhere else that reads the catalog (and the dictionary in Part 2).

### Files

- **New** `zerver/src/lib/.../oracle_tag_descriptions.rs` (or a `.json` asset +
  `include_str!` loader) — the authored `slug -> &'static str` map. Server-only
  (keep `zwipe-core` pure; only the server serves descriptions).
- `zerver/.../card/mod.rs get_oracle_tags` — apply the merge.
- Tests: our description wins over Scryfall's; our description fills a Scryfall
  `NULL`; a slug with neither stays `None`.

### Authoring workflow

Add entries over time, priority order: the **curated ~48** (`CURATED_ORACLE_TAGS`)
first, then the most-populated tags, then the long tail. Each batch is a small
commit + deploy. The dictionary (Part 2) doubles as a progress view: tags with no
description render "No description yet."

**No `MIN_CLIENT_VERSION` bump, no migration (Option A), additive.**

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
