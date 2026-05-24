# Theme Guidelines

Rules for the CSS theme system. Follow these when adding or modifying themes.

---

## Variable System

Every theme defines exactly 18 CSS variables:

| Variable | Purpose |
|----------|---------|
| `--bg-primary` | Main background |
| `--bg-overlay` | Semi-transparent overlay (modals, dialogs) |
| `--text-primary` | Main text |
| `--text-muted` | Secondary/dimmed text |
| `--text-subtle` | Tertiary text (between primary and muted) |
| `--border-primary` | Standard borders (buttons, inputs, chips) |
| `--border-secondary` | Darkest borders (dividers, deep separators) |
| `--border-muted` | Subtle borders (between primary and secondary) |
| `--color-error` | Error/destructive status |
| `--color-success` | Positive/success status |
| `--color-warning` | Warning/caution status |
| `--border-error` | Error border |
| `--border-success` | Success border |
| `--border-warning` | Warning border |
| `--accent-primary` | Primary accent (interactive/selected states) |
| `--accent-secondary` | Secondary accent (contrasting personality color) |
| `--accent-tertiary` | Tertiary accent |
| `--shadow-sm` | Small elevation shadow |
| `--shadow-md` | Medium elevation shadow |

---

## Contrast Ratio Targets

All dark themes must hit these contrast ratios against `--bg-primary`. Ranges are wide enough to accept each theme's canonical foreground while still satisfying WCAG AAA (7.0x) for primary text and AA (4.5x) for muted/subtle. Aspirational target shown for reference.

| Variable | Aspirational | Acceptable Range | Floor |
|----------|--------------|-----------------|-------|
| `--text-primary` | **11.0x** | 7.0 -- 14.0x | WCAG AAA |
| `--text-muted` | **5.2x** | 4.5 -- 5.4x | WCAG AA |
| `--text-subtle` | **7.0x** | 6.0 -- 7.5x | between AA and AAA |
| `--border-primary` | **2.3x** | 2.1 -- 2.5x | aesthetic |
| `--border-secondary` | **1.2x** | 1.1 -- 1.35x | aesthetic |
| `--border-muted` | **1.6x** | 1.4 -- 1.8x | aesthetic |

Accent contrast varies by theme personality (4x -- 13x range across themes). No strict target, but accents should be legible as text.

Light themes invert the relationship (dark text on light background) but follow the same ratio targets. Prefer each theme's canonical foreground when it lands in band — do not brighten or darken beyond canonical just to hit the aspirational midpoint.

### Shadow conventions

- Dark themes: `rgba(0, 0, 0, 0.3)` for sm, `rgba(0, 0, 0, 0.5)` for md
- Light themes: `rgba(0, 0, 0, 0.1)` for sm, `rgba(0, 0, 0, 0.15)` for md

---

## How to Add a New Theme

1. **Pick your base palette** -- start with the source theme's canonical colors (e.g., Catppuccin Mocha bg `#1e1e2e`, text `#cdd6f4`).

2. **Derive text/border values at target ratios.** Use the contrast formula to find colors that match the theme's hue/tone while hitting the targets above. Scale brightness using HSV while preserving hue and saturation.

3. **Choose 3 accents** that express the theme's personality. These have the most creative freedom.

4. **Add both dark and light variants** as `.theme-{name}-dark` and `.theme-{name}-light` classes in `shared/themes.css`.

5. **Register in `ALLOWED_THEMES`** in `zwipe-core/src/domain/user/models/preferences.rs`.

6. **Verify contrast** by running the calculation script (see below).

### Contrast verification script

```python
def luminance(hex_color):
    hex_color = hex_color.lstrip('#')
    r, g, b = [int(hex_color[i:i+2], 16)/255.0 for i in (0, 2, 4)]
    def linearize(c):
        return c/12.92 if c <= 0.04045 else ((c+0.055)/1.055)**2.4
    return 0.2126*linearize(r) + 0.7152*linearize(g) + 0.0722*linearize(b)

def contrast(c1, c2):
    l1, l2 = luminance(c1), luminance(c2)
    if l1 < l2: l1, l2 = l2, l1
    return (l1 + 0.05) / (l2 + 0.05)

bg = '#YOUR_BG'
pairs = {
    'text-primary': '#YOUR_TP',
    'text-muted': '#YOUR_TM',
    'text-subtle': '#YOUR_TS',
    'border-primary': '#YOUR_BP',
    'border-secondary': '#YOUR_BS',
    'border-muted': '#YOUR_BM',
}
for name, color in pairs.items():
    print(f'{name}: {contrast(bg, color):.2f}x')
```

---

## File Locations

| File | Role |
|------|------|
| `shared/themes.css` | Single source of truth for all theme CSS variables |
| `zwipe-core/src/domain/user/models/theme.rs` | `ThemeConfig` (default theme + css class) |
| `zwipe-core/src/domain/user/models/preferences.rs` | `ALLOWED_THEMES` registry + `UserPreferences` default |
| `zwiper/build.rs`, `zite/build.rs` | Copy `shared/themes.css` into asset directories at build time |
| `zite/src/main.rs`, `zwiper/src/lib/inbound/screens/profile/preferences.rs` | `display_theme_name(slug)` helper — capitalizes hyphen-split words; special-cases (e.g. `rose-pine` → "Rosé Pine") go here, not in the slug |

Theme CSS files in `zwiper/assets/` and `zite/assets/` are build artifacts (gitignored). Edit `shared/themes.css` only.

For text/casing/font rules across the app see `context/dev/ui_text_conventions.md`.
