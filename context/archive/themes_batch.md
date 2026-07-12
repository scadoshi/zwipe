# Batch-add coding themes (+ Vantablack / Whiteout)

**Status: SHIPPED (owner-confirmed 2026-07-12; planned 2026-07-11). Pure additive, client + shared-const only — no
server, DB, or contract changes. Theme validation lives in `ALLOWED_THEMES`;
the picker and CSS derive everything else from it.**

## Goal

Grow the theme catalog with the "coding-essential" editor palettes the list is
missing, plus a pure-black **Vantablack** and its pure-white counterpart. We have
a clean pipeline for this, so add the whole batch at once.

## The pipeline (every touch point)

Adding one theme = **one const entry + 6 CSS blocks + maybe one label special-case**.
Concretely:

1. **`zwipe-core/src/domain/user/models/preferences.rs`** — append the slug to
   `ALLOWED_THEMES` (line 11). This is the single source of truth: it's the
   validation gate *and* the picker iterates it. **Keep it alphabetical** — the
   picker preserves this order (`theme_picker.rs:45`). Add the slug to the
   `accepts_all_allowed_themes` test's coverage automatically (it loops the const,
   so no test edit needed).

2. **Three identical `themes.css` files** — `zite/assets/`, `zwipe-components/assets/`,
   `zwiper/assets/`. Today all three are **byte-identical** (653 lines, 28 blocks =
   14 themes × dark/light). They **must stay in sync**; the safest workflow is edit
   one, then copy it over the other two and diff to confirm. Each theme needs a
   `.theme-{slug}-dark` and `.theme-{slug}-light` block, each defining all **19
   CSS variables** (see the gruvbox block at `zwipe-components/assets/themes.css:53`
   as the canonical template):
   `--bg-primary --bg-overlay --text-primary --text-muted --text-subtle`
   `--border-primary --border-secondary --border-muted`
   `--color-error --color-success --color-warning`
   `--border-error --border-success --border-warning`
   `--accent-primary --accent-secondary --accent-tertiary`
   `--shadow-sm --shadow-md`
   (Note: gruvbox-dark is *also* mirrored in `:root` at line 620 as the default;
   new themes don't touch `:root`.)

3. **Display labels** — auto-derived by `display_theme_name()`, which title-cases
   slug words (`tokyo-night` → "Tokyo Night"). It's **duplicated in two files** that
   must match:
   - `zwipe-components/src/theme_picker.rs:21`
   - `zwiper/src/lib/inbound/screens/profile/preferences.rs:20`
   Slugs that don't title-case cleanly need a special-case in **both** (like the
   existing `rose-pine` → "Rosé Pine"). For this batch: `vscode` → "VS Code",
   `github` → "GitHub", `night-owl` → "Night Owl" (title-cases fine, no special
   case), `vantablack`/`whiteout` title-case fine.

4. **Colorblind grouping** (only if we add an accessibility theme) — the
   `COLORBLIND_THEMES` const is *also* duplicated in the same two files
   (`theme_picker.rs:17`, `preferences.rs:38`). A new colorblind slug goes in both
   or it lands in the wrong picker section.

## Vantablack modeling decision (RESOLVED)

Vantablack (pure black) and its pure-white twin are a natural **dark/light pair**,
and the app already has a per-theme dark/light toggle. So model as **one slug**:

- `vantablack-dark` = OLED pure black (`--bg-primary: #000000`, text near-white)
- `vantablack-light` = pure white (`--bg-primary: #ffffff`, text near-black)

One entry, toggle flips black↔white. **Rejected:** two separate slugs
("vantablack" + "whiteout") — redundant with the toggle and clutters the list.
(If the owner wants "Whiteout" to appear as its own named entry regardless, add a
second slug whose dark block *is* the white palette; flag before doing so.)

**Monochrome caveat:** true black/white leaves error/success/warning
indistinguishable. Keep those three as **minimally desaturated** hues (dim red /
green / amber) so status colors still read, everything else grayscale. This theme
also effectively covers **achromatopsia** (total color blindness) — worth noting,
but leave it in the regular group unless the owner wants it under "Color blind".

## The batch (alphabetical slugs)

Strong tier (add all):

| slug         | label       | dark bg / fg              | light bg / fg             | notes |
|--------------|-------------|---------------------------|---------------------------|-------|
| `ayu`        | Ayu         | `#0f1419` / `#bfbdb6`      | `#fcfcfc` / `#5c6166`      | signature orange accent `#ffb454`, blue `#39bae6` |
| `github`     | GitHub      | `#0d1117` / `#c9d1d9`      | `#ffffff` / `#24292f`      | accent `#58a6ff`/`#0969da`; canonical, add special-case label |
| `kanagawa`   | Kanagawa    | `#1f1f28` / `#dcd7ba`      | `#f2ecbc` / `#545464`      | wave(dark)/lotus(light); accent `#7e9cd8`/`#4d699b` |
| `night-owl`  | Night Owl   | `#011627` / `#d6deeb`      | `#fbfbfb` / `#403f53`      | accent `#82aaff`; low-light designed |
| `vantablack` | Vantablack  | `#000000` / `#f5f5f5`      | `#ffffff` / `#0a0a0a`      | monochrome + dim status hues (see above) |
| `vscode`     | VS Code     | `#1e1e1e` / `#d4d4d4`      | `#ffffff` / `#1e1e1e`      | Default Dark+/Light+; accent `#569cd6`/`#0070c1`; special-case label |
| `zenburn`    | Zenburn     | `#3f3f3f` / `#dcdccc`      | derived warm light        | dark-native; light is synthetic low-contrast warm |

Optional lower tier (add only if the owner wants the full sweep): `gruvbox-material`,
`material` (Palenight), `monokai-pro`, `cobalt2`, `oceanic-next`. Skip anything
glow/neon (e.g. Synthwave '84) per the no-glow-effects preference.

### Per-theme variable mapping

For each theme, only the anchors above are canonical; derive the remaining vars by
following the **gruvbox pattern**:
- `--text-muted` / `--text-subtle`: two steps interpolated between fg and bg.
- `--border-primary/secondary/muted`: bg lightened (dark) or darkened (light) in
  three steps; `secondary` closest to bg, `primary` most contrast.
- `--color-error/success/warning`: theme's native red / green / yellow.
- `--border-error/success/warning`: slightly darker/saturated variants of those.
- `--accent-primary/secondary/tertiary`: theme's headline accent, then a secondary
  (red/pink) and tertiary (yellow/orange), matching how gruvbox picks green/red/yellow.
- `--bg-overlay`: `rgba(0,0,0,0.6)` dark / `rgba(0,0,0,0.3)` light (constant).
- `--shadow-sm/md`: copy the existing constants (`0.3/0.5` dark, `0.1/0.15` light).

## Build order

1. Append all slugs to `ALLOWED_THEMES` (alphabetical), plus any label/colorblind
   special-cases in **both** picker files.
2. Author all new blocks in **one** `themes.css`, then propagate to the other two
   and `diff` all three to confirm they're identical again.
3. `cargo build -p zwipe-core` (const), `cargo test -p zwipe-core` (the
   `accepts_all_allowed_themes` loop now covers the new slugs for free).

## Verify

Per `context/development/commit_guidelines.md`: `cargo +nightly fmt`, clippy,
`cargo test --workspace`. Then `dx serve`, open the theme picker on both zwiper
(preferences sheet) and zite (nav dropdown), and for each new theme: confirm it
appears in the right group, the label reads correctly (esp. "VS Code" / "GitHub"),
and the dark/light toggle flips cleanly — spot-check Vantablack goes true black ↔
true white with status colors still legible. Confirm all three `themes.css` are
byte-identical (`diff`).
