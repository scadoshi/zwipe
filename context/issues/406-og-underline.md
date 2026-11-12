# 406. The link cards underline Magic: The Gathering

State: fixing | Owner: thinker | PR: - | Build seen: production

## Report
Composer, 2026-11-12: app.zwipe.net's Open Graph image (`og-app.png`) underlines "Magic: The Gathering", against the no-underlined-links rule. Redraw it without the underline, accent kept, same layout, 1200x630, and give the app's head `twitter:image` and `twitter:image:alt` as zwipe.net has. Check zwipe.net's and the share page's images too.

## Cause
zwipe-context `marketing/og_app.html` and `og_default.html` styled the accent span as a link (`text-decoration: underline`). zwipe.net's `og-default.png` came from the same rule. The share page's preview is drawn by zerver's `share_preview` and has no underline. The app's head had `twitter:image` but no `twitter:image:alt`.

## Fix
thinker:
- zwipe-context: both sources call the span `.accent`, with the color and no underline.
- `og-app.png`: rendered again from `og_app.html` in headless Chromium. Before the change the source reproduced the old PNG pixel for pixel, and the new one differs only in the underline's two rows.
- `og-default.png`: the underline's two rows (y 362 and 363, x 260 to 549) painted with the panel color `#282828`. A Linux render draws the rating's stars in another font and moves the card 2px, so the shipped image keeps its pixels apart from the line.
- `zwiper-web/index.html`: `twitter:image:alt`, the same text as `og:image:alt`. `tests/link_card.rs` checks X's image and alt match Open Graph's.

## Verification
-
