# 401. The app's Stacks columns hold each row to its taller stack

State: fixing | Owner: halo | PR: - | Build seen: e556ab671

## Report
From the composer, after Scotty tested phone Stacks: each row of two stacks takes the height of the taller one, so a 1-card stack next to a 50-card stack leaves a gap, and the next row starts below the 50. The two columns should have independent heights (masonry): each stack sits right under the one above it in its own column, each next group goes in the shorter column (by current height, so a folded group counts as just its header), and group order stays readable. Folding a group lets the stack below it in that column rise to meet it, and revealing a card pushes only its own column down, both through the shared settle, with no snap. Two flex columns, not CSS multi-column (the Firefox slowness in 399). Phone touch Stacks only.

## Fix
halo:
- **Placement.** zwipe-ui `deck_cards::columns::masonry` puts each stack under the shorter column so far, the left on a tie. `stack_height` weighs a stack in thousandths of its column's width: a header (500), the first card whole (1393, from 488 by 680) and a 181 strip per card after it (the 13% a tuck leaves). A folded stack counts as its header only. A reveal counts for nothing, so it never moves a stack across.
- **Layout.** components `DeckStacks` with `touch` renders `.deck-stacks-columns` with two `.deck-stacks-column`s, each holding its stacks in order. The app's `main.css` makes them two flex columns. The pointer stacks (the web) keep their one grid.
- **Movement.** The screen is already a settle trigger in Stacks, so a fold, a reveal and a tap off slide the keyed stacks and cards. Fold all is not a settle trigger, so in Stacks it starts the watch itself (`settle::watch`).

## Checked (headless Chrome, 360 wide, the app's sheets)
- Creatures (12), Planeswalkers (1), Instants (3), Artifacts (2), Enchantments (1), Lands (4): the left column holds Creatures, Artifacts and Lands, and the right holds Planeswalkers, Instants and Enchantments. Each stack sits under the one above it in its column, with no gap waiting on the other column.
- Revealing a card in Creatures moves Artifacts from 675 to 868 and Lands from 1045 to 1238, while the right column stays where it was.

## Verification
-
