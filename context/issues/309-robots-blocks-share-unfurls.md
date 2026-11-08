# 309. X shows no card for shared deck links

State: approved | Owner: laptop | PR: - | Build seen: 02102ab5e

## Report
Scotty, 2026-11-08: pasting zwipe.net/deck/... links into X shows no image. Composer: the page and image serve fine to a Twitterbot user agent (200, twitter:card summary_large_image, PNG 200), but zwipe.net/robots.txt has `User-agent: *` ... `Disallow: /deck/` (meant to keep shared decks out of search). Twitterbot honors robots.txt, so it never fetches the page. Other unfurlers that honor robots (LinkedInBot, Slackbot, Discordbot, facebookexternalhit) are blocked the same way.

## Fix
In zite's robots.txt, give the link-preview bots their own group that allows /deck/ (Twitterbot, facebookexternalhit, LinkedInBot, Slackbot-LinkExpanding, Discordbot, TelegramBot, WhatsApp, SkypeUriPreview/Teams), keep `*` disallowing /deck/ for search engines. Note Cloudflare prepends its managed block to robots.txt. Test that robots.txt allows /deck/ for Twitterbot and disallows it for Googlebot. Verify with X's card behavior after deploy (new link, since X caches per URL).
