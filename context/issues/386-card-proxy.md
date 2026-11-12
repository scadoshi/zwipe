# 386. One no-image card for every image spot

State: fixing | Owner: laptop | PR: - | Build seen: local, web

## Report
Composer for Scotty, 2026-11-12 (grown from 384): every place a card image can be missing or fail gets the same card, replacing the swipe and Featured proxy and the stacks' and Printings' different fallbacks, with List row thumbnails getting one where they had none.
- **Shape:** always the card's 63:88 ratio, the exact box the image would fill.
- **Content:** laid out exactly as the expanded card row, from the row's own pieces.
- **Text:** shrinks to fit the box.
- **Thumbnails:** only what reads at that size.
- **Theming:** theme tokens only.

Scotty approved the renders (`target/386-renders/`), and asked for the chip order to follow the row.

## Fix
laptop:
- **The component:** `components::CardProxy`, in a 63:88 size container.
  - **Order:** the name with the mana cost, a rule, the row's own `KindChips` (type line, rarity), the rules text, the Keywords and Card roles rows (plain chips with the row's classes, since a proxy sits inside clickable cards), then the row's own `FactsChips` (set, prices, power and toughness).
  - **Sizing:** type in container units. A shared script (`use_card_proxy_fit`, installed at the web, app and zwipe.net roots) steps `--proxy-scale` down until nothing overflows.
  - **Thumbnails:** under 110px wide, the name only.
- **Wired into:**
  - `FlippableCardImage`, which covers every swipe stack (Add, Remove, the commander swipe, Suggested commanders, Examples), the image previews (Featured flavor on app.zwipe.net Home, the app's Home and zwipe.net), the featured cards and the app's printing sheet. Its proxy lies over a transparent stand-in the image's size, so every host's image rules give it the image's exact box. The swipe stacks pass the card's profile for its roles.
  - `CardStack`: a missing or failed image draws the proxy.
  - The `CardRow` thumbnail: in the thumbnail's own box, the name only.
  - `CoverFlow` (Printings): a missing or failed printing draws the proxy.
- **zwipe.net Featured flavor** always offers Image, as the app does, so a card without art opens on its proxy.
- **Removed:** the old proxy (`no-image-card`), the stacks' name-only fallback and the cover flow's set-name tile.

## Verification
-
