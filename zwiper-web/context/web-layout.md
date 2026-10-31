# zwiper-web layout

The breakpoint is 900px: at 900px and up the window is wide, below it the web app is the phone layout in one column. Screens with three panels add the third at 1280px and up; between 900px and 1279px they show two. A left sidebar holds Home, Decks, Activity, Playstyle, Account, Config and Help, and folds into a top bar with a menu button below 900px. Bottom sheets become side panels when they edit something on the page and centered modals when they are short forms or lists; dialogs stay modal.

Esc closes the top overlay (dialog, then modal, then side panel, then full overlay), browser back does the same for overlays the app puts on its back stack, Enter submits the focused form, and `/` focuses the screen's search or quick add field where there is one. Single-key shortcuts only fire while no text field has focus.

## Shared chrome

- The sidebar shows the six screens plus `Help`, which opens the same `Help & feedback` dialog as the app's `!` button. The current screen is highlighted; screens reached from a deck (Deck view, Edit, Import, Export, the card screens) highlight `Decks`. Logged-out screens have no sidebar.
- The header is the app's `ScreenHeader` at the top of the main area: title, then the `?` hint button on the right when the screen has one. The `!` moves into the sidebar as `Help`. Below 900px the top bar is menu button, title, `?`, and `Help` sits in the menu.
- On a wide window the action bar becomes a toolbar under the header, buttons in the app's order, so actions sit next to the content instead of at the bottom of a tall window. Below 900px it is pinned at the bottom as in the app. `Back` stays in the toolbar on every screen that has it.
- Toasts keep the app's kinds, text, durations, stack of three and click to expand. On a wide window they sit top right so they do not cover the sidebar; below 900px top left as in the app.
- Inline `?` info buttons and the screen `?` open the same `Got it` dialogs as the app, centered. One-time hints fire on the same triggers.
- `More actions` sheets become a popover menu anchored to `More`, same buttons in the same order; a dialog opened from it closes the popover where the app closes the sheet and leaves it open where the app leaves the sheet open.
- Every `AlertDialog` stays a centered modal with its actions in order. Enter runs the non-danger primary action; danger actions never fire on Enter.
- Skeletons are drawn per panel, so side by side panels load and ghost on their own, and each skeleton keeps the app's ghost shapes.
- Card rows (`CardRow`) on every screen: hovering the compact line for a moment shows the card image beside the cursor; click still expands the row. `Image` opens the full `ImagePreview`, where Esc or a click closes it and the `Flip` control also answers the F key.
- Accordions with one section open at a time stay that way below 900px. On a wide window, screens with room show every section open at once (named per screen below) and drop the disclosure arrow there.
- Swipe surfaces (Add, Remove, Swipe select, Oracle tag examples) keep mouse drag on the card, since it works the same as touch, and add arrow keys plus four buttons under the card, one per direction the surface allows.

## Login

- On a wide window the `HomeHero` panel (logo, counters, `Everybody`, `Live`) sits on the left and the form on the right, both vertically centered.
- Enter in either field runs `Log in`; `Create profile` and `Forgot password` stay as buttons under the form. Tab moves identity, password, `Show`, then the buttons.
- Below 900px the hero stacks above the form as in the app.

## Register

- Same split as Login: hero left, the four fields with their live error lines right, the `?` beside `Password` opening the `Password` hint.
- Enter in any field runs `Create profile`; `Back to login` is a button under the form.
- Below 900px it stacks like the app.

## Forgot password

- Same split: hero left, the `Email` field (or the `Check your email` panel after success) right.
- Enter runs `Send reset link`; `Back to login` sits beside it.
- Below 900px it stacks like the app.

## Update required

- Shown instead of everything else, with no sidebar, as in the app: the empty header bar, the pulsing `Update required` card, then a `Reload` button in place of the store link, since a web build updates by loading the page again.
- The layout is one centered column at every width.

## Home

- On a wide window the hero panel spans the top (logo left, the three counters and the `Live`, `Everybody`/`Only me`, `Activity` chips right). Below it the `Color map` panel and the `Featured flavor` panel sit side by side.
- The featured flavor panel shows the card image next to the quote, which the phone keeps behind `Image`; `Image` still opens the full preview and `Buy` still opens the `Buy card` dialog.
- The app's action bar (`Decks`, `Account`, `Config`) is dropped on a wide window because the sidebar holds the same three; below 900px it stays.
- `Playstyle` under the color map and `Activity` in the hero stay as buttons.
- Below 900px it is the phone stack: hero, color map, featured flavor.

## Activity

- On a wide window the counts panel sits on top, then the `Activity` section full width with the heatmap unscrolled (a year fits) and `Totals` beside it. Under that, `Card roles` and `Oracle tags` sit side by side, both open, with `Most skipped` full width below.
- All three accordion sections are open at once, so the shared `Show` rate chips appear above both charts and one click switches both, as in the app.
- Hovering a heatmap square shows the floating day chip the spec describes for hover devices; clicking a square still drives the readout for 4 seconds.
- Role and tag chips, `Examples`, and `Most skipped` rows (`Image`, `Printing`) work by click as in the app.
- Below 900px it is the phone accordion with `Activity` open.

## Playstyle (Suggested commanders)

- On a wide window the left column holds the profile sections (`Color map`, `Summary`, `Card roles`, `Oracle tags`) all open, stacked. The right column holds the `Show` row with `Art`, the `Suggested commanders` list and `Staples` under it.
- Rows expand inline on click with their footer (`Why this commander?`) and actions `Image`, `Printing`, `Create deck`, `Maybeboard`, `Remove`.
- `Swipe` opens the Swipe select overlay in suggested mode (see Edit deck); `More` is a popover with `Clear skips`, which opens the `Clear skips` dialog.
- Below 900px it is the phone column with the profile accordion and `Color map` open.

## Commander maybeboard

- On a wide window the left column holds `Quick add` with its floating results, the `Show` row (`All`, five color pips, `Art`) and the `Commanders` list. The filter sheet opens as a right side panel beside the list, so the list re-filters in view on `Apply`.
- `/` focuses `Quick add`; arrow keys move through the result chips and Enter adds the highlighted one.
- Rows expand on click with `Image`, `Printing`, `Create deck`, `Remove`. `Swipe` opens Swipe select in commander mode; `More` is a popover with `Clear maybeboard`.
- Below 900px it is the phone layout with the filter as a bottom sheet.

## Oracle tag dictionary and examples

- The dictionary opens as a large centered modal over its host (the oracle tags picker or the filter panel). On a wide window the letter rail becomes a vertical column on the left and the list fills the rest; each row keeps `Examples` and `Use`.
- `/` focuses the search; with search empty and unfocused, typing a letter jumps to that letter. Esc is `Back`.
- Examples opens stacked on top as a modal with the card image on the left and its details (the `CardDetailsSheet` content) shown on the right instead of behind the eye button. ← goes to the next card and ↓ goes back one, matching the app's left and down swipes; two buttons under the card do the same.
- Below 900px both are full-screen overlays as in the app.

## Account

- On a wide window the `Account` and `About` lists sit side by side.
- `Change username`, `Change email` and `Change password` open as centered modals with the same fields and `Back` / `Save changes` footers; Enter runs `Save changes`. The `Change password` confirm dialog stacks on top, and Esc closes only the top one.
- `Resend` and the refresh icon button (with a `title` tooltip on hover) stay in the `Verification` row.
- `Log out` stays in the toolbar and opens its dialog; `More` is a popover with `Delete account`, which opens the countdown dialog.
- Below 900px the two lists stack and the sheets slide up as in the app.

## Config

- On a wide window the `Appearance` and `Cards` lists sit side by side. The `App icon` row is left out, as the spec allows where the platform has no app icon.
- `Themes` opens as a right side panel so the live preview repaints the whole page beside it. Clicking a row previews it, ↑ and ↓ move the preview through the list, Enter is `Save` and Esc is `Back` (restore and `Theme unchanged`).
- `Exceptions` opens as a centered modal with the `N selected` row and the franchise chips; `Back` and `Save` behave as in the app.
- `Dark`/`Light`, `Theme`/`Natural` and `Hide`/`Show` stay one-click toggles in their rows.
- Below 900px the lists stack and both sheets slide up.

## Privacy policy

- On a wide window a sticky contents list on the left names the section headings (`Data we collect` through `Contact`) and scrolls to each; the section cards sit in one reading column on the right.
- Below 900px it is the column of cards alone.

## Changelog

- On a wide window the chip rows (`Latest`, years, months, minor versions) become a sticky filter column on the left, keeping the app's levels and the click-the-deepest-chip-to-go-up rule. Release cards fill the right column.
- Below 900px the chip rows sit above the cards as in the app.

## Deck list

- On a wide window the `Group by` and `Show` rows sit above the list, and the `Tags` row stays unfolded since there is room. Deck rows lay out two per line inside each group, with the art crops at a larger size.
- Clicking a row opens the deck; group headers collapse on click as in the app.
- `Create` keeps its verify-email guard; `More` is a popover with `Commander maybeboard` and `Playstyle`.
- Below 900px it is the phone list with the `Tags` fold chip.

## Create deck

- On a wide window the form splits into two columns: `Profile` (name, format, command zone fields, power level) on the left, `Budget` above `Tags` on the right.
- The `Format`, `Deck tags` and `Oracle tags` pickers open as a right side panel over the right column, with the definition bar on top, the chip grid below and `Back` / `Save` at the panel foot. The form stays visible, so the format cascade clearing the commander shows as it happens. `/` focuses the picker's search; Esc is `Back`.
- Command zone fields: typing shows the floating results; ↑ and ↓ move through them and Enter picks the highlighted card. `Filter` and `Swipe` chips stay in the label row.
- `Swipe` opens Swipe select as a full-window overlay laid out like Add cards: card and its info on the left, the card details beside it, `Back`, `Filter`, `Refresh` in the toolbar. → picks, ← skips, ↑ saves to the commander maybeboard (commander and partner modes), ↓ undoes. Its `Filter` opens the filter as a side panel inside the overlay.
- `Save` sits in the toolbar; Enter in the deck name field runs it. Below 900px it is the phone form with full-screen pickers.

## Edit deck

- Same layout, pickers and Swipe select as Create deck; the toolbar shows `Back` and, only when something changed, `Save changes`.
- The loading skeleton follows the two-column shape.
- Below 900px it is the phone form.

## Import

- On a wide window the left column holds the fixed controls (`From`, `Mode`, `Board`, the explainer line) and the decklist textarea or URL field; the right column holds the `Imported` and `Unresolved` results so the user can fix the paste while reading what failed, which the phone pushes below the input.
- Ctrl+Enter (Cmd+Enter on a Mac) runs the submit button (`Import` or `Replace`) when it is enabled.
- Below 900px the results sit under the input as in the app.

## Export

- On a wide window the `Export` chips (`Deck`, `Maybe`, `Side`) sit in a narrow left column and the read-only `Decklist` fills the right at full height.
- `Copy` stays in the toolbar.
- Below 900px it is the phone stack.

## Deck view

- At 1280px and up there are three columns: `Profile` with `Budget` and `Tags` under it; the stats (`Distributions`, `Mana`, `Draw odds`, `Warnings`); and a cards column listing the mainboard by type with the command zone on top, as compact rows that expand on click. Between 900px and 1279px the stats move under the profile in the left column and the cards column stays on the right.
- Every section is open at once, so the phone's single-open accordion and its default `Budget` go away on a wide window; `Warnings` keeps its warning border and fix buttons (`Add cards`, `Import`, `Fix to {n}`, `Clear`, `Remove`).
- The toolbar keeps `Back`, `Edit`, `Cards`, `More`. `Cards` opens Deck cards for full editing; the cards column here is for reading. `More` is a popover with `Import cards`, `Export cards`, `Buy deck`, `Clone deck`, `Share deck`, `Clear skips`, `Delete deck`, and their dialogs are the app's.
- The `Draw odds` stepper `-` / `+` buttons also answer ← and → while the stepper has focus.
- Below 900px it is the phone screen: profile card, accordion with `Budget` open, no cards column.

## Add cards (swipe)

- At 1280px and up there are three columns: the card stack with `CardInfoDisplay` under it; the card details (the `CardDetailsSheet` content: cost, type, oracle text, keywords, card roles) for the top card, which the phone hides behind the eye button; and the filter as a docked panel. Between 900px and 1279px the filter becomes a side panel opened by `Filter`, and the card and details stay side by side.
- The `From` row (`Search`, `Maybeboard`) and `Synergy` sit above the card. The toolbar keeps `Back`, `Filter`, `Refresh`; the eye button opens the full details sheet as a modal where `Printings` lives.
- Keys: → adds, ← skips, ↑ sends to the maybeboard, ↓ undoes (Ctrl+Z also undoes). F flips a double-faced card, P opens the printing modal, R is `Refresh`. Four buttons under the card do the same as the arrows, and mouse drag still works.
- The printing sheet opens as a centered modal with the carousel; ← and → step printings while it is open, Enter is `Save`, Esc is `Back`.
- At the end of results, ↓ (or the undo button) steps back into the pile in place of the phone's down-pull.
- Below 900px it is the phone screen: card, info, filter and details as bottom sheets.

## Card filter sheet

- Where docked (Add and Remove at 1280px and up) it is a right column titled `Filter` with its `?`, the accordion, and `Back`, `Apply`, `Reset` at its foot. Elsewhere on a wide window it opens as a right side panel with the same parts.
- Draft and apply rules stay: edits are a draft until `Apply`; `Back` restores the draft and toasts `Filter unchanged`. When docked, `Back` reverts but the panel stays, and leaving the screen with an unapplied draft reverts it the same way.
- Section `×` buttons, the Any/All, Exact/Range and Include/Exclude chips, and picker result lists work by click; in a picker, ↑ and ↓ move through results and Enter picks. The `Dictionary` chips open the dictionary modal.
- Arrow keys typed in a filter field never swipe the card.
- Below 900px it is the app's bottom sheet.

## Deck cards view

- At 1280px and up there are three columns: a left column with the header block (name, format, power, tags), the featured cards, `Quick add` and the `Group by`, `Boards`, `Show` rows; the card groups in the middle; and a detail pane on the right showing the selected row's image, `CardDetails` and its actions (`-` / `+`, `Printing`, `Star`/`Unstar`, the move buttons, `Flip`, `Image`). Between 900px and 1279px the controls sit above the groups and the detail pane stays on the right.
- Clicking a row selects it into the detail pane instead of expanding it inline; one row at a time as in the app. With `Maybe` or `Side` on, their groups list above the mainboard as in the app.
- Keys: `/` focuses `Quick add`, ↑ and ↓ move the selected row, `+` and `-` change its quantity (the app's debounced writes still apply), Ctrl+Z is `Undo`. F flips the selected card everywhere a card shows, so `Filter` has no key here.
- Drag adds one thing here: dragging a row onto the `Maybeboard`, `Sideboard` or a mainboard group header moves it to that board, the same as the move buttons, with the same toast.
- The toolbar keeps `Back`, `Add`, `Remove`, `Filter`, `Undo` (only with history) and `Reset filter` (only with a filter). The filter opens as a side panel over the detail pane.
- Below 900px it is the phone list with rows expanding inline.

## Remove cards

- Same columns as Add cards: card and info, details, docked filter at 1280px and up; the filter becomes a side panel between 900px and 1279px. The `Boards` row (`Main`, `Maybe`, `Side`, `All`) sits above the card.
- Keys: → removes, ← keeps, ↑ moves between main and maybeboard, ↓ undoes (Ctrl+Z too). F flips, P opens the read-only printing modal, R is `Refresh`. Four buttons under the card match the arrows.
- The toolbar keeps `Back`, `Filter`, `Refresh` and the eye button.
- Below 900px it is the phone screen.

## Owner decisions (approved 2026-10-31)

- Buttons under swipe cards use the app's own action names for each mode: `Skip`, `Maybeboard`, `Undo`, `Add` on Add cards; `Keep`, `Move`, `Undo`, `Remove` on Remove; `Skip`, `Save`, `Undo`, `Pick` in Swipe select; `Next`, `Back one` in Examples.
- Update required shows `Reload` in place of the store link.
- `Report a problem ↗` names the platform `Web` in the mail body.
- The one-time `filter` hint fires the first time the user opens a filter section, not when Add or Remove mounts with the filter docked.
- A command zone input clears its chosen card only when typing starts, never on a click.
- Deck view's cards column stays read-only; Deck cards is the editor.
- Commander maybeboard's `Back` follows browser history instead of pushing the deck list.
