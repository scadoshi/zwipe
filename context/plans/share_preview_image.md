# Share preview image

Step 2 of [share_preview.md](share_preview.md), issue 285. A shared deck's link preview (iMessage, Discord, Slack, X) shows a generated 1200x630 image with the deck's name, art and basic stats instead of the commander's bare art crop. Plan only; nothing here is built.

## Open questions for Scotty

Asked one at a time, in this order. Each has a default the plan assumes until he answers.

1. **Where it renders.** zerver draws the image in Rust and Cloudflare caches it (recommended, no new cost), or a Pages Function draws it at the edge, which needs the Workers Paid plan ($5 a month) because the free plan's CPU limit is too small for image rendering. Default: zerver.
2. **Theme.** One fixed look, Gruvbox dark like zwipe.net and the current `og-default.png`, or the deck owner's theme? The owner's theme means the share API carries the theme name (no identity, but it is a setting of theirs). Default: fixed Gruvbox dark.
3. **Price.** Show the deck's price? Prices come from the nightly Scryfall sync, can be missing on some printings, and would only refresh when the deck changes. Default: show it in USD when every main-deck card has a USD price, else leave it off.
4. **Mana curve.** Include a small 0 to 6+ curve, or keep the image to name, art, format, card count and colors? Default: include it.
5. **Art for decks without a commander.** Default: the first starred (MVP) card, else the deck's most expensive card, else no art.

## What the image shows

1200x630, the size every unfurler crops to cleanly. The left 520px is the art and the right side is the text, so the art still reads when Slack or iMessage shrinks the image to a thumbnail.

- **Art.** The commander's `art_crop`. Partner commanders split the panel top and bottom. A Background or signature spell doesn't take art. "Art by {artist}" sits small along the bottom of the art panel, from `scryfall_data.artist`.
- **Deck name.** JetBrains Mono 700, up to two lines, then an ellipsis.
- **Format line.** The format's display name and the bracket when set, such as "Commander · Bracket 3".
- **Led by.** The commander's name, and the partner's when there is one.
- **Color identity.** Pips in WUBRG order. Colorless shows the C pip.
- **Stats row.** The card count, counted the same way as the share page (`cardCount` in the Function), and the price when question 3 allows it.
- **Mana curve.** Seven small bars, 0 to 6+, from `DeckMetrics::mana_curve_bars`, lands left out.
- **Brand.** The Zwipe mark and "zwipe.net" in the bottom right.

Text always comes from the same zwipe-core functions the app uses (format names, curve, price, card count, color identity), so the image can't drift from the deck page. The Pages Function today repeats the format names in JavaScript and needs a test to keep them matched; approach B doesn't add more of that.

## Approaches

### A. Pages Function on Cloudflare: Satori with resvg-wasm, or workers-og

The Function at `/deck/:token/preview.png` fetches the share payload, lays the card out as JSX-like nodes with Satori (yoga layout to SVG), rasterizes the SVG to PNG with resvg-wasm and caches it with the Cache API keyed by token and version. `workers-og` packages the same two libraries.

- **Limits.** The free plan gives a Function about 10ms of CPU per request and a small compressed bundle limit (3MB, as I understand Cloudflare's current limits; to confirm before building). A 1200x630 Satori and resvg render usually takes 50 to 200ms of CPU, and resvg-wasm alone is about 2.5MB. So A needs Workers Paid: 30s of CPU by default and a 10MB bundle.
- **Fonts.** Satori reads TTF, OTF and WOFF, not WOFF2. zite only ships JetBrains Mono as WOFF2, so the Function would bundle TTF copies (about 270KB per weight).
- **Art.** The Function fetches `art_crop` from `cards.scryfall.io` and passes it in as a data URL.
- **Good.** Rendering sits beside the tags and never touches the VPS.
- **Costs.**
  - $5 a month.
  - Layout logic in JavaScript, so format names, card count and curve get re-implemented next to Rust and need drift tests.
  - Satori's CSS subset is narrower than the app's.
  - Cold starts on a heavy bundle.

### B. zerver renders it (recommended)

A new public route on the API returns the PNG. zerver builds an SVG for the card from a template, renders it with the `resvg` crate (pure Rust, with `usvg` and `tiny-skia`) and returns it with long-lived cache headers. Cloudflare already proxies `api.zwipe.net` (the tunnel), so its edge caches the PNG and zerver draws each deck version about once per edge location.

- **Route.** `GET /api/share/deck/{token}/preview.png?v={version}`. The handler loads the shared deck the way the share handler does, so a revoked token stops serving the image.
- **Layout.** An SVG string built in Rust from a fixed template. Text sizes are fixed, and the deck name is cut to two lines by measuring with the embedded font's advance widths (`usvg`'s fontdb has the metrics). There's no general layout engine to carry.
- **Fonts.** JetBrains Mono 400 and 700 TTF embedded with `include_bytes!` (OFL), plus the Mana font's pip glyphs (the same font the app draws `ms` pips with), or plain colored circles with the letter if that font's license doesn't allow embedding. To confirm before building.
- **Art.** Fetched with zerver's existing `reqwest` client from `cards.scryfall.io`, with a short timeout. Kept in a small in-memory cache keyed by URL, since one commander's art serves many decks.
- **CPU.** One render is tens of milliseconds on the VPS. Three things keep a burst of unfurls from turning into a burst of renders: an in-process LRU of finished PNGs keyed by (token, version) with about 64 entries, single-flight per key so concurrent requests share one render, and the share route's existing rate limit.
- **Good.**
  - No new paid plan.
  - Rust, so it follows scadoshi-rust and gets tested like the rest of zerver.
  - Uses zwipe-core directly for every number on the card.
  - The edge cache does the heavy lifting.
- **Costs.**
  - The `resvg` dependency (compile time and binary size; a few MB).
  - Fonts in the zerver binary.
  - Renders land on the one VPS, so they are cached and capped as above.

### C. Prerender on change and store

zerver renders the PNG whenever a shared deck changes (debounced) or a share token is made, uploads it to R2 or a table, and `og:image` points at the stored file.

- **Good.** Unfurls never wait on a render.
- **Costs.**
  - An R2 bucket and its credentials, or image blobs in Postgres.
  - A background job with retry.
  - Renders for decks nobody ever shares again.
  - Stale images if a job fails.

B gets the same "render once per version" result lazily, with none of that.

**Recommendation: B.** It costs nothing new, keeps every number in zwipe-core, and Cloudflare's edge plus the versioned URL make it effectively render once per deck version. A is the fallback if Scotty would rather keep image work off the VPS and pay for Workers.

## Caching and invalidation (B)

- **Version.** The image's version is the deck's `updated_at`. `touch_deck` already bumps it on every card add, remove, quantity, board or printing change, and profile updates set it too, so a commander change, a rename or a card edit each give a new version.
- **URL.** The Pages Function writes `og:image` as `https://api.zwipe.net/api/share/deck/{token}/preview.png?v={unix seconds}`. A new version is a new URL, so nothing needs purging.
- **Headers.**
  - When `v` matches the deck's current version: `Cache-Control: public, max-age=31536000, immutable`.
  - A stale or missing `v` (an old preview someone re-opens): the current image with `max-age=300`, so it stays correct without pinning an old render.
- **Edge.** Cloudflare caches PNG by extension and honors the origin's `Cache-Control` on the free plan. The query string is part of the cache key at the default caching level, so each version caches separately. To check with `curl -sI` for `cf-cache-status: HIT` after the first request.
- **Unfurlers.** Discord, Slack and iMessage keep their own copy of a preview per message. An already-posted preview never updates; a new share of the edited deck gets the new URL and the new image.
- **Price drift.** Prices change nightly without bumping `updated_at`, so the price on the image only refreshes when the deck changes. Fine if question 3 keeps the price; otherwise not an issue.

## Fallbacks

- **No art** (no commander, no MVP, or the art fetch fails or times out): the no-art layout, with the theme background and the color pips large in the art panel.
- **Render error:** a 302 to `https://zwipe.net/assets/og-default.png` with `max-age=60`, so the next request tries again.
- **Unknown, revoked or malformed token:** 404. The Function never points at the image for those tokens anyway; it keeps today's generic tags.
- **The Function can't reach the API:** today's behavior, the generic tags with the site-wide image.
- **Very long names, many colors, both partners:** covered by layout tests (below).

## Theming

Default: one fixed look, Gruvbox dark, the same palette `og-default.png` and zwipe.net's default use, read from the same theme values the app's Gruvbox profile has so the colors match exactly. If Scotty picks the owner's theme (question 2), the share payload gains an optional `theme` and the template reads that profile's colors, still dark.

## What the share API adds (additive only)

Both changes follow [api_evolution.md](../architecture/api_evolution.md) and go to the composer before they're built.

1. **`HttpSharedDeck.updated_at: Option<DateTime<Utc>>`** with `#[serde(default)]`. The Function uses it to version the image URL. Older clients ignore it, and a payload without it gets an unversioned URL with the short cache.
2. **A new route, `GET /api/share/deck/{token}/preview.png`**, public, `image/png`. Added to `route_coverage.rs` and the path constants in zwipe-core.
3. **Only if question 2 picks the owner's theme:** `HttpSharedDeck.theme: Option<String>`, also `#[serde(default)]`.

Approach A needs only the first change, since the Function would render the image itself.

## Tests

- **Layout** (zerver unit tests):
  - The SVG for a fixture deck holds the name, format, count, curve bars and pips.
  - A 120-character name wraps to two lines and ends in an ellipsis.
  - Partners split the art panel.
  - A deck with no commander takes the MVP's art.
- **Render.** The PNG decodes at 1200x630. A small checksum or pixel-sample test against a fixture catches accidental layout changes, with a tolerance so font hinting can't flake it.
- **Route** (integration test):
  - 200 `image/png` with the immutable headers for the current `v`, and the short headers for a stale one.
  - 404 for an unknown or revoked token.
  - 302 to the default image when the art fetch fails. The art client sits behind a trait so the test can fail it.
- **Function.** zite's `og_function` test checks the image URL carries the token and `v`.

## Rollout

1. Composer approves the two additive wire changes.
2. zerver: the route, the template, fonts, the art cache, the LRU and single-flight, and the tests. One PR.
3. zite: the Function writes the versioned image URL when `updated_at` is present, else keeps the art crop. One PR.
4. After the deploy, Surfer checks a real share in Discord, Slack and iMessage, plus `cf-cache-status` on the image.

## References

- Scryfall's image guidelines: art crops are fine to show and should credit the artist; don't add other marks over the art. To re-read before building, since the layout puts text beside the art, not on it.
- Cloudflare limits for Workers and Pages Functions (CPU per request, bundle size), to confirm on the current plan page before choosing A.
