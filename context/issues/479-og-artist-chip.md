# 479. A shared deck's link preview credits the art with a plain "Art" label and text

State: fixing | Owner: laptop | Type: feature | PR: - (pushed branch, hash in the composer's log) | Build seen: release bb316783f

## Report
Scotty, via the composer: credit the artist in the shared deck's Open Graph image (the link preview for `/:owner/deck/:slug/:token`) the way 475's Printings chip does: the Image icon then the artist's name, in the `detail-chip-artist` style. Render before and after at 1200x630 before it ships; deploy nothing.

## Before
Local zerver, the shared Atraxa Superfriends deck (`/api/share/deck/8726b1ec-.../preview.png`), 1 of 1: the footer shows a muted `Art` label chip, then `Justin Hernandez & Alexis Hernandez` as plain text in the text color.

## Cause
`zerver/src/lib/inbound/http/share_preview/layout.rs`, `draw_footer`, draws the credit as a `Tone::Muted` chip reading `Art`, then the artists' names as a separate `text` element.

## Fix
laptop:
- **Credit chip:** `draw_footer` draws one chip (`artist_chip`) in the `--palette-6` tone, as `detail-chip-artist` is (12% tint, 35% edge, the theme's `--ink-palette-6`). It holds the Image icon (the same Lucide path as `components::IMAGE_ICON`, 22px, stroked) and then the artists, joined with "and" and cut to the room left of the brand.
- **Ink:** `palette.rs` reads `--ink-palette-6` into `Inks`.
- **Layout version:** `SHARE_PREVIEW_LAYOUT` goes from 2 to 3 (zwipe-core and the share Function's `PREVIEW_LAYOUT`), so unfurlers and the edge fetch the new drawing.
- **Test:** `the_art_credit_is_the_artist_chip_with_the_image_icon` fails on the old footer.

## Checked
Before and after at 1200x630 from the local zerver, same deck.

## Verification
-
