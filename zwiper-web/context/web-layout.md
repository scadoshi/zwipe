# zwiper-web layout

The breakpoint is 900px: at 900px and up the window is wide and the web is panels on zite's page grammar; below it the web is the app, as `context/plans/unified_ui.md` lays it out. Screens with three panels add the third at 1280px and up; between 900px and 1279px they show two. A top nav like zite's holds the Z mark (Home), Decks, Activity, Playstyle, Account, Settings and Community. Bottom sheets become side panels when they edit something on the page and centered modals when they are short forms or lists; dialogs stay modal, and the web uses as few as it can: where a link or a page does the job, it replaces the app's dialog.

Below 900px every screen is the app's: the app's header (the back arrow on every screen but Home and the sign-in screens, the title with its `?`, the hamburger) that slides away on a scroll down, the app's menu dropping from it, no bars and no footers anywhere, a screen's one or two main actions floating in the dock at the bottom right, the swipe screens' pile chips beside the dock, pickers and filters as bottom sheets that close from an × or a drag down with Apply or Save as the dock's checkmark. The rules on actions hold on a wide window too: no action bar, no More sheet, no form footers, no navigation-only buttons. The per-screen sections below describe the wide layouts, which are the web's own.

Esc closes the top overlay (dialog, then modal, then side panel, then full overlay), browser back does the same for overlays the app puts on its back stack, Enter submits the focused form (and, with nothing focused, the page's form panel when it has exactly one), and `/` focuses the screen's search or quick add field where there is one. Single-key shortcuts only fire while no text field has focus.

## Page grammar

The wide layout copies zite's home page (`zite/assets/style.css`). Screens move onto it one at a time, with the owner reviewing each; the ones below that say so are on it.

- The page is one centered column at most 74rem wide inside a 1.5rem gutter, and it starts at the top: the screen's 1rem top padding under the nav is the only gap, as zite's page sits 1rem under its nav. Nothing is centered vertically on a wide window.
- A screen with a hero puts it at the top of the page. The rest of its panels share two equal columns under it, each a stack one panel gap (1rem) apart, and the shorter column ends where its last panel does. Panels are ordered so the columns come out about even and related panels sit together.
- A hero that is a list of rows (a deck's Profile, Playstyle's Summary) stays 40rem wide and centered, since label and value would sit a page apart at full width.
- A form is its own panel at every width, as wide as the page (74rem) with its fields held to 26rem in its middle so an input never stretches across the window (below 900px it keeps the page's 1rem side gutter), with its submit full width under the last field, as in the app. A form is never folded into a hero or stretched across the columns.
- Every input panel wears the kit panel's title bar at every width, zite's panel title over a full-bleed rule, named with the app's own title for its screen: `Login`, `Create profile`, `Reset password` (its sent panel too, with `Check your email` as its first line). A panel beside a form is titled after what it holds, such as Register's `Password`; it sits to the form's right on a wide window and under it below 900px. Create, Edit, Import and the change forms take their screen's title when they move onto the page grammar.
- A form panel has no footer. The sign-in forms put the submit (`Log in`, `Create profile`, `Send`) full width under the last field and the links to the other sign-in screens as chips in one centered row beneath it. The deck form's Save or Create is the dock's checkmark, Import's import and Export's Copy the dock's icon.
- Below 900px the hero and the columns dissolve into one column in the app's order; the panels and their title bars stay.
- In `web.css` the pieces are `.web-page`, `.web-hero` (with `.web-hero-narrow`), `.web-columns`, `.web-column`, `.web-form-panel` and `.web-auth-pair` (a form with a panel beside it), and in Rust `chrome::form_panel`'s `FormPanelHead`.

## Shared chrome

- The top nav is zite's bar, the kit's `NavBar`: the Z mark (zwipe-core's `logo::Z`, the app icon's art) on the left in `var(--logo)`, sized to stand as tall as a nav pill and linking Home, and on the right `Decks`, `Activity`, `Playstyle`, `Account`, `Settings` and `Community` as pills. `Community` opens zite's Community page (Discord and email support), the same site so no `↗` and no new tab, in place of the app's `!` and its `Help & feedback` dialog. The current screen's pill is lit; screens reached from a deck (Deck view, Edit, Import, Export, the card screens) light `Decks`, the Commander maybeboard lights `Playstyle`, and Privacy policy lights About.
- Below 900px the bar is the app's header, with no Z mark: the back arrow in the left corner, the title and its `?` in the middle, the hamburger on the right. The menu drops from it as the app's: Decks, Playstyle, Activity and Commander maybeboard as rows, the current one lit, then one footer row with the `!` for Help and feedback, Home, Account, Settings and About as icons, the theme pair (a palette for the Themes sheet and the sun or moon that flips the mode) and a small Log out. A tap on an entry, a tap outside or a new route closes it. The kit folds its bar at 60rem, so `web.css` keeps it open from 900px.
- Logged out, the nav holds the mark, `Log in` and `Create profile` (the current one lit) and `Community`, which takes the place of the app's `!`.
- Page head. On a wide window a screen with a description opens with zite's page header: a panel at the top of the page with the screen's title in its title bar, the screen's `?` at its right end, and a one-line description of what the screen does under a rule, one panel gap above the panels under it. Below 900px the page head hides and the title sits centered in the folded nav between the mark and the hamburger, with the `?` right after it. Every screen has a description except Home (its hero introduces it) and Login, Create profile and Reset password (their form panel is titled after the screen); those still put their title in the folded nav. On a wide window a screen without a page head keeps its `?` on its own line at the page's end, at the right. The title also names the browser tab.
- Buttons whose only job is to go somewhere go. Back is the nav's arrow at every width (owner, 2026-11-04: the wide layout has no bars left to hold it), beside the browser's own. The sign-in screens link each other with chips under the submit: Login to Create profile and Forgot password, the other two back to Login. An overlay (Swipe select, Examples, the dictionary, a deck picker) closes from the back arrow in its header's corner; a sheet from its × or a drag down. Every button that does something stays.
- The web app has no site footer. The Scryfall credit and the Fan Content notice live on zwipe.net and in the Privacy policy screen, as in the app. Privacy policy is open signed in or out: the public nav carries a `Privacy policy` pill, and signed in it is on About with `Website` and the changelog. A screen scrolls as one column under the nav.
- No screen has a bar at its end. A screen's one or two main actions float in the action dock, a squircle bar at the bottom right, as icons where the symbol is standard (+, −, the funnel, undo, refresh, a checkmark, the eye, two sheets for Copy, a paper plane for Send, chain links for Synergy); the deck view's Cards floats there as a chip, the maybeboard's Filter too. Less used actions sit inline in panels, destructive ones in red in a Danger zone panel at the page's end, and the page leaves room under them so the dock never covers the last row. There is no toolbar under the nav.
- Toasts keep the app's kinds, text, durations, stack of three and click to expand. On a wide window they sit top right; below 900px top left as in the app.
- Inline `?` info buttons and the screen `?` open the same `Got it` dialogs as the app, centered. One-time hints fire on the same triggers.
- Nothing hides behind More. There is no More sheet and no More popover: each action one held sits inline where it belongs, the destructive ones in a Danger zone panel.
- Every `AlertDialog` stays a centered modal with its actions in order. From 900px up a dialog sizes to its content up to the page width, so a long hint spreads out instead of scrolling and a short confirmation stays compact; below 900px it keeps the app's width. Enter runs the non-danger primary action; danger actions never fire on Enter.
- Skeletons are drawn per panel, so side by side panels load and ghost on their own, and each skeleton keeps the app's ghost shapes.
- Card rows (`CardRow`) on every screen: hovering the compact line for a moment shows the card image beside the cursor; click still expands the row. `Image` opens the full `ImagePreview`, where Esc or a click closes it and the `Flip` control also answers the F key.
- Accordions with one section open at a time stay that way below 900px. On a wide window, screens with room show every section open at once (named per screen below) and drop the disclosure arrow there.
- Swipe surfaces (Add, Remove, Swipe select, Oracle tag examples) keep mouse drag on the card, since it works the same as touch, and on a wide window add arrow keys plus four buttons under the card, one per direction the surface allows. Below 900px only the card's prices sit under it, the pile chips share the bottom row with the dock, and a swipe down is undo.

## Login

- On the page grammar with no hero: the page opens on the form panel, centered at the top under the nav. The counters stay on Home and Activity.
- `Log in` is the full-width button under the fields, with `Create profile` and `Forgot password` as chips in one centered row beneath it. Enter in either field runs `Log in`.
- Below 900px it is the same panel in one column.

## Register

- On the page grammar with no hero: the four fields with their live error lines in their own form panel at the top under the nav.
- The `?` beside `Password` opens the `Password` hint, as in the app, so the form stays the only panel.
- `Create profile` is the full-width button under the fields, with `Back to login` as a chip beneath it. Enter in any field runs `Create profile`. Success goes back to Login with the username filled in.
- Below 900px it is the same panel in one column.

## Forgot password

- On the page grammar with no hero, as Login: the `Email` field in its own form panel with `Send` full width under it and `Back to login` and `Create profile` as chips beneath.
- After a send, the `Check your email` panel takes the form panel's place at the same width, holding only its message.
- Enter runs `Send`. Below 900px it is the same panel in one column.

## Update required

- Shown instead of everything else, with no nav, as in the app: the empty header bar, the pulsing `Update required` card, then a `Reload` button in place of the store link, since a web build updates by loading the page again.
- The layout is one centered column at every width.

## Home

- On the page grammar. The hero card spans the top of the page (the mark left, the three counters and the `Live` and `Everybody`/`Only me` chips right), then zite's tagline under a full-bleed rule, without zite's "built for mobile" ("The Magic: The Gathering deck builder. Swipe through cards for any format, and pick a commander to get the best synergy first.", the game's name linking Wizards in a new tab). Under it the app's `Decks` panel (the three latest decks as tiles, `All decks` in its header) and `Colors` panel (six bars, `Playstyle` in its header), both from `components`, share the columns with `Featured flavor`.
- The featured flavor panel is the app's: the quote, then the card's name and price, `Image` (the full preview), and in place of `Buy` the `TCGplayer ↗` and `Card Kingdom ↗` links that open in a new tab, as on zite.
- Home has no actions of its own beyond the hero's `Activity` and the panel headers' chips; the menu holds the rest.
- Below 900px it is the app's Home: hero, Decks, Colors, Featured flavor.

## Activity

- On the page grammar. The counts panel is the hero. The `Activity` section spans the page under it, the year's heatmap scaled to the page width over `Totals`. Under that the left column holds `Most skipped` with the app's card-list header (the accent-tertiary color, bold), and the right column holds `Card roles` over `Oracle tags`.
- No actions of its own. Back is the folded nav's arrow; the screen's `?` is in the page head.
- All three accordion sections are open at once, so the shared `Show` rate chips appear above both charts and one click switches both, as in the app.
- Hovering a heatmap square shows the floating day chip the spec describes for hover devices; clicking a square still drives the readout for 4 seconds.
- Role and tag chips, `Examples`, and `Most skipped` rows (`Image`, `Printing`) work by click as in the app.
- Below 900px it is the phone accordion with `Activity` open.

## Playstyle (Suggested commanders)

- On the page grammar, in rows of two panels, all open: `Summary` (its table) beside `Color map`, `Card roles` beside `Oracle tags`, then `Suggested commanders` beside `Staples`. The two card lists keep the app's pink card-list headers and hold their own filter chips (the `Show` row with `Art`; `Type` and `Color`) and empty lines inside their panels.
- Rows expand inline on click with their footer (`Why this commander?`) and actions `Image`, `Printing`, `Create deck`, `Maybeboard`, `Remove`.
- `Swipe` sits in the commander list's header and opens the Swipe select overlay in suggested mode (see Edit deck). `Clear skips` is a Danger zone panel at the page's end, red, and opens its dialog.
- Below 900px it is the phone column with the profile accordion, `Summary` back in it, and `Color map` open.

## Commander maybeboard

- On a wide window the left column holds `Quick add` with its floating results, the `Show` row (`All`, five color pips, `Art`) and the `Commanders` list. The filter sheet opens as a right side panel beside the list, so the list re-filters in view on `Apply`.
- `/` focuses `Quick add`; arrow keys move through the result chips and Enter adds the highlighted one.
- Rows expand on click with `Image`, `Printing`, `Create deck`, `Remove`. `Swipe` in the list's header opens Swipe select in commander mode; `Filter` floats as a chip; `Clear maybeboard` is a Danger zone panel at the page's end.
- Below 900px it is the phone layout with the filter as a bottom sheet.

## Oracle tag dictionary and examples

- The dictionary opens as a large centered modal over its host (the oracle tags picker or the filter panel). On a wide window the letter rail becomes a vertical column on the left and the list fills the rest; each row keeps `Examples` and `Use`.
- `/` focuses the search; with search empty and unfocused, typing a letter jumps to that letter. Esc and the back arrow in the header's corner close it.
- Examples opens stacked on top as a modal with the card image on the left and its details (the `CardDetailsSheet` content) shown on the right instead of behind the eye button. ← goes to the next card and ↓ goes back one, matching the app's left and down swipes; two buttons under the card do the same.
- Below 900px both are full-screen overlays as in the app.

## Account

- The `Account` list, with the Danger zone panel at the bottom; the About list moved to its own screen.
- `Change password` keeps the `?` beside its `Password` field for the rules, as Register does.
- `Change username`, `Change email` and `Change password` open as centered modals with the same fields, closed from an × and saved from the dock's checkmark; Enter runs `Save changes`. The `Change password` confirm dialog stacks on top, and Esc closes only the top one.
- `Resend` and the refresh icon button (with a `title` tooltip on hover) stay in the `Verification` row.
- `Log out` and `Delete account` sit in the Danger zone panel, Delete in red; the first opens its dialog, the second the countdown dialog. The menu's footer carries Log out as well.
- Below 900px the two lists stack and the sheets slide up as in the app.

## Settings

- On a wide window the `Appearance` and `Cards` lists sit side by side. The `App icon` row is left out, as the spec allows where the platform has no app icon.
- `Themes` opens as a right side panel so the live preview repaints the whole page beside it. Clicking a row previews it, ↑ and ↓ move the preview through the list, Enter is Save (the dock's checkmark) and Esc closes it (restore and `Theme unchanged`).
- `Exceptions` opens as a centered modal with the `N selected` row and the franchise chips; its × closes it and the dock's checkmark saves, as in the app.
- `Dark`/`Light`, `Theme`/`Natural` and `Hide`/`Show` stay one-click toggles in their rows.
- Below 900px the lists stack and both sheets slide up.

## Privacy policy

- On a wide window a sticky contents list on the left names the section headings (`Data we collect` through `Contact`) and scrolls to each; the section cards sit in one reading column on the right.
- Below 900px it is the column of cards alone. Back is the header's arrow, to About.

## About

- The screen in the menu's footer, replacing Changelog: a panel of rows (Version with its `?`, Website with the zwipe.net chip, Privacy policy with a View chip), a Credits panel (the Scryfall credit and the Fan Content notice) and a Changelog panel with the shared changelog as rows, the version filter first. `/changelog` redirects here.

## Deck list

- On a wide window the `Group by` and `Show` rows sit above the list, and the `Tags` row stays unfolded since there is room. Deck rows lay out two per line inside each group, with the art crops at a larger size.
- Clicking a row opens the deck; group headers collapse on click as in the app.
- `Create` floats as the dock's + and keeps its verify-email guard; Commander maybeboard and Playstyle are in the menu.
- Below 900px it is the phone list with the `Tags` fold chip.

## Create deck

- On a wide window the form splits into two columns: `Profile` (name, format, command zone fields, power level) on the left, `Budget` above `Tags` on the right.
- The `Format`, `Deck tags` and `Oracle tags` pickers open as a right side panel over the right column, with the definition bar on top, the chip grid below, a back arrow in its header and Save as the dock's checkmark. The form stays visible, so the format cascade clearing the commander shows as it happens. `/` focuses the picker's search; Esc is `Back`.
- Command zone fields: typing shows the floating results; ↑ and ↓ move through them and Enter picks the highlighted card. `Filter` and `Swipe` chips stay in the label row.
- `Swipe` opens Swipe select as a full-window overlay laid out like Add cards: card and its info on the left, the card details beside it, a back arrow in its header and `Filter`, `Refresh` and the eye in its dock. → picks, ← skips, ↑ saves to the commander maybeboard (commander and partner modes), ↓ undoes. Its `Filter` opens the filter as a side panel inside the overlay.
- `Create` is the dock's checkmark once the form is valid; Enter in the deck name field runs it. Below 900px it is the phone form with full-screen pickers.

## Edit deck

- Same layout, pickers and Swipe select as Create deck; Save is the dock's checkmark, shown only once something changed.
- The loading skeleton follows the two-column shape.
- Below 900px it is the phone form.

## Import

- On a wide window the left column holds the fixed controls (`From`, `Mode`, `Board`, the explainer line) and the decklist textarea or URL field; the right column holds the `Imported` and `Unresolved` results so the user can fix the paste while reading what failed, which the phone pushes below the input.
- Ctrl+Enter (Cmd+Enter on a Mac) runs the dock's import icon when it is enabled.
- Below 900px the results sit under the input as in the app.

## Export

- On a wide window the `Export` chips (`Deck`, `Maybe`, `Side`) sit in a narrow left column and the read-only `Decklist` fills the right at full height.
- `Copy` is the dock's icon.
- Below 900px it is the phone stack.

## Deck view

- On the page grammar. The `Profile` card is the hero, 40rem wide. The left column holds `Budget`, `Tags`, `Distributions`, `Mana` and `Draw odds`; the right column holds `Warnings` over a cards column that lists the mainboard by type with the command zone on top, as compact rows that expand on click.
- Every section is open at once, so the phone's single-open accordion and its default `Budget` go away on a wide window; `Warnings` keeps its warning border and fix buttons (`Add cards`, `Import`, `Fix to {n}`, `Clear`, `Remove`).
- Edit, Clone and Share are icons in the Profile panel's header. Buy is a label in Budget with the `TCGplayer ↗` and `Card Kingdom ↗` chips. `Cards` floats in the dock as a chip and opens Deck cards for full editing; the cards column here is for reading. Import and Export sit in a Data panel and Clear skips and Delete in a Danger zone panel at the page's end; the dialogs are the app's. Back after a delete, or a deck the server no longer has, lands on Decks.
- The `Draw odds` stepper `-` / `+` buttons also answer ← and → while the stepper has focus.
- Below 900px it is the phone screen: profile card, accordion with `Budget` open, no cards column.

## Add cards (swipe)

- At 1280px and up there are three columns: the card stack with `CardInfoDisplay` under it; the card details (the `CardDetailsSheet` content: cost, type, oracle text, keywords, card roles) for the top card, which the phone hides behind the eye button; and the filter as a docked panel. Between 900px and 1279px the filter becomes a side panel opened by `Filter`, and the card and details stay side by side.
- The `From` label and the one `Search`/`Maybe` toggle chip sit in the bottom row beside the dock, which holds Synergy as the link icon, `Filter`, `Refresh` and the eye; the eye opens the full details sheet as a modal where `Printings` lives.
- Keys: → adds, ← skips, ↑ sends to the maybeboard, ↓ undoes (Ctrl+Z also undoes). F flips a double-faced card, P opens the printing modal, R is `Refresh`. Four buttons under the card do the same as the arrows, and mouse drag still works.
- The printing sheet opens as a centered modal with the carousel; ← and → step printings while it is open, Enter is `Save`, Esc is `Back`.
- At the end of results, ↓ (or the undo button) steps back into the pile in place of the phone's down-pull.
- Below 900px it is the app's screen: the card with its prices under it, the bottom row, the filter and details as bottom sheets, the down-pull at the end of the pile.

## Card filter sheet

- Where docked (Add and Remove at 1280px and up) it is a right column titled `Filter` with its `?`, the accordion, and its own dock with Reset as an undo icon and Apply as the checkmark; an × in its corner closes it. Elsewhere on a wide window it opens as a right side panel with the same parts.
- Draft and apply rules stay: edits are a draft until `Apply`; closing restores the draft and toasts `Filter unchanged`. When docked, the × reverts but the panel stays, and leaving the screen with an unapplied draft reverts it the same way.
- Section `×` buttons, the Any/All, Exact/Range and Include/Exclude chips, and picker result lists work by click; in a picker, ↑ and ↓ move through results and Enter picks. The `Dictionary` chips open the dictionary modal.
- Arrow keys typed in a filter field never swipe the card.
- Below 900px it is the app's bottom sheet.

## Deck cards view

- At 1280px and up there are three columns: a left column with the header block (name, format, power, tags), the featured cards, `Quick add` and the `Group by`, `Boards`, `Show` rows; the card groups in the middle; and a detail pane on the right showing the selected row's image, `CardDetails` and its actions (`-` / `+`, `Printing`, `Star`/`Unstar`, the move buttons, `Flip`, `Image`). Between 900px and 1279px the controls sit above the groups and the detail pane stays on the right.
- Clicking a row selects it into the detail pane instead of expanding it inline; one row at a time as in the app. With `Maybe` or `Side` on, their groups list above the mainboard as in the app.
- Keys: `/` focuses `Quick add`, ↑ and ↓ move the selected row, `+` and `-` change its quantity (the app's debounced writes still apply), Ctrl+Z is `Undo`. F flips the selected card everywhere a card shows, so `Filter` has no key here.
- Drag adds one thing here: dragging a row onto the `Maybeboard`, `Sideboard` or a mainboard group header moves it to that board, the same as the move buttons, with the same toast.
- The dock holds + (Add) and − (Remove), a divider, the funnel (with a small × while a filter is on, which resets at once) and undo. The filter opens as a side panel over the detail pane.
- Below 900px it is the phone list with rows expanding inline.

## Remove cards

- Same columns as Add cards: card and info, details, docked filter at 1280px and up; the filter becomes a side panel between 900px and 1279px. The board chips (`Main`, `Maybe`, `Side`, `All`) sit in the bottom row beside the dock.
- Keys: → removes, ← keeps, ↑ moves between main and maybeboard, ↓ undoes (Ctrl+Z too). F flips, P opens the read-only printing modal, R is `Refresh`. Four buttons under the card match the arrows.
- The dock holds `Filter`, `Refresh` and the eye.
- Below 900px it is the phone screen.

## Not found

- zite's dead end, under the nav the user would see anyway (signed in or out): a 25rem panel titled `Page not found` in the error red, uppercase and pulsing, zite's sentence ("Nothing lives at this address. It may have moved with a site update, or the link may be incomplete.") and `Home` in its footer.

## Owner decisions (approved 2026-10-31)

- Buttons under swipe cards, on a wide window, use the app's own action names for each mode: `Skip`, `Maybeboard`, `Undo`, `Add` on Add cards; `Keep`, `Move`, `Undo`, `Remove` on Remove; `Skip`, `Save`, `Undo`, `Pick` in Swipe select; `Next`, `Back one` in Examples.
- Update required shows `Reload` in place of the store link.
- `Buy` is a link per retailer (`TCGplayer ↗`, `Card Kingdom ↗`) opening in a new tab, as on zite, for a single card and for a deck; the app's `Buy` dialog goes.
- The one-time `filter` hint fires the first time the user opens a filter section, not when Add or Remove mounts with the filter docked.
- A command zone input clears its chosen card only when typing starts, never on a click.
- Deck view's cards column stays read-only; Deck cards is the editor.
- Commander maybeboard's `Back` follows browser history instead of pushing the deck list.
