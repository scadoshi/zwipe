# zwiper-web layout

The breakpoint is 900px: at 900px and up the window is wide, below it the web app is one column. The web is panels at every width: the app's bare forms and bottom action bar do not carry over (the app itself is unchanged). Screens with three panels add the third at 1280px and up; between 900px and 1279px they show two. A top nav like zite's holds the Z mark (Home), Decks, Activity, Playstyle, Account, Config and Community, and below 900px folds behind a hamburger. Bottom sheets become side panels when they edit something on the page and centered modals when they are short forms or lists; dialogs stay modal, and the web uses as few as it can: where a link or a page does the job, it replaces the app's dialog.

Esc closes the top overlay (dialog, then modal, then side panel, then full overlay), browser back does the same for overlays the app puts on its back stack, Enter submits the focused form, and `/` focuses the screen's search or quick add field where there is one. Single-key shortcuts only fire while no text field has focus.

## Page grammar

The wide layout copies zite's home page (`zite/assets/style.css`). Screens move onto it one at a time, with the owner reviewing each; the ones below that say so are on it.

- The page is one centered column at most 74rem wide inside a 1.5rem gutter, and it starts at the top: the screen's 1rem top padding under the nav is the only gap, as zite's page sits 1rem under its nav. Nothing is centered vertically on a wide window.
- A screen with a hero puts it at the top of the page. The rest of its panels share two equal columns under it, each a stack one panel gap (1rem) apart, and the shorter column ends where its last panel does. Panels are ordered so the columns come out about even and related panels sit together.
- A hero that is a list of rows (a deck's Profile, Playstyle's Summary) stays 40rem wide and centered, since label and value would sit a page apart at full width.
- A form is its own panel at every width, as wide as the page (74rem) with its fields held to 26rem in its middle so an input never stretches across the window (below 900px it keeps the page's 1rem side gutter), with its actions in the panel's footer under the fields (below). A form is never folded into a hero or stretched across the columns, and a screen whose only actions are its form's has no action bar.
- Every input panel wears the kit panel's title bar at every width, zite's panel title over a full-bleed rule, named with the app's own title for its screen: `Login`, `Create profile`, `Reset password` (its sent panel too, with `Check your email` as its first line). A panel beside a form is titled after what it holds, such as Register's `Password`; it sits to the form's right on a wide window and under it below 900px. Create, Edit, Import and the change forms take their screen's title when they move onto the page grammar.
- A form panel's buttons sit in its footer at every width, the kit panel's action area: a full-bleed rule across the panel, then the app's compact buttons in one centered row, the primary first, wrapping only when they don't fit (Login: `Log in`, `Create profile`, `Forgot password`). Every input panel does this when it moves onto the page grammar.
- Below 900px the hero and the columns dissolve into one column in the app's order; the panels, their title bars and footers stay.
- In `web.css` the pieces are `.web-page`, `.web-hero` (with `.web-hero-narrow`), `.web-columns`, `.web-column`, `.web-form-panel`, `.web-form-actions` and `.web-auth-pair` (a form with a panel beside it), and in Rust `chrome::form_panel`'s `FormPanelHead` and `FormPanelActions`.

## Shared chrome

- The top nav is zite's bar, the kit's `NavBar`: the Z mark (zwipe-core's `logo::Z`, the app icon's art) on the left in `var(--logo)`, sized to stand as tall as a nav pill and linking Home, and on the right `Decks`, `Activity`, `Playstyle`, `Account`, `Config` and `Community` as pills. `Community` opens zite's Community page (Discord and email support), the same site so no `↗` and no new tab, in place of the app's `!` and its `Help & feedback` dialog. The current screen's pill is lit; screens reached from a deck (Deck view, Edit, Import, Export, the card screens) light `Decks`, the Commander maybeboard lights `Playstyle`, and Privacy policy and Changelog light `Account`.
- Below 900px the links fold behind the kit's hamburger and open as its dropdown under the bar; a link, a tap outside or a new route closes it. The kit folds its bar at 60rem, so `web.css` keeps it open from 900px.
- Logged out, the nav holds the mark, `Log in` and `Create profile` (the current one lit) and `Community`, which takes the place of the app's `!`.
- Under the nav there is no screen header at any width: the page starts right under the nav. The title stays for assistive tech and names the browser tab. A screen's `?` hint button sits at the right end of its action bar. Every screen with a `?` has an action bar.
- Buttons whose only job is to go somewhere (`Back`, `Back to login`, `Back to decks` and the like) go, since the nav and the browser's back do their job. Login keeps `Create profile` and `Forgot password` as buttons under `Log in`; Register and Forgot password have no `Log in` button, since the nav's `Log in` and the browser's back cover it. Every button that does something stays, and so does the `Back` that closes a screen's own sheet, picker or overlay. Screens drop theirs as they move onto the page grammar.
- The web app has no site footer. The Scryfall credit and the Fan Content notice live on zwipe.net and in the Privacy policy screen, as in the app. Privacy policy is open signed in or out: the public nav carries a `Privacy policy` pill, and signed in it is under Account with `Changelog` and `Website`. A screen scrolls as one column under the nav.
- A screen's action bar is the last thing in it and is pinned to the window's bottom edge at every width (`position: sticky; bottom: 0`), so its actions are always on screen: the app's bar, a top rule over the panel background with the buttons centered in the app's order. The content ends above it, so nothing stays hidden behind it. There is no toolbar under the nav.
- Toasts keep the app's kinds, text, durations, stack of three and click to expand. On a wide window they sit top right; below 900px top left as in the app.
- Inline `?` info buttons and the screen `?` open the same `Got it` dialogs as the app, centered. One-time hints fire on the same triggers.
- `More actions` sheets become a popover menu anchored to `More`, same buttons in the same order; a dialog opened from it closes the popover where the app closes the sheet and leaves it open where the app leaves the sheet open.
- Every `AlertDialog` stays a centered modal with its actions in order. From 900px up a dialog sizes to its content up to the page width, so a long hint spreads out instead of scrolling and a short confirmation stays compact; below 900px it keeps the app's width. Enter runs the non-danger primary action; danger actions never fire on Enter.
- Skeletons are drawn per panel, so side by side panels load and ghost on their own, and each skeleton keeps the app's ghost shapes.
- Card rows (`CardRow`) on every screen: hovering the compact line for a moment shows the card image beside the cursor; click still expands the row. `Image` opens the full `ImagePreview`, where Esc or a click closes it and the `Flip` control also answers the F key.
- Accordions with one section open at a time stay that way below 900px. On a wide window, screens with room show every section open at once (named per screen below) and drop the disclosure arrow there.
- Swipe surfaces (Add, Remove, Swipe select, Oracle tag examples) keep mouse drag on the card, since it works the same as touch, and add arrow keys plus four buttons under the card, one per direction the surface allows.

## Login

- On the page grammar with no hero: the page opens on the form panel, centered at the top under the nav. The counters stay on Home and Activity.
- `Log in`, `Create profile` and `Forgot password` sit in one row of compact buttons in the form panel's footer, `Log in` first. Enter in either field runs `Log in`.
- Below 900px it is the same panel in one column.

## Register

- On the page grammar with no hero: the four fields with their live error lines in their own form panel at the top under the nav.
- The `?` beside `Password` opens the `Password` hint, as in the app, so the form stays the only panel.
- `Create profile` sits in the form panel's footer. The app's `Back to login` goes; the nav's `Log in` covers it. Enter in any field runs `Create profile`. Success goes back to Login with the username filled in.
- Below 900px it is the same panel in one column.

## Forgot password

- On the page grammar with no hero, as Login: the `Email` field in its own form panel with `Send reset link` in its footer. The app's `Back to login` goes; the nav's `Log in` covers it.
- After a send, the `Check your email` panel takes the form panel's place at the same width, holding only its message.
- Enter runs `Send reset link`. Below 900px it is the same panel in one column.

## Update required

- Shown instead of everything else, with no nav, as in the app: the empty header bar, the pulsing `Update required` card, then a `Reload` button in place of the store link, since a web build updates by loading the page again.
- The layout is one centered column at every width, and its empty action bar is left out.

## Home

- On the page grammar. The hero card spans the top of the page (the mark left, the three counters and the `Live` and `Everybody`/`Only me` chips right). Under it `Color map` takes the left column and `Featured flavor` the right.
- The featured flavor panel is the app's: the quote, then the card's name and price, `Image` (the full preview), and in place of `Buy` the `TCGplayer ↗` and `Card Kingdom ↗` links that open in a new tab, as on zite.
- The app's buttons that only navigate go at every width, since the nav holds them: the action bar (`Decks`, `Account`, `Config`), `Activity` in the hero and `Playstyle` under the color map.
- Below 900px it is the phone stack: hero, color map, featured flavor.

## Activity

- On the page grammar. The counts panel is the hero. The left column holds the `Activity` section, with the heatmap unscrolled (a year fits) over `Totals`, and `Most skipped` under it; the right column holds `Card roles` over `Oracle tags`.
- All three accordion sections are open at once, so the shared `Show` rate chips appear above both charts and one click switches both, as in the app.
- Hovering a heatmap square shows the floating day chip the spec describes for hover devices; clicking a square still drives the readout for 4 seconds.
- Role and tag chips, `Examples`, and `Most skipped` rows (`Image`, `Printing`) work by click as in the app.
- Below 900px it is the phone accordion with `Activity` open.

## Playstyle (Suggested commanders)

- On the page grammar. `Summary` is the hero, 40rem wide. The left column holds the other profile sections (`Color map`, `Card roles`, `Oracle tags`), all open; the right column holds the `Show` row with `Art`, the `Suggested commanders` list and `Staples` under it.
- Rows expand inline on click with their footer (`Why this commander?`) and actions `Image`, `Printing`, `Create deck`, `Maybeboard`, `Remove`.
- `Swipe` opens the Swipe select overlay in suggested mode (see Edit deck); `More` is a popover with `Clear skips`, which opens the `Clear skips` dialog.
- Below 900px it is the phone column with the profile accordion, `Summary` back in it, and `Color map` open.

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
- `Change password` keeps the `?` beside its `Password` field for the rules, as Register does.
- `Change username`, `Change email` and `Change password` open as centered modals with the same fields and `Back` / `Save changes` footers; Enter runs `Save changes`. The `Change password` confirm dialog stacks on top, and Esc closes only the top one.
- `Resend` and the refresh icon button (with a `title` tooltip on hover) stay in the `Verification` row.
- `Log out` stays in the action bar and opens its dialog; `More` is a popover with `Delete account`, which opens the countdown dialog.
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
- `Swipe` opens Swipe select as a full-window overlay laid out like Add cards: card and its info on the left, the card details beside it, `Back`, `Filter`, `Refresh` in the action bar. → picks, ← skips, ↑ saves to the commander maybeboard (commander and partner modes), ↓ undoes. Its `Filter` opens the filter as a side panel inside the overlay.
- `Save` sits in the action bar; Enter in the deck name field runs it. Below 900px it is the phone form with full-screen pickers.

## Edit deck

- Same layout, pickers and Swipe select as Create deck; the action bar shows `Back` and, only when something changed, `Save changes`.
- The loading skeleton follows the two-column shape.
- Below 900px it is the phone form.

## Import

- On a wide window the left column holds the fixed controls (`From`, `Mode`, `Board`, the explainer line) and the decklist textarea or URL field; the right column holds the `Imported` and `Unresolved` results so the user can fix the paste while reading what failed, which the phone pushes below the input.
- Ctrl+Enter (Cmd+Enter on a Mac) runs the submit button (`Import` or `Replace`) when it is enabled.
- Below 900px the results sit under the input as in the app.

## Export

- On a wide window the `Export` chips (`Deck`, `Maybe`, `Side`) sit in a narrow left column and the read-only `Decklist` fills the right at full height.
- `Copy` stays in the action bar.
- Below 900px it is the phone stack.

## Deck view

- On the page grammar. The `Profile` card is the hero, 40rem wide. The left column holds `Budget`, `Tags`, `Distributions`, `Mana` and `Draw odds`; the right column holds `Warnings` over a cards column that lists the mainboard by type with the command zone on top, as compact rows that expand on click.
- Every section is open at once, so the phone's single-open accordion and its default `Budget` go away on a wide window; `Warnings` keeps its warning border and fix buttons (`Add cards`, `Import`, `Fix to {n}`, `Clear`, `Remove`).
- The action bar keeps `Edit`, `Cards`, `More`; the app's `Back` goes, as the nav's `Decks` covers it. `Cards` opens Deck cards for full editing; the cards column here is for reading. `More` is a popover with `Import cards`, `Export cards`, the `TCGplayer ↗` and `Card Kingdom ↗` links in place of `Buy deck`, `Clone deck`, `Share deck`, `Clear skips`, `Delete deck`, and the other dialogs are the app's.
- The `Draw odds` stepper `-` / `+` buttons also answer ← and → while the stepper has focus.
- Below 900px it is the phone screen: profile card, accordion with `Budget` open, no cards column.

## Add cards (swipe)

- At 1280px and up there are three columns: the card stack with `CardInfoDisplay` under it; the card details (the `CardDetailsSheet` content: cost, type, oracle text, keywords, card roles) for the top card, which the phone hides behind the eye button; and the filter as a docked panel. Between 900px and 1279px the filter becomes a side panel opened by `Filter`, and the card and details stay side by side.
- The `From` row (`Search`, `Maybeboard`) and `Synergy` sit above the card. The action bar keeps `Back`, `Filter`, `Refresh`; the eye button opens the full details sheet as a modal where `Printings` lives.
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
- The action bar keeps `Back`, `Add`, `Remove`, `Filter`, `Undo` (only with history) and `Reset filter` (only with a filter). The filter opens as a side panel over the detail pane.
- Below 900px it is the phone list with rows expanding inline.

## Remove cards

- Same columns as Add cards: card and info, details, docked filter at 1280px and up; the filter becomes a side panel between 900px and 1279px. The `Boards` row (`Main`, `Maybe`, `Side`, `All`) sits above the card.
- Keys: → removes, ← keeps, ↑ moves between main and maybeboard, ↓ undoes (Ctrl+Z too). F flips, P opens the read-only printing modal, R is `Refresh`. Four buttons under the card match the arrows.
- The action bar keeps `Back`, `Filter`, `Refresh` and the eye button.
- Below 900px it is the phone screen.

## Owner decisions (approved 2026-10-31)

- Buttons under swipe cards use the app's own action names for each mode: `Skip`, `Maybeboard`, `Undo`, `Add` on Add cards; `Keep`, `Move`, `Undo`, `Remove` on Remove; `Skip`, `Save`, `Undo`, `Pick` in Swipe select; `Next`, `Back one` in Examples.
- Update required shows `Reload` in place of the store link.
- `Buy` is a link per retailer (`TCGplayer ↗`, `Card Kingdom ↗`) opening in a new tab, as on zite, for a single card and for a deck; the app's `Buy` dialog goes.
- `Report a problem ↗` names the platform `Web` in the mail body.
- The one-time `filter` hint fires the first time the user opens a filter section, not when Add or Remove mounts with the filter docked.
- A command zone input clears its chosen card only when typing starts, never on a click.
- Deck view's cards column stays read-only; Deck cards is the editor.
- Commander maybeboard's `Back` follows browser history instead of pushing the deck list.
