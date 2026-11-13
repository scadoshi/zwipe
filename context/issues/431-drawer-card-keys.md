# 431. With a card's details drawer open on the web deck page, the card keys do nothing

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-13: with a card's details drawer open (Scotty clicked an MVP featured card), `p`, `i`, `v`, `+`, `-` and Shift+S do nothing, though they work on hover with the drawer closed. Rule: while the drawer shows a card, that card stays the keyed card, so every card key acts on it as on hover, with 422's commander and read-only exceptions; `d` closes the drawer. Check drawers opened from featured cards, List rows and Stacks, owner and shared. In his screenshot the page behind the drawer is dimmed on a wide window, where 424 leaves the details drawer undimmed. Added: Esc leaves Quick add (blurs the field, clears its suggestions) so the card keys and `u` work again. Added (droider, narrow web, Chrome): with Printings open over the details drawer, Back closes Printings, but a second Back closes the tab instead of the drawer; every open sheet and drawer should own a back step on the web.

## Fix
scotland:
- The details drawer carries its card's id (`data-card`), and the deck page's keys read the drawer's card first, ahead of the card under the cursor, as a stack card so `d` closes it. The keys found the card through the Stacks slot the drawer opened from, which a featured card, and any drawer in List, has none of. A key presses the button in the drawer showing its card.
- The rule that leaves the deck page live behind the drawer (no dim, no click catch) applied only while Stacks showed; it follows the deck page's drawer now, so a featured card's drawer in List leaves the page undimmed as in Stacks.
- Esc in Quick add closes its suggestions and takes focus off the box.
- Back: the browser's back closed the top overlay by putting the screen's URL back as a new entry. Chrome on Android skips an entry a page adds without a tap, so the next back skipped it and left. Each overlay that opens now owns a history entry, added as it opens; the browser's back goes over it and closes the top overlay, an overlay closed another way (×, Esc, the backdrop) takes its entry back out with a back the page swallows, and the router's own back goes past the entries in one move (`chrome/overlay_steps.rs`, orangui 3bd2f9b `OverlayBackStack::depth`). The history follows the stack once it holds still for 50ms, since an overlay drawn again leaves and rejoins the stack at once.

## Verification
- Wide deck page, Stacks: open a stack card, hover another card, press Shift+S, `+`, `-`, `p`, `i`: each acts on the drawer's card; `d` closes the drawer. The same from a featured MVP card, and in List from a featured card. On the commander, `+` does nothing. On a shared deck, `p`, `i` and `d` work and the editing keys do nothing.
- A featured card's drawer in List leaves the page undimmed on a wide window.
- Type in Quick add, press Esc: the suggestions close, the box loses focus, and `p` on a hovered card opens its Printings.
- Open a card's drawer, then its Printings; Back closes Printings, Back closes the drawer, Back leaves the page. Open both again and close them with Esc; one Back then leaves the page. On a phone (Android Chrome) the same, with no tab closing early.
