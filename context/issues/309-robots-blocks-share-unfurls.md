# 309. X shows no card for shared deck links

State: verified | Owner: laptop | PR: #449 | Build seen: 02102ab5e

## Report
Scotty, 2026-11-08: pasting zwipe.net/deck/... links into X shows no image. Composer: the page and image serve fine to a Twitterbot user agent (200, twitter:card summary_large_image, PNG 200), but zwipe.net/robots.txt has `User-agent: *` ... `Disallow: /deck/` (meant to keep shared decks out of search). Twitterbot honors robots.txt, so it never fetches the page. Other unfurlers that honor robots (LinkedInBot, Slackbot, Discordbot, facebookexternalhit) are blocked the same way.

## Fix
In zite's robots.txt, give the link-preview bots their own group that allows /deck/ (Twitterbot, facebookexternalhit, LinkedInBot, Slackbot-LinkExpanding, Discordbot, TelegramBot, WhatsApp, SkypeUriPreview/Teams), keep `*` disallowing /deck/ for search engines. Note Cloudflare prepends its managed block to robots.txt. Test that robots.txt allows /deck/ for Twitterbot and disallows it for Googlebot. Verify with X's card behavior after deploy (new link, since X caches per URL).

## Fix (laptop)
Branch `nits/laptop-309`. zite's robots.txt keeps `User-agent: *` disallowing /deck/ (and /verify/, /confirm-email/, /reset/) and adds one group for the link-preview bots (Twitterbot, facebookexternalhit, LinkedInBot, Slackbot-LinkExpanding, Discordbot, TelegramBot, WhatsApp, SkypeUriPreview, Teams) that allows / and still disallows the three private paths; a crawler obeys only the group naming it, so those lines repeat. Shared deck pages carry `noindex`, so search stays out either way. app.zwipe.net served its index.html at /robots.txt through the single-page fallback; zwiper-web now ships `public/robots.txt` (`User-agent: *`, `Allow: /`), which dx copies to the site root like its icons. zite has no AI-crawler rules, so neither file adds any. Tests: zite's `tests/robots.rs` reads the file as RFC 9309 crawlers do (the named group over `*`, longest rule wins, Allow wins a tie): Twitterbot and the other preview bots may fetch /deck/x and not the private paths, Googlebot, Bingbot and DuckDuckBot may not fetch /deck/x but may fetch / (failed before); zwiper-web's `tests/robots.rs` checks its file is plain rules allowing everything. Cloudflare prepends its managed block to zwipe.net's robots.txt; verify X's card with a new deck link after deploy (X caches per URL).
