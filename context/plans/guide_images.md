# Guide images

**Status: ACTIVE — 7 guides shipped (getting-started, swipe-to-build,
commander-and-formats, filtering, commander-maybeboard, organize-and-browse,
share-your-deck; 19 webps + 1 shared reference).**

**Resume here:**
1. ~~Verify shipped images sit in the right places.~~ Done 2026-08-17: all 9
   shipped webps viewed and matched against their `Block::Image` alt text and
   `guide_image()` registry arms — no mismatches.
2. **Next capture session: the single-shot run — `remove-cards`, `swipe-memory`,
   `synergy` (1 each, tracker below); they can come in one batch.**
   Owner drops raw PNGs on the Desktop (not staging — that's been the real
   flow) with rough names; assistant converts (`cwebp -q 82 -resize 860 0`),
   places, adds registry arms + `Block::Image` entries, compiles, commits,
   ticks the tracker.

**Layout note (settled 2026-08-17, don't re-derive):** guide screenshots
render in ONE prev/next gallery per guide — a sidecar `Panel` (eyebrow
"Screens" / title "In the app") in the LEFT column, prose right, sticky
below the nav via the measured `--nav-height` var (body ResizeObserver in
`zite/src/main.rs`). `Block::Image` position in the data no longer affects
layout (the gallery harvests them in order) but keep entries next to their
sections as documentation. Article header = h1 with `panel-title` sizing +
tag chips + `panel-rule`. One guide at a time, in the tracker order below. Built to
survive interruption: the tracker + per-guide shot lists say exactly where we
left off and what to capture next.

## Decisions (owner, 2026-08-16)

- **Density varies by guide** — the shot list below is the authority (most
  guides 2–3, simple ones 1, concept guides can be 0).
- **Owner captures, assistant finishes.** Raw simulator PNGs land in the
  staging dir with rough names; assistant renames, optimizes, wires, commits.
- **Order starts at `getting-started`** (the flagship), then the tracker.
- **Every shot in the default dark theme**, iPhone 11 Pro Max simulator,
  portrait. (One exception: the share-page shots are desktop-browser
  captures of zwipe.net.)

## The loop (per guide)

1. Owner captures the guide's shots (list below) and drops the raw PNGs into
   `zite/assets/guides/_staging/` — rough names are fine ("maybeboard
   expanded.png"). The dir is scratch; it never ships.
2. Assistant: renames to convention, downsizes (target ~860px wide, the 2×
   of the rendered column), compresses, places under
   `zite/assets/guides/<slug>/`, wires the `Block::Image` entries into the
   guide, compiles, commits.
3. Owner eyeballs the deployed page, checks the tracker box, next guide.

## Naming + wiring

- **Files:** `zite/assets/guides/<guide-slug>/<nn>-<what-it-shows>.webp`
  (fall back to `.png` if webp tooling is missing on the Mac — check
  `cwebp`/`sips` at first optimize). Example:
  `zite/assets/guides/getting-started/02-add-screen.webp`.
- **Content:** new `Block::Image { src, alt, caption }` variant in
  `guides/content.rs`, rendered by `guides/mod.rs` with a
  `.guide-img` style (bordered, rounded like the app's cards, lazy-loaded,
  max-width 100%). Alt text is written per shot in the lists below.
- **Asset registry:** Dioxus `asset!()` is a compile-time macro, so a data
  array can't mint assets dynamically. `content.rs` gets a small
  `guide_image(path) -> Asset` match that lists each shipped image literally
  — one line added per image at wiring time.

## One-time infra (before the first guide)

- [x] `Block::Image` variant + `render_block` arm + `.guide-img` CSS
- [x] `_staging/` dir created + gitignored
- [x] `guide_image` asset registry scaffold (in content.rs, empty until first image)

## Tracker + shot lists

Format: `nn — what the image must show` (alt text ≈ the same sentence).

- [x] **getting-started** (3) — shipped 2026-08-17
  - 01 — create-deck form with name, format picked, commander filled
  - 02 — add screen mid-stack: a card up top, Synergy/Filter chips visible
  - 03 — deck view with a real deck: featured cards + stats sections
- [x] **swipe-to-build** (3) — shipped 2026-08-17
  - 01 — add screen with the swipe hint dialog open (the four directions)
  - 02 — card filter sheet open over the add screen
  - 03 — "From" row on Maybeboard: the maybeboard stack being swiped
- [x] **commander-and-formats** (2) — shipped 2026-08-17
  - 01 — format picker open, a commander format's details showing
  - 02 — commander Zwipe select: a legendary on top of the pile
- [x] **commander-maybeboard** (4) — shipped 2026-08-17 (owner added a 4th: the screen's own hint dialog)
  - 01 — Swipe select hint open showing the up-swipe bullet
  - 02 — the maybeboard screen with several saved commanders (art on)
  - 03 — one row expanded: Printing / Create deck / Remove buttons visible
  - 04 — the Commander maybeboard hint dialog (Swipe / Quick add / Create deck bullets)
- [x] **organize-and-browse** (3) — shipped 2026-08-17 (01 grouped by Tag not
  Color: the color group-header pip has a size/centering bug, logged in
  progress/todo.md; recapture by Color once fixed if desired)
  - 01 — deck list grouped by Tag with Show pips + tag chips visible
  - 02 — card list grouped by Type with Creatures collapsed (arrow sideways)
  - 03 — quick add results floating over the list
- [x] **share-your-deck** (3) — shipped 2026-08-17 (owner added the dialog's
  shared state, so the guide shows both halves of the toggle)
  - 01 — the Share deck dialog before sharing (Create link)
  - 02 — a shared deck page on zwipe.net, featured cards + sections visible
    (desktop browser capture, kept at its native 1236px — not the 860px phone
    width; don't upscale it on a re-convert)
  - 03 — the dialog once shared (Copy link / Stop share)
- [x] **filtering** (2) — shipped 2026-08-17 (02 became the Filters hint dialog; the sheet shot is shared with swipe-to-build)
  - 01 — filter sheet: colors + type section
  - 02 — add screen with the Filter dot lit (active filter)
- [ ] **remove-cards** (1)
  - 01 — remove screen with a card mid-swipe
- [ ] **swipe-memory** (1)
  - 01 — deck More sheet with Clear skips visible
- [ ] **synergy** (1)
  - 01 — add screen, Synergy chip on, an on-theme suggestion up top
- [ ] **deck-stats** (2)
  - 01 — Distributions section open (type/color bars)
  - 02 — Draw odds section open with the turn stepper
- [ ] **budgeting** (1)
  - 01 — Budget section open: total vs target, currency chips
- [ ] **land-targets** (1)
  - 01 — Mana section: Lands actual/target row + curve
- [ ] **deck-mvps** (1)
  - 01 — deck view featured strip with starred MVPs
- [ ] **deck-tags** (1)
  - 01 — deck tag picker open with a few tags selected
- [ ] **oracle-tags** (2)
  - 01 — oracle tag select with search results
  - 02 — deck view Tags section showing chosen oracle tags
- [ ] **card-roles** (1)
  - 01 — expanded card row: Card roles chips under the details
- [ ] **tags-roles-and-oracle-tags** (0 — concept guide, links carry it)
- [ ] **oracle-tag-dictionary** (1)
  - 01 — dictionary screen mid-letter with a description open
- [ ] **import-export** (2)
  - 01 — import screen with a pasted list
  - 02 — export screen with boards toggled
