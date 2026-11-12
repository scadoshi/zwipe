# 359. f and v gaps on Commander maybeboard and meld cards

State: fixing | Owner: scotland | PR: #559 | Build seen: live, web, after #554

## Report
surfer via composer, 2026-11-12, web:
- (a) `f` does nothing on Commander maybeboard (`/maybeboard`), though it has a Filter button.
- (b) `v` on a hovered meld row or a hovered meld stack card does nothing, and the stack image never shows the meld. The open-row case was #555.
- (c) On Add, `v` melds the card image, but the details panel keeps the front's text and its Meld toggle stays off.

Skipped on composer's word: the key swallowed right after Escape closes the filter.

## Cause
- (a) The maybeboard's key handler only knew `/`.
- (b) and (c): each image, details panel and row kept its own Meld state (`use_meld`'s local signal), so turning Meld on in one left the others off. `v` on a hovered card could only press a Meld button that was drawn, and a collapsed row or a stack card has none. The stack card draws its own `img` from the card and never asked about meld.

## Fix
scotland, #559:
- `MeldSource` holds the halves whose Meld is on (`on`, by Scryfall id), so a half's Meld is one switch wherever it shows: the card image, the details (panel, drawer, open row) and the stack card.
- `MeldToggle::of` builds a half's toggle outside a component, and `use_meld` is that over the context. The app and the web provide `on`.
- The stack card shows the melded image while Meld is on.
- `v` on a hovered one-faced meld half toggles its Meld directly (`HoverAction::Meld`), with its row open or not. A one-faced card that doesn't meld does nothing.
- On Add and the other swipe screens, `v` still presses the image's flip or Meld, and the details now follow it.
- Commander maybeboard's keys go through a `screen_key` map that takes `f` from `shortcuts::card_key`.

Tests:
- `components/tests/meld_row.rs`: with a half's Meld on, its details, its image and its stack card all show the melded card. This fails without the stack change.
- `hover_keys`: a hovered meld half gives `Meld`; a plain one-faced card gives nothing.
- `commander_maybeboard`: `f` maps to Filter exactly where `card_key` says so.

Headless on scotland:
- `f` on `/maybeboard` opens the Filter sheet.
- `v` on a hovered Bruna stack card shows Brisela on the stack (`target/shots-359/359-stack-meld.png`). The open row's Meld then reads on, and `v` on the hovered row turns it off.
