#!/usr/bin/env python3
"""Render the alternate app icons, one per orangui theme and mode, or with
--web the browser icons of app.zwipe.net and zwipe.net.

    zcripts/ios/icon.py
    zcripts/ios/icon.py --web

Draws the Z from zwipe-core/src/domain/logo/z.txt in JetBrains Mono,
each theme's logo color on its background, the text color in dark and the
accent in light like orangui's --logo, read from
the themes.css of the orangui build Cargo.lock pins. Sized and placed to
match the gruvbox primary icon in zwiper/assets/favicon, which was made with
asciier and stays as it is (context/operations/ios/app-store/submission/
icon_update.md).

Writes zwiper/assets/app-icons/<theme>-<mode>.png. zcripts/ios/icons.sh
compiles them into the alternate icons the App icon sheet switches between.

With --web, draws the same mark at the same size, line height and place in
the web icons' colors, then scales it to each file both sites link: the
favicon PNGs and .ico on a rounded tile, the touch and manifest icons on a
full square. Writes them into zwiper-web and zite.

Needs Pillow and fonttools: pip3 install pillow fonttools brotli
"""

import json
import pathlib
import re
import subprocess
import sys
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
# The web icons' colors: gruvbox dark's background and a light gray mark.
WEB_BG = "#282828"
WEB_FG = "#ebebeb"
# A favicon's tile radius, as a share of its side.
FAVICON_RADIUS = 0.2
# The .ico's frames.
ICO_SIZES = (16, 32, 48, 64)
# Each site's directory for the favicons and touch icon, then for the
# manifest's icons, from the repo root.
WEB_SITES = (
    ("zwiper-web/assets", "zwiper-web/public/icons"),
    ("zite/assets", "zite/public/assets"),
)


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


def tile(img: Image.Image, radius: float) -> Image.Image:
    """`img` with its corners rounded off, a `radius` share of its side."""
    side = img.size[0]
    mask = Image.new("L", img.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        (0, 0, side - 1, side - 1), radius=round(side * radius), fill=255
    )
    out = img.convert("RGBA")
    out.putalpha(mask)
    return out


def scaled(img: Image.Image, side: int) -> Image.Image:
    return img.resize((side, side), Image.LANCZOS)


def web() -> None:
    full = render(mark(), ttf_path(), WEB_BG, WEB_FG).convert("RGBA")
    tiled = tile(full, FAVICON_RADIUS)
    for icons, manifest in WEB_SITES:
        icons = ROOT / icons
        manifest = ROOT / manifest
        for side in (16, 32):
            scaled(tiled, side).save(icons / f"favicon-{side}x{side}.png")
        frames = [scaled(tiled, side) for side in ICO_SIZES]
        frames[-1].save(
            icons / "favicon.ico",
            sizes=[(side, side) for side in ICO_SIZES],
            append_images=frames[:-1],
        )
        scaled(full, 180).save(icons / "icon-180.png")
        for side in (192, 512):
            scaled(full, side).save(manifest / f"icon-{side}.png")
    print("wrote the web icons for", len(WEB_SITES), "sites")


def main() -> None:
    if "--web" in sys.argv[1:]:
        web()
        return
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
