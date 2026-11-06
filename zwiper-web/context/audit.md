# zwiper-web audit against the specs

Checked on 2026-10-31 against a local zerver, in headless Chrome at 1440 and 420 wide, with one seeded throwaway account (four decks, a year of activity, swipe signals, a commander maybeboard) and one empty, unverified account. Copy was also checked in bulk: every quoted string in the four specs was looked up in the sources the web build draws from (zwiper-web, zwipe-ui, components, orangui, zwipe-core), and the only misses were the app's store links, which the kept `Reload` deviation replaces, and zite's reset page, which is not an app screen.

Status is `ok` when copy, actions and the states named were seen live and match the spec or a kept deviation in `parity-plan.md`; `open` names what is left.

## Spec 1: auth, chrome, Account, Config

- Login: ok. Hero with live counters, both fields, `Log in`, `Create profile`, `Forgot password`, the empty-submit toast, `!` Help dialog.
- Register: ok. Live field errors in the spec's wording, the `Password` hint, `Back to login`.
- Forgot password: ok. Form, the fixed invalid-email line, `Back to login`.
- Update required: ok. Seen with zerver's `MIN_CLIENT_VERSION` above the build; `Reload` is the kept deviation.
- Not found: ok. Not in the app; a web-only fallback with a `Home` button.
- Chrome: ok. Sidebar `Help` (menu below 900px), header `?` on every screen that has a hint, one-time hints firing once per account, toasts top right on a wide window.
- Account: ok. Both lists, `Verified` and `Unverified` with the `Verification` row, the three change modals, `Log out` and the counting `Delete account` dialog, `Privacy policy` and `Changelog` and back again.
- Config: ok. Themes side panel with arrow-key preview, dark and light, pips, Universes Beyond `Show`/`Hide` and the Exceptions modal with its one-time hint.
- Privacy policy: ok. Contents column, every section, `Last updated` chip.
- Changelog: ok. Filter column and release cards.

## Spec 2: Home, Activity, Playstyle, maybeboard, dictionary

- Home: ok. Welcome hint, `Verify your email!` for an unverified user, `Everybody` and `Only me`, color map (and `Build a deck to see your playstyle` when empty), featured flavor with `Image` and retailer links when the card has a price, no `?` as in the app.
- Activity: ok. Hint, counts, heatmap, Totals, role and tag charts, Most skipped; `Swipe some cards to see your activity` when empty, `Could not load your activity` with the error toast when offline.
- Playstyle: ok. Hint, profile sections open together, oracle tags table, suggestions with rows, `More` and `Clear skips`, Swipe select in suggested mode, `Create deck` handing the commander to Create; empty state `Build a deck to see your playstyle`.
- Commander maybeboard: ok. Quick add, `Show` row, rows with `Create deck`, `Filter` as a side panel, Swipe in commander mode with its own `Filter`, `More` with `Clear maybeboard`, `No commanders` when empty.
- Oracle tag dictionary: ok. Opens from the oracle tag picker and the filter's `Dictionary` chips, hint, letter rail, search, `Examples` and `Use`.
- Oracle tag examples: ok. From the dictionary and every role reveal host (Activity, Playstyle, maybeboard, deck view, deck cards); `Next` and `Back one` by key and button, `No more cards. Swipe down to go back.` at the end, where `Back one` returns the last card and nothing else.

## Spec 3: decks

- Deck list: ok. Hint, `Group by`, `Show`, groups, `More` with `Commander maybeboard` and `Playstyle`, `No decks` when empty, `Could not load decks` with the error toast when offline.
- Create deck: ok. Two columns, every field, the format picker as a side panel, the commander search, Swipe select per slot, the commander seeded from Playstyle and the maybeboard.
- Edit deck: ok. Hint, the same form, `Save changes` only once something changed.
- Import: ok. From, Mode, Board, the hint, `Imported 1, 1 unresolved` with both result lists.
- Export: ok. Board chips, hint, `Copy` and `Copied to clipboard`.
- Deck view: ok. Three columns, every section open, role reveals with `Examples`, Warnings with `Add cards` and `Import` reaching their screens, `Cards`, `Edit`, `More` with Import, Export, retailer links in a new tab, Clone, Share, Clear skips and Delete dialogs.

## Spec 4: cards and filters

- Add cards: ok. Stack with details beside it, docked filter at 1280px, every swipe by key and button, toasts, the details modal and printings, back to the deck cards view.
- Card filter: ok. Draft until `Apply` (docked and as a sheet), `Back` reverting with `Filter unchanged`, `Reset` staging the default, `Filter applied`, the one-time `Filters` hint on the first section opened rather than on mount.
- Deck cards view: ok. Header, featured cards, Quick add, Group by, Boards, Show, rows selected into the pane, `Image`, `Filter`, `Add` and `Remove`.
- Remove cards: ok. Boards row, swipes, the docked filter, `No cards` on an empty board.

## Owner rules, every screen

- `Buy` is never a modal: Home's featured card and the deck's `More` show `TCGplayer ↗` and `Card Kingdom ↗` as links with `target="_blank"`.
- The ASCII logos (sidebar, hero, splash) take `var(--logo)`: the text color in dark mode, the accent in light mode, measured on the page in both.
- The `filter` hint waits for the first section opened: seen on Add at both widths and on Remove, and the other hosts open the same panel.
- The filter keeps its draft until `Apply` wherever it opens.

## Open

- Browser back leaves the screen even while an overlay is open; Esc and the overlays' own buttons close them. Closing the top overlay on back needs a history entry per open overlay, and orangui's `OverlayBackStack` has no way to tell zwiper-web how many are open. Belongs in orangui: a depth reading, or the history entries themselves.
- The orangui dialog host draws each `AlertDialog` body in its own scope, so a dialog whose button label or input reads the owner's signals (Delete account's countdown, Clone deck's name) logs Dioxus's hoisted-value warning on every change. Debug builds only; the fix belongs in orangui.
- Loading skeletons were not caught live this time: the local zerver answers before a screenshot lands. They were checked when each wave built its screen.
- The ASCII logo renders with gaps in headless Chrome, which lacks a font for its block glyphs; a desktop browser draws it whole.
