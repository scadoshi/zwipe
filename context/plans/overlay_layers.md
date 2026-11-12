# Overlay layers

Issue 424. Which overlay sits on top when two are open, which ones dim the page, and how Esc and Back pick the one to close.

## The bug

Web deck page, Stacks, 1450 wide: with a card's details drawer open on the right, opening Themes from the nav leaves the drawer on top. Every sheet is z-index 200 (backdrop) and 201 (sheet), so two open sheets tie and DOM order decides. Themes loses outright because its sheet renders inside `.nav-wrapper`, which is sticky at z-index 101: sticky makes it a stacking context, and nothing inside it can rise above a root-level 201.

Other pairs are held apart today by hand-picked numbers that only cover the pairs someone hit: `.sheet-over` (204/205, Printings over the details drawer), `.web-flow` (204/205, the wide Printings), `.web-shortcuts-layer` (240/241), `.web-dict-sheet` (220/221), `.dict-overlay` (220), `.examples-overlay` (230), `.image-preview-container` (202), `.swipe-select-screen` (210).

## The rule

From Scotty:

- An overlay opened later sits over one opened earlier, whatever kind each is. The latest wins.
- "On top" and "dims" are separate choices, made per sheet (table below).
- Live-preview sheets (Themes, and the card filter, whose results update live) do not dim on the web, so the page shows the change behind them. A click outside closes them; it never reaches the page.
- Decision sheets (confirms, destructive actions, forms you would lose) dim and block.
- On a phone every sheet, drawer and bottom sheet dims, Themes included. The no-dim rule is web only (from the kit's modal width).
- Menus and dropdowns (the phone and narrow hamburger nav menu, popover menus) never dim, on any platform.
- Esc and the OS back close the topmost overlay first. That is already the order of orangui's `OverlayBackStack`; the layering makes what you see match it.

## The approach

One order, the one that already exists. orangui's `OverlayBackStack` holds every open overlay in the order it opened (each `BottomSheet` and `AlertDialog` registers through `use_overlay_back_action`), and `close_top` already closes the last one. The stack gains a read of each entry's place in that order, and the overlay draws its z-index from it. No second registry.

- **Tiers.** Two, plus toasts: overlays (every sheet, drawer, pane and full-screen overlay) from 200, and dialogs (`AlertDialog`, hints) from 1000. Toasts stay at 9999. Within a tier, z-index is the tier's base plus twice the overlay's place in the stack: its backdrop takes the even number and its sheet the odd one, so the latest backdrop dims everything opened before it, panes included.
- **Why no separate pane tier.** A decision sheet blocks the page, so nothing opens over it except from inside it. A live-preview sheet closes on an outside click, so nothing opens behind it. Open order alone gives the right answer in every pair in the inventory.
- **Where it lands.** `use_overlay_back_action` returns the overlay's place (a `Memo<Option<usize>>`, `None` while closed or with no stack, as on zwipe.net). `BottomSheet` and `AlertDialog` write it as `--overlay-z` on their backdrop and panel, and the kit's CSS reads `z-index: var(--overlay-z, 200)` and `calc(var(--overlay-z, 200) + 1)`, so a host with no stack keeps today's numbers. Overlays that are not a `BottomSheet` (the examples and dictionary overlays, the image preview, swipe select) take the same hook and variable.
- **The nav trap.** A sheet inside the nav cannot outrank anything while the nav is its stacking context. While a sheet inside it is shown, the nav takes the open sheet's z-index (`--overlay-z` written on the nav by `ThemePicker`, read by `.nav-wrapper:has(.bottom-sheet.show)`), so the whole nav rises with the sheet to its place in the order.
- **Dimming.** `BottomSheet` takes `dim: SheetDim`: `Always` (the default, today's behavior) or `Phone` (dims below the kit's modal width only; above it the backdrop is transparent but still takes the click that closes the sheet). Themes and the card filter pass `Phone`.
- **The hand-picked numbers go.** `.sheet-over`, `.web-flow`, `.web-shortcuts-layer`, `.web-dict-sheet`, `.dict-overlay`, `.examples-overlay`, `.image-preview-container` and `.swipe-select-screen` drop their z-index rules; open order covers them.
- **Esc and Back.** Unchanged: both already call `close_top`. Issue 415's order (the composer's note) stays as it is.

## Inventory

Every overlay that can be open with another, on the web (W) and in the app (A). "Dims" is the backdrop above the kit's modal width on the web; on a phone, and in the app, everything dims.

| Overlay | Where | Kind | Dims (web, wide) | Phone and app |
|---|---|---|---|---|
| Themes (`ThemePicker`, nav) | W A | live preview | no, outside click closes | dims |
| Card filter (`DrawerSheet`, `CardFilterPanel`) | W A | live preview | no, outside click closes | dims |
| Card details drawer / Stacks drawer (`CardDetailsSheet`, `card-details-drawer`) | W A | pane | no (today: transparent backdrop) | dims |
| Printings, narrow and wide (`PrintingSheet`, `CoverFlowSheet`) | W A | sheet | yes | dims |
| Oracle tag examples (`oracle_tag_examples`, `.examples-overlay`) | W A | sheet | yes | dims |
| Oracle tag dictionary (`.web-dict-sheet`, `.dict-overlay`) | W A | sheet | yes | dims |
| Shortcuts sheet (`ShortcutsSheet`) | W | sheet | yes | n/a |
| Image preview (`.image-preview-container`) | W A | full screen | yes | dims |
| Swipe select (`.swipe-select-screen`) | W A | full screen | yes | dims |
| Pickers in the deck form (format, tags, oracle tags) | W A | sheet | yes | dims |
| Buy (`buy.rs`) | W A | decision | yes, blocks | dims |
| Deck edit, import, clone | W A | decision (form) | yes, blocks | dims |
| Account: email, password, username, verification, delete | W A | decision | yes, blocks | dims |
| Sign-in prompt, log out, leave guard (`web-leave-dialog`) | W A | dialog | yes, blocks | dims |
| Hints and the `?` dialogs (`ContentHintDialog`, one-time hints) | W A | dialog | yes | dims |
| Commander maybeboard, playstyle, exceptions, app icon, Universes Beyond sheets | W A | sheet | yes | dims |
| Quick add and search results (`.search-float-results`) | W A | inline float | n/a, closes on blur | n/a |
| Nav menu (hamburger) and popover menus (`.nav-dropdown-*`, `.nav-backdrop`) | W A | menu | no | no: menus never dim, on any platform |
| Toasts | W A | toast | n/a, always on top | on top |

## Order of work

1. orangui: the stack's place read, `use_overlay_back_action` returning it, `--overlay-z` in `BottomSheet` and `AlertDialog`, `SheetDim`, `ThemePicker` lifting the nav, the kit CSS. Tests for the order (open A then B: B above; close B: A back; reopen A: A above).
2. zwipe: bump orangui, pass `SheetDim::Phone` for Themes and the filter, move the custom overlays onto the hook, delete the hand-picked z-indexes, regenerate zwiper-web's sheets.
3. Check headless on the web at 1450 and 390: drawer then Themes (Themes on top, page undimmed, outside click closes it), Themes then a card (closes Themes first), drawer then Printings then Esc twice, filter then a card, a dialog over a sheet, the leave guard over everything but toasts.
