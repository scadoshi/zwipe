# 468. Phone web's featured strip is small; the app's is big and scrolls (465: the app's third card is cut off and the page jumps)

State: fixing | Owner: thinker | Type: feature | PR: - | Build seen: -

## Report
Composer for Scotty, 2026-11-13: phone web's featured strip shows its cards small, while the app shows them big with a sideways scroll. Make phone web match the app exactly (card size, side-scroll, deal-in, gap), reusing the app's strip component and CSS. 465, from droider's break pass (halo:611/after/after-448-newstar-app.png): in the app, starring a third card makes the strip a clipped row (the third card cut off at the right edge), and Undo unstarring it shifts the page below up about 19px. Nothing below may move when a card joins or leaves.

## Cause
- 468: phone web drew the deck page's head row (`DeckHead`'s `.deck-head-featured`, `components/assets/magic.css`), a different component from the app's strip (`zwiper` `FeaturedCards`, `.deck-featured` in `screens-b.css`).
- 465: the strip's cards shared the row by `flex: 1 0 min(11rem, calc((100% - 1rem) / 2.3))`: two cards grew to 175px at 390 (half the row), a third dropped every card to the 148px basis, so the art and the strip's height shrank (the page below moved) and the third card sat at 2.3 to the screen, cut off at the right edge.

## Fix
thinker:
- The app's strip moved to `components::featured_strip::FeaturedStrip`, which the app draws on its Cards and the web draws on its phone Cards layer under the head (the head keeps the name and facts and drops its own featured row there). Its deal ends on the last card's `animationend` rather than a timer, so it runs on the browser and in the app alike; a `deal_key` prop (the web's Cards entries) and the screen's `ScreenReturns` (the app's) deal it again.
- Every card takes one width whatever the count, `min(11rem, (100% - 0.5rem) / 2)`: two fill the row, a third and later scroll in from the faded edge, and the strip's height holds when one joins or leaves.

## Verification
- Phone web and app at 390, a deck with a commander and MVPs: the strip's cards are the same size on both (half the row), two fill it, a third scrolls; starring or unstarring a third card moves nothing below the strip.

## Follow-up: a half card at the strip's edge (droider, after 468)
- Cause: the cards were `min(11rem, half the row)`. At 412px the row's content is 380px but two 11rem (176px) cards take 360px, so at the scroll's end (scrollLeft = scrollWidth - clientWidth, droider's 348) the last two sit right-aligned and the 20px left shows a sliver of the card before them; snap was on (the app's stop at 184 is one card plus the gap) but the end isn't a card start.
- Fix: the cap is 14rem, so on any phone each card is exactly half the row and two fill it; every stop, the end included, lands on a card start.
- Checked headless at 412 and 390 with four cards: at the end the visible cards start at 0 and half the row (0/194 at 412, 0/183 at 390), and a scroll left at 100px snaps to the next card start.

