# 393. No-image stack strips wrap and peek, and Home's deck tiles show a broken image

State: fixing | Owner: laptop | PR: - | Build seen: live #589, web, images blocked

## Report
Surfer, via the composer, 2026-11-11, on live #589 with images blocked:
- A tucked stack card shows only its top strip, but the proxy's name wraps to two lines and the second hides under the next card.
- The name's size differs card to card in one stack, and covered cards' chips peek under some names.
- app.zwipe.net Home's Decks tiles show the browser's broken-image icon.

## Fix
laptop, web (components, so the app's stacks too):
- **Stacks.** A covered stack card's proxy carries `card-proxy-strip`. It shows only the name and cost on one line, at one size for every card in the stack (the card's width sets it, and the fit script leaves strips alone), cut with an ellipsis. Nothing under the head shows. The full proxy, fitted, shows once the card is revealed or is the last; the fit script refits when the class changes.
- **Home tiles.** A deck tile's art that fails to load shows the deck's name in the no-image card's dress, in the tile's square, wrapped between words and fitted like a thumbnail's.

Headless, images blocked, 1422: on the deck page's Stacks, 32 strips, one name size (12px), none on two lines. Home's failed tile shows its name, with no broken image left. Shots: `target/393-shots/` on the laptop. The html golden `card_stack_dealing` gains the strip class on its covered card.

## Verification
-
