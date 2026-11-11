# 346. GitHub-style shared deck links, "by <username>", and the deal on the web deck page

State: fixing | Owner: thinker | PR: - | Build seen: release 1.18.17

## Report
Scotty, 2026-11-11 (feature, approved), via the composer:
1. Shared deck links become `zwipe.net/<username>/deck/<deck-name-kebab>/<id>`. Only the id identifies the deck, and a stale username or slug goes to the canonical link. Scotty is fine if old links break; new builds share the new form, and nothing a shipped app calls may break.
2. The shared deck page shows "by <username>" under the deck name.
3. The app's deck screen gets the share page's card deal-in entrance.

Scotty explicitly approved publishing the owner's username on shared decks, in the link and as the byline.

## Fix
thinker, 2026-11-11, on `nits/thinker-share-links`.
- **Link** (`zwipe-core::domain::share_link`): `deck_slug` makes the kebab slug: lower case, common Latin accents folded, every other run one dash, trimmed, at most 60 characters cut at a dash, `deck` when empty. `encode_segment` percent-encodes the username, which may hold any non-whitespace character. `shared_deck_path`/`shared_deck_url` build `/<owner>/deck/<slug>/<token>`. The `<id>` is the share token, the capability re-sharing rotates. Tests first.
- **Server:** `HttpSharedDeck` gains an additive, optional `owner` (the username); `GET /api/share/deck/{token}` fills it, and a failed lookup leaves it out. Its doc records Scotty's OK. Shipped apps ignore the field.
- **Apps:** the deck view model (`State::for_owner`) and web's share dialog build the new link from the session's username, the deck name and the token.
- **zite:** a new route `/:owner/deck/:slug/:token` loads by token alone. The old `/deck/:token` stays as a route to the same page. Once the deck loads, the page replaces the URL with the canonical one whenever the owner or slug differs, which covers old links, renamed decks and renamed users. With no `owner` (an older server) the link stays.
- **Byline:** "by <username>" sits under the name through `DeckHead`'s new `byline`, in the existing `text-muted` style.
- **Link previews:** `functions/[owner]/deck/[slug]/[token].js` re-exports the `/deck/:token` handler. `og:url` and canonical use the new form when the API names the owner.
- **Static pages:** zite's routes are one or two segments, so only the four-segment `/x/deck/y/z` shape reaches the new route.
- **Deal:** zite's `.sd-featured` deal moved unchanged into components' `magic.css` as `deck-head-deal`, which both sites load. zite uses it, and app.zwipe.net's wide deck page now holds its featured row until the images load (or 4 s pass), then deals. Reduced motion shows the cards plainly. The native app already deals its featured strip.
