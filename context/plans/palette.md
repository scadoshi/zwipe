# Palette: six decorative color slots per theme

**Status: PLANNED 2026-10-08. Nothing built yet.**

**One sentence:** every theme gets six decorative colors (`--palette-1` to `--palette-6`) so the apps stop borrowing the status colors for variety, with a test that keeps each theme's six legible and distinct without flattening the theme's own contrast.

## The problem

Each theme defines three semantic accents (`--accent-primary` for titles and links, `--accent-secondary` for selected state, `--accent-tertiary` for eyebrows) and three status colors (`--color-success`, `--color-warning`, `--color-error`). Three accents were not enough for tags, chart series and chip rows, so the status colors got used as a fourth, fifth and sixth decorative color. Measured 2026-10-08: 304 accent call sites and 126 status call sites across zwipe-components, zite, zwiper, portfolio and cairn. The sixth tag in any zite row is error red, which is why "Swiping" on the guides index looked like a warning. Gruvbox has seven hues and the app shows three of them.

## The contract

Three tiers, each with one job. No variable crosses tiers.

| Tier | Variables | Job | Decorative use |
|---|---|---|---|
| Semantic accents | `--accent-primary`, `--accent-secondary`, `--accent-tertiary` | title and link, selected state, eyebrow | no |
| Status | `--color-success`, `--color-warning`, `--color-error` (and their `--border-*`) | outcomes: saved, over budget, failed | never |
| Palette | `--palette-1` to `--palette-6` | variety with no meaning: tags, chart series, chip tints, logo stripes | only here |

Six, because it is the common chromatic count of a terminal scheme (red, yellow, green, cyan, blue, magenta) and every theme in `themes.css` can fill six without inventing hues. Numbered, not hue-named, because the same slot is pink in Rosé Pine and purple in Dracula.

Slot order is fixed across themes so a given element has the same temperature everywhere, and so neighbors in a row alternate warm and cool:

| Slot | Role | Gruvbox dark |
|---|---|---|
| 1 | cool, the theme's blue | `#83a598` |
| 2 | warm, the theme's orange or red-orange | `#fe8019` |
| 3 | the theme's green | `#b8bb26` |
| 4 | the theme's violet or pink | `#d3869b` |
| 5 | the theme's yellow | `#fabd2f` |
| 6 | the theme's cyan or aqua | `#8ec07c` |

A palette slot may hold the same value as a status color. Measured 2026-10-08: in 34 of 62 blocks the theme's only green and yellow are its success and warning colors, so excluding them would leave four slots and hide a theme's best hues from everyday use. The reservation is on the variable, not the color: `--color-success` means an outcome and ships with its context (a toast, a check, an error box), while `--palette-3` means nothing. The guard below enforces the variable. Where a theme has fewer than six hues (Nord, Vantablack, Miasma), slots repeat a hue at a different lightness.

## Contrast: floors, not a target

Earlier themes were tuned to one contrast level. That is dropped: a theme's contrast is part of its identity (Solarized is soft on purpose, Vantablack is harsh on purpose). The test enforces floors only:

1. **Legible on the surface.** Each slot against `--bg-primary` at 3:1 or better, which is the mark and chip threshold. Text never wears a palette color; labels stay in `--text-*`.
2. **Distinct from each other.** Every pair of the six at OKLab distance 8 or better under normal vision (all pairs, since tags sit side by side in any order), with 15 reported as the target. Measured 2026-10-08: a hard 15 fails 26 of 30 published palettes (Gruvbox green and aqua sit at 8), and the low-variety themes (Miasma, Hackerman, Matte Black, Osaka Jade, Ristretto, Zenburn, Vantablack) clear 8 only by spreading lightness within a hue, which is how their six are built.
3. **Distinct under color-vision deficiency, as a warning.** Adjacent pairs at OKLab distance 6 or better under simulated protanopia and deuteranopia. A warning, not a failure, because every palette use in the apps carries a label (a tag's text, a chart's legend), which is the secondary encoding the method requires.
4. **Lightness band and chroma floor are reported, not enforced.** They are the parts of the validator that would push every theme toward the same contrast.

The four accessibility themes are the exception that proves the rule. Achromatopsia has no hue, so its six are lightness steps and check 3 is skipped. Deuteranopia, protanopia and tritanopia are built for their deficiency, so check 3 runs against that deficiency and is a failure, not a warning.

## Where the values come from

Published palettes, with the source recorded in a comment above each theme block. Nothing eyeballed.

- Named schemes: Ayu, Catppuccin, Dracula, Everforest, GitHub, Gruvbox, Kanagawa, Monokai, Night Owl, Nord, One Dark, Rosé Pine, Solarized, Tokyo Night, VS Code, Zenburn. Each has a canonical ANSI or accent table.
- Omarchy themes: Ethereal, Hackerman, Matte Black, Osaka Jade, Ristretto, Synthwave, Vantablack. Their terminal palettes are in the Omarchy repository.
- docs.rs, PowerShell, Miasma, Rustbox: site or terminal colors where published; otherwise picked within the theme's existing hues and flagged for review.
- The four accessibility themes: built, not sourced, from the deficiency outward (blue and orange plus lightness steps for the red-green pair, red and cyan for tritanopia, six lightness steps for achromatopsia). The 2026-10-08 swatch derived them from their own accents and got pairs at distance 0; that is a placeholder, not a palette.

Collection is a research pass that produces one table per theme (dark and light, hex per slot, source URL) before any CSS changes. A theme whose source has no light variant says so; the light palette is then derived by the same rules and marked derived.

## The test

A Rust test in zwipe-components, `themes_palette_floors`, parses `assets/themes.css`, and for every theme block checks the four floors above. It is the gate: a theme block without six palette slots, or with a pair that collides, fails `cargo test` and so fails every deploy. OKLab conversion and the Machado-Oliveira-Fernandes 2009 CVD matrices are about eighty lines and carry no dependency. The dataviz skill's `validate_palette.js` is the cross-check while authoring (run in headless Chrome on scotland, which has no Node), not a CI dependency: zwipe is public and the script is not ours to vendor.

A swatch page, local only and never deployed, renders all sixty variants with their six slots beside their three accents and three status colors, so the owner reviews every theme in one screenshot per mode.

## The remap

Only decorative uses move. The audit is per call site, by one question: does this color mean something? If yes (a swipe direction's outcome, a saved toast, an over-budget chip), it stays on status. If it is variety (a tag's tint, a chart series, a rotating accent), it moves to a palette slot.

Known movers, from the 2026-10-08 survey:

- zite `.tag:nth-child(6n+4..6)`: the tag cycle. Becomes `3n` over slots 1 to 3, or `6n` over all six.
- zite chart tints and the About crate cards' colored chips.
- zwipe-components: chart series in `charts.rs`, the logo stripes, any `tint-{n}` rotation.
- zwiper: the celebration tint rotation picks from the three accents; it can pick from six.
- portfolio and cairn: whatever the audit finds; cairn has 37 sites.

Known stayers: swipe-direction colors (right is add, left is skip: those are outcomes), the review rating pill, every toast, the 404 title.

## Guard

Each repo's CI gets a grep that fails when a `--color-(success|warning|error)` reference appears outside an allowlist of selectors (toasts, status chips, form errors, the swipe directions, dead ends). The allowlist lives beside the grep, one line per selector, so adding a legitimate status use is a visible diff.

## Phases, each a green deploy on its own

1. **Themes and test.** Add six slots to all sixty blocks with sources, add `themes_palette_floors`, add the swatch page under `zwipe-components/scripts/`. Nothing visible changes. One commit; consumers re-pinned.
2. **zwipe-components.** Move its decorative uses to palette slots. Re-pin.
3. **zite**, then **portfolio**, then **cairn**, then **zwiper**: audit and move, one repo per commit. The zwiper change rides the next app release (changelog entry under `UPCOMING`).
4. **Guard.** The CI grep in each repo, last, once every mover has moved.

## Known gotchas

- Prerendered sites put the shared stylesheet before inline styles whatever the source order, so a consumer override of a palette slot needs higher specificity. Compare effective CSS, not HTML (see `shared_ui_kit.md`).
- zwipe CI formats with nightly rustfmt; run `cargo +nightly fmt` before each commit (`development/commit_guidelines.md`).

## Open decisions

- Six slots and the name `--palette-N`: confirmed 2026-10-08.
- Palette may equal status values: confirmed 2026-10-08 (the owner wants every theme hue visible in normal use).
- Distinctness floor 8 enforced, 15 reported: proposed 2026-10-08, pending the owner.
- Custom themes' palettes (Miasma, Rustbox, PowerShell, docs.rs): picked by the agent, reviewed by the owner on the swatch page.
