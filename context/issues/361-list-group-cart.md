# 361. The List's card groups have no cart

State: fixing | Owner: thinker | PR: - | Build seen: production, web (screenshot of the deck page List view)

## Report
Scotty, 2026-11-11, web: put the same per-group cart the stacks have (353) on each List group header too (Commander, Creatures (23), Enchantments (11), ...), so List and Stacks both have it: `CartButton` and `StoreDialog` with that group's cards, tokens out, at the right end of the header row.

## Cause
353 put the cart in the stack header only; the List's bands (`DeckBand`) had no slot for it.

## Fix
thinker: `DeckBand` passes an `accessory` to its `DeckFold`, whose header already holds one at its right end (Budget's currency chips use it). Each band with something to buy gets the cart there. The group buy state, its cards and its dialog move from `stacks.rs` into `deck/view/group_buy.rs`, shared by both views, and the dialog's copy becomes the group's ("Buy this group's cards from a retailer."). Golden `deck_band_with_cart`; `a_group_sends_its_cards_with_their_copies_and_leaves_tokens_out`. Headless at 900 and 1440: every band has its cart, the dialog names the group with both stores linking its cards, and the band stays open.
