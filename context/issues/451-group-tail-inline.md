# 451. A List group's price and cart sit on a line of their own under its title

State: fixing | Owner: laptop | PR: - (pushed branch, hash in the composer's log) | Build seen: release 9fe1c9363 | Type: feature

## Report
Scotty, phone web (app.zwipe.net, a deck's Cards, List, 390 wide): each group header puts its price chip and cart on a second line under `Creatures (24)`, the right side of the title row empty, Commander's lone cart too. Put them on the title row at the right edge, wrapping only when the title doesn't fit; the same in the app; the wide web stays unless it wastes the same space; the chips unchanged. droider confirmed it on phone web and in the app (main fce83f664, deck Surfer 399).

## Cause
`components/assets/magic.css`, `.card-group-header-tailed > .card-group-toggle` and `> .card-group-tail`: each had `flex: 1 0 100%`, so in the wrapping header each took a whole line and the tail always fell under the title, whatever the width. The app and the phone web both draw `CardGroup` with that sheet, which is why both show it; the wide web's groups are the deck page's own bands, which already end in the price and cart.

## Fix
laptop, in `magic.css`: the title takes `flex: 1 1 auto` and the tail `flex: none; margin-left: auto`, with the tail's under-title indent gone. The tail sits at the right end of the title's line, and wraps under it, still at the right edge, only when the line can't hold both. `components/tests/group_tail_inline.rs` fails on the old rules.

## Checked
- Headless Chrome, a shared Commander deck (Atraxa Superfriends, 100 cards), Cards, List: before and after at 390 and 360. After, Commander's cart and `Creatures (9) $59.01 cart` share the title's line; at 390 `Planeswalkers (13)` and `Enchantments (13)` are too long to share it with their price and cart and wrap, at the right edge.
- Wide (1280): the band headers already put the price and cart at the right end; unchanged.

## Verification
-
