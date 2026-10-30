#!/usr/bin/env python3
"""Writes src/lib/domain/theme/palettes.rs from orangui's themes.

Reads assets/themes.css for the colors and src/theme.rs for the picker's groups, from an orangui checkout beside the zwipe checkout or the path given as the first argument. Run it again whenever orangui's themes change:

    python3 -I zwipe-tui/scripts/gen_themes.py [path/to/orangui]
"""

import pathlib
import re
import sys

CRATE = pathlib.Path(__file__).resolve().parent.parent
ORANGUI = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else CRATE.parent.parent / "orangui"
OUT = CRATE / "src" / "lib" / "domain" / "theme" / "palettes.rs"

# zwipe-tui's role, from orangui's CSS variable.
ROLES = [
    ("background", "bg-primary"),
    ("text", "text-primary"),
    ("muted", "text-muted"),
    ("border", "border-primary"),
    ("accent", "accent-primary"),
    ("success", "color-success"),
    ("error", "color-error"),
    ("warning", "color-warning"),
    ("mana_white", "mana-white"),
    ("mana_blue", "mana-blue"),
    ("mana_black", "mana-black"),
    ("mana_red", "mana-red"),
    ("mana_green", "mana-green"),
]


def hex_rgb(value):
    value = value.strip().lstrip("#")
    if len(value) == 3:
        value = "".join(c * 2 for c in value)
    if not re.fullmatch(r"[0-9a-fA-F]{6}", value):
        raise SystemExit(f"not a hex color: #{value}")
    return [int(value[i : i + 2], 16) for i in (0, 2, 4)]


def palettes(css):
    found = {}
    for slug, mode, body in re.findall(r"\.theme-([a-z0-9-]+)-(dark|light)\s*\{(.*?)\}", css, re.S):
        values = dict(re.findall(r"--([a-z0-9-]+):\s*([^;]+);", body))
        found[(slug, mode)] = {role: hex_rgb(values[var]) for role, var in ROLES}
    return found


def groups(theme_rs):
    body = theme_rs.split("pub const THEME_GROUPS", 1)[1].split("];", 1)[0]
    return [
        (label, re.findall(r'"([a-z0-9-]+)"', slugs))
        for label, slugs in re.findall(r'label:\s*"([^"]+)",\s*themes:\s*&\[(.*?)\]', body, re.S)
    ]


def colors(c):
    fields = ", ".join(f"{role}: [{', '.join(f'0x{x:02x}' for x in c[role])}]" for role, _ in ROLES)
    return f"Colors {{ {fields} }}"


def main():
    css = (ORANGUI / "assets" / "themes.css").read_text()
    found = palettes(css)
    grouped = groups((ORANGUI / "src" / "theme.rs").read_text())

    slugs = sorted({slug for slug, _ in found})
    listed = [slug for _, members in grouped for slug in members]
    if sorted(listed) != slugs:
        raise SystemExit(f"groups and palettes disagree: {sorted(set(listed) ^ set(slugs))}")
    lines = [
        "//! Orangui's theme palettes, written by `zwipe-tui/scripts/gen_themes.py` from orangui's `assets/themes.css` and `src/theme.rs`. Regenerate rather than edit.",
        "",
        "use super::{Colors, Family, Group};",
        "",
        "/// Every theme family, alphabetical by slug.",
        "pub(crate) const FAMILIES: &[Family] = &[",
    ]
    for slug in slugs:
        lines += [
            "    Family {",
            f'        slug: "{slug}",',
            f"        dark: {colors(found[(slug, 'dark')])},",
            f"        light: {colors(found[(slug, 'light')])},",
            "    },",
        ]
    lines += ["];", "", "/// The theme picker's sections, in orangui's order: warm to cool to neutral.", "pub(crate) const GROUPS: &[Group] = &["]
    for label, members in grouped:
        quoted = ", ".join(f'"{m}"' for m in members)
        lines.append(f'    Group {{ label: "{label}", slugs: &[{quoted}] }},')
    lines.append("];")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT.relative_to(CRATE)}: {len(slugs)} themes in {len(grouped)} groups")


main()
