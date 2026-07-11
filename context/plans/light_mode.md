# Light-mode polish pass

**Status: IN PROGRESS (2026-07-11).** Dark mode is the clean baseline; light
mode has accumulated rough edges (contrast, muddy fills, effects tuned for dark
backgrounds). This is the running list for a cross-cutting light-mode cleanup,
opened alongside the deck-cards-screen revamp (squircle pips, price/stat tags,
flowing rows) that surfaced several of these.

## Guiding principle: never regress dark mode

Dark mode looks finished, so every fix here must leave it byte-identical. Two
levers make that safe:

- Theme classes ride on the `.app-shell` element as `theme-{name}-light` /
  `theme-{name}-dark` (see `zwiper/src/bin/zwipe.rs` `ThemeWrapper`). So a
  light-only override is just `.app-shell[class$="-light"] { … }` — dark classes
  end in `-dark` and never match.
- Prefer theme variables (`themes.css`) over literals. Muddiness usually comes
  from mixing an off-palette literal (e.g. pure `#000`) into a light base;
  mixing the theme's own `--text-primary` / `--bg-primary` instead stays
  harmonious.

## Items

### 1. Grid-panel "sink" darkening — DONE (brighter)
The scroll area (`.screen-content`) and one other panel fill with
`--bg-sink`, defined in `.app-shell` as `color-mix(bg-primary, #000 15%)`. On a
cream palette that 15% black reads as a dirty grey-cream panel. Fixed for light
themes only by lightening the sink toward white so the content sits a touch
brighter than the chrome (fresh paper feel) instead of dirtier:
```css
.app-shell[class$="-light"] { --bg-sink: color-mix(in srgb, var(--bg-primary), #fff 35%); }
```
`main.css` ~L81. Dark mode's recessed sink is untouched. (Flat — sink =
`bg-primary` — was tried first but felt depthless.)

### 2. Mana-pip drop shadow — DONE (both modes)
Two shadows were in play: our custom offset `box-shadow` (removed) and, the real
offender, mana-font's own `.ms-cost.ms-shadow` — a hard *double* solid `#111`
offset (`-0.06em 0.07em 0 #111, 0 0.06em 0 #111`, zero blur) that read as a
chunky sticker outline, worst in light mode. Overridden on the global
`i.ms.ms-cost` rule (higher specificity + later load) with a plain soft drop
shadow `box-shadow: 0 1px 2px rgba(0,0,0,0.25)` that follows the squircle.
Not light-specific but surfaced during this pass.

## Backlog (unverified, add as spotted)

- Tag tint contrast in light mode: the `color-mix(… 12%, transparent)` fills and
  `35%` borders (price/stat tags, deck-list stat-chips) were eyeballed on dark —
  re-check legibility on each light palette.
- Price-green (`--color-success`) legibility on light cream backgrounds.
- Audit any other `rgba(0,0,0,…)` / `#000` / `#fff` literals in `main.css` that
  bake a fixed direction instead of a theme var (these are the usual
  light-mode offenders).
- zite (`zite/assets/style.css`) has its own grid/background stack — sweep it
  separately once the app light-mode is dialed in.

## Ship

Client-only CSS. No server, no migration, no min-version gate. Verify by
toggling each light theme in Profile → Preferences on a `dx serve` pass. Rides
the next store build.
