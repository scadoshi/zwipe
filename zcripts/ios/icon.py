#!/usr/bin/env python3
"""Render the alternate app icons, one per orangui theme and mode.

    zcripts/ios/icon.py

Draws the Z from zwipe-core/src/domain/logo/z.txt in JetBrains Mono,
each theme's logo color on its background, the text color in dark and the
accent in light like orangui's --logo, read from
the themes.css of the orangui build Cargo.lock pins. Sized and placed to
match the gruvbox primary icon in zwiper/assets/favicon, which was made with
asciier and stays as it is (context/operations/ios/app-store/submission/
icon_update.md).

Writes zwiper/assets/app-icons/<theme>-<mode>.png. zcripts/ios/icons.sh
compiles them into the alternate icons the App icon sheet switches between.

Needs Pillow and fonttools: pip3 install pillow fonttools brotli
"""

import json
import pathlib
import re
import subprocess
import tempfile

from fontTools.ttLib import TTFont
from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
PRIMARY = ("gruvbox", "dark")
# Font size, line height and where the ink's top-left lands, fitted to the
# primary icon's ink box, (127, 224) to (890, 810).
SIZE = 92
LINE_H = 0.84
INK_AT = (127, 224)


def mark() -> list[str]:
    return (ROOT / "zwipe-core/src/domain/logo/z.txt").read_text().rstrip("\n").split("\n")


def ttf_path() -> str:
    font = TTFont(ROOT / "zwiper/assets/fonts/jetbrains-mono-400.woff2")
    font.flavor = None
    out = pathlib.Path(tempfile.gettempdir()) / "zwipe-jbm-400.ttf"
    font.save(out)
    return str(out)


def palettes() -> dict[tuple[str, str], tuple[str, str]]:
    """Each (theme, mode)'s (background, text) from the pinned orangui themes.css."""
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1"],
            cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout
    )
    manifest = next(p["manifest_path"] for p in meta["packages"] if p["name"] == "orangui")
    css = (pathlib.Path(manifest).parent / "assets/themes.css").read_text()
    out = {}
    for name, mode, body in re.findall(
        r"^\.theme-([a-z0-9-]+)-(dark|light)\s*\{(.*?)^\}", css, re.M | re.S
    ):
        # The mark takes orangui's --logo: the text color in dark, the accent in light.
        ink = "--text-primary" if mode == "dark" else "--accent-primary"
        bg = re.search(r"--bg-primary:\s*(#[0-9a-fA-F]{6})", body)
        fg = re.search(ink + r":\s*(#[0-9a-fA-F]{6})", body)
        if bg and fg:
            out[(name, mode)] = (bg.group(1), fg.group(1))
    return out


def render(lines: list[str], ttf: str, bg: str, fg: str, canvas: int = 1024) -> Image.Image:
    font = ImageFont.truetype(ttf, SIZE)
    ink = Image.new("L", (canvas, canvas), 0)
    draw = ImageDraw.Draw(ink)
    for i, line in enumerate(lines):
        draw.text((0, SIZE + i * LINE_H * SIZE), line, font=font, fill=255, anchor="ls")
    box = ink.getbbox() or (0, 0, 0, 0)
    mask = ink.crop(box)
    img = Image.new("RGB", (canvas, canvas), bg)
    img.paste(Image.new("RGB", mask.size, fg), INK_AT, mask)
    return img


def main() -> None:
    lines = mark()
    ttf = ttf_path()
    out = ROOT / "zwiper/assets/app-icons"
    out.mkdir(exist_ok=True)
    for stale in out.glob("*.png"):
        stale.unlink()
    for (name, mode), (bg, fg) in sorted(palettes().items()):
        if (name, mode) != PRIMARY:
            render(lines, ttf, bg, fg).save(out / f"{name}-{mode}.png")
    print("wrote", len(list(out.glob("*.png"))), "alternate icons")


if __name__ == "__main__":
    main()
