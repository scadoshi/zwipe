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

## Wave A: auth, chrome, account, config (spec 1), branch `feat/web-a`

- Login parity: the shared 60 s counter cache and its stored copy across visits, the prefill from Register.
- Register and Forgot password, every field, rule and toast.
- Chrome: header `?` hints, the HintHost topics, one-time hints wired to `mark_hint_shown`, the side panel and modal forms of the app's sheets, skeletons, the entrance animations.
- Account: both lists, the three change sheets, verification, the More popover, Log out and Delete account dialogs.
- Config: themes as a live-preview side panel, dark mode, pips, Universes Beyond and its exceptions.
- Privacy policy with its contents column, and Changelog with its filter column.
- The authed-call facade (refresh, run, toast) every later wave calls through.

## Wave B: home, activity, playstyle (spec 2), branch `feat/web-b`

- Home: the scoped hero, the color map, the featured flavor card with its image beside the quote.
- Activity: the heatmap and readout, Totals, the role and oracle tag rate charts, Most skipped.
- Playstyle: the profile sections open together, the oracle tags table, the suggestions with Show, Staples, Clear skips, the suggested-mode swipe.
- Commander maybeboard with quick add, the color filter, swipe, remove, clear.
- The oracle tag dictionary and Examples modals.

## Wave C: decks (spec 3), branch `feat/web-c`

- Deck list: Group by, Show filters, tags, collapsible groups, the Create gate, the More popover.
- Create and Edit in two columns: every field, the command-zone searches, the format, tag and oracle tag pickers as side panels, Swipe select, the clears, change detection, every validation and server message.
- Import and Export.
- Deck view in columns: profile, stats, the cards column, warnings and their fixes, the More popover and its dialogs.

## Wave D: cards, add, filters (spec 4), branch `feat/web-d`

- The stores: per-deck filters, parked add stacks, the shared undo log, in-flight adds.
- Add cards: the stack with details and the docked filter, From and Synergy, keys and buttons for every swipe, undo, every toast, land and budget crossings, the printing modal, end of results.
- The card filter panel: all 15 sections, draft and Apply, reset rules, picker sources.
- Deck cards view with the detail pane, quick add, Group by, Boards, Show, quantities, moves (buttons and drag), printings, MVP, Undo, Reset filter.
- Remove cards.

## Integration and live audit, branch `feat/web`

Wire the cross-links the waves leave open (Playstyle `Create deck` to Create, the maybeboard to the filter, deck view to the card screens, import and export), then audit each spec against the running web app one screen at a time beside the phone app and fix what differs. Done means every screen in the specs is reachable with the app's copy and states, a live run with the test account goes through every screen on a wide and a narrow window and leaves the account as it was found, the gate is green, and the README is current.

## Kept deviations

- Update required shows `Reload` instead of a store link; a web build updates by loading the page again.
- The problem report names its platform `Web`.
- The one-time `filter` hint fires on the first use of a filter section, not when the screen opens with the filter docked.
- A command zone input clears its chosen card when typing starts, not on a click.
- Commander maybeboard's `Back` follows browser history; the app pushes the deck list.
- The `!` header button moves into the sidebar as `Help` on authed screens; the public screens keep it in the header.
- There is no splash; the first screen's entrances run at once.
- Platform-only pieces are skipped: the app icon picker, haptics, crash capture, the OS back gesture.
- The session lives in `localStorage`, where any script on the origin can read it. The app keeps it in the OS keychain.
- The web layout choices in `web-layout.md`, once approved, count as kept deviations from the phone layout.

## Launch blocker

Before zwiper-web is public, the refresh token moves out of `localStorage` into an httpOnly cookie. That needs zerver to set and read the cookie, so it is out of scope for the wave branches; building on `localStorage` is fine until then.
