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

All dark themes must hit these contrast ratios against `--bg-primary`. These ratios are consistent across all 14 current themes.

| Variable | Target | Acceptable Range |
|----------|--------|-----------------|
| `--text-primary` | **11.0x** | 10.6 -- 11.6x |
| `--text-muted` | **5.2x** | 5.0 -- 5.4x |
| `--text-subtle` | **7.0x** | 6.8 -- 7.2x |
| `--border-primary` | **2.3x** | 2.1 -- 2.5x |
| `--border-secondary` | **1.2x** | 1.1 -- 1.3x |
| `--border-muted` | **1.6x** | 1.4 -- 1.8x |

Accent contrast varies by theme personality (4x -- 13x range across themes). No strict target, but accents should be legible as text.

Light themes invert the relationship (dark text on light background) but follow the same ratio targets.

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

Theme CSS files in `zwiper/assets/` and `zite/assets/` are build artifacts (gitignored). Edit `shared/themes.css` only.
