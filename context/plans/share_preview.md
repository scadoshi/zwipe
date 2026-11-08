# Share link previews

Scotty, 2026-11-07: moving zwipe.net to Cloudflare Pages is approved in principle. Do it when he's home, since he has to attach zwipe.net and www to the new Pages project in the Cloudflare dashboard and then turn off GitHub Pages. Until then, #336 stays held and nobody touches DNS or the Pages setup.

## Why

zwipe.net is on GitHub Pages, which serves /deck/:token as the SPA's 404.html with HTTP 404. People see the deck because the page's JavaScript loads it. Link unfurlers (iMessage, Discord, Slack) and search crawlers don't run JavaScript, so they get a 404 and the site's generic og tags.

## Step 1: per-deck tags (built, #336)

A Pages Function at zite/functions/deck/[token].js reads the public share API and rewrites title, description and og/twitter tags with the deck's name, format, commander and card count, using the commander's art_crop as the image. Cutover steps are in deploy_web.md.

## Step 2: a generated preview image (planned, see [share_preview_image.md](share_preview_image.md))

Scotty wants a share preview that shows basic deck stats: an og image generated per deck (deck name, commander art, format, card count, color identity, maybe price and mana curve). Scotty mentioned an earlier thread about this that hasn't been found. Plan it after the cutover. One option is to render the image in a Pages Function or Worker (Satori + resvg-wasm, or workers-og) and cache it per deck version.
