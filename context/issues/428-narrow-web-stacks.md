# 428. Narrow web's deck cards have no View chip, no Stacks and no group price or cart

State: fixing | Owner: laptop | PR: - (pushed branches, hashes in the composer's log) | Build seen: a442e8f97 | Type: feature

## Report
Scotty: the web app below 900px (the deck's Cards screen) has no View chip and no Stacks, which the phone app has, and its List group headers (Commander, Creatures) show no price or cart, which the app's do (#620). Narrow web should match the app with the same shared pieces: the View chip with the same remembered setting, Stacks in touch mode (the app's two independent columns from #615, first tap reveals and floats, second opens the details sheet, a tap off settles), and the List headers' price and cart. Touch or pointer goes by the input (the hover and pointer media), not the width, so a narrow window with a mouse keeps hover reveals.

## Fix
laptop. components: `CardGroup` (the app's folding List group, moved out of zwiper) and `group_tail` (a group's price and cart, for a List header or a stack's chip) are shared, and `DeckStacks` draws its stack tails with it; `use_pointer` reads `(hover: hover) and (pointer: fine)`; touch stacks mark their columns `.deck-stacks-touch`, and the app's touch Stacks and tailed-header rules move from zwiper's `main.css` to `magic.css` under that class, so both clients load them. zwiper-web: the narrow Cards screen builds its groups from `deck_sections` as the wide page does, draws them as `CardGroup`s with their tails, adds the View chip (`CardsViewSwitch`, the stored choice) and in Stacks draws `DeckStacks` with `touch` from the input, the `DeckStackDrawer` and the group store dialog; a tap off the cards dismisses the reveal as in the app; the stacks' deal moves to `use_stacks_dealing`, shared with the wide page. The app uses the moved pieces with no change in behavior.

## Checked (headless Chrome, app.zwipe.net local, deck "Krenko goblins")
- 390 and 600, touch: View chip under Show (Art disabled in Stacks); two columns; a first tap reveals and floats the card, a second opens its details drawer, a tap off settles it.
- 390 and 600, List: every group header with its price and cart on the line under the title (Commander shows its cart only, the card has no price).
- 600 with a mouse: the stacks lay out by width with no touch class, and hover reveals and floats.

## Verification
-
