# zwipe-ui inventory

Step 1 of the zwipe-ui plan. The owner approved every recommendation below; step 2 built the crate and the core moves, and the Status column says where each piece landed. Screen models (Part 2) are not built yet. `shared-plan.md` holds the corrections made to this file on 2026-11-01, the drift still to settle and the ordered branches that follow.

zwipe-ui is the crate all three clients pull their screen logic from: zwiper (the app), zwiper-web and zwiper-tui. It names no UI crate, so it builds for iOS, Android, the terminal and wasm32 alike. The owner's direction is to share as much as possible, so this goes past pure helpers to a screen model per screen: its state, the actions it takes, what each action asks the client to do, and its verbatim copy. Each client then maps a tap, click or key to an action and draws the state.

Sources read: zwiper and components on `origin/main` (72f67e16), zwiper-tui on `origin/feat/tui` (5b770f0e), zwiper-web on `origin/feat/web` (b8d7c242) and `origin/feat/web-a`, orangui at the pinned e6719eb. Paths below are short: `app/` is `zwiper/src/lib/inbound/`, `tui/` is `zwiper-tui/src/lib/`, `web-a/` is `zwiper-web/src/lib/` on `feat/web-a`.

## What zwipe-ui can depend on

zwipe-core, orangui-core, chrono, uuid and email_address. Not orangui, because orangui depends on Dioxus. Not zwipe-client (decision 3).

Resolved by decision 1: orangui's pure pieces (`short`, `with_separators`, `peak_indices`, the toast bands, the count-up and decode math) now live in orangui-core, which tracks orangui main on the same source as components' orangui, so Cargo.lock holds one copy.

## Part 1: the pieces

Home is where each piece goes: **ui** is zwipe-ui, **core** is zwipe-core, or a client name when only that client needs it. Waves are zwiper-web's: A (auth, chrome, account, config, mostly built on `feat/web-a`), B (home, activity, playstyle), C (decks), D (cards, add, filters).

| Piece | App today | TUI today | Web | Home | Drift | Waves | Status |
|---|---|---|---|---|---|---|---|
| Color map model: growth, pockets, shades, rings, ghost | `components/src/color_map.rs` (model plus Dioxus in one file, 18 tests) | `tui/domain/color_map` with `golden.txt` | B: Home | ui; components is to keep the Dioxus half and call it (shared plan K4) | none; all 20 constants equal | B | in zwipe-ui `color_map`, golden test passing; the TUI switched; components still has its own `ColorMapModel` and `seed_from_id` |
| Count-up rolling figures, `short`, `with_separators` | orangui `entrance.rs` (private fns inside signal loops) | `tui/domain/count_up` | A, B (heroes, figures) | see decision 1 | none in what was ported; TUI lacks reduced motion, splash pacing, shimmer | A, B | orangui-core (`number`, `count_up`); `short` and `with_separators` used from there; the TUI switched |
| Logo decode frames | orangui `entrance.rs` | `tui/domain/decode` | none (web has no splash) | see decision 1 | none; TUI rounds 850 and 450 ms to whole 40 ms ticks on purpose | none | orangui-core `decode`; the TUI switched |
| Activity math: heatmap layout, readout tags, totals, rate kinds, role and tag rate bars | `app/screens/activity/sections.rs` (pure, 16 tests) | `tui/domain/activity` | B: Activity | ui; the baseline weight to core | `year_line` missing in TUI | B | in zwipe-ui `activity`, `year_line` added; peaks from orangui-core; weight from core; the TUI switched |
| Home figures, count scope, flavor card expiry, has-data | `app/components/home_hero.rs`, `zwiper/src/lib/outbound/count_scope_store.rs`, `session_upkeep.rs` `next_hour` | `tui/domain/home` | A built `counts.rs`; B: Home | ui | three edge cases, see Drift | A, B | in zwipe-ui `home`; drift 1 to 3 fixed; the TUI switched |
| Buy links, deck and single card | `zwiper/src/lib/outbound/buy_links.rs` (pure, untested) | `tui/domain/deck/buy.rs`, `tui/domain/home` (deck only) | B: flavor card; C: Buy | ui | TUI lacks single-card links | B, C | in zwipe-ui `buy`, with single-card links; the TUI switched |
| Playstyle helpers: identities, Show filter, reasons, summary, roles, tag table, staples | `app/screens/deck/suggested_commanders.rs` (pure, ~26 tests) | `tui/domain/playstyle` | B: Playstyle | ui; identity and type-line helpers to core | none | B | in zwipe-ui `playstyle`; identity and type-line helpers in core; the TUI switched |
| Deck list Show filter and Group by | `app/screens/deck/list.rs` (2 tests) | `tui/domain/deck/list.rs` | C: Deck list | ui | none | C | in zwipe-ui `deck::list`; the TUI switched |
| Deck row count and pips | `list.rs` `row_card_count` | `tui/domain/deck/mod.rs` | C | ui | web writes `"{count} cards"` always, so one card reads "1 cards"; `cards_label` writes "1 card", which the owner chose for every client (shared plan D1) | C | in zwipe-ui `deck`; the TUI and the app use `cards_label` (the app since `shared/app-followups`) |
| Deck form: land stepper, price box, format details, command-zone search rules, knock-on clears, oracle tag reseed, change detection | `app/screens/deck/components/deck_fields.rs`, `format_select.rs`, `create.rs`, `edit.rs` (signals, untested) | `tui/domain/deck/form.rs`, `tui/inbound/screens/deck_form` | C: Create and Edit | ui | none; app writes the reseed twice | C | in zwipe-ui `deck::form`: the land stepper, price box, format text, command-zone search rules, reseed and import outcome, plus `FormValues` (every field Create and Edit hold) with `saved`, `create_request`, `update_request` against the saved copy (change detection), the knock-on clears (`pick_format`, `clear_format`, `set_commander` returning a `Cascade`, and `clears_mates` for the first-load rule) and slot visibility (`shows`), shared plan branch 5. The app switched its Create, Edit and `deck_fields.rs` to these; the TUI switched to the first set. The picker flow is branch 6; the app's land stepper uses `step_land_target` and its `MAX_LAND_TARGET` copy is gone |
| Import outcome toast | `app/screens/deck/import.rs` | `form.rs` `import_outcome` | C: Import | ui | none | C | in zwipe-ui `deck::form`; the TUI switched |
| Export text | `app/screens/deck/export.rs` | `tui/domain/deck/export.rs` | C: Export | ui | none | C | in zwipe-ui `deck::export`; the TUI switched |
| Warning fix choice | `app/screens/deck/components/deck_warnings.rs` | `tui/domain/deck/warning.rs` | C: Deck view | ui now, core later (decision 6) | none | C | in zwipe-ui `deck::warning`; the TUI switched |
| Create gate and deck limits | `app/screens/deck/list.rs`, `suggested_commanders.rs`, `commander_maybeboard.rs` (3 copies) | `tui/inbound/screens/decks`, `playstyle` | C: Deck list, Playstyle, maybeboard | ui; the four limits to core | none: all 8 copies matched | C | `zwipe_ui::deck::create_gate` and `VERIFY_TO_CREATE` (warning, LONG); the four limits in `zwipe_core::domain::deck::limits`, re-exported by zerver; the app switched, web and TUI to switch |
| Deck view rules and toasts: budget, tags, metrics, share link, total price, clone name, success toasts | `app/screens/deck/view.rs`, `deck_budget_section.rs`, `deck_tags_section.rs`, `more_buttons.rs`, `clone_deck_dialog.rs`, `deck_warnings.rs` | `tui/inbound/screens/deck` | C: Deck view | ui | the TUI's clone toast names the server's returned name, the app the typed one; the app wins | C | in zwipe-ui `deck::view` (`Done` toasts are Info, QUICK); the app switched, web and TUI to switch |
| Commander maybeboard rules and swipe select re-deal | `app/screens/deck/commander_maybeboard.rs`, `deck/components/swipe_select.rs` | `tui/inbound/screens/commander_maybeboard`, `commander_swipe` | C: maybeboard, Swipe select | ui | `step_highlight` takes `Step` (web used a bool); the TUI keeps its plain usize cursor for now | C | in zwipe-ui `commander::{maybeboard, swipe_select}`; the app switched, web and TUI to switch |
| Card filter Apply decision | `card_filter_sheet.rs` `apply_action` (4 tests) | `tui/domain/filter/mod.rs` | D: filter panel | ui | Apply when unchanged, see Drift | D | in zwipe-ui `filter`; drift 10 fixed in `filter::draft`; the TUI switched |
| Filter sections, fields, pickers (15 sections, mode chips, picker sources and caps) | the 25 files under `app/screens/deck/card/filter/` (inside components) | `tui/domain/filter/{sections,fields,picker}.rs` | D | ui | none | D | in zwipe-ui `filter::{sections, fields, picker}`; the TUI switched |
| Filter draft, snapshot, reset rules per host | `card_filter_sheet.rs`, `add.rs` `clear_filters` | `tui/inbound/screens/card_sheets/filter` | D | ui, returning an outcome instead of toasting | none | D | in zwipe-ui `filter::draft`, returning toasts; the TUI switched |
| Add deck context: lands excluded, legality plus identity defaults, serve gate | `add.rs` | `tui/domain/filter/mod.rs`, `tui/domain/add` | D | ui; possibly core (decision 6) | none | D | in zwipe-ui `filter` and `add` (decision 6); the TUI switched |
| Per-deck filter store | `app/screens/deck/card/components/filter_store.rs` | `tui/domain/stores/filters.rs` | D: the stores | ui | none | D | in zwipe-ui `stores::filters`; the TUI switched |
| Parked add stacks (20 decks, 50 behind) | `add_stack_cache.rs` | `tui/domain/stores/parked.rs` | D | ui | app can panic on trim, see Drift | D | in zwipe-ui `stores::parked`, clamped (decision 4); the TUI switched |
| Shared undo log (cap 100) | `undo_log.rs` (3 tests) | `tui/domain/stores/undo.rs` | D | ui | none; TUI merges the live and parked halves into one store | D | in zwipe-ui `stores::undo`; the TUI switched |
| In-flight adds | `in_flight_adds.rs` (4 tests) | `tui/domain/stores/in_flight.rs` | D | ui | none | D | in zwipe-ui `stores::in_flight`; the TUI switched |
| Catalog cache: which lists, freshness | `app/components/catalog_cache.rs` | `tui/domain/stores/catalog.rs` | D: picker sources | ui (freshness and slots); fetching stays in clients | TUI has no 24 h TTL and 3 fewer lists | B, C, D | in zwipe-ui `stores::catalog`, 24 h TTL and all ten lists; the TUI switched |
| Card stack, linear and cycling, and its actions | `card_stack.rs`, `action_history.rs` (no tests) | `tui/domain/stack` (8 tests) | C: Swipe select; D: Add, Remove | ui | app lacks a clamp, see Drift | C, D | in zwipe-ui `stack`, clamped (decision 4); the TUI switched |
| Add rules: page 25, load-more at 5, seed, land target, empty-results copy, related ids | `add.rs` (closures over signals) | `tui/domain/add` | D: Add | ui | none | D | in zwipe-ui `add` and `timing`; the TUI switched |
| Deck cards pass, featured strip, quantity bursts, undo messages, stale check | `app/screens/deck/card/view.rs` (5 tests) | `tui/domain/deck_cards` | D: deck cards view | ui | none | D | in zwipe-ui `deck_cards`; the TUI switched |
| Land and budget crossings | written three times: `view.rs`, `add.rs`, `remove.rs` | `tui/domain/deck_signals` | D | ui | none | D | in zwipe-ui `deck_signals`; the TUI switched |
| Changelog filing and filter | `components/src/changelog.rs` (pure, tested) | `tui/domain/changelog` | A: Changelog | ui; web keeps its anchors | two edge cases, see Drift | A | in zwipe-ui `changelog`; drift 7 fixed; the TUI switched; components still files releases itself (`components/src/changelog.rs:158`, shared plan K5) |
| Card as text: which faces, stats line, rules | `components/src/card_details.rs` | `tui/domain/card` | D: details | ui for face choice and stats; mana symbol parsing stays in TUI | three differences, see Drift | B, D | in zwipe-ui `card`; drift 6 fixed; the TUI switched |
| Privacy policy HTML to text | renders the HTML as is | `tui/domain/legal` | renders the HTML as is | tui | n/a | none | stays in zwiper-tui |
| Theme palettes as RGB per role | n/a (CSS) | `tui/domain/theme`, `scripts/gen_themes.py`, `gen_theme_extras.py` | n/a (CSS) | tui | n/a | none | stays in zwiper-tui |
| Account form wording and checks | the account and auth screens (`zwiper/src/lib/domain/error.rs` until branch 3) | `tui/domain/account`, `tui/inbound/screens/register` | `web-a/domain/validation.rs`, `account.rs` | ui | link expiry boundary; TUI repeats the password rule | A | in zwipe-ui `account`; drift 14 matched to the app; the TUI switched to the first set. Branch 3 added Register's `password_error`, Forgot password's `reset_email_error` and the `PASSWORDS_DIFFER`, `PASSWORD_REQUIRED`, `PASSWORD_UNCHANGED`, `INVALID_RESET_EMAIL` constants; the app switched to every account check and its `domain/error.rs` is gone; web and TUI switch to branch 3's pieces on their branches. The app's own the app's pending email note now comes from `account::pending_note` (shared plan D3) |
| Log in gate | `app/screens/auth/login.rs` `submit_gate` | `tui/screens/login/keys.rs`, `EMPTY_FORM` | `web/domain/login.rs` | ui | none: all three trim the identifier only and raise an error at 5 s | A | in zwipe-ui `account::login_gate`, `MISSING_INPUTS` and `missing_inputs_toast` (branch 3); the app switched; web and TUI switch on their branches |
| Resend and Delete account labels | `app/screens/account/components/email_verification.rs`, `delete_account_dialog.rs` | `tui/screens/account/mod.rs` `resend_label`, `DeleteDialog::label` | `web/domain/account.rs` | ui | none in the words; the TUI folds "Deleting..." into its label, which the app keeps on its busy button | A | in zwipe-ui `account::{resend_label, delete_label}` (branch 3); the app switched; web and TUI switch on their branches |
| Home greeting | `app/screens/home.rs` | `tui/screens/home/mod.rs` `greet` | `web/inbound/screens/home/mod.rs` | ui; the once-per-launch flag stays in each client | the TUI nudges as an error and both toasts sit at the 5 s band; the app and web use a warning and an info at 3 s (shared plan D6) | B | in zwipe-ui `home::greeting` with `Greeting` and `UserSource` (branch 3), the app's kinds and band; the app switched; web and TUI switch on their branches |
| Session freshness and refresh-rejected rule | `app/components/auth/ensure_session.rs` | `tui/outbound/account` | `web-a/domain/session.rs` | ui; storage and timers stay in clients | TUI checks in a different order | A | in zwipe-ui `session`; drift 8 fixed; the TUI switched |
| Version gate change | `session_upkeep.rs` | `tui/inbound/app/session.rs` | `web-a/domain/version_gate.rs` | ui | none | A | in zwipe-ui `session::update_required`; the TUI switched |
| Problem report mailto, platform as a parameter | `app/components/support.rs` | `tui/inbound/widgets/help.rs` | `web-a/domain/support.rs` | ui | none; `urlencode` is written three times | A | in zwipe-ui `support`; the TUI switched |
| `?` hint topics: titles and bodies | `app/components/hint_host.rs`, `concept_explainers.rs` (RSX) | `tui/inbound/widgets/hint/topics.rs` (markup strings) | `web-a/inbound/chrome/hint_topic.rs` (RSX) | ui, as typed copy (decision 2) | TUI has stale RoleRates and TagRates copy | A, and every wave | in zwipe-ui `hint::topics` as typed spans, app copy, AppIcon included; the TUI switched |
| One-time hints: ids and copy | ids in `zwipe-core/src/domain/user/models/hints.rs`; copy in 19 screens | `tui/inbound/widgets/hint/keys.rs` | `web-a/inbound/chrome/one_time_hint.rs` (an enum over the ids) | ids enum in core, copy in ui | none; verbatim in all three | A, and every wave | `OneTimeHint` in core; copy in zwipe-ui `hint::one_time`; the TUI switched |
| Toast bands 3, 5 and 8 s | orangui `toast_timing.rs` | `tui/inbound/widgets/toast.rs` (copied) | orangui | see decision 1 | none | all | orangui-core `toast`, used by zwipe-ui `toast`; the TUI switched |
| Behavior timings: debounces 300 and 800 ms, quantity flush 300, resend 60 s, delete 5 s, upkeep 60 s, counts 60 s, catalog 24 h, heat pick 4 s, maybeboard toast 1.5 s | spread across the app | spread across the TUI | partly in web-a | ui, as named constants | none | all | in zwipe-ui `timing`; the TUI switched; `HEAT_PICK` derives from orangui-core `heat::PICK_MS`, one source (shared plan branch 2) |
| Search limits: min query length, result caps 8 and 5, page 25, held size, tile counts | spread | spread | none yet | ui | none | B, C, D | in zwipe-ui `timing`; the TUI switched; `EXAMPLES_MAX_CARDS` is `stack::MAX_CARDS_IN_STACK`, as the app's examples screen uses it |
| Picker change check (`same_set`: two slug lists equal ignoring order, duplicates counted) | `app/screens/config/universes_beyond.rs`, `deck/components/tag_select.rs`, `oracle_tag_select.rs` | `tui/inbound/screens/deck_form/pickers.rs`, `config/mod.rs` | `domain/deck.rs`, `domain/preferences.rs` | ui | none: all seven copies agree | C | in zwipe-ui `filter::fields::same_set`; the app switched (shared plan branch 2); web and TUI switch on their branches |
| Deck form pickers: draft and Save/Back, tag toggles and caps, format, deck tag and oracle tag results, the dictionary's Use, bar copy, `?` content, partner autofill guard and toast | `app/screens/deck/components/{format_select,tag_select,oracle_tag_select,deck_fields}.rs` | `tui/inbound/screens/deck_form/{pickers,mod}.rs` | `domain/deck.rs`, `deck/form/pickers.rs`, `form/command_zone.rs` | ui | D10: the TUI's format Back, oracle grid and hardcoded caps; the app wins | C | in zwipe-ui `deck::pickers`; the app switched (its deck and oracle tag `?` bodies stay as explainers), web and TUI to switch |
| Oracle tag dictionary: letter buckets, the A to Z rail with `#`, browse and capped search rows | `app/screens/oracle_tag_dictionary.rs` | `tui/inbound/screens/oracle_tags/mod.rs` | `inbound/screens/oracle_tag_dictionary.rs` | ui | none: all three copies agree | D | in zwipe-ui `dictionary` (`bucket_of`, `letters`, `rows_for`, `OTHER`); the app switched, web and TUI to switch |
| Config: preference changes and undo, the sparse update, stale-reply settle for the toggles, exception chip names, saved and unchanged toasts | `app/screens/config/{mod,preferences,universes_beyond}.rs` | `tui/inbound/screens/config` | `domain/preferences.rs` | ui | none in words, kinds or bands; the app icon toast stays in the app | D | in zwipe-ui `preferences`; the app switched, web and TUI to switch (`Toggles` is for the TUI) |
| Filter chip rows: options, which chip shows selected, what a tap writes | inline in the app's filter screens | `tui/.../card_sheets/filter/mod.rs` (by index) | `domain/filter_chips.rs` | ui | the TUI highlights the Sort order chip, the app doesn't; the app wins | D | in zwipe-ui `filter::chips`; the app switched Is commander in, Is legal in, Sort by, currency, Doesn't produce and Card roles exclude; its rows with a mode chip, basic types, color identity, rarity and Config stay inline; web and TUI to switch |
| Card info lines: price, set, release, artist | `app/screens/deck/card/components/card_info.rs` | `tui/inbound/widgets/card_info.rs` | `inbound/cards/card_info.rs` | ui | none in words; each client picks its own blank line (the app a non-breaking space) | C | in zwipe-ui `card::info`; the app switched, web and TUI to switch |
| Command zone: role labels, group headers, row art order, the slot update request | `app/screens/deck/card/view.rs` (featured strip, group headers, printing swap and its undo), `deck/view.rs` (clear commander), `deck/components/deck_profile.rs`, `deck_fields.rs`, `deck/list.rs` `command_zone_art`, `card/components/undo_log.rs` `CommandZoneSlot` | `tui/inbound/screens/deck_form/draw.rs`, `deck/draw.rs`, `deck_cards/mod.rs` (`push_zone`, `save_zone_printing`) | `domain/deck.rs` `command_zone_art`, `deck/view/cards.rs` `slot_label`, `deck/cards/view.rs` `zone_header` and `slot_request`, `deck/view/sections.rs`, `deck/view/mod.rs`, `deck/form/fields.rs` | ui | none: every client uses the app's words and order (shared plan D2) | C, D | in zwipe-ui `deck::command_zone`: `Slot` (moved from `deck::form`, which re-exports it) with `label`, `group_header` and `update_request`, and `art` (shared plan branch 4); the app switched and its `CommandZoneSlot` is gone; web and TUI switch on their branches |
| HTTP status of a client error | n/a | matches on variants (`tui/outbound/account`) | private `status(&ClientError)` in `inbound/session/ensure_fresh.rs` | zwipe-client, `ClientError::status` | none | A | in zwipe-client; web switches on its branch |
| Animation timings: splash, stagger, entrance, wipe, spinner, sweeps, card motion | orangui CSS and JS, `main.css` | `tui/inbound/widgets` | CSS | each client | none | none | stays in each client |
| Generation ticket for out-of-order replies | `app/components/generation.rs` (Dioxus `CopyValue`) | in the TUI's loop | none yet | ui as a plain counter; app keeps its hook over it | n/a | C, D | in zwipe-ui `generation`; the TUI switched |
| Swipe gesture engine | `app/components/interactions/swipe` (33 tests) | keys | keys and buttons | zwiper | n/a | none | stays in zwiper |
| Usage telemetry buffer and screen vocabulary | `app/components/telemetry` (11 tests) | `zwiper-tui/src/lib/outbound/usage_buffer.rs`, copied from the app; sends errors only, swipes and signals as zero | anonymous funnel only | ui (shared plan decision 2, which reverses decision 8) | TUI and web swipes count nowhere | none | in zwipe-ui `telemetry` (`UsageBuffer`, a plain struct the client locks; `HandledError`, `ErrorKind`, `SwipeDirection`, `vocabulary`); the app switched behind a wrapper, its flush loop, `ClientError` mapping and logging stay in zwiper; web adds a flush, TUI swaps its copy; decision 8 resolved by decision 2 |

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
| Login | form state, submit, the Register prefill, error and toast copy | inputs and focus | S, built: `screens::login`; the app switched, web and TUI to switch; storing the session stays in the client |
| Register | fields, live errors (web-a's `validation.rs` is the model), submit, toasts | inputs | S, built: `screens::register`; the app switched, web and TUI to switch |
| Forgot password | field, check, submit, toasts | inputs | S, built: `screens::forgot_password`; the app switched, web and TUI to switch |
| Update required | copy | the action differs on purpose (store link, Reload, terminal message) | S, no model by decision: only fragments of copy are shared |
| Help and feedback | copy, the mailto builder with platform | opening mail or the browser | S, no model: copy and the mailto live in `zwipe_ui::support`; the app switched |
| Splash | decode frame math (decision 1) | all drawing; web has none | none |
| Home | hero scope and figures, color map model, flavor card and its hourly expiry, buy link, Playstyle panel states, one-time hint | color map drawing, counter animation, image | M, built: `screens::home`; the app switched, web and TUI to switch; the caches above the router stay in each client |
| Activity | heatmap layout, readout selection and its 4 s release, totals, rate kind switch, both rate charts, Most skipped, empty lines | heat grid drawing, sweeps, chart drawing | M, built: `screens::activity`; the app switched, web and TUI to switch; the heat pick's 4 s release stays in each client |
| Playstyle | profile sections, tag table, suggestions with Show, Staples filters, Clear skips flow, reasons, every empty line | chart and card drawing; sections open together on web | L, built: `screens::playstyle`; the app switched, web and TUI to switch; section openness, the role reveal and the swipe overlay stay in the client |
| Commander maybeboard | list, quick add with its 300 ms debounce, color filter, remove, clear, hand-off to Create | swipe gesture | M, built: `screens::commander_maybeboard`; the app switched, web and TUI to switch; the card filter, Art chip and swipe overlay stay in the client |
| Swipe select (commander and suggested modes) | the stack, the per-mode constraint, maybeboard save and undo, toasts | the gesture engine and card motion | L, built: `screens::swipe_select`; the app switched, web and TUI to switch; gestures, the entering signal and the filter counter stay in the client |
| Oracle tag dictionary | search, definition, Use hand-off | overlay or panel layout | S, built: `screens::oracle_tag_dictionary`; the app switched, web and TUI to switch; Use keeps the dictionary open (the TUI goes back, drift to settle), and the catalog failure toasts each time (web and TUI once per visit) |
| Oracle tag examples | load, paging, empty and error lines | card drawing | S, built: `screens::oracle_tag_examples`; the app switched, web and TUI to switch; a failed first page shows the empty line, as the app does |
| Deck list | the whole TUI `DeckList` minus focus: decks, Group by, Show, tag fold, collapsed groups, Create gate, More items | chip and list drawing | S, built: `screens::deck_list`; the app switched, web and TUI to switch |
| Create and Edit | every field, validation, knock-on clears, change detection, the four command-zone searches with debounce and generation tickets, the format, tag and oracle tag pickers (draft, Save disabled when unchanged, Back with "unchanged"), submit and server messages | text inputs, picker layout (sheet, side panel, overlay) | L, built: `screens::deck_form`; the app switched, web and TUI to switch; text inputs and the swipe overlays stay in the client |
| Import | paste, submit, outcome toast | text area | S, built: `screens::deck_import`; the app switched, web and TUI to switch |
| Export | board toggles with the at-least-one rule, the text, copy toast | clipboard API | S, built: `screens::deck_export`; the app switched, web and TUI to switch |
| Deck view | load, profile, stats, sections, warnings and their fixes, More items, the Buy, Clone, Share, Clear skips and Delete dialogs and their flows, first-deck hint | accordion or columns, charts | M, built: `screens::deck_view`; the app switched, web and TUI to switch; which dialog is up stays in the client |
| Add cards | sources (Search, Maybeboard), From and Synergy, the stack, paging, every swipe outcome, undo through the shared log, in-flight adds, land and budget crossings, end of results, all 43 toasts | gestures and card motion, image loading | XL, built: `screens::add_cards`; the app switched, web and TUI to switch; the deck's command zone now comes from the deck reply instead of four card fetches |
| Card filter | draft and Apply, 15 sections, per-host reset, picker sources, contradiction and catalog toasts | accordion or columns, steppers and chips as drawn | L, built: `screens::card_filter`; the app switched the sheet-level flow (open, Apply, Back, Reset, section clears, the dictionary); the section bodies still edit the host's signal; web and TUI to switch |
| Deck cards view | load, filter and group pass, boards, Show, quick add, quantities with the 300 ms flush, moves, printings, MVP, Undo, Reset filter, every toast | detail pane, drag to move, row drawing | XL, built: `screens::deck_cards`; the app switched, web and TUI to switch; the filter, its counter and the undo log stay the client's; retire `deck_cards::Bursts` once web and TUI switch |
| Remove cards | load, board filter, cycling stack, swipe outcomes, undo, crossings, toasts | gestures | M, built: `screens::remove_cards`; the app switched, web and TUI to switch; the undo log stays the client's (the app's `undo_log.rs` still duplicates `zwipe_ui::stores::undo`, so `remove.rs` converts between them) |
| Printing sheet | list, pick, toasts | layout | S, built: `screens::printing_sheet`; the app switched, web and TUI to switch; the carousel index stays in the client |
| Image preview | which image and face | all of it is drawing | none |
| Account | both lists, the three change forms, verification with resend cooldown, delete countdown, More, dialogs | inputs, dialogs as drawn | M, built: `screens::account`; the app switched, web and TUI to switch; the clock stays in the client (`Wait`/`Tick`) |
| Config | theme pick and preview, dark mode, pips, Universes Beyond and its exceptions, toasts | the app icon picker (app only), theme drawing | M, built: `screens::config`; the app switched, web and TUI to switch; theme and pips stay the client's, the app icon stays in the app |
| Privacy policy | nothing beyond the source text already in core | rendering; the TUI's parser stays in the TUI | none |
| Changelog | filing and the filter | web anchors | S, built: `screens::changelog`; the app takes status, copy and Back; components still draws its own chips until K5 |

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

Found after the port:

17. Deck row count: the app and web write `"{count} cards"` always, so one card reads "1 cards" (`app/screens/deck/list.rs:365`); the TUI uses `cards_label`, which writes "1 card". The owner decided "1 card" everywhere, app included. The app fix is a later branch.

Duplicates inside one client: the app has `MAX_LAND_TARGET` again beside core's, the oracle tag reseed in both `create.rs` and `edit.rs`, and the land and budget crossings in three screens. `urlencode` is written in all three clients, `front_face` three times (app, TUI and a core variant), and the email wording table three times.

The spec `4-cards-add-filters.md` has a "Gaps and differences" section (from line 895) written before the port. It says the TUI has no filter sheet, no stores and no serve gate, and it has all three now.

## Moves to zwipe-core (done)

- `PASS_BASELINE_WEIGHT` (10.0) lives in zerver's domain; the app's `TAG_BASELINE_WEIGHT` copies it and says the server weighs the same way. One constant in core.
- `identity_of`, `identities_present`, `has_type` and `front_face` are card rules that sit beside core's `colors.rs` and `commander_eligibility.rs`.
- A `OneTimeHint` enum over core's 20 `HINT_*` strings, which web-a already wrote.
- Not done yet: the below-minimum deck warning is matched on its text (`"requires at least"`). It should be a `WarningAction` variant, once a minimum-version floor lets the app rely on it.

## Stays in one client

The TUI's palettes and their generator scripts, the privacy policy parser, mana symbol tokens, and the terminal's preview of the next card. The app's gesture engine, app icon, haptics, keyring, crash capture and shake to edit. The web's changelog anchors, `localStorage` and its timers.

## Decisions (all approved as recommended)

1. **orangui's pure pieces.** `short`, `with_separators`, `peak_indices`, the toast bands and the count-up and decode math sit in orangui, which depends on Dioxus, so the TUI and zwipe-ui can't use them. Options: (a) a Dioxus-free `orangui-core` crate or module in the orangui repo that orangui, zwipe-ui and the TUI all use; (b) move them into zwipe-ui and leave orangui's own copies, which keeps two copies across two repos; (c) leave the TUI's copies where they are. I'd pick (a). It touches a repo outside zwipe.
2. **Hint copy format.** The app and web write emphasis in RSX with CSS variable names; the TUI uses a small string markup (`[Key|role]`, `{word|role}`). A shared table needs one neutral form. Typed spans (`Line` and `Bullet` blocks of `Text`, `Key` and `Emph` spans with a `Role` enum) or the TUI's string markup with a parser per client. I'd pick typed spans; the app then renders them through its existing `HintKey` and `HintColored`, and the brief's rule that lifted pieces render byte-identically applies.
3. **zwipe-client.** Screen models produce effects naming zwipe-core contracts and take replies as results. Should zwipe-ui depend on zwipe-client for `ClientError`, so the error-to-message mapping is shared too? It pulls in reqwest, which builds for wasm32 already. I'd say no for now: each client maps its error to a message before handing it in.
4. **App bugs the TUI fixed** (drift 15 and 16). The brief says keep the app's behavior. A panic isn't behavior worth keeping. I'd lift the clamped versions and note the change in the PR.
5. **Copy the app changed later** (drift 4). The app's current RoleRates and TagRates copy goes in, and the TUI picks it up when it switches.
6. **Deck context and the warning fix in core.** `ensure_lands_excluded`, the legality-plus-identity default and the serve gate read like deck rules. I haven't confirmed the server applies the same ones. Put them in zwipe-ui for now and revisit?
7. **web-a's domain files.** Wave A ported account validation, counts, session, version gate and support, but `feat/web-a` hasn't merged into `feat/web`. Move those into zwipe-ui before or after it merges?
8. **Telemetry.** The app's usage buffer and screen vocabulary are pure and tested, but neither port sends usage counts yet. Leave them in zwiper until a second client needs them? Overtaken: the TUI has since copied the buffer, and the owner decided web and TUI send what the app sends (shared plan decision 2, branch 12).
9. **Screen-model scope.** All 28 screens, or the waves B to D screens first, with Wave A's screens lifted when the TUI or the app needs them?

## Follow-ups

- web-a's domain files (validation, counts, session, version gate, support) stay in zwiper-web until `feat/web-a` merges, then move to zwipe-ui (decision 7).
- The deck rules in `filter` and `add` (`ensure_lands_excluded`, the deck context default, the serve gate) move to core once the server is confirmed to apply the same ones (decision 6).
- The pending email note says "Link expires in less than a minute" for anything under a minute (owner-approved). The app used to say "1 minute" from 1 to 59 seconds and "0 minutes" under a second; it now calls `account::pending_note`.
- The app's deck row count moves to `cards_label` so one card reads "1 card" (drift 17, shared plan D1).
- components switches its color map and changelog filing to zwipe-ui (shared plan K4 and K5).
- zwiper-tui uses zwipe-ui for every piece above marked switched, on `feat/tui`. zwiper-web is next; the app moves last, one piece at a time.
- zwiper depends on zwipe-ui on main since shared plan branch 2, through `same_set`, its first switched piece.
