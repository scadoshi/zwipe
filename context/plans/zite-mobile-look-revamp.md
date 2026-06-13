# Plan: Make the marketing site (zite) echo the mobile client's look

**Status:** Planning only — no `zite/` changes yet. This document is the spec.

**Goal:** Restyle the `zite` marketing site so it visually simulates the revamped
mobile app (`zwiper`): a darker background carrying a faint grid, with all content
lifted off that grid into floating, shadowed "bubble" containers. Nothing readable
should sit naked on the grid — text is either inside a panel or, for short
labels/headings, rendered in an accent color.

This is a **CSS-led** change. The DOM/component structure of zite mostly stays;
most work lands in `zite/assets/style.css`, with a few small wrapper elements
added in the page components where bare text currently sits on the background.

---

## Why this is low-risk to port

zite already shares the design system with the app:

- `zite/assets/themes.css` is the same theme file as the app (same variables:
  `--bg-primary`, `--text-primary`, `--text-muted`, `--text-subtle`,
  `--border-primary`, `--border-secondary`, `--border-muted`, `--accent-primary`,
  `--accent-secondary`, `--accent-tertiary`, `--color-error/success/warning`,
  `--shadow-sm`, `--shadow-md`). zite has a working theme switcher, so whatever we
  build must read from these vars (never hardcode colors).
- zite already loads JetBrains Mono and uses `--accent-primary` for links.
- The app's new look is built entirely from these same tokens plus two derived
  ones (below), so the visual language transfers 1:1.

---

## Design tokens to introduce in zite (copy from the app)

Add these derived tokens once, high in the cascade (on `:root`/`html`, or
wherever the theme class is applied — see "Where the theme class lives" below) so
every theme gets them for free:

```css
--grid-line: color-mix(in srgb, var(--text-primary) 4%, transparent);
--grid-size: 28px;
--bg-sink:  color-mix(in srgb, var(--bg-primary), #000 15%);
```

- `--grid-line` — the faint grid stroke, derived from the theme's text color so it
  auto-matches every palette (light themes included). 4% is the app's value; the
  app tuned `--bg-sink` to 15% black. Keep these identical to the app for
  consistency; tune later if the larger desktop canvas reads differently.
- The grid itself is two stacked 1px linear-gradients tiled at `--grid-size`:

```css
background-image:
    linear-gradient(var(--grid-line) 1px, transparent 1px),
    linear-gradient(90deg, var(--grid-line) 1px, transparent 1px);
background-size: var(--grid-size) var(--grid-size);
```

### Where the theme class lives (verify before building)

In the app the theme class sits on a root wrapper (`ThemeWrapper`), and the derived
tokens are defined on that wrapper so both screens and modals inherit them. In zite,
confirm where the `theme-*` class is applied (check `zite/src/main.rs` /
`theme switcher` section of `style.css`) and define `--grid-line/--grid-size/--bg-sink`
on that same element. If the class is on `<body>`/`<html>`, define them there.
**Pitfall we already hit in the app:** defining these on an inner element means
overlays/sections outside it can't resolve `var(--bg-sink)` and silently fall back
to nothing. Define them at the theme root.

---

## The two rules that define the look

1. **Layered background.** The page background becomes the darker `--bg-sink` with
   the grid painted on it. Chrome (nav, footer) and content panels paint their own
   solid `--bg-primary` (or `--border-secondary` for nav) so the grid only shows in
   the gutters/gaps — exactly like the app, where `.screen-content` is `--bg-sink`+grid
   and the header/util-bar stay `--bg-primary`.

2. **Nothing naked on the grid.** Any text/element currently sitting directly on the
   page background must either:
   - be wrapped in a **floating bubble**: `background-color: var(--bg-primary)`,
     `border: 1px solid var(--border-secondary)`, `border-radius` (~1rem),
     `box-shadow: var(--shadow-md)` (hero/large) or `var(--shadow-sm)` (small); **or**
   - if it's a short label/heading/inline highlight, be given an **accent color**
     (`--accent-primary` for primary, `--accent-tertiary` for section labels,
     `--accent-secondary` for sheet/section titles — matching how the app colors
     `.card-info`, `.pref-section-label`, sheet titles).

   Long-form prose (about, privacy) → always a bubble, never accent-colored
   paragraphs (accent text is for short strings only; body copy must stay
   `--text-primary` for readability).

### Border convention (from the app)

- **Inputs / buttons** → `border-primary` (border-1).
- **Panels / cards / displays / dialogs / containers** → `border-secondary` (border-2).
- Full-width dividers inside panels → 1px `border-secondary` rules (the app's
  `.dialog-rule` pattern: break out of padding with negative margins to span edge-to-edge).

---

## Section-by-section work (maps to `style.css` sections + `home.rs` classes)

### base / layout
- `html, body`: change `background-color: var(--bg-primary)` → `var(--bg-sink)` and
  add the grid `background-image`/`background-size`. (Decide: grid on `body` for the
  whole canvas, vs. on `.page` only. Recommend `body` so the full viewport including
  side gutters shows grid, with nav/footer solid on top.)
- Define the derived tokens at the theme root here.

### nav (`.nav-wrapper`, `nav`, `.nav-panel*`)
- Already `background-color: var(--border-secondary)` and sticky — that's good chrome
  (solid, masks grid). Keep solid. Consider adding `box-shadow: var(--shadow-sm)` /
  keeping the bottom border so it reads as a raised bar over the grid.
- Nav links/pills: leave border-1 (they're buttons/links).

### hero (`.hero`, `.logo`, `.tagline`)
- The ASCII `.logo` can stay transparent (it's art, like the app's logo on
  `.screen-content`) — it reads fine as glyphs over the grid.
- `.tagline` currently sits on the background → wrap in a hero bubble: `--bg-primary`
  + `border-secondary` + `--shadow-md` + rounded, centered, max-width. This becomes
  the visual anchor of the page, mirroring the app's `.home-flavor` floating box.

### sections / cards + features (`.features-grid`, `.feature-card`, `.card-title`,
`.card-category`, `.card-summary`, `.card-bullets`)
- `.feature-card`: ensure opaque `--bg-primary` fill + `border-secondary` +
  `border-radius` + `box-shadow: var(--shadow-sm)` (likely already card-ish — audit
  and align to the panel convention). These are the canonical "bubbles."
- `.card-category` → accent label (`--accent-tertiary`, small, letter-spaced) like
  the app's `.pref-section-label` / `.chip-row-label`.
- `.card-title` → `--accent-primary` (matches app `.label` / card name accents).
- `.card-summary` / `.card-bullets` → `--text-primary`/`--text-muted` inside the card
  (already on a panel, so no accent needed).

### gallery (`.project-gallery`, `.gallery-frame`, `.gallery-video`,
`.gallery-nav`, `.gallery-caption`, `.gallery-counter`, `.gallery-meta`)
- `.gallery-frame` → bubble: `--bg-primary` + `border-secondary` + `--shadow-md` +
  rounded; the video sits inside with matching corner rounding (`overflow: hidden`).
- `.gallery-caption` / `.gallery-meta` / `.gallery-counter` → either inside the frame
  bubble, or a small pill (`--bg-primary` + `border-secondary` + `--shadow-sm`) so
  captions never float on bare grid.
- `.gallery-nav` prev/next buttons → button styling (border-1) with `--bg-primary`
  fill + shadow so they're tappable bubbles over the grid (mirror app `.util-btn`).

### about / contribute / privacy (long-form pages)
- Wrap prose blocks in content bubbles (`--bg-primary` + `border-secondary` +
  `--shadow-sm` + generous padding, `border-radius`). One bubble per logical section
  rather than one giant card, so the grid breathes between them.
- Headings → accent (`--accent-primary`); body stays `--text-primary`.
- Likely needs small wrapper `div`s added in `about.rs` / `privacy.rs` /
  `contribute.rs` (the only component-level edits beyond CSS).

### forms (verify / reset) (`forms (verify / reset)` section)
- Inputs → border-1, opaque `--bg-primary` fill, `box-shadow: var(--shadow-sm)`
  (match app `.input`).
- Wrap each form in a centered bubble panel (border-2 + shadow), like the app's
  auth screens' content sitting in `container-sm` over the grid.
- Buttons → `--bg-primary` fill + border-1 + shadow (match app `.util-btn`/`.btn`).

### footer
- Keep solid (`--bg-primary` or `--border-secondary`) so it's chrome, not grid.
  Optionally a top border + `--shadow-md` (upward) to read as a raised bar.

### theme switcher
- Its control should be a bubble/pill (border-2 + shadow). If we want to go further,
  port the app's swatch-dots idea (each theme previews its colors via
  `theme-{name}` class on a swatch strip reading `var(--accent-*)`), but that's a
  stretch goal — out of scope for v1 of this revamp.

---

## Accessibility & quality checks

- **Contrast:** verify text-on-bubble and accent-on-bubble meet WCAG AA across the
  light themes too (light bg + `--bg-sink` darkening + grid can muddy contrast). The
  app uses 4% grid / 15% sink; if light themes look dirty on the bigger canvas, gate
  a lower grid alpha for light themes.
- **Reduced motion / performance:** the grid is static CSS gradients — negligible
  cost. Don't use `background-attachment: fixed` (janky, and unnecessary here since
  the page scrolls as one document — one continuous grid avoids the seam problem we
  hit on the app's import screen).
- **Single grid surface:** paint the grid on exactly one element (body). Don't also
  paint it on `.page` or sections, or adjacent grids will double-line at seams (the
  bug we fixed by merging the app's import controls into one grid container).
- **Theme parity:** test every theme + light/dark via the switcher; all tokens are
  derived, so this should "just work," but verify `--bg-sink`/`--grid-line` resolve
  on every page (including overlay-ish elements).

---

## Suggested phasing

1. **Foundation:** add derived tokens at the theme root; set `body` to `--bg-sink` +
   grid; confirm grid renders under all themes. (Pure CSS.)
2. **Chrome:** nav + footer stay solid, add raised shadows. (Pure CSS.)
3. **Home bubbles:** hero tagline bubble, feature cards, gallery frame + caption
   pills + nav buttons. (CSS + minor wrappers if needed.)
4. **Long-form pages:** about / privacy / contribute prose bubbles. (CSS + small
   wrapper divs.)
5. **Forms:** verify / reset inputs + form panels. (CSS.)
6. **Polish pass:** accent-color audit (no naked text on grid), border-1 vs border-2
   audit, contrast check across themes, responsive check at mobile widths.

---

## Open questions for the user (decide before building)

- **Grid on `body` vs `.page`?** Recommend `body` (full-canvas grid, gutters
  included) with solid nav/footer/panels on top.
- **Nav tone:** keep current `--border-secondary` nav, or move to `--bg-primary` to
  match the app's chrome exactly?
- **How "bubbled" should long-form pages be** — one card per section (recommended)
  vs. a single page-wide card vs. lighter treatment (just accent headings, prose
  stays on grid)? The brief says "floating bubble with drop shadow on pretty much
  everything," so per-section cards is the default.
- **Match the app's exact values (4% grid / 15% sink)** or tune for the larger
  desktop canvas? Start matched, adjust in the polish pass.

---

## Files this will touch (when implemented)

- `zite/assets/style.css` — the bulk (tokens, body grid, every section above).
- `zite/assets/themes.css` — only if we decide light themes need a different grid
  alpha (otherwise untouched; derived tokens can live in `style.css`).
- `zite/src/pages/home.rs`, `about.rs`, `privacy.rs`, `contribute.rs`,
  `verify.rs`, `reset.rs` — small wrapper `div`s where bare text needs a bubble.
- No backend / `zwipe-core` / `zerver` impact — marketing site only.
