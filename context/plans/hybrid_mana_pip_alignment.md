# Hybrid mana pip glyph misalignment — investigation & fix plan

**Status: PLANNED (2026-07-12).** The glyph inside a **hybrid/twobrid** mana pip sits off-center
in its squircle; single-color pips and generic numerals are fine. Owner-reported across both
zite and the app (todo: Bugs section). This doc is the investigation + how to resolve.

## Reproduction

- Inline oracle-text costs and the mana-cost pip row in the expanded card detail.
- Clear example: **Reaper King** `{2/W}{2/U}{2/B}{2/R}{2/G}` (twobrid). Also plain hybrid
  (`{W/U}` → `ms-wu`) and Phyrexian (`{W/P}` → `ms-wp`).
- **Both zite and the app**, because both share the rendering (below).

## How pips render (grounded)

- **Base font:** mana-font loaded via CDN in *both* clients —
  `@import url("https://cdn.jsdelivr.net/npm/mana-font@latest/css/mana.css")`
  (`zwiper/assets/main.css:1`, `zite/assets/style.css:3`). It defines the glyph codepoints and
  a default `.ms-cost` circle. **No `.ms-hybrid`/`.ms-split` marker class exists** (checked).
- **Emission:** `zwipe-components/src/oracle_text.rs` `sym_to_class` lowercases and **drops
  slashes** (`W/U` → `wu`, `2/W` → `2w`), emitting `i.ms.ms-{code}.ms-cost.ms-shadow`
  (oracle text at `oracle_text.rs:63`; mana cost via `card_row.rs`). So hybrids arrive as
  `ms-wu` / `ms-2w` / `ms-wp` — no distinguishing hook.
- **Our overrides** (`zwipe-components/assets/components.css`): `i.ms.ms-cost` is a squircle
  (`display:inline-flex; align-items/justify-content:center; line-height:1; padding:0 1px;`
  border + box-shadow). Plus two special-cases: single-color border/color for `ms-w/u/b/r/g`
  only, and a `::before { font-size:0.82em }` **numeral shrink** for `ms-0..ms-20/x/y/z` only.

## Root cause

The squircle overrides + the numeral shrink were calibrated for **single-color glyphs and
generic numerals**. Hybrid glyphs are a different glyph family in the mana font with their own
em-box geometry/baseline, and they match **none** of our special-cases:
- Not in the numeral-shrink list (`ms-2w` ≠ `ms-2`), so the denser split glyph renders at full
  `1em` and crowds/offsets inside the squircle.
- Not in the single-color list, so they also fall back to base grey (`#393835`) instead of
  two-tone — a *secondary* cosmetic miss.

The visible symptom is a vertical/positional offset of the `::before` glyph within the flex-
centered squircle.

## Resolution

**Step 1 — measure.** In dev tools, compare the `::before` box of a hybrid pip (`ms-2w`) to a
single-color one (`ms-w`): baseline, glyph height, and offset within the `1.5em` squircle.
That pins whether the fix is vertical-align, line-height, a `translateY`, and/or the shrink.

**Step 2 — give hybrids a target.** Two options:
- **(preferred) Emit-time marker.** `sym_to_class` already knows a symbol contained a `/`
  (hybrid). Have it also emit `ms-cost-hybrid` (from both emit sites: `oracle_text.rs:63` and
  the `card_row.rs` mana-cost render). Then CSS corrects **one class** instead of enumerating.
  Lives in the shared component → app + zite both fixed. ⚠ These are files the otags agent has
  been editing — coordinate before touching.
- **(no-Rust) enumerated selector.** Target the hybrid classes directly in CSS: two-color
  `ms-wu, ms-wb, ms-ub, ms-ur, ms-br, ms-bg, ms-rg, ms-rw, ms-gw, ms-gu`; twobrid
  `ms-2w..ms-2g`; Phyrexian `ms-wp..ms-gp` (+ hybrid-Phyrexian). Verbose but pure CSS, no
  component change.

**Step 3 — apply the correction** on the hybrid target: the measured `vertical-align` /
`line-height` fix (and the `0.82em` shrink if the split glyph is oversized like the numerals).
All in `components.css`, so both clients inherit it.

**Step 4 (optional, cosmetic) — two-tone color.** Our fixed-hex single-color border/color
scheme can't express two colors in one pip. Either leave hybrids to mana-font's native two-tone
fill (don't override their color), or accept the grey. Decide separately from the alignment fix.

## Verification

- Dev-tools measurement before/after.
- Visual: Reaper King `{2/W}..{2/G}`, a `{W/U}` card, a `{W/P}` Phyrexian card.
- **Both zite and the app**, **light + dark** themes (border/color hexes are theme-fixed).

## Notes

- Single-color pips and numerals must stay unchanged — scope every rule to the hybrid target.
- Fix is shared-CSS (and optionally one shared-component emit tweak), so it lands in both
  clients in one pass. No server/wire involvement.
