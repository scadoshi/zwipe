# Card stacks (1.19.0)

Decided with Scotty on 2026-11-11. A second view of a deck's cards, like Archidekt's stacks. Web first, phones after. Follow the existing structures: grouping, boards, headers, drawers and sheets as they are today.

## Where

A List | Stacks switch on the deck page. The list stays the default, and the choice is remembered. Web first, at wide widths.

## Structure

- One stack per group, from the existing Group by (Type, Mana value, Color, Card role) and the board chips (Main, Maybe, Side). Commander first. Groups come from the cards' own data and tags; there is no custom grouping and no dragging between groups.
- Each stack has a header in the existing group-header style: group name, card count, total price.
- Cards overlap vertically, each showing its title strip; the last card in a stack shows in full. A card with more than one copy gets a small quantity badge.
- Stacks flow into as many columns as the width fits.

## Interaction (web)

- Hover reveals a card in place: the hovered card shows in full and the cards below it make room. The exact motion is designed with Scotty before it is built; start with a plain reveal.
- Click opens the existing card details drawer, with every action it already has (quantity, Printings, star, board, remove).
- While the drawer is open, clicking another card swaps it in; Esc or the close button closes it.
- The drawer does not follow hover (it would flicker as the cursor crosses a stack).

## Phones (after web)

Two columns. First tap reveals a card in place (as hover does on the web); a second tap opens the existing bottom sheet with details and actions. Same switch. Keep the stack layout and reveal state shared so phones mostly wire taps to it.

## Not in v1

Fan-out animation when a stack is clicked, custom groups, drag between groups, inline quantity controls on the stack.

## Later: height cap with a fisheye

Scotty, 2026-11-11, to try after v1: cap each stack at a share of the window (about 70%). A taller stack compresses its spacing: cards near the cursor keep their title strip (the hovered card shows in full) and cards farther away pile almost on top of each other. As the cursor moves down the stack the spacing follows it, so the stack's height never changes and the page never jumps. With no cursor, a tall stack shows evenly compressed. The cost is fewer titles visible at once. Phones use the same math on tap.

