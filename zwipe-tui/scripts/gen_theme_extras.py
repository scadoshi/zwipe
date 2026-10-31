#!/usr/bin/env python3
"""Writes src/lib/domain/theme/extras.rs from orangui's themes.

The colors the chrome needs beyond gen_themes.py's roles: the six palette colors (theme swatches, hint keys), the second and third accents, the muted border (skeletons) and the three status borders (toasts). Reads assets/themes.css from an orangui checkout beside the zwipe checkout or the path given as the first argument:

    python3 -I zwipe-tui/scripts/gen_theme_extras.py [path/to/orangui]
"""

import pathlib
import re
import sys

CRATE = pathlib.Path(__file__).resolve().parent.parent
ORANGUI = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else CRATE.parent.parent / "orangui"
OUT = CRATE / "src" / "lib" / "domain" / "theme" / "extras.rs"

# zwipe-tui's role, from orangui's CSS variable.
ROLES = [
    ("accent_secondary", "accent-secondary"),
    ("accent_tertiary", "accent-tertiary"),
    ("border_muted", "border-muted"),
    ("border_success", "border-success"),
    ("border_error", "border-error"),
    ("border_warning", "border-warning"),
]
SWATCHES = [f"palette-{i}" for i in range(1, 7)]


def hex_rgb(value):
    value = value.strip().lstrip("#")
    if len(value) == 3:
        value = "".join(c * 2 for c in value)
    if not re.fullmatch(r"[0-9a-fA-F]{6}", value):
        raise SystemExit(f"not a hex color: #{value}")
    return [int(value[i : i + 2], 16) for i in (0, 2, 4)]


def rgb(c):
    return "[" + ", ".join(f"0x{x:02x}" for x in c) + "]"


def extras(values):
    fields = [f"{role}: {rgb(hex_rgb(values[var]))}" for role, var in ROLES]
    swatches = ", ".join(rgb(hex_rgb(values[var])) for var in SWATCHES)
    fields.append(f"swatches: [{swatches}]")
    return "Extras { " + ", ".join(fields) + " }"


def main():
    css = (ORANGUI / "assets" / "themes.css").read_text()
    found = {}
    for slug, mode, body in re.findall(r"\.theme-([a-z0-9-]+)-(dark|light)\s*\{(.*?)\}", css, re.S):
        found[(slug, mode)] = dict(re.findall(r"--([a-z0-9-]+):\s*([^;]+);", body))
    slugs = sorted({slug for slug, _ in found})
    lines = [
        "//! The chrome's extra colors for each orangui theme, written by `zwipe-tui/scripts/gen_theme_extras.py` from orangui's `assets/themes.css`. Regenerate rather than edit.",
        "",
        "use super::Extras;",
        "",
        "/// Each theme's extra colors, dark then light, alphabetical by slug.",
        "pub(crate) const EXTRAS: &[(&str, Extras, Extras)] = &[",
    ]
    for slug in slugs:
        lines += [
            f'    ("{slug}",',
            f"        {extras(found[(slug, 'dark')])},",
            f"        {extras(found[(slug, 'light')])}),",
        ]
    lines.append("];")
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT.relative_to(CRATE)}: {len(slugs)} themes")


main()
