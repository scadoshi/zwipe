# zwiper-web parity plan

The goal is every screen and behavior of the Zwipe app (zwiper) in a browser, laid out for a big window. The specs in `specs/` are the source of truth for copy, timings, client calls and rules. `web-layout.md` says how each screen sits on a wide window and how its actions are reached, and the owner approves it before screen work starts. This file says what gets built in what order and when a piece counts as done.

Specs:

- `specs/1-auth-chrome-account-config.md`: session upkeep, Login, Register, Forgot password, app chrome (header, help, hints, toasts, sheets, dialogs, skeletons, animations), Account, Config, Privacy policy, Changelog
- `specs/2-home-activity-playstyle.md`: Home, Activity, Playstyle (suggested commanders), Commander maybeboard, the oracle tag dictionary and examples, the ColorMap
- `specs/3-decks.md`: deck list, Create, Edit, Import, Export, deck view and its More sheet
- `specs/4-cards-add-filters.md`: the add (swipe) screen, the card filter sheet, the deck cards view, Remove cards

## Ground rules

- Copy the app. Strings are verbatim, behavior follows the spec, and where the spec records an app quirk, keep it unless this plan lists it below.
- The web changes layout and input only. What the app hides behind a sheet, accordion or tab may show at once; every app action stays reachable by click, with keys where `web-layout.md` names them.
- zwiper-web depends on `components` (and through it orangui), `zwipe-client` and `zwipe-core`, and changes no other crate. Logic the app keeps inside zwiper is ported into `src/lib/domain/` as pure, tested functions. Shared look comes from the kit's stylesheets and zwiper's own, loaded as they are; `assets/web.css` holds only what a browser window changes.
- Below 900px every screen falls back to the app's single column.
- Gate per commit, from the worktree root: `cargo +nightly fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo clippy -p zwiper-web --target wasm32-unknown-unknown -- -D warnings`, `cargo test -p zwiper-web`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p zwiper-web`, and `dx build --platform web` from `zwiper-web/`.

## Pace and branches

Packages are built one at a time, in the order below. Each lives on its own branch `feat/web-<x>` cut from `feat/web` and merges back into `feat/web` when its done list is met. Merging `feat/web` to main is the owner's call.

## Phase 0: foundation (done)

The crate, the router with a route per screen at the app's paths (unbuilt screens are placeholder panels naming the screen), the sidebar shell with its narrow fallback, theming through the kit, the session in `localStorage` with the 60 s upkeep loop (refresh and min-version check), the auth gate, the `Help & feedback` dialog, Logout behind the Account placeholder, and Login end to end with the app's copy. The docs in this folder.

## Wave A: auth, chrome, account, config (spec 1), branch `feat/web-a` (done)

- Login parity: the shared 60 s counter cache and its stored copy across visits, the prefill from Register.
- Register and Forgot password, every field, rule and toast.
- Chrome: header `?` hints, the HintHost topics, one-time hints wired to `mark_hint_shown`, the side panel and modal forms of the app's sheets, skeletons, the entrance animations.
- Account: both lists, the three change sheets, verification, the More popover, Log out and Delete account dialogs.
- Config: themes as a live-preview side panel, dark mode, pips, Universes Beyond and its exceptions.
- Privacy policy with its contents column, and Changelog with its filter column.
- The authed-call facade (refresh, run, toast) every later wave calls through.

## Wave B: home, activity, playstyle (spec 2), branch `feat/web-b` (done)

- Home: the scoped hero, the color map, the featured flavor card.
- Activity: the heatmap and readout, Totals, the role and oracle tag rate charts, Most skipped.
- Playstyle: the profile sections open together, the oracle tags table, the suggestions with Show, Staples, Clear skips, the suggested-mode swipe.
- Commander maybeboard with quick add, the color filter, swipe, remove, clear.
- The oracle tag dictionary and Examples modals.
- zwiper-web takes its screen logic, hint copy, timings and caps from zwipe-ui; the copies it still keeps are marked `// Belongs in zwipe-ui`.
- Left for later waves: the commander maybeboard's `Filter` and commander-mode Swipe select's filter (Wave D), and a host for the dictionary in the filter panel (Wave D); the oracle tag picker hosts it since Wave C.

## Wave C: decks (spec 3), branch `feat/web-c` (done)

- Deck list: Group by, Show filters, tags, collapsible groups, the Create gate, the More popover.
- Create and Edit in two columns: every field, the command-zone searches, the format, tag and oracle tag pickers as side panels, Swipe select for each slot, the clears, change detection, every validation and server message. Create takes the commander Playstyle and the maybeboard hand it.
- Import and Export.
- Deck view in columns: profile, stats, the cards column, warnings and their fixes, the More popover and its dialogs.
- Left for Wave D: `Cards`, the below-minimum warning's `Add cards`, Swipe select's `Filter` in every mode, and clearing the deck's undo history after an import, each marked `TODO(wave D)`.

## Wave D: cards, add, filters (spec 4), branch `feat/web-d` (done)

- The stores: per-deck filters, parked add stacks, the shared undo log, in-flight adds.
- Add cards: the stack with details and the docked filter, From and Synergy, keys and buttons for every swipe, undo, every toast, land and budget crossings, the printing modal, end of results.
- The card filter panel: all 15 sections, draft and Apply, reset rules, picker sources.
- Deck cards view with the detail pane, quick add, Group by, Boards, Show, quantities, moves (buttons and drag), printings, MVP, Undo, Reset filter.
- Remove cards.
- The filter on the commander maybeboard and in Swipe select's search modes, the deck view's `Cards` and the warning's `Add cards`, and an import clearing the deck's undo log.
- zwiper-web takes the stores, the stack and its actions, Add's deck context and search rules, the filter's draft, sections, fields and pickers, the deck cards pass and bursts, the undo entries and the land and budget signals from zwipe-ui. The filter's chip rows (`domain::filter_chips`) are marked `// Belongs in zwipe-ui`.

## Integration and live audit, branch `feat/web-int` (done)

- The cross-links the waves left open are wired and checked live both ways: Playstyle and the commander maybeboard hand their commander to Create, the maybeboard opens the filter, the deck view reaches Cards, Add, Remove, Import and Export (from More and from the warnings), the dictionary and Examples open from every host the app has, and Help and the `?` hints open from every screen. Browser back and forward move between screens; closing an open overlay on back is still open (see `audit.md`).
- The end-of-pile `Undo` in Swipe select and `Back one` in Examples call their undo directly instead of pressing the swipe control with no card on top.
- The signals that root tasks (`spawn_forever`) write live in the root scope, so the catalog, caches, card stores and deck tags no longer raise Dioxus's hoisted-value warnings.
- `audit.md` holds the screen-by-screen check against the four specs, with what is left.
- Done means every screen in the specs is reachable with the app's copy and states, a live run with a throwaway account goes through every screen on a wide and a narrow window and deletes the account after, the gate is green, and the README is current.

## Kept deviations

- Update required shows `Reload` instead of a store link; a web build updates by loading the page again.
- The problem report names its platform `Web`.
- The one-time `filter` hint fires on the first use of a filter section, not when the screen opens with the filter docked.
- A command zone input clears its chosen card when typing starts, not on a click.
- Commander maybeboard's `Back` follows browser history; the app pushes the deck list.
- The printing sheet steps through printings with `←` and `→` buttons and keys in place of the app's swipe carousel.
- Home greets once per page load, a browser's launch.
- `Buy` is a link per retailer that opens in a new tab, as zite's are, for a card and for a deck, where the app opens a dialog to pick one.
- The app's screen header goes under the top nav: the `!` is the nav's `Help` on every screen, signed in or not, and a screen's `?` sits at the right end of its action bar.
- Buttons that only navigate (`Back`, `Back to login`, Home's `Decks`, `Account`, `Config`, `Activity` and `Playstyle`) go as each screen moves onto the page grammar; the top nav and the browser's back cover them, and a form's neighbor screen gets a text link in its panel.
- `Clear skips` on the deck view clears at once; the app first sends the skips its telemetry holds, and the web records none.
- Platform-only pieces are skipped: the app icon picker, haptics, crash capture, usage telemetry (a failed call is logged, not reported), the OS back gesture.
- Escape closes an open dialog before anything else, even where the app's back stack would close the sheet under it first (the Exceptions sheet's hint).
- `Back to login` on Register and Forgot password goes to Login when there is no history to go back to, such as a page opened from a link.
- The session lives in `localStorage`, where any script on the origin can read it. The app keeps it in the OS keychain.
- The web layout choices in `web-layout.md`, once approved, count as kept deviations from the phone layout.
- On the deck cards view the `Main`, `Maybe` and `Side` chips also take a dragged row, so a row can move to a board whose section is hidden.
- The deck cards view reads the deck's profile from the deck it loads; the app fetches the profile and each command zone card again.

## Launch blockers

Both need zerver changes, so they are out of scope for the wave branches.

- The refresh token moves out of `localStorage` into an httpOnly cookie, which zerver sets and reads. Building on `localStorage` is fine until then.
- Tag funnel events with the platform (zerver change), then turn the web's on. The web's anonymous events (app opened, Register viewed, Register submitted) are wired but held off by `SEND_FUNNEL_EVENTS` in `session/funnel.rs`, so web visits stay out of the app's funnel numbers.
