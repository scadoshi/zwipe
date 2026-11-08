# 302. Add's card details panel overlaps the swipe card at 900 to 1060px

State: reported | Owner: - | PR: - | Build seen: 1d7053e88

## Report
Surfer, live web 1d7053e88, 2026-11-08, found in the 298 width sweep (same-origin iframe in the app tab). On Add, with card details open, the details panel on the right overlaps the swipe card from about 1060 CSS px down to 900 (1440 at 150% = 960 is in the range). It covers the right edge of the card art and rules text. Below 890 the panel hides and the card centers; at 1100 and up there's a gap.

## Repro
1. Open Add on a deck with card details showing (the eye at the bottom right).
2. Make the viewport 960 CSS px wide (e.g. a 1440px window at 150% zoom).
3. The details panel (left edge 585) sits over the card (right edge 641).

Measured overlap (card right − panel left): 1060: 6px, 1040: 16px, 1024: 24px, 970: 51px, 960: 56px, 930: 71px, 900: 86px; none at 1100 (gap 14px) or wider. From 900 to 960 the panel's title is also hidden under the three-tier header (298).

Shot on halo: ~/Developer/surfer-shots/302-add-960-details-overlap.png.
