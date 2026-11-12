# Shared deck page in the web app (418)

Status: approved 2026-11-12 (Scotty, via composer: plan it and start it). Owner: scotland.

A shared deck's link opens on app.zwipe.net instead of zwipe.net. The web app already draws the deck page, its Stacks, keys, printings and Oracle tag examples, so a reader gets all of it, signed in or not. zwipe.net's own copy of the page goes once the web's is live.

## Hard requirement: link previews

A shared link pasted into Teams, WhatsApp, X, iMessage, Discord or Slack shows the deck's name, its description and its preview image, as zwipe.net's does today. Crawlers do not run the wasm, so the page's tags come from the server side:

- A Cloudflare Pages Function on the deck route serves the web app's `index.html` with the deck's `og:title`, `og:description`, `og:image` (the existing `/api/share/deck/{token}/preview.png`), `og:url`, `twitter:card` and `<title>` in place of the generic ones. It is zite's `functions/deck/[token].js`, which already does exactly this, pointed at the web app's shell and host.
- Unknown, revoked or malformed token: the generic tags with a 404. API down: the generic tags with a 200, and the app shows its own retry.
- Test, run before and after deploy: `curl -sA "Twitterbot/1.0" https://app.zwipe.net/<owner>/deck/<slug>/<token>` returns the deck's `og:title`, `og:description` and `og:image`, and the image URL answers 200 `image/png`. The same with `facebookexternalhit/1.1`, `WhatsApp/2` and `Slackbot-LinkExpanding 1.0`. Locally: `wrangler pages dev` over the built site with the function's `API_BASE` pointed at a local zerver.
- zite's `og_function` test, which keeps the function's format names in step with zwipe-core, moves with the function.

## The URL

One shape, the one links already use, on the web app's host: `https://app.zwipe.net/<owner>/deck/<slug>/<token>`. Only the token identifies the deck; the owner and slug are cosmetic and corrected in place, as zite does now. The two-segment `/deck/<token>` form is not carried over.

## Steps

1. **The route** (zwiper-web). `#[route("/:owner/deck/:slug/:token")] SharedDeck` in `OpenFrame`, outside `AuthGate`, so a signed-in reader keeps their nav and session and a signed-out reader gets the public nav. Four segments with `deck` second collide with no web route.
   - **One page, two modes** (Scotty): the deck page's body is one component, `DeckPageBody` (`screens/deck/view/body.rs`): the head, controls, List or Stacks, stat panels, dock, filter, stores, Stacks drawer, image preview and printings, and the Oracle tags' examples. The owner's page fills it from its editor and passes its edit actions, Quick add, the foot and the dock's edit buttons; the shared deck fills it from `GetSharedDeck` (public) and passes none of them, with `use_deck_hover_keys` read only. Both modes go through the same code, so a fix to one is a fix to both.
   - The owner's phone layout stays its own screens (the deck view and `/cards`); the shared deck on a phone is the same body under `DeckPage`'s Profile and Cards areas.
2. **Public sources on the web.** The web's printings and examples sources go through `use_authed`, which a signed-out reader has no session for. On the shared page they use the public routes: printings `GET /api/card/{oracle_id}/printings`, examples `GET /api/card/oracle-tag/{slug}/examples` (c63ef8780), meld `GET /api/card/{id}` (already public). Signed-in readers can take the same public calls; nothing on the page is personal.
3. **Previews.** `zwiper-web/functions/[owner]/deck/[slug]/[token].js` (zite's, with `SITE = https://app.zwipe.net`) and `deploy-web.yml` runs `wrangler pages deploy` from a directory holding `functions/`, as `deploy-zite.yml` does. Crawler test above.
4. **The link.** `share_link::shared_deck_url` builds on `APP_BASE` instead of `WEB_BASE`. Both clients mint links through it: zwiper-web, and the phone app's deck dock Share (#593, through `zwipe_ui::deck::view::share_link`). A phone build already in the stores keeps minting `zwipe.net/<owner>/deck/<slug>/<token>` until it updates.
5. **The 301 (required).** zite's functions answer `zwipe.net/<owner>/deck/<slug>/<token>` and `/deck/<token>` with a 301 to the same path on app.zwipe.net (`/deck/<token>` keeps its token in the owner form's place: `/_/deck/deck/<token>`, which the page corrects in place). Shipped phones' links and links already pasted into chats keep opening, and a crawler following the 301 reads the web app's deck tags. Ship 1 to 5 together, so a new link never points at a page that isn't there yet.
6. **zwipe.net's page, after surfer's checks.** Remove zite's shared deck page, its `og_function` test and its now-unused share CSS; the 301 functions stay.

## Wire and API (`development/api_evolution.md`)

No contract changes. The shared-deck, printings, card and examples routes exist and are public. The share URL is built in the client, so moving its host is a client change only. Deploy order: zerver needs nothing; the web deploy carries the route, the function and the new link at once; zite's removal (or redirect) follows.

Dead links: shipped phone builds keep minting zwipe.net links, and links shared before the switch point there too. Step 5's 301 keeps all of them working, so it stays for as long as a phone build that predates step 4 is in use.

## Out of scope

zite's examples styling (parked on `nits/scotland-zite-examples`, not pushed): the web draws Examples natively, so zite no longer needs it. The orangui `SCREEN_CSS` split (57d01ad) is harmless and stays.
