# Hybrid mana pip glyph misalignment — investigation & fix plan

**Status: SHIPPED 2026-07-12 (`bb4bef05`).** Fixed via Path A: for the ~20 hybrid classes,
restore mana-font's box geometry (`display: inline-block; 1.3em; line-height: 1.35em`) plus
`box-sizing: content-box` so the 1px squircle border sits *outside* the padding box the
half-offsets target — matching mana-font's native border-less box. Kept our border/radius/
shadow. Verified on Reaper King (`{2/W}..{2/G}`) in both zwiper and zite. The `::before`
font-size approach was a confirmed no-op (below). Minor cosmetic note deferred: the twobrid
numeral sits close to the pip edge, judged too small to matter. The record below is kept for
context.

The glyph inside a **hybrid/twobrid** mana pip sat off-center in its squircle; single-color
pips and generic numerals were fine. Owner-reported across both zite and the app.

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

## Root cause (CONFIRMED via the mana-font source, 2026-07-12)

Hybrid/twobrid symbols are **not a single `::before` glyph** — mana-font renders them as **two
absolutely-positioned pseudo-elements** over a gradient split-circle background:

```css
.ms-cost.ms-2w::before, .ms-cost.ms-2w::after { font-size: 0.55em !important; position: absolute; }
.ms-cost.ms-2w::before { top: -0.38em; left: 0.28em; }   /* top-left half */
.ms-cost.ms-2w::after  { top:  0.5em;  left: 1em;   }   /* bottom-right half */
.ms-cost.ms-2w { background: linear-gradient(135deg, var(--ms-split-top) 50%, var(--ms-split-bottom) 50%); }
```

Those `top`/`left` offsets are hardcoded against **mana-font's native `.ms-cost` box**:
`width/height: 1.3em; line-height: 1.35em; font-size: 0.95em; display: inline-block`.

**Our `i.ms.ms-cost` override changes that box** — `width/height: 1.5em`, `line-height: 1`,
`display: inline-flex`, `padding`, squircle radius. The absolute half-offsets were calibrated
for the 1.3em/1.35em inline-block box, so in our larger flex box they land in the wrong place →
the visible misalignment. (Flex centering is irrelevant here — the halves are `position:
absolute`, so they ignore `align-items`/`justify-content` and depend only on the box geometry +
their `top`/`left`.)

**Why the first attempt did nothing:** it set `::before { font-size: 0.72em }` — beaten by
mana-font's `font-size: 0.55em !important` — plus a 0.5px `translateY`, and it never touched
`::after` or the `top`/`left` offsets that actually position the halves. So zero visible change
across rebuilds.

Secondary: the two-tone color comes from `--ms-split-top`/`--ms-split-bottom` (gradient
background) + the glyph `color`; our fixed `color: #393835` tints both glyphs grey.

## Resolution — the fix targets absolute positioning, NOT glyph size

The lever is the two halves' `top`/`left` (absolute), scaled to whatever box we give the pip.
`font-size` needs `!important` to move at all (mana-font pins it). Two paths:

**Path A (recommended) — normalize the hybrid box so mana-font's native offsets land.** For the
hybrid classes, override *our* `i.ms.ms-cost` back toward what mana-font's offsets assume:
`display: inline-block` (drop the flex), and the `line-height`/dimensions its `top`/`left` were
tuned for — while keeping our border + border-radius for the squircle look. Fewest magic
numbers; the halves fall where mana-font intends.

**Path B — re-place the halves for our box.** Keep our 1.5em flex squircle and override the
hybrid `::before`/`::after` `top`/`left` (and `font-size … !important`) to re-center the two
halves in it. More hand-tuned offsets; enumerate the ~20 hybrid classes (or add an
`ms-cost-hybrid` emit-time marker — but that touches `oracle_text.rs`/`card_row.rs`, the otags
agent's files, so prefer the enumerated CSS for now).

**This must be tuned live in dev tools.** It's absolute positioning against a resized box —
blind/headless iteration won't converge. Whoever executes needs to inspect a real `ms-2w` pip
(Reaper King), read the computed `::before`/`::after` box, and dial `top`/`left` until both
halves sit in the squircle. Start by proving the rule is live (set an obviously-large offset,
confirm a half jumps), then dial in.

**Two-tone color (separate, optional):** the split colors come from `--ms-split-top`/
`--ms-split-bottom` (gradient bg) + glyph `color`; our `color: #393835` greys both glyphs. To
restore two-tone, stop overriding `color` on hybrids and let mana-font's vars drive it. Decide
independently of alignment.

## Verification

- Inspect the real pip's `::before`/`::after` computed box in dev tools before/after.
- Visual: Reaper King `{2/W}..{2/G}` (twobrid), a `{W/U}` card (hybrid), a `{W/P}` card
  (Phyrexian).
- **Both zite and the app**, **light + dark** themes (our border/color hexes are theme-fixed).
- Remember: `COMPONENTS_CSS` is `include_str!` — restart `dx serve` / rebuild, not hot-reload.

## Notes

- Single-color pips and numerals must stay unchanged — scope every rule to the hybrid target.
- Fix is shared-CSS (and optionally one shared-component emit tweak), so it lands in both
  clients in one pass. No server/wire involvement.
