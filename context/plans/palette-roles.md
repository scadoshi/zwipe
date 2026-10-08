# Palette roles: one job per hue

**Status: PLANNED 2026-10-08. Phase 1 (roles) is next; phase 2 (accent survey) follows once roles are reviewed. Builds on `palette.md`, which is done.**

**One sentence:** give every colored thing in the app and the sites a role (title, label, number, name, link, action, glyph, selected), pin each role to one variable, and let each theme decide which of its hues fills each variable, the way a syntax theme gives "keyword" and "string" fixed roles and lets Gruvbox make them red and green while Hackerman makes them blue and cyan.

## The problem

With six palette slots in every theme (`palette.md`), the app still reads as one color per theme. The 2026-10-08 audit of zwiper's 92 accent references (plus the shared pieces it renders) found why: the three accents each do four or five jobs, and the same entity changes hue between screens.

- `--accent-tertiary` carries titles, eyebrows, card names, counts and rarity chips.
- `--accent-primary` carries page titles, field labels, numbers, action chips, the focus ring and the toggle fill.
- `--accent-secondary` is cleanly "selected", except `.btn-xs` and `.otag-examples-btn` borrow it, so on the filter sheet a selected chip and an action button look alike.
- Card names are primary in the detail view (`.card-detail-name`) and tertiary on rows and image overlays (`.card-info-name`, `.nic-name`).
- Hackerman is the extreme case: its block puts green in `accent-primary`, so every title on every screen is green, and five of its six palette slots are greens. The scheme's own identity is blue with green as the accent.

Adding colors on top would repeat the status-color mistake. The fix is fewer jobs per variable.

## Phase 1: roles, pinned to variables

Sites keep drawing from fixed variables. Each role gets exactly one.

| Role | Today | Variable | Sites (from the audit) |
|---|---|---|---|
| Titles: page, card, sheet, dialog | split primary / tertiary | `--accent-primary` | `.page-header > h2`, `.card h3`, `.card-title`, `.modal-title`, `.alert-dialog-title`, the "Filter" and "Printings" sheet titles, `.privacy-content h2` |
| Labels and captions | split primary / tertiary | `--accent-tertiary` | `.label`, `.label-sm`, `.label-xs`, `.tag-search-label`, `.chips-label`, `.pref-section-label`, `.chip-row-label`, `.card-group-header`, `.hl-key`, chart headings in `charts.rs` |
| Selected, including toggle-on and the active carousel dot | secondary, plus primary in two spots | `--accent-secondary` | `.chip.selected`, `.chip-xs.selected`, `.type-box.selected`, `.mana-box.selected`, `.copy-max-box.selected`, `.pref-row.selected`, `.keyword-chip.active`, `.toggle-track.on`, `.carousel-dot.active` |
| Focus ring | primary | `--accent-primary` (unchanged) | `.input:focus` |
| Numbers and stats | primary and tertiary | `--palette-5` (yellow) | `.home-hero .stat-num`, `.stat-chip`, `.stat-chip-other`, `.card-row-stat`, `.field-count`, `.tag-count`, `.card-info`; zite `.stat-chip-power`, `.stat-chip-price`; zwiper `.stat-chip-power`, `.flavor-price` (these two moved to slot 3 in `palette.md`'s pass and move again here, so one role has one hue) |
| Names: cards, decks, commanders, tags, formats | primary and tertiary | `--palette-3` (green) | `.card-detail-name`, `.card-info-name`, `.nic-name`, `.tag-def-name`, `.dict-slug`, `.stat-chip-zone`, `.stat-chip-format`, the "Zwipe" in `update_required.rs` |
| Links and tappable text | primary | `--palette-1` (blue) | `.privacy-link`, `.password-toggle-btn:hover`, `.flavor-source-link`, `.keyword-chip`, `.card-detail-otags .detail-chip` |
| Utility actions: small buttons and action chips | secondary and primary | `--palette-2` (orange) | `.btn-xs`, `.otag-examples-btn`, `.chip-xs.chip-primary`, `.card-action-btn:hover` |
| Small glyphs | secondary and primary | `--palette-6` (cyan) | `.filter-dot`, the info button, `.skeleton-chip-active` |
| Card type vs rarity chips | primary vs tertiary | `--palette-1` vs `--palette-4` | `.detail-chip-type/rarity`, `.nic-chip-type/rarity` |
| Hint keys (`HintKey`) | hand-picked per call site | the role variable of the thing they point at | 17 call sites; the default becomes `--palette-2` (most keys name a button) |
| Left rules and tinted surfaces | all three | follow the role of the text they frame | `.keyword-reveal-text`, `.otag-reveal-block`, `.otag-def`, `.card-row.expanded` |

Status colors are untouched; the guard from `palette.md` keeps it so.

Rules that fall out of the table:

- Three accents, three jobs: primary is the title hue, secondary is the selected hue, tertiary is the label hue. Nothing else draws from them except the focus ring.
- Palette slots 1 to 6 are role hues now, not just a decorative cycle: 1 links, 2 actions, 3 names, 4 rarity, 5 numbers, 6 glyphs. Tag cycles and chart series keep using them positionally, which is fine: a tag is decoration.
- The sites follow the same map where the role exists (zite's stat chips and form labels, the portfolio's chips are already positional).

### How to do it

One pass per crate, in this order, each a green deploy: zwipe-components (app-side CSS and `HintKey` default), zwiper (CSS, inline styles, hint call sites), zite, portfolio. Re-pin consumers after the crate change. Local check: zwiper's web build (`dx serve` in `zwiper/`) reaches Home, Login and Register without a session, which exercises title, label, number and link roles; the deck and card views need an account against the local database (see memory note on the local zwipe DB) or a phone build. Changelog: one entry under `UPCOMING`, phrased as what the user sees ("Numbers, names and links each have their own color in every theme").

### Known gotchas

- `HintKey` colors are passed as variable names from call sites; grep for `color: "--accent-` across zwiper to find them all (the audit lists 17).
- `.stat-chip-format` uses the selected hue for a non-selected format name today; it becomes a name (slot 3).
- The portfolio's `.tag-c0..c4` and zite's `.tag-c1..c6` are positional cycles and stay as they are.

## Phase 2: which hue fills which accent, per theme

Once roles are fixed, the remaining lever is per theme: which of its hues go into `accent-primary`, `accent-secondary` and `accent-tertiary`. This is where "Gruvbox's aqua is its least-used hue" and "Hackerman is a green explosion" get fixed, without touching a site.

Rule per theme, from the scheme's own conventions (sources in `palette.md` and this morning's research):

- `--accent-primary` (titles): the hue the scheme gives its most prominent syntax role, keywords or headings.
- `--accent-secondary` (selected): the scheme's selection, cursor or search-match hue.
- `--accent-tertiary` (labels): the hue it gives its quietest role, comments or line numbers, as long as it clears 3:1 on the background.

Known cases, from the owner 2026-10-08: Hackerman's primary becomes its blue (`#829dd4`), green moves to a role that shows less; Gruvbox's primary is up for review (aqua is its least-used hue in its own editor grammar; yellow, orange, green and blue carry most of it). Everything else is reviewed on the same renders as before: the swatch page from `zwipe-components/scripts/swatch/` and the portfolio home under each theme, dark then light, five per sheet.

Published colors still never change; only which published hue sits in which variable.

## Open decisions

- Phase 1 map: proposed 2026-10-08, awaiting the owner's read of this plan.
- Phase 2 per-theme accents: to be rated on renders after phase 1 ships.
