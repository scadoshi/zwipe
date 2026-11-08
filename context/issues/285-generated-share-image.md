# 285. Generated preview image for shared decks

State: fixing | Owner: thinker | PR: - | Build seen: -

## Report
Scotty, 2026-11-07: a shared deck's link preview shows a generated image with basic deck stats, not just the commander's art crop. Step 2 of `context/plans/share_preview.md`. Plan only for now: write `context/plans/share_preview_image.md` and bring Scotty the open questions one at a time.

## Plan
thinker, 2026-11-07: [share_preview_image.md](../plans/share_preview_image.md). Recommends rendering in zerver (Rust `resvg`) behind Cloudflare's edge cache with the image URL versioned by the deck's `updated_at`; compares a Pages Function with Satori/resvg-wasm (needs Workers Paid) and prerendering to R2. Five open questions for Scotty at the top.

## Answers
Scotty's answers: 1. zerver draws it, Cloudflare caches it.
2. The owner's theme (share API may carry the theme name).
3. Show the price as is (USD sum of priced cards; unpriced cards just don't count).
4. Include the mana curve, and the deck tags too. Final set: name, art, format (with power level when set), card count, colors, price, deck tags, mana curve.
5. Art only from the command zone (commander, partner, Oathbreaker and the like). A deck with no command zone gets no art; the text side takes the full width.
