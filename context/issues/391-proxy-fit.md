# 391. The no-image card clips its facts row in the app's Featured card, and thumbnails break names mid-word

State: fixing | Owner: laptop | PR: #594 | Build seen: #589, app (Android), images blocked

## Report
Droider, via the composer, 2026-11-11, on #589 with images blocked:
- The commander card at the top of the app's deck cards screen clips its last row: the P/T chip is cut off under the printing and price row.
- Name-only thumbnails break names mid-word (Thembercha/ud).

## Cause
- The app's Featured card is about 150px wide on a phone, so the card's base type is about 7.5px. Android WebView's minimum font size (8px) holds every line at 8px, so stepping `--proxy-scale` down shrank nothing, and the fit stopped at its floor with the content still overflowing.
- The thumbnail name used `word-break: break-word`, which splits a word that doesn't fit, before the type has a chance to shrink.

## Repro
Render `cargo test -p components --test card_proxy_render -- --ignored` in Chrome with `webkit.webprefs.minimum_font_size` 8:
- the long-text card at 150px loses its facts row;
- the long-named card's thumbnails read "Thember/cha" and "Overwatche/r".

## Fix
laptop, #594, web and app (one component):
- When the type can't shrink the card into its box, `FIT_JS` drops rows one at a time and fits again: first the rules text, then the Keywords and Card roles rows, then the kind chips. The name and the facts row always show.
- A thumbnail's name wraps only between words, and the fit shrinks it until each word and line fits. As a last resort, the widest words are cut to an ellipsis.
- The full card's name breaks a word only when the word alone is wider than the line.
- The renders gain a 150px phone Featured cell, a 64px app thumbnail cell and a long-named card. Before and after under the 8px floor: `target/391-renders/` on the laptop.

## Verification
-
