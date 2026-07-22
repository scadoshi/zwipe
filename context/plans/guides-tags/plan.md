# Guides page: tags + filter (retire categories)

## Why

The `/guides` index groups 17 guides under 4 category headings, but the split is
lopsided: **Decks 11, Build 3, Cards 2, Start 1**. "Decks" has become a 60%
dumping bucket, so the category axis no longer organizes anything and the page
reads flat. Replace category grouping with per-guide **tags** (1-3 each) and a
**chip filter** so people sift by subject, mirroring the Changelog's filter and
the app's chip grammar (oracle tags, roles, About page).

Guide index cards are already `Panel {}` (from the earlier alignment pass), so
this builds on that.

## Taxonomy (finalized)

Vocabulary — **9 tags**. Display/filter order:

`Getting started` · `Swiping` · `Filtering` · `Cards` · `Commander` ·
`Oracle tags` · `Deck building` · `Deck stats` · `Importing`

(No "Reference" tag — the dictionary and the "how they fit" overview live under
`Oracle tags`; their reference nature is served by the **related-guides
cross-links** in step 5, not a tag.)

Per-guide assignment (slug → tags; **first tag is the primary**, used for the
breadcrumb + JSON-LD):

| slug | tags |
|---|---|
| getting-started | Getting started |
| swipe-to-build | Swiping, Deck building |
| remove-cards | Swiping, Deck building |
| swipe-memory | Swiping |
| filtering | Filtering, Cards |
| synergy | Commander, Cards |
| commander-and-formats | Commander, Deck building |
| budgeting | Deck building |
| land-targets | Deck building |
| deck-tags | Deck building, Oracle tags |
| oracle-tags | Oracle tags, Cards |
| oracle-tag-dictionary | Oracle tags |
| card-roles | Cards, Oracle tags |
| tags-roles-and-oracle-tags | Oracle tags, Cards |
| deck-mvps | Deck building, Cards |
| deck-stats | Deck stats |
| import-export | Importing, Deck building |

Notes:
- **"Deck building" is the broad one (7 guides).** That's acceptable under tags
  (unlike the old category) because each also carries a sharper second tag, so
  people can still narrow. If it still feels like a pile after building, split it
  into `Building` (swipe/remove/MVPs) + `Setup` (budget, land target, format) —
  one extra tag, mechanical to reassign.
- Primary tag per guide is chosen so the breadcrumb still reads sensibly
  (e.g. `Guides → Commander → Choose a commander & format`).

## Files in play

- `zite/src/pages/guides/content.rs` — `Guide` struct + `GUIDES` data.
- `zite/src/pages/guides/mod.rs` — index (`Guides`) + detail (`GuidePage`),
  `CATEGORY_ORDER` const, JSON-LD, breadcrumb.
- `zite/assets/style.css` — `.guide-cat*` (remove), new filter + tag styles.

`category` is currently read in three places, all of which move to the primary
tag `g.tags[0]`:
1. index grouping (`CATEGORY_ORDER` loop) — deleted outright
2. JSON-LD `"articleSection"` (mod.rs ~L151)
3. detail breadcrumb `.crumb-cat` (mod.rs ~L170)

## Steps

### 1. Data model — `content.rs`
- In `struct Guide`, **replace** `pub category: &'static str,` with
  `pub tags: &'static [&'static str],`.
- Also add `pub related: &'static [&'static str],` (slugs of guides this one
  references or complements; empty slice is fine). Powers step 5.
- For each of the 17 guides, replace the `category: "..."` line with
  `tags: &["...", "..."]` per the table above (primary tag first), and add a
  `related: &[...]` line (seed list in step 5; `&[]` where none yet).
- (Every guide has ≥1 tag, so `g.tags[0]` is always safe downstream.)

### 2. Index page — `guides/mod.rs`
- Delete the `CATEGORY_ORDER` const (~L81-83) and its doc comment.
- Add filter state inside `Guides`:
  `let mut selected = use_signal(|| Option::<&'static str>::None);`
- Define the tag list for the filter row (a module const):
  `const GUIDE_TAGS: &[&str] = &["Getting started","Swiping","Filtering","Cards","Commander","Oracle tags","Deck building","Deck stats","Importing"];`
- Replace the `for cat in CATEGORY_ORDER` block (~L99-114) with:
  - a **filter row** `div.guide-filter`: an "All" chip
    (`button`, active when `selected().is_none()`, onclick sets `None`) followed
    by one `button.tag` per `GUIDE_TAGS` (active when `Some(tag)`, onclick sets
    `Some(tag)`); toggling the active one back to `None` is a nice-to-have.
  - a single `div.card-grid` iterating
    `GUIDES.iter().filter(|g| selected().map_or(true, |t| g.tags.contains(&t)))`,
    rendering the existing `Link { class:"guide-card", Panel { title, ... } }`.
- **Inside each guide Panel**, after the `p.card-summary`, add a tag chip row:
  `div { class: "guide-tags", for t in g.tags { span { class: "tag", "{t}" } } }`
  (non-interactive; the whole card is already a `Link`, so card chips are visual
  only — filtering happens via the top row).

### 3. Detail page — `guides/mod.rs` (`GuidePage`)
- JSON-LD: `"articleSection": g.category` → `g.tags[0]`.
- Breadcrumb: `span { class:"crumb-cat", "{g.category}" }` → `"{g.tags[0]}"`.
- (Optional polish: render all `g.tags` as chips under the guide title too.)

### 4. Styling — `zite/assets/style.css`
- **Remove** `.guide-cat` and `.guide-cat-heading` rules (now unused).
- **Filter row**: `.guide-filter { display:flex; flex-wrap:wrap; gap:0.5rem; margin-bottom:1.5rem; }`.
- **Filter chips** reuse the colored `.tag` look but as buttons, so normalize:
  `button.tag { font-family:inherit; cursor:pointer; }` (the base `.tag`
  border/padding/color already applies). Add an **active** state — e.g.
  `.tag.active { filter:brightness(1.35); }` or a filled variant — and style the
  "All" chip (a plain neutral `.tag`).
  - Note: `.tag` cycles 6 theme colors by `:nth-child`, so filter chips get
    stable colors (fixed order) while card chips color by position. That's
    consistent with the About page. If you want a **fixed color per tag** across
    both places, map each tag to an explicit color class instead of nth-child —
    optional follow-up, more code.
- **Card tag row**: `.guide-tags { display:flex; flex-wrap:wrap; gap:0.4rem; margin-top:0.75rem; }`
  (sits inside `.panel-body`; last child, so the panel-rule/footer spacing is
  already handled).

### 5. Related-guides cross-links — `guides/mod.rs` (`GuidePage`) + `content.rs`
The `related` field (step 1) drives a **"Related guides"** block at the bottom of
each detail page, so guides that reference each other actually link.
- In `GuidePage`, after the content section, if `!g.related.is_empty()` render a
  `.guide-related` block: for each slug in `g.related`, resolve
  `GUIDES.iter().find(|x| x.slug == slug)` and render a `Link` to
  `GuidePage { slug }` showing the title (and its primary tag chip). Skip any
  slug that doesn't resolve (defensive).
- Style: reuse a `Panel` with an eyebrow like "Related", body = the list of
  chip-style `Link`s. Or a plain `.guide-related` list — keep it light.
- **Seed relationships** (author expands; keep them mutual where it reads well):
  - `swipe-to-build` ↔ `remove-cards`, `swipe-memory`, `filtering`
  - `filtering` ↔ `card-roles`, `oracle-tags`
  - `synergy` ↔ `commander-and-formats`, `oracle-tags`
  - `oracle-tags` ↔ `oracle-tag-dictionary`, `card-roles`, `tags-roles-and-oracle-tags`
  - `tags-roles-and-oracle-tags` ↔ `deck-tags`, `card-roles`, `oracle-tags`
  - `budgeting` ↔ `land-targets`, `deck-stats`
  - `import-export` ↔ `deck-stats`
- **Stretch (optional):** turn in-prose references (e.g. the filtering guide's
  "See Read a card at a glance") into real inline links. Needs a link token in
  the `inline()` renderer that maps a guide slug → its route — more involved than
  the curated block, which already delivers most of the value. Defer unless
  wanted.

### 6. Verify
- `cargo check -p zite` (struct field rename touches every guide + both pages).
- `cargo +nightly fmt` before committing (CI gate).
- Manual: filter selects/clears; "All" resets; chips render on cards; breadcrumb
  + JSON-LD show the primary tag; related block links resolve; no leftover
  `.guide-cat` or `category` references
  (`grep -rn "category\|guide-cat\|CATEGORY_ORDER" zite/src/pages/guides`).
- Sanity-check no dangling `related` slugs (each must match a real guide slug).

### 7. Commit (follow `context/development/commit_guidelines.md`)
Suggested split:
- `feat(guides): tag guides, add a chip filter, and link related guides`
  (content.rs + mod.rs + style.css together — one cohesive change), or split the
  related-guides block into its own follow-up commit if you want smaller diffs.

## Open decisions (defaults chosen, flip if you disagree)
- **Reference tag:** dropped — lame tag; the dictionary/overview guides live
  under `Oracle tags` and are connected via related-guides cross-links instead.
- **Deck building breadth:** left broad; split into `Building` + `Setup` only if
  it still reads as a pile after building.
- **Filter select mode:** single-select + "All" (matches changelog, simplest).
  Multi-select OR is a later upgrade if wanted.
- **Related links:** curated `related` slugs + a detail-page block now; inline
  prose links deferred (stretch in step 5).
- **Per-tag fixed colors:** not done; chips color by position like the rest of
  the app. Optional follow-up.
