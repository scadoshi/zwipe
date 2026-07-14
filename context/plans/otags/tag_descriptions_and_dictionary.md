# Oracle tag descriptions + dictionary

**Status: Part 1 mechanism SHIPPED (2026-07-13, `0114cb38`); 1,100 descriptions
authored (oracle-text-verified) — high-traffic head fully covered, tail ongoing.
Part 2 (dictionary) BUILT 2026-07-14, ships in the 1.7.0 client — see
[`dictionary_client.md`](dictionary_client.md).**
Two linked pieces the owner asked for:
1. **Our own descriptions layer** — Scryfall describes only ~29% of tags (1,302 of
   4,494; the biggest tags are often blank); author our own over time until we
   describe **all** of them (fully replacing Scryfall's). **Mechanism done; 1,100
   authored (oracle-text-verified); tail authoring ongoing.**
2. **Oracle tag dictionary** — a browsable page of all ~4,500 tags + descriptions
   to surf. **BUILT 2026-07-14** (letter-first browse + search; ships in 1.7.0).

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

**Runbook:** [`../../development/runbooks/otag_description_authoring.md`](../../development/runbooks/otag_description_authoring.md)
— the repeatable AI-orchestrated loop (fan out subagents to draft + verify each
description against real card oracle text, then splice into the const). Ships a
reusable `Workflow` script alongside it. Follow that to run a batch.

Add entries to `ORACLE_TAG_DESCRIPTIONS`, push, deploy; the next `zervice` run
writes them in. Priority order: **highest card-population blanks first** (e.g.
`triggered-ability` @ 7,885, `attack-trigger`, `removal-creature`), then the
curated ~48, then the long tail — until coverage is satisfactory, then all of them.
Coverage so far: 7 (starter) → 82 → 257 → 500 → 700 → **1,100** (2026-07-13). Descriptions are user-facing:
short, plain, em-dash-free; do **not** carry Scryfall's `[label](slug)` cross-link
syntax (we overwrite with our own plain text everywhere). A catalog dump with
per-tag populations to prioritize from lives in the sweep scratchpad
(`otag-sweep/catalog.tsv`) — regenerate with the query in Part 1's authoring notes
if gone.

---

## Part 2 — Oracle tag dictionary (browsable page)

**Full client UX plan (letter-first, components, routing):**  
[`dictionary_client.md`](dictionary_client.md) — **source of truth for implementers.**

**Backend/serving:** [`dictionary_backend.md`](dictionary_backend.md) (endpoint +
CF already ready). **Catalog prefetch:**  
[`../catalog_session_cache.md`](../catalog_session_cache.md) (session cache + 1-day
TTL; dictionary can ship on Phase 0 otag-only cache).

### What (summary)

In-app read-only reference of all ~4,500 tags: **slug** (mono) + **description**
(+ optional label / parent_slugs). Tags with no description show **"No description
yet"**.

**Primary browse:** top-row **letter chips** (horizontal scroll, not wrap). Only
the **selected letter's tags render** — other letters never mount.  
**Secondary:** optional search over **slug + label + description** (no auto-focus /
forced keyboard).  
**Chrome:** `ScreenHeader` (`!` / `?`), skeleton while loading, toast on fetch fail,
`ActionBar` Back.

### Data source

Existing **`GET /api/card/oracle-tags`** + **`ClientGetOracleTags`** — no new
backend. Prefer session cache over per-open `use_resource` (see catalog plan).

### Where (decided 2026-07-12): in-app (zwiper), read-only

Not on zite. Reachable from the Oracle tags picker + `?` hint.

---

## Sequencing

1. **Part 1 mechanism** (overlay + tests) — SHIPPED.
2. **Bulk authoring** — 1,100 done (high-traffic head covered); tail ongoing via
   runbook.
3. **Part 2 dictionary** — next client build; UX locked in
   [`dictionary_client.md`](dictionary_client.md). Can pair with catalog-cache
   Phase 0 (oracle tags only).

## Decisions

- **Descriptions store (2026-07-12):** repo file compiled into the server; overlay
  at sync; no admin UI.
- **Dictionary home (2026-07-12):** in-app (zwiper), read-only. Not zite.
- **Browse model (2026-07-13):** letter rail primary, search optional; only active
  letter mounts. Details in `dictionary_client.md`.
