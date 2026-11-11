# Keyboard shortcuts (web, v1)

Decided with Scotty on 2026-11-11. Web only: phones have no keyboard. Keep it to the basics on the swipe screens; anything else waits until people use these.

## Swipe screens (Add, Remove, Commander swipe)

| Key | Vim | Does |
|---|---|---|
| → | `l` | Add (swipe right) |
| ← | `h` | Skip (swipe left) |
| ↑ | `k` | Maybeboard (swipe up) |
| ↓ | `j` | Undo (swipe down) |
| `p` | | Open Printings |
| `d` | | Card details |
| `v` | | Flip the card to its other face ("view the back"), for a card with two |
| `f` | | Filter |
| `r` | | Refresh the pile |
| `s` | | Synergy on or off (Add only; Remove and Commander swipe have no Synergy) |

Each key does exactly what its dock button or swipe does today, through the same action, so a keypress and a swipe can never disagree. A key whose action is unavailable (Undo with nothing to undo, Maybeboard on a screen without one) does nothing.

## Deck cards screen (List and Stacks)

Each key acts on one card: the card under the cursor (a list row or a stack card), else the card whose details are open (the open list row, or the card in the Stacks view's details drawer). With neither, it does nothing. It does exactly what that card's own button in its details does, through the same call.

| Key | Does |
|---|---|
| `p` | Open Printings for the card (a command zone card keeps its slot; a token has none, so nothing) |
| `i` | Open its image, as its Image button does |
| `v` | Flip it to its other face, as its Flip button does; nothing for a single-faced card |
| `m` | Move it to the maybeboard, as the details' To maybeboard button does |
| `s` | Move it to the sideboard, as the details' To sideboard button does |
| `S` (Shift+S) | Star or unstar it, as the details' Star button does, cap included: a fourth star is refused with the server's "This deck already has 3 MVPs" toast and rolls back |
| `-` | One copy fewer, as the stepper's `-` does: at one copy it removes the card, with Undo |
| `+` (or `=` without Shift) | One copy more, as the stepper's `+` does, with no cap |

A key whose button the card doesn't have does nothing: `m` on a maybeboard card, `s` on a sideboard card, Shift+S off the main board, `m`, `s`, `-` and `+` on a command zone card or a token, `v` on a single-faced card or a stack card whose details aren't open (a stack card shows no buttons of its own), and `i` anywhere in the Stacks view, whose cards and details drawer show no Image button. `s` here is Sideboard; Synergy's `s` belongs to Add.

These keys also work while the Stacks view's card details drawer is open, the one sheet they pass through. Every other key waits for it to close, and every other sheet (Filter, Printings, the shortcuts sheet) and every dialog keeps all keys from the screen as usual.

`H` (Shift+H) opens the shortcuts sheet with this screen's keys only, under the line "Hover or open a card, then press its key": `p`, `i`, `v`, `m`, `s`, Shift+S, `-`, `+`, then Esc, `?` and `H`. The screen has no dock, so no keyboard button.

## Printings

← / → (or `h` / `l`) move between printings, Enter selects the one in the center, Esc closes.

## Everywhere

- Esc closes the open sheet or drawer.
- `?` opens the page's existing hint, the same one its `?` button opens.

## The shortcuts sheet

- `H` (Shift+H) opens the shortcuts sheet on the swipe screens and the deck cards screen, listing only that screen's keys. On a swipe screen: the four directions with `hjkl`, named as that screen's dock names them, `p`, `d`, `v`, `f`, `r`, `s` where the screen has Synergy, Esc, `?` and `H`. The sheet lists only keys the screen answers. Lowercase `h` stays Skip; with Caps Lock on, `h` opens the sheet instead (accepted). On every other screen Shift+H does nothing.

## Showing the shortcuts

No key labels on buttons and no settings toggle: they would clutter the screens for people who never use a keyboard. Instead, the swipe screens get one keyboard icon in the dock, on a computer's wide layout only (`(hover: hover) and (pointer: fine)` and the web's 900px layout switch), which opens the same shortcuts sheet as `H`. Shift+H works at any width.

## Rules

- Shortcuts are off while focus is in a text field, select or contenteditable, so typing never swipes.
- Modifier combos (Ctrl, Cmd, Alt) pass through to the browser.
- While a sheet is open, only its own keys and Esc work; the swipe keys don't reach the screen behind it.
- Keys show in the existing HintKey style in the shortcuts sheet; the sheet is the existing sheet component, nothing new.

## Not in v1

`/` to search, `n` for a new deck, decks list shortcuts.
