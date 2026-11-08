# 285. Generated preview image for shared decks

State: approved (plan first) | Owner: thinker | PR: - | Build seen: -

## Report
Scotty, 2026-11-07: a shared deck's link preview shows a generated image with basic deck stats, not just the commander's art crop. Step 2 of `context/plans/share_preview.md`. Plan only for now: write `context/plans/share_preview_image.md` and bring Scotty the open questions one at a time.

## Plan
thinker, 2026-11-07: [share_preview_image.md](../plans/share_preview_image.md). Recommends rendering in zerver (Rust `resvg`) behind Cloudflare's edge cache with the image URL versioned by the deck's `updated_at`; compares a Pages Function with Satori/resvg-wasm (needs Workers Paid) and prerendering to R2. Five open questions for Scotty at the top.
