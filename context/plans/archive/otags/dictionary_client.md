# Oracle-tag dictionary — client UX (Part 2)

**Status: PLANNED 2026-07-13. Client-only (zwiper). No new backend.**  
**Companion:** [`tag_descriptions_and_dictionary.md`](tag_descriptions_and_dictionary.md)
(Part 1 shipped; Part 2 overview), [`dictionary_backend.md`](dictionary_backend.md)
(endpoint + CF already ready).

**One sentence:** an in-app, read-only dictionary of all ~4,500 oracle tags where
**letter navigation is primary**, optional search is secondary, and only the
active letter's tags mount in the DOM.

---

## Goals

- Teach what oracle-tag slugs mean (1,100+ authored descriptions; tail shows
  "No description yet").
- Keyboard-optional: players can browse without ever focusing a search field.
- Instant open after cold start once the catalog is session-cached (see
  [`../catalog_session_cache.md`](../catalog_session_cache.md) — dictionary can
  ship with a dedicated otag cache first; the general catalog plan unifies it).

## Non-goals (MVP)

- In-app authoring or editing of descriptions.
- zite / marketing-site dictionary.
- Full tree browser, role-grouped taxonomy, or "All letters" dump.
- New shared components in `zwipe-components` (piece together existing ones).
- New API endpoints.

---

## Layout

```
[!]     Oracle tags          [?]     ← ScreenHeader (support + hint, as every screen)
────────────────────────────────────
[ search optional… ]                 ← secondary; not required to use the screen
[*A*] [B] [C] …  →                   ← Chip rail: nowrap + horizontal scroll
────────────────────────────────────
  only selected letter’s tags        ← other letters do not render at all
  slug (mono)
  label (optional if useful)
  description | "No description yet"
  parent_slugs (muted / chips, light)
────────────────────────────────────
[ Back ]                             ← ActionBar + Button Util
```

### Letter rail (primary)

- **Top row**, not a left-side index (phone width + existing chip patterns).
- **Require horizontal scroll** — do **not** wrap 26 chips onto one screen.
- Each letter is a `zwipe_components::Chip` with `selected` for the active letter.
- Letters **A–Z** always present so the strip is predictable; optional trailing
  `#` only if any slug starts with a non-letter.
- **Default selection:** `A` (or first letter that has tags if we prefer).
- **Render rule:** `tags.filter(|t| first_letter(t.slug) == selected)` only.
  Switching letters remounts the list (Changelog-style key with letter prefix so
  ease-in can replay).
- Empty letter → muted copy, e.g. "No tags for S" (fine; rare if catalog is full).

**Grouping key:** first character of **slug** (stable identity; mono display).
Lowercase; chips may display as `A`…`Z`.

### Search (secondary)

- Optional search field under the header / above the letter rail.
- When the query is **empty** → letter mode (above).
- When the query is **non-empty** → search **entire catalog** (override letter):
  match **slug + label + description** (case-insensitive). Cap results if needed
  for DOM safety (e.g. first ~100 ranked by slug); letter rail can stay visible
  but inactive, or clear on letter tap.
- Placeholder sentence case, e.g. "Search tags or descriptions".
- **Do not** auto-focus the field on open — no forced keyboard.

### List rows

Local markup + CSS (not `Panel`, not `CardRow`, not keyword expand-chips):

| Field | Treatment |
|-------|-----------|
| slug | Mono emphasis (app is already JetBrains Mono) |
| description | Body text; missing → "No description yet" |
| label | Optional second line if it adds clarity |
| parent_slugs | Optional muted chips/text; MVP can be display-only |

**Avoid:** picker-style "tap chip → fill def bar" — dictionary rows always show
the description. **Avoid:** mounting all 4.5k tags at once.

### Header / footer

- `ScreenHeader { title, hint }` — `!` and `?` already built; wire hint body only.
- Hint: short explainer (letter rail primary; search optional; descriptions
  author-over-time). New key `HINT_ORACLE_TAG_DICTIONARY` in
  `zwipe-core` hints (shape-only server validation).
- `ActionBar` + Util **Back**.

### Loading / error

| State | Behavior |
|-------|----------|
| Catalog loading | Skeleton: fake letter chips + a few entry bars (existing skeleton CSS language) |
| Catalog failed | Toast (sentence case); muted empty body. Prefetch fails at app load — toast when the screen first observes `Failed` (ToastProvider is below upkeeper, so toast from the screen, not from startup) |
| Empty letter | Muted inline message |

---

## Data

| Layer | What |
|-------|------|
| Endpoint | Existing `GET /api/card/oracle-tags` (public, CF Rule 1) |
| Client | Existing `ClientGetOracleTags` — **no** `Authorization` (keeps CF cache) |
| Wire | `OracleTag { slug, label, description, parent_slugs }` |
| Cache | Prefer session cache (startup prefetch). Until
  [`catalog_session_cache.md`](../catalog_session_cache.md) lands as a unified
  system, implement a dedicated `OracleTagCache` mirror of `ChangelogCache` in
  `session_upkeep` — same lifecycle. Dictionary + picker + filter otag UI all
  read it. |

Picker (`oracle_tag_select.rs`) and card-filter oracle-tags today each
`use_resource` their own fetch — migrate them to the session cache when the
cache lands (same PR as dictionary or immediately after).

---

## Components to reuse (no new shared primitives)

| Need | Source |
|------|--------|
| Letter / parent chips | `zwipe_components::Chip` |
| Footer | `ActionBar` + `Button` Util |
| Header | `ScreenHeader` |
| Hint shell | `HintDialog` + bullets / `OracleTagsExplainer` sibling copy |
| Filter→list mental model | `Changelog` (chip filter + remount keys) — **not** its wrap CSS |
| Scroll body | Screen + local CSS: `.letter-rail` (nowrap, `overflow-x: auto`),
  `.dictionary-list` (vertical, only active set) |
| Skeleton | Existing `.skeleton-bar` / changelog-skeleton patterns in `main.css` |

**Do not use for rows:** `Panel`, `KeywordChips`, `CardRoleChips`, `CardRow`,
`CardDetails` (wrong job or too heavy).

**Local CSS only for the rail:** existing `.changelog-filter` / `.chip-row`
**wrap**; dictionary requires **nowrap + horizontal scroll**.

---

## Routing & entry points

| Item | Detail |
|------|--------|
| Route | e.g. `/oracle-tags` → `OracleTagDictionary` |
| Guard | `Bouncer` (same as other in-app reference screens) |
| Entry | Button/link from `oracle_tag_select` ("Browse dictionary" / similar,
  sentence case); optional mention in Oracle tags `?` / explainer |
| Back | `navigator.go_back()` — returns to create/edit with picker still open if
  it was |

Not on Profile for MVP (picker + hint is enough); Profile entry is a later nicety.

---

## Files (expected)

- **New** `zwiper/.../screens/oracle_tag_dictionary.rs` (or under `deck/` if
  preferred — top-level screen is fine; not zite)
- `zwiper/.../router.rs` — route
- `zwiper/.../screens/mod.rs` — module
- `zwiper/.../session_upkeep.rs` — `OracleTagCache` + startup fetch (or later
  fold into unified catalog cache)
- `zwiper/.../oracle_tag_select.rs` (+ filter otag if present) — read cache;
  entry button
- `zwiper/assets/main.css` — letter rail + dictionary list/entry
- `zwipe-core/.../hints.rs` — `HINT_ORACLE_TAG_DICTIONARY`
- Docs: mark this plan + Part 2 sequencing done when shipped

---

## Sequencing

1. Session `OracleTagCache` (or first slice of
   [`catalog_session_cache.md`](../catalog_session_cache.md) for otags only).
2. Dictionary screen (letter rail + optional search + skeleton).
3. Entry from picker + hint.
4. Point picker/filter at the cache (drop per-open `use_resource` for otags).
5. Later: unify other filter catalogs under the same cache plan.

Ship on the next client build (e.g. 1.7.0); additive, no `MIN_CLIENT_VERSION`
bump, no server deploy required for the UI (endpoint already live).

---

## Decisions (2026-07-13)

| Decision | Choice |
|----------|--------|
| Primary browse | Letter rail, horizontal scroll |
| Secondary browse | Optional search (slug + label + description) |
| DOM | Only selected letter's tags (or search hits) render |
| Letter placement | Top row chips, not left rail |
| Group by | Slug first character |
| "All" | No (would re-render thousands) |
| Keyboard | Never auto-focus search |
| Home | In-app only |
| Shared components | Reuse `Chip` + chrome; no new crate components for MVP |
