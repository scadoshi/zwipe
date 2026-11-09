# App Store product page header

The header App Store Connect shows above the product page: the app's grid-tiles backdrop as texture, with no text or logo, in Gruvbox dark like the store screenshots.

- `header-16x9-5244x2950.png` and `header-21x9-3840x1646.png`: the two sizes App Store Connect takes. Both are opaque RGB, because it refuses alpha.
- `header.html` draws the tiles as orangui's `grid-cells.js` does: the same seed, shades, gap and corner radius. The field is even across the image, with a soft vignette, instead of fading down a screen. Tiles are about 146px, 36 across the 16:9 header.
- `./render.sh` renders both PNGs again, with Chrome and ImageMagick. `BG` and `INK` set another theme's `--bg-primary` and `--text-primary`.
