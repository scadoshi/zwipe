# 421. The revealed card in Stacks sits still under an outline

State: fixing | Owner: laptop | PR: - (pushed branch, hash in the composer's log) | Build seen: f24dda35e

## Report
Composer, following 417: in Stacks the single revealed card should float like the featured cards, toned down (a small lift, about half the sway). On the web, the card under the mouse once it is revealed, starting after the reveal settles, not the name bars around it or the whole stack. In the app, the card a first tap reveals floats while it is revealed and stops on tap-off. Off under reduced motion, transform only, no layout shift, smooth in Firefox, easing back without a snap, shared code where possible. Scotty, added: no outline on the selected card in Stacks on the web or in the app; a ring only for keyboard focus (`:focus-visible`).

## Fix
laptop: the float is shared CSS in `components/assets/magic.css` ("Float"): `floatable` on an element, plus `floating` while it should float, or `float-on-hover` to float under a mouse hover. `--float-lift`, `--float-grow`, `--float-swing` and `--float-delay` tune it (defaults are 417's featured values). The turn is a paused sway phase times an eased reach, both registered numbers, so a card stops from wherever it was. The featured image is `floatable float-on-hover`. `CardStack` gives every card `floatable` and the revealed one `floating`, in both its hover and tap modes, with a lift of 0.15rem, 1.5% larger, a 0.6 degree swing and a delay of the stack's 500ms move; the revealed slot sits over the next only while it is revealed. The `open` outline (and `CardStack`'s `open` prop, used only for it) is gone; `.card-stack-card:focus-visible` keeps its ring.

## Checked
- Chrome, web deck page at 1400: hovering a stack card's strip reveals it, nothing floats for 500ms, then it eases to 0.15rem up and 0.6 degrees; leaving eases it back from 0.6 degrees in about 0.35s; the slot's z-index holds until it settles. The featured card's hover is unchanged (1.2 degrees back to 0 in about 0.35s).
- Click opens the details with no outline on the card; Tab rings the focused card.
- Firefox 160 (headless, WebDriver BiDi, a page with the same CSS): the float waits 500ms, eases in, and eases out from 0.55 degrees with no snap.
- Tap mode: the web host forced into `touch` locally (not committed) with phone-width columns: a tap reveals the card and it floats after the push settles. Stopping is the same class change as the web's leave.

## Follow-up: tap-off pop (droider, app, 2a5de3562)
Tap-off on Sol Ring: the cards below slid up under the still-raised card for about 0.6s, then in one frame it dropped behind and they popped on top. The revealed slot held its z-index for 0.45s after the reveal ended, past the app's slide. Fix: the slot's raise ends the moment it is no longer revealed (no hold), so the cards below slide back over it in their own order while it settles. Checked in Chrome: in tap mode (local, forced) moving the reveal leaves the old slot at z-index auto from the first frame with the next card on top; on the web's leave the cards below pass over its lower edge one by one as they slide up and it never comes back on top.

## Verification
-
