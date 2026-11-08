# 308. Share preview images may be too big for WhatsApp

State: approved | Owner: thinker (after the 303 gate step) | PR: - | Build seen: 02102ab5e

## Report
Composer, 2026-11-08, answering Scotty's "when do share images show in Teams, WhatsApp, X?". The tags are right (og:image 1200x630 with width/height, twitter:card summary_large_image), but the PNG is 378 KB for a one-commander deck and was 538 KB for partners. WhatsApp is known to skip og:image above roughly 300 KB, so WhatsApp may show no picture.

## Fix direction
Serve the preview as JPEG (quality ~85) or a palette-reduced PNG so a two-art deck stays well under 300 KB, keeping 1200x630 and the og:image:type in step. Bump SHARE_PREVIEW_LAYOUT so caches refresh. Test pins a size ceiling on the golden renders.
