# 404. The app's List groups have no cart

State: fixing | Owner: halo | PR: #620 | Build seen: 023c1f769

## Report
From Scotty, through the composer: the phone app's Deck > Cards should have the group shopping cart on both views, the List group headers and the Stacks headers, as the web has. Stacks headers got a cart in #611 and #613. The List headers have none.

## Fix
halo, #620:
- **Stacks:** every group with a priced card or a card to buy shows its price and cart: the boards, the command zone and the main groups. The tokens have nothing to buy, so they show neither, on both views and on the web.
- **List:** the app's `CollapsibleGroup` takes a `trailing` tail. With it, only the arrow and the title fold the group (a `.card-group-toggle`), and the price chip and `CartButton` sit on the line under the title, the cart at the end, so a tap on the cart never folds. The screen reads each group's rows from the model's `stack_groups`, the same groups under the same fold keys, and the cart opens the existing `GroupBuyDialog` (Buy <group>, TCGplayer and Card Kingdom) through `GroupBuy::offer`, as a stack's does.
- **CSS:** `.card-group-header-tailed` in the app's `main.css`.
- **Tests, written first:** a tailed group folds only from its title and keeps the cart outside the toggle; a group without a tail keeps its whole-header fold.

## Checked (headless Chrome, 393 wide, the app's sheets)
- Creatures, a folded Instants and sorceries, and Lands each show the price under the title, starting under its first letter, with the cart at the right end. Tokens shows neither, as before.

## Verification
-
