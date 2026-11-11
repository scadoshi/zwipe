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
| `s` | | Synergy on or off |

Each key does exactly what its dock button or swipe does today, through the same action, so a keypress and a swipe can never disagree. A key whose action is unavailable (Undo with nothing to undo, Maybeboard on a screen without one) does nothing.

## Printings

← / → (or `h` / `l`) move between printings, Enter selects the one in the center, Esc closes.

## Everywhere

- Esc closes the open sheet or drawer.
- `?` opens the page's existing hint, the same one its `?` button opens.

## Swipe screens only

- `H` (Shift+H) opens the shortcuts sheet: the four directions with `hjkl`, `p`, `d`, `v`, `f`, `r`, `s`, Esc, `?` and `H`. Lowercase `h` stays Skip; with Caps Lock on, `h` opens the sheet instead (accepted). On every other screen Shift+H does nothing.

## Showing the shortcuts

No key labels on buttons and no settings toggle: they would clutter the screens for people who never use a keyboard. Instead, the swipe screens get one keyboard icon in the dock, on a computer's wide layout only (`(hover: hover) and (pointer: fine)` and the web's 900px layout switch), which opens the same shortcuts sheet as `H`. Shift+H works at any width.

## Rules

- Shortcuts are off while focus is in a text field, select or contenteditable, so typing never swipes.
- Modifier combos (Ctrl, Cmd, Alt) pass through to the browser.
- While a sheet is open, only its own keys and Esc work; the swipe keys don't reach the screen behind it.
- Keys show in the existing HintKey style in the shortcuts sheet; the sheet is the existing sheet component, nothing new.

## Not in v1

`/` to search, `n` for a new deck, deck page and decks list shortcuts.
