# 457. On a phone, the deck's Profile, Cards and card details don't behave like route changes

State: fixing | Owner: scotland | Type: feature | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-13: every tap that isn't a bottom sheet or side drawer must feel exactly like a route change. In the app the iOS back swipe on Cards reveals the deck's Profile underneath as it slides away. Profile to Cards, and on a phone Cards to a card's details, should look and behave like any other route: no screen visible underneath during a back swipe or predictive back, and the content animating in as if entered fresh every time (the same entrances, the featured strip's deal-in included), with the data and images from the held deck so it's instant. Scotty took option B (route-shaped layers, no real routes). Droider: a fresh load straight onto `?view=cards` shows the deck's name in the header instead of "Cards".

## Cause
- The Profile stays painted under the Cards layer, and the Cards under the details screen (both mounted, both visible to the browser); the Cards layer's 0.3s slide-out is what showed the Profile during the app's back swipe (that slide went to 1ms in 4059d64f8). Nothing replays a screen's entrances when the screen over it closes: they run once, at mount.
- The header: `zwiper-web/src/lib/inbound/shell/frame.rs`, `ScreenFrame` sets the nav's one title slot in an effect that runs again whenever its title changes. On a fresh `?view=cards` load the Cards layer takes the slot first and the deck's name arrives after, so the page's effect writes over the layer's "Cards"; from the Profile the name is already in, so the layer's write is the last.

## Fix
scotland:
- `components::screen_return`: a screen another covers carries `under-layer` (not painted) and `inert` (not focused or tapped) while the one on top shows; `use_screen_returns` counts its returns, `replay_entrances` runs its entrances (`content-enter`, `content-enter-delayed`, `chip-in`) again in place, and `ScreenReturns` (context) lets whatever deals on entry deal again. Nothing unmounts, so scroll, filter, folds, the cursor and Undo stay.
- Web (`body.rs`, `cards_layer.rs`): the phone Profile while the Cards layer shows, and the Cards layer while a card's details screen shows; on each return the entrances replay and the featured strip deals again (thinker's `deal()`), and the stacks deal again on each Cards open and each return (`DealCue::again`, `use_stacks_dealing`'s `entries`).
- App (`deck/view.rs`, `deck/card/view.rs`, `featured_cards.rs`): the same for the Profile under Cards and, at phone width, the Cards under a card's details; `FeaturedCards` deals again on its screen's `ScreenReturns`.
- The header: the nav keeps the page's title and the overlays' titles apart (`OverlayTitles`, a stack); it shows the newest overlay's, else the page's, so a page setting its title late no longer takes the nav from an overlay over it.

- 466 takes the card details out of this: they are a drawer over the Cards again, so the Cards stay in view under them, dimmed, and nothing replays when the drawer closes. Profile to Cards keeps the route feel.

- Regression on 31b3ab105 (droider, emulator and Scotty's phone, 3 of 3): in the app, Cards opened to the grid tiles alone. Cause: `zwiper/src/lib/inbound/screens/deck/view.rs`, the Cards layer renders inside the Profile's `.screen` (the `if cards_layer.shown()` block sits before that div's closing brace), so `under-layer` and `inert` on the Profile's screen hid and froze the Cards with it; the web's Profile wrapper is a sibling of its layer, so phone web was fine. Fix: the Profile's screen takes `profile-covered`, which hides its children but `.app-cards-layer` (`magic.css`, pinned by `components/tests/profile_covered.rs`), and `inert` goes on its content and dock instead.

## Verification
- Phone web (412): open a deck's Cards: the Profile is hidden and inert under it; Stacks deal and the strip deals in. Open a card's details (two taps on a stack card): the Cards are hidden and inert under them. Back: the Cards come back with the strip and the stacks dealing again. Back: the Profile comes back with its entrance.
- Fresh load on `/decks/<id>?view=cards`: the header reads "Cards".
- App: the same on a phone; the iOS back swipe and Android back show nothing underneath.
