# 402. zwipe.net's logos look heavier than the app's

State: fixing | Owner: laptop | PR: PENDING | Build seen: zwipe.net and app.zwipe.net live, 2026-11-12

## Report
Scotty, a screen recording flipping between zwipe.net and app.zwipe.net home at the same window size: the ASCII logos are the same size, but on zwipe.net the hero ZWIPE and the nav Z look heavier and chunkier, with solid blocks, while the app's show thinner vertical stripes.

## Cause
Headless Chrome at 1634 wide. Every computed style of the nav Z (`.nav-logo`) and the hero ZWIPE (`.hero-head .logo`) matches on both sites: family, weight, size, line height, letter spacing, transform and smoothing. Only the font Chrome actually renders differs (`CSS.getPlatformFontsForNode`):
- zwipe.net hero: Menlo for 241 glyphs, JetBrains Mono for 208. The nav Z: Menlo 50, JetBrains Mono 23.
- app hero: JetBrains Mono for all 449. The nav Z: all 73.

zite served @fontsource's Latin and Latin-ext subsets of JetBrains Mono. The block characters the logos are drawn in (U+2580-259F, ▎ and ▉) are in neither, so they fell back to the system's monospace font, which draws them heavier. The app serves the complete font files.

## Fix
laptop, PENDING: one font rule for both sites, in components.
- `components::FONTS_CSS` (`components/assets/fonts.css`): JetBrains Mono at 400, 500 and 700 from the complete files at `/assets/`.
- zwiper-web's `01-fonts.css` is its copy, kept equal by the sheets test like the other shared sheets.
- zite inlines `FONTS_CSS` first and bundles the app's complete font files unhashed at `/assets/`. Its subset faces in `style.css` and `public/fonts` are gone. A test checks that zite's files are the app's byte for byte and that the shared rule names each one.

## Checked (headless Chrome, 1634)
After the fix, the computed-style and platform-font dump is identical on both sites, the widths included: the hero ZWIPE is 517.45px wide on both (zwipe.net's was 518.89px), with JetBrains Mono for all 449 glyphs. The before and after crops, at 2x and 1x, are in the PR.

## Verification
-
