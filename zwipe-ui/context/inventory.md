# zwipe-ui inventory

Step 1 of the zwipe-ui plan, for the owner's review. Nothing is built yet.

zwipe-ui is the crate all three clients pull their screen logic from: zwiper (the app), zwiper-web and zwiper-tui. It names no UI crate, so it builds for iOS, Android, the terminal and wasm32 alike. The owner's direction is to share as much as possible, so this goes past pure helpers to a screen model per screen: its state, the actions it takes, what each action asks the client to do, and its verbatim copy. Each client then maps a tap, click or key to an action and draws the state.

Sources read: zwiper and components on `origin/main` (72f67e16), zwiper-tui on `origin/feat/tui` (5b770f0e), zwiper-web on `origin/feat/web` (b8d7c242) and `origin/feat/web-a`, orangui at the pinned e6719eb. Paths below are short: `app/` is `zwiper/src/lib/inbound/`, `tui/` is `zwiper-tui/src/lib/`, `web-a/` is `zwiper-web/src/lib/` on `feat/web-a`.

## What zwipe-ui can depend on

zwipe-core, chrono, uuid, serde and thiserror. Not orangui, because orangui depends on Dioxus. Not zwipe-client, unless the owner decides otherwise (open decision 3).

That rules out reusing orangui's `short`, `with_separators`, `peak_indices`, the toast bands and the entrance math from zwipe-ui as they stand. The TUI copied all five by hand. Open decision 1 covers how to keep one copy.

## Part 1: the pieces

Home is where each piece goes: **ui** is zwipe-ui, **core** is zwipe-core, or a client name when only that client needs it. Waves are zwiper-web's: A (auth, chrome, account, config, mostly built on `feat/web-a`), B (home, activity, playstyle), C (decks), D (cards, add, filters).

| Piece | App today | TUI today | Web | Home | Drift | Waves |
|---|---|---|---|---|---|---|
| Color map model: growth, pockets, shades, rings, ghost | `components/src/color_map.rs` (model plus Dioxus in one file, 18 tests) | `tui/domain/color_map` with `golden.txt` | B: Home | ui; components keeps the Dioxus half and calls it | none; all 20 constants equal | B |
| Count-up rolling figures, `short`, `with_separators` | orangui `entrance.rs` (private fns inside signal loops) | `tui/domain/count_up` | A, B (heroes, figures) | see decision 1 | none in what was ported; TUI lacks reduced motion, splash pacing, shimmer | A, B |
| Logo decode frames | orangui `entrance.rs` | `tui/domain/decode` | none (web has no splash) | see decision 1 | none; TUI rounds 850 and 450 ms to whole 40 ms ticks on purpose | none |
| Activity math: heatmap layout, readout tags, totals, rate kinds, role and tag rate bars | `app/screens/activity/sections.rs` (pure, 16 tests) | `tui/domain/activity` | B: Activity | ui; the baseline weight to core | `year_line` missing in TUI | B |
| Home figures, count scope, flavor card expiry, has-data | `app/components/home_hero.rs`, `zwiper/src/lib/outbound/count_scope_store.rs`, `session_upkeep.rs` `next_hour` | `tui/domain/home` | A built `counts.rs`; B: Home | ui | three edge cases, see Drift | A, B |
| Buy links, deck and single card | `zwiper/src/lib/outbound/buy_links.rs` (pure, untested) | `tui/domain/deck/buy.rs`, `tui/domain/home` (deck only) | B: flavor card; C: Buy | ui | TUI lacks single-card links | B, C |
| Playstyle helpers: identities, Show filter, reasons, summary, roles, tag table, staples | `app/screens/deck/suggested_commanders.rs` (pure, ~26 tests) | `tui/domain/playstyle` | B: Playstyle | ui; identity and type-line helpers to core | none | B |
| Deck list Show filter and Group by | `app/screens/deck/list.rs` (2 tests) | `tui/domain/deck/list.rs` | C: Deck list | ui | none | C |
| Deck row count and pips | `list.rs` `row_card_count` | `tui/domain/deck/mod.rs` | C | ui | none | C |
| Deck form: land stepper, price box, format details, command-zone search rules, knock-on clears, oracle tag reseed, change detection | `app/screens/deck/components/deck_fields.rs`, `format_select.rs`, `create.rs`, `edit.rs` (signals, untested) | `tui/domain/deck/form.rs`, `tui/inbound/screens/deck_form` | C: Create and Edit | ui | none; app duplicates `MAX_LAND_TARGET` and writes the reseed twice | C |
| Import outcome toast | `app/screens/deck/import.rs` | `form.rs` `import_outcome` | C: Import | ui | none | C |
| Export text | `app/screens/deck/export.rs` | `tui/domain/deck/export.rs` | C: Export | ui | none | C |
| Warning fix choice | `app/screens/deck/components/deck_warnings.rs` | `tui/domain/deck/warning.rs` | C: Deck view | ui now, core later (decision 6) | none | C |
| Card filter Apply decision | `card_filter_sheet.rs` `apply_action` (4 tests) | `tui/domain/filter/mod.rs` | D: filter panel | ui | Apply when unchanged, see Drift | D |
| Filter sections, fields, pickers (15 sections, mode chips, picker sources and caps) | the 25 files under `app/screens/deck/card/filter/` (inside components) | `tui/domain/filter/{sections,fields,picker}.rs` | D | ui | none | D |
| Filter draft, snapshot, reset rules per host | `card_filter_sheet.rs`, `add.rs` `clear_filters` | `tui/inbound/screens/card_sheets/filter` | D | ui, returning an outcome instead of toasting | none | D |
| Add deck context: lands excluded, legality plus identity defaults, serve gate | `add.rs` | `tui/domain/filter/mod.rs`, `tui/domain/add` | D | ui; possibly core (decision 6) | none | D |
| Per-deck filter store | `app/screens/deck/card/components/filter_store.rs` | `tui/domain/stores/filters.rs` | D: the stores | ui | none | D |
| Parked add stacks (20 decks, 50 behind) | `add_stack_cache.rs` | `tui/domain/stores/parked.rs` | D | ui | app can panic on trim, see Drift | D |
| Shared undo log (cap 100) | `undo_log.rs` (3 tests) | `tui/domain/stores/undo.rs` | D | ui | none; TUI merges the live and parked halves into one store | D |
| In-flight adds | `in_flight_adds.rs` (4 tests) | `tui/domain/stores/in_flight.rs` | D | ui | none | D |
| Catalog cache: which lists, freshness | `app/components/catalog_cache.rs` | `tui/domain/stores/catalog.rs` | D: picker sources | ui (freshness and slots); fetching stays in clients | TUI has no 24 h TTL and 3 fewer lists | B, C, D |
| Card stack, linear and cycling, and its actions | `card_stack.rs`, `action_history.rs` (no tests) | `tui/domain/stack` (8 tests) | C: Swipe select; D: Add, Remove | ui | app lacks a clamp, see Drift | C, D |
| Add rules: page 25, load-more at 5, seed, land target, empty-results copy, related ids | `add.rs` (closures over signals) | `tui/domain/add` | D: Add | ui | none | D |
| Deck cards pass, featured strip, quantity bursts, undo messages, stale check | `app/screens/deck/card/view.rs` (5 tests) | `tui/domain/deck_cards` | D: deck cards view | ui | none | D |
| Land and budget crossings | written three times: `view.rs`, `add.rs`, `remove.rs` | `tui/domain/deck_signals` | D | ui | none | D |
| Changelog filing and filter | `components/src/changelog.rs` (pure, tested) | `tui/domain/changelog` | A: Changelog | ui; web keeps its anchors | two edge cases, see Drift | A |
| Card as text: which faces, stats line, rules | `components/src/card_details.rs` | `tui/domain/card` | D: details | ui for face choice and stats; mana symbol parsing stays in TUI | three differences, see Drift | B, D |
| Privacy policy HTML to text | renders the HTML as is | `tui/domain/legal` | renders the HTML as is | tui | n/a | none |
| Theme palettes as RGB per role | n/a (CSS) | `tui/domain/theme`, `scripts/gen_themes.py`, `gen_theme_extras.py` | n/a (CSS) | tui | n/a | none |
| Account form wording and checks | `zwiper/src/lib/domain/error.rs`, the account and auth screens | `tui/domain/account`, `tui/inbound/screens/register` | `web-a/domain/validation.rs`, `account.rs` | ui | link expiry boundary; TUI repeats the password rule | A |
| Session freshness and refresh-rejected rule | `app/components/auth/ensure_session.rs` | `tui/outbound/account` | `web-a/domain/session.rs` | ui; storage and timers stay in clients | TUI checks in a different order | A |
| Version gate change | `session_upkeep.rs` | `tui/inbound/app/session.rs` | `web-a/domain/version_gate.rs` | ui | none | A |
| Problem report mailto, platform as a parameter | `app/components/support.rs` | `tui/inbound/widgets/help.rs` | `web-a/domain/support.rs` | ui | none; `urlencode` is written three times | A |
| `?` hint topics: titles and bodies | `app/components/hint_host.rs`, `concept_explainers.rs` (RSX) | `tui/inbound/widgets/hint/topics.rs` (markup strings) | `web-a/inbound/chrome/hint_topic.rs` (RSX) | ui, as typed copy (decision 2) | TUI has stale RoleRates and TagRates copy | A, and every wave |
| One-time hints: ids and copy | ids in `zwipe-core/src/domain/user/models/hints.rs`; copy in 19 screens | `tui/inbound/widgets/hint/keys.rs` | `web-a/inbound/chrome/one_time_hint.rs` (an enum over the ids) | ids enum in core, copy in ui | none; verbatim in all three | A, and every wave |
| Toast bands 3, 5 and 8 s | orangui `toast_timing.rs` | `tui/inbound/widgets/toast.rs` (copied) | orangui | see decision 1 | none | all |
| Behavior timings: debounces 300 and 800 ms, quantity flush 300, resend 60 s, delete 5 s, upkeep 60 s, counts 60 s, catalog 24 h, heat pick 4 s, maybeboard toast 1.5 s | spread across the app | spread across the TUI | partly in web-a | ui, as named constants | none | all |
| Search limits: min query length, result caps 8 and 5, page 25, held size, tile counts | spread | spread | none yet | ui | none | B, C, D |
| Animation timings: splash, stagger, entrance, wipe, spinner, sweeps, card motion | orangui CSS and JS, `main.css` | `tui/inbound/widgets` | CSS | each client | none | none |
| Generation ticket for out-of-order replies | `app/components/generation.rs` (Dioxus `CopyValue`) | in the TUI's loop | none yet | ui as a plain counter; app keeps its hook over it | n/a | C, D |
| Swipe gesture engine | `app/components/interactions/swipe` (33 tests) | keys | keys and buttons | zwiper | n/a | none |
| Usage telemetry buffer and screen vocabulary | `app/components/telemetry` (11 tests) | none | anonymous funnel only | not audited; see decision 8 | n/a | none |

## Part 2: screen models

The shape I'd propose for every screen:

- `State`: the data, loading, error and empty states, and selections that mean something to the user (Group by, Show chips, a picked card, a collapsed group).
- `Action`: what a user does, plus each reply arriving (`Loaded(Result<..>)`).
- `fn update(&mut State, Action) -> Vec<Effect>`, where an `Effect` is a client call named by its zwipe-core contract, a navigation, a toast with its text, kind and band, a one-time hint, or a timer (debounce, flush).
- Copy as constants or functions on the screen's module, so every title and empty line has one source.

Keyboard focus, scroll position, hover, gestures, animation frames and layout stay in each client. The TUI is already close to this: each screen is a view struct, a `Request` and `Update` pair and an `apply`. Its views mix focus fields in with screen state, and its handlers toast and navigate through a terminal `Context`, so the lift is mostly splitting those two apart. The app is the costly one: its screens hold state in many signals inside components (Add has 25 signals and 43 toast calls, the deck cards view 48 signals), and the plan moves it last.

Effort is the work to build the shared model from the TUI's version and test it against the app's behavior, not to switch the app over. S is under a day, M one to two days, L three to five, XL more than a week.

| Screen | What can be shared | What stays in each client, and why | Effort |
|---|---|---|---|
| Login | form state, submit, the Register prefill, error and toast copy | inputs and focus | S |
| Register | fields, live errors (web-a's `validation.rs` is the model), submit, toasts | inputs | S |
| Forgot password | field, check, submit, toasts | inputs | S |
| Update required | copy | the action differs on purpose (store link, Reload, terminal message) | S |
| Help and feedback | copy, the mailto builder with platform | opening mail or the browser | S |
| Splash | decode frame math (decision 1) | all drawing; web has none | none |
| Home | hero scope and figures, color map model, flavor card and its hourly expiry, buy link, Playstyle panel states, one-time hint | color map drawing, counter animation, image | M |
| Activity | heatmap layout, readout selection and its 4 s release, totals, rate kind switch, both rate charts, Most skipped, empty lines | heat grid drawing, sweeps, chart drawing | M |
| Playstyle | profile sections, tag table, suggestions with Show, Staples filters, Clear skips flow, reasons, every empty line | chart and card drawing; sections open together on web | L |
| Commander maybeboard | list, quick add with its 300 ms debounce, color filter, remove, clear, hand-off to Create | swipe gesture | M |
| Swipe select (commander and suggested modes) | the stack, the per-mode constraint, maybeboard save and undo, toasts | the gesture engine and card motion | L |
| Oracle tag dictionary | search, definition, Use hand-off | overlay or panel layout | S |
| Oracle tag examples | load, paging, empty and error lines | card drawing | S |
| Deck list | the whole TUI `DeckList` minus focus: decks, Group by, Show, tag fold, collapsed groups, Create gate, More items | chip and list drawing | S |
| Create and Edit | every field, validation, knock-on clears, change detection, the four command-zone searches with debounce and generation tickets, the format, tag and oracle tag pickers (draft, Save disabled when unchanged, Back with "unchanged"), submit and server messages | text inputs, picker layout (sheet, side panel, overlay) | L |
| Import | paste, submit, outcome toast | text area | S |
| Export | board toggles with the at-least-one rule, the text, copy toast | clipboard API | S |
| Deck view | load, profile, stats, sections, warnings and their fixes, More items, the Buy, Clone, Share, Clear skips and Delete dialogs and their flows, first-deck hint | accordion or columns, charts | M |
| Add cards | sources (Search, Maybeboard), From and Synergy, the stack, paging, every swipe outcome, undo through the shared log, in-flight adds, land and budget crossings, end of results, all 43 toasts | gestures and card motion, image loading | XL |
| Card filter | draft and Apply, 15 sections, per-host reset, picker sources, contradiction and catalog toasts | accordion or columns, steppers and chips as drawn | L |
| Deck cards view | load, filter and group pass, boards, Show, quick add, quantities with the 300 ms flush, moves, printings, MVP, Undo, Reset filter, every toast | detail pane, drag to move, row drawing | XL |
| Remove cards | load, board filter, cycling stack, swipe outcomes, undo, crossings, toasts | gestures | M |
| Printing sheet | list, pick, toasts | layout | S |
| Image preview | which image and face | all of it is drawing | none |
| Account | both lists, the three change forms, verification with resend cooldown, delete countdown, More, dialogs | inputs, dialogs as drawn | M |
| Config | theme pick and preview, dark mode, pips, Universes Beyond and its exceptions, toasts | the app icon picker (app only), theme drawing | M |
| Privacy policy | nothing beyond the source text already in core | rendering; the TUI's parser stays in the TUI | none |
| Changelog | filing and the filter | web anchors | S |

Adding it up: about 45 days of work on the screen models if one agent does it all, plus the pieces in Part 1, much of which comes along with the screens. Waves B to D need Home through Remove. Wave A is built already, and moving web-a's domain files into zwipe-ui is a small job (decision 7).

## Drift

The ports are faithful. Every timing, cap and page size matches, the one-time hint copy matches word for word in all three clients, and so do the toast bands. What differs:

Where the TUI differs from the app:

1. Home, cards swiped: past i64 the app shows the figure as still loading (`checked_add`, tested); the TUI saturates and shows i64::MAX (`tui/domain/home/mod.rs:36`).
2. Home, `next_hour`: the app keeps the fraction of a second, leaves at least a second and returns the max time on overflow; the TUI truncates and returns `now` on overflow, which has already expired.
3. Home, has-data: the TUI counts any color code with cards; the app counts only WUBRG after conversion. The TUI also has this check twice (home and playstyle).
4. `?` topics, RoleRates: the TUI says "rate" where the app now says "share", drops "A grey role has none yet", and keeps ", for roles dealt at least 10 times", which the app removed. TagRates bullet 1 has the same rate and share swap. The TUI's copy is from before the app changed.
5. `?` topics, AppIcon: missing in the TUI and in web-a. That's probably deliberate in both.
6. Card faces: the TUI uses `card_faces` whenever there are any; the app only when the card has no top-level oracle text. The app passes no defense for a face; the TUI shows it. The app drops faces with no text and no type line.
7. Changelog: when the newest release has no readable date, the app files upcoming releases under the first release with one; the TUI files them under Earlier. The TUI also sorts year and month chips by key where the app keeps first-seen order (same result while the server sends newest first).
8. Session: the app and web-a check refresh-token expiry first; the TUI checks access-token freshness first, so it can hand out a session whose refresh token is already dead.
9. Catalog: the TUI never refreshes a loaded list (the app refreshes after 24 h) and lacks keyword reminders, Universes Beyond franchises and deck tags.
10. Filter Apply when nothing changed: the app disables the button; the TUI still runs it, so Add Search with an empty filter toasts "Filter is already empty".
11. The TUI's register screen repeats the password rule instead of calling `domain/account`.
12. Activity's `year_line` (the screen reader's summary) isn't ported.
13. `tui/inbound/widgets/stats.rs` holds an unused second counter roller; it should be deleted rather than lifted.

Where both ports differ from the app:

14. Link expiry: the app tests `left <= 0`; both ports test whole seconds, so with under a second left the app says "1 minute" and the ports say expired.

Where the app has the bug and the TUI doesn't:

15. Parked stack trim: `add_stack_cache.rs:57` drains `..cut` without clamping and can panic if the index passes the end; the TUI clamps.
16. `retreat_wrapping`: after a removal the app can leave the index out of range (`card_stack.rs:214`); the TUI clamps to the last card.

Duplicates inside one client: the app has `MAX_LAND_TARGET` again beside core's, the oracle tag reseed in both `create.rs` and `edit.rs`, and the land and budget crossings in three screens. `urlencode` is written in all three clients, `front_face` three times (app, TUI and a core variant), and the email wording table three times.

The spec `4-cards-add-filters.md` has a "Gaps and differences" section (from line 895) written before the port. It says the TUI has no filter sheet, no stores and no serve gate, and it has all three now.

## Moves to zwipe-core

- `PASS_BASELINE_WEIGHT` (10.0) lives in zerver's domain; the app's `TAG_BASELINE_WEIGHT` copies it and says the server weighs the same way. One constant in core.
- `identity_of`, `identities_present`, `has_type` and `front_face` are card rules that sit beside core's `colors.rs` and `commander_eligibility.rs`.
- A `OneTimeHint` enum over core's 20 `HINT_*` strings, which web-a already wrote.
- The below-minimum deck warning is matched on its text (`"requires at least"`). It should be a `WarningAction` variant, once a minimum-version floor lets the app rely on it.

## Stays in one client

The TUI's palettes and their generator scripts, the privacy policy parser, mana symbol tokens, and the terminal's preview of the next card. The app's gesture engine, app icon, haptics, keyring, crash capture and shake to edit. The web's changelog anchors, `localStorage` and its timers.

## Open decisions for the owner

1. **orangui's pure pieces.** `short`, `with_separators`, `peak_indices`, the toast bands and the count-up and decode math sit in orangui, which depends on Dioxus, so the TUI and zwipe-ui can't use them. Options: (a) a Dioxus-free `orangui-core` crate or module in the orangui repo that orangui, zwipe-ui and the TUI all use; (b) move them into zwipe-ui and leave orangui's own copies, which keeps two copies across two repos; (c) leave the TUI's copies where they are. I'd pick (a). It touches a repo outside zwipe.
2. **Hint copy format.** The app and web write emphasis in RSX with CSS variable names; the TUI uses a small string markup (`[Key|role]`, `{word|role}`). A shared table needs one neutral form. Typed spans (`Line` and `Bullet` blocks of `Text`, `Key` and `Emph` spans with a `Role` enum) or the TUI's string markup with a parser per client. I'd pick typed spans; the app then renders them through its existing `HintKey` and `HintColored`, and the brief's rule that lifted pieces render byte-identically applies.
3. **zwipe-client.** Screen models produce effects naming zwipe-core contracts and take replies as results. Should zwipe-ui depend on zwipe-client for `ClientError`, so the error-to-message mapping is shared too? It pulls in reqwest, which builds for wasm32 already. I'd say no for now: each client maps its error to a message before handing it in.
4. **App bugs the TUI fixed** (drift 15 and 16). The brief says keep the app's behavior. A panic isn't behavior worth keeping. I'd lift the clamped versions and note the change in the PR.
5. **Copy the app changed later** (drift 4). The app's current RoleRates and TagRates copy goes in, and the TUI picks it up when it switches.
6. **Deck context and the warning fix in core.** `ensure_lands_excluded`, the legality-plus-identity default and the serve gate read like deck rules. I haven't confirmed the server applies the same ones. Put them in zwipe-ui for now and revisit?
7. **web-a's domain files.** Wave A ported account validation, counts, session, version gate and support, but `feat/web-a` hasn't merged into `feat/web`. Move those into zwipe-ui before or after it merges?
8. **Telemetry.** The app's usage buffer and screen vocabulary are pure and tested, but neither port sends usage counts yet. Leave them in zwiper until a second client needs them?
9. **Screen-model scope.** All 28 screens, or the waves B to D screens first, with Wave A's screens lifted when the TUI or the app needs them?
