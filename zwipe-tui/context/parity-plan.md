# zwipe-tui parity plan

The goal is every screen and behavior of the ZWIPE app (zwiper) in the terminal, matched as closely as a character grid allows. The specs in `specs/` are the source of truth for copy, timings, client calls and rules; this file says who builds what, in what order, and how a piece counts as done.

Specs:

- `specs/1-auth-chrome-account-config.md`: session upkeep, Login, Register, Forgot password, app chrome (header, help, hints, toasts, sheets, dialogs, skeletons, every animation), Account, Config, Privacy policy, Changelog
- `specs/2-home-activity-playstyle.md`: Home, Activity, Playstyle (suggested commanders), Commander maybeboard, oracle tag dictionary and examples, the ColorMap tile algorithm
- `specs/3-decks.md`: deck list, Create, Edit, Import, Export, deck view and its More sheet
- `specs/4-cards-add-filters.md`: the add (swipe) screen, the card filter sheet, the deck cards view, Remove cards

## Ground rules

- Copy the app. Strings are verbatim, behavior follows the spec, and where the spec records an app quirk, keep it unless this plan says otherwise.
- Logic the app keeps in Dioxus crates (`zwiper`, `components`, orangui) is ported into `zwipe-tui/src/lib/domain/` as pure, tested functions. zwipe-tui depends only on zwipe-core and zwipe-client, and this branch changes no other crate.
- Taps become keys. Each screen's action bar becomes its key hints, and the same letter means the same thing everywhere it can (`?` hint, `!` help, `Esc` back, `q` quit outside text fields).
- Layout must survive a small terminal without panicking; a render test at a tiny size guards each screen.
- Gate per commit: nightly fmt, clippy `-D warnings`, tests, rustdoc `-D warnings`, all on `-p zwipe-tui`.

## Kept deviations

- "1 card" stays singular; the app's "1 cards" is an app bug.
- A deck with nothing to narrow its search follows the app: Add refuses with "Try a filter to start swiping" instead of serving the whole pool.
- Platform-only pieces are skipped: the app icon picker, haptics, telemetry.
- Images use the terminal's graphics protocol, with half-blocks as the fallback.
- The deck view shows several sections at once on wide terminals and collapses to tabs on narrow ones, with every app action still available. Its sections are panels instead of the app's accordion: Tab moves between them, Enter zooms one to fill the screen, and Esc unzooms before it goes back.

## Phase 0: foundation (done before the waves)

Screen modules under `inbound/screens/<name>/`, a back stack, and the shared widgets: toasts, dialog, sheet, text input, chip row, accordion, `?` popup, one-time hint, action bar, skeleton, stats strip, open URL, OSC 52 copy. `ARCHITECTURE.md` holds the recipe for adding a screen.

## Wave 1: four packages built in parallel

### A. Auth, chrome, account, config (spec 1)

- Session upkeep: the 60 s loop (refresh and version check), the silent return to Login when the session clears, a 5 s logout timeout.
- Login: the logo-and-counters panel, app copy and errors, untrimmed input.
- Register: every field and rule, the prefill back to Login.
- Forgot password: the request flow and its copy. The reset itself happens on the website.
- Splash and every animation in spec 1 section 2, with the app's timings: the logo decode at 22 frames, rolling counters, grid rain, screen entrance as far as a terminal can.
- Chrome: the `!` Help & feedback dialog, all `?` hint topics, all one-time hint keys wired to `mark_hint_shown`, toasts with the app's durations and stacking.
- Account parity: the More sheet, the Log out confirm, the expiry wording, the red Unverified badge.
- Config parity: the load-failure state, the swatches in the theme list, the app's failure behavior.
- Privacy policy (from `zwipe_core::legal`, HTML to text) and Changelog (cache, skeleton, filter chips).

### B. Home, Activity, Playstyle (spec 2)

- Home parity: the three caches, Live chip, rolling numbers, compact `28.0k` figures, Activity entry, featured flavor Image and Buy, toasts and the welcome hint.
- ColorMap: the tile algorithm ported exactly, drawn two cells per tile with the four shades and the ring sweep.
- Activity: the 371-day heatmap and readout, Totals, the role and oracle tag rate charts, Most skipped.
- Playstyle: Color map, Summary, Card roles, the Oracle tags table, the top-10 list with Show, each row's actions, Staples with its filters, Clear skips, the suggested-mode swipe.
- Commander maybeboard: quick add, the color filter, the card filter sheet from package D, swipe, remove, clear.
- Oracle tag dictionary and Examples overlays.

### C. Decks (spec 3)

- Deck list: Group by, Show filters, the tags fold, collapsible groups, the app's rows, the Create gate, the More sheet.
- Create and Edit: every field, the four command-zone search pickers, the format, tag and oracle tag pickers, targets, the knock-on clears, the change detection, every validation and server message.
- Import and Export.
- Deck view: the Profile card, the sections with Roles expanding into tags, warning fixes, the first-deck hint, and the More sheet (Import, Export, Buy, Clone, Share and Stop share with clipboard copy, Clear skips, Delete).

### D. Cards (spec 4)

- In-memory stores: the per-deck filter store, parked add stacks, and the shared undo log.
- Add screen parity: the From and Synergy switches, deck context, every gesture, an undo that lands before its add, every toast, land and budget crossings, the printing and details sheets, end of results.
- The card filter sheet: all 15 sections, the draft and Apply flow, reset rules, picker sources.
- Deck cards view: header, featured cards, quick add, Group by, Boards, Show toggles, row actions with debounced quantity changes, board moves, printing changes, MVP, Undo, Reset filter.
- Remove cards.

## Wave 2: integration

Merge the packages, wire cross-links (Playstyle "Create deck" to Create, Commander maybeboard to the filter sheet, deck view to cards, import and export), then audit each spec against the running app one screen at a time and fix what differs.

Wired: Create with a commander from Playstyle, the commander swipe and the maybeboard; the deck list's More sheet to the maybeboard and Playstyle; the deck view's Roles tag Examples; the deck form's and the filter sheet's oracle tag Dictionary; the maybeboard's and the commander swipe's card filter; tag Examples in the details sheet and the deck cards view. Home and Login share one copy of everybody's counts.

## Done means

- Every screen in the specs reachable from the TUI, with the app's copy and states.
- A live run with the test account through every screen in tmux, leaving the account as it was found.
- The full workspace gate green, README updated, branch pushed to `feat/tui`. Merging to main stays the owner's call.
