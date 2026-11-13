# Shared guides: one guide source for zwipe.net and the web app

Issue 419. Status: built on release 2026-11-12, every phase plus Scotty's extras (gallery keys, tag links, the index filter in the address); waiting on the deploy and Surfer's live check. The screenshots sit at `zite/public/guide-images/` and serve from `https://zwipe.net/guide-images/`, not the `/guides/img/` path planned below; a guide's back arrow in the web app goes to the index; the web app's hint links open its guide in a new tab, as every dialog link does. The issue's Fix section has the details.

## Scotty's decisions

1. Guide content (text, images, figures) lives in one shared place that zwipe.net (zite) and the web app (zwiper-web) both render from, with no copies.
2. The web app gets public guide routes, `/guides` and `/guides/:slug`. They work signed in (the normal nav) and signed out (the public nav, with sign in and sign up).
3. Every guide link inside the web app opens the web app's guide. zwipe.net keeps its own guide pages for SEO and links only to its own.
4. The phone app's guide links open app.zwipe.net's guide in the browser. Guides aren't compiled into the app.
5. The web app's guide pages carry `rel=canonical` to the zwipe.net guide URL, so SEO stays with zwipe.net.

## Where things are today

- `zite/src/pages/guides/content.rs` (1248 lines): `Guide` and `Block` as typed data (`GUIDES`), the image sizes (`guide_image_size`) and an `asset!` registry (`guide_image`).
- `zite/src/pages/guides/mod.rs` (653 lines): the index (tag chips), the detail page (breadcrumb, JSON-LD, related guides), `render_block` and the `[[icon]]` and backtick tokens.
- 22 screenshots, 4.2 MB, in `zite/assets/guides/<slug>/`.
- Links: zwipe-ui `hint::guide::url(slug)` builds `https://zwipe.net/guides/<slug>/`. components' `guide_for_*` actions use it, so today the web app and the phone app both send readers to zwipe.net.
- zwiper-web's `OpenFrame` already serves routes to everyone: signed in under the normal `Shell`, signed out under `PublicFrame` (About, Privacy policy, Not found via `is_open`). Scotland's 418 is building on the signed-out frame for shared decks.

## Where the shared pieces go

| Piece | Where | Why |
|---|---|---|
| `Guide`, `Block`, `GUIDES`, tags, related, image sizes | zwipe-core `content::guides` | pure data like `content::changelog`; zite, zwiper-web and later the app read one copy. `MediaSize` becomes a plain `(width, height)` there. |
| Screenshots | once, under zite's `public/guides/img/<slug>/<file>`, served as-is at `https://zwipe.net/guides/img/...` | dx bundles only a crate's own files (`zwiper-web/src/lib/inbound/sheets.rs`), so an `asset!` in a shared crate can't reach the web build. One public copy keeps Scotty's "no copies": zite and the web app both point at the zwipe.net URL (`localhost:8080` in a debug build, from `site::WEB_BASE`). A changed screenshot gets a new file name, since the URL is no longer hashed. |
| Article and index rendering (`render_block`, the tokens, the related block, the index chips) | components `guides` | one renderer. The host passes how a guide link is built (zite `/guides/<slug>/`, the web app its `Route`) and where images live (both: the zwipe.net URL). |
| SEO pieces (JSON-LD, sitemap, breadcrumb's site name, `<title>`) | zite | zwipe.net keeps them; the web app adds only the canonical link. |

The alternative for images was a copy in `zwiper-web/assets/guides` held equal by a test, as the stylesheets are. That doubles 4.2 MB in the repo and is a second copy, so I recommend the public URL.

## Links

- zwipe-ui `hint::guide::url` takes the base: `GuideBase::Site` (`https://zwipe.net/guides/`, zite only), `GuideBase::WebApp` (the relative `/guides/`, so a web app link stays in the web app and opens in the same tab) and `GuideBase::Phone` (`https://app.zwipe.net/guides/`, which the app opens in its in-app browser as it does today).
- components' `guide_for_*` take that base. zwiper-web passes `WebApp`, and zwiper passes `Phone`.
- Inside a guide, a related guide link follows the host: on zwipe.net to zwipe.net, in the web app to the web app.

## The web app's routes

- `/guides` and `/guides/:slug` join `OpenFrame` and `is_open`, so a signed-out reader gets the public nav and can sign in or sign up from it. A signed-in reader keeps the normal nav.
- Each guide page puts `document::Link { rel: "canonical", href: "https://zwipe.net/guides/<slug>/" }` in the head (the index points at `https://zwipe.net/guides/`).
- If 418 changes `OpenFrame` or the signed-out frame, the guides take its version. I'll check with the composer before touching `frame.rs`.

## Phases (one push each)

1. **Content to zwipe-core:** `content::guides` with the data and image sizes. zite renders from it with no visible change. Tests move with it: every guide's related slugs exist, every image has a size, and the sizes match the files' headers.
2. **Images to one public path:** zite's `public/guides/img/`, with the `asset!` registry gone. zite's pages load them from the shared URL, checked at 1400 and 390.
3. **Rendering to components:** zite adopts with no visible change (headless shots before and after).
4. **The web app's guide routes,** in the open frame, with the canonical link, plus the link bases: web app links in-app, the phone to app.zwipe.net. This waits on 418's frame if it changes `OpenFrame`.

No wire or server change.
