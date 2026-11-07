# Rust audit, 2026-11-07

A scadoshi-rust review of every Rust line changed in the 24 hours from 2026-11-06 12:00 (Pacific): zwipe `origin/release` from `74cd82f8d` (PRs #159 to #290, 252 files, about 11,900 lines added), orangui `main` from `7a9a9ee` (#6 to #15, about 1,000 lines) and Cairn `main` from `29d5982` (the day rollover, lifecycle and Android work, about 230 lines). CSS-only and markdown-only changes are skipped, and so are the open PRs (#289, #291). Cairn's backup and import (`179e25f`, `aaca8b1`) and its page-kill recovery (`18e31c7`) landed later and were added in a second pass, as #65 to #67.

Library or application: zwipe-core, zwipe-ui, zwipe-client, components and orangui are libraries and get the API rules strictly. zerver, zwiper, zwiper-web, zite and Cairn are applications.

Tools first. clippy with the full lint set passes in CI on all three repos, so there are no `unwrap`, `expect`, `panic!`, indexing or unexplained `as` casts outside tests. A grep of every added line found none either, and no `#[allow]`. Every `unsafe` block has a `// SAFETY:` comment and does one operation. What follows is what clippy can't see.

Paths are in the zwipe repo unless they start with `orangui:` or `cairn:`. Severity runs panic, unsafe and behavior first, then docs, naming and style.

## Counts

| Severity | Count |
|---|---|
| panic / unsafe / behavior | 11 |
| docs | 8 |
| naming | 12 |
| style | 36 |
| total | 67 |

Not counted: every new zwiper-web, zwiper and components file opens with `use dioxus::prelude::*`, a glob import (§7). That's the convention across all three crates (85 files at the base in zwiper-web alone), so it's a crate-wide decision rather than a finding against these changes.

## Panic, unsafe and behavior

| # | Where | Rule | Finding | Fix |
|---|---|---|---|---|
| 1 | `zwipe-ui/src/screens/remove_cards/mod.rs:756`, `:638` | behavior: reachable bad state | An undo held while its delete is still out puts the card back in the stack but not in `entries`. A second right swipe on it finds no entry, so it records the default `Board::Deck` and quantity 1 and sends a second `DeleteDeckCard`, which can race the held `RestoreDeckCard`. The undo log then has the wrong board and count. | Put the entry back when the undo is held, or refuse a remove while a restore for that `card_id` is pending. |
| 2 | `orangui:src/lifecycle.rs:119` | §3 no panics | `use_app_lifecycle()` is public and panics by design (`use_context`) when no root is above it. It's documented under `# Panics`, but the library rule is no panics. | Return `Option<AppLifecycle>` through `try_use_context`, as `use_on_resume` already does. |
| 3 | `orangui:src/page_kill.rs:187` | §11 unsafe fn needs `# Safety`; catch_unwind at FFI | `terminated` is an `unsafe extern "C-unwind" fn` that the Objective-C runtime calls, but it has no `# Safety` section and no `catch_unwind`. A panic in it would unwind into UIKit frames. Nothing in it panics today. | Add `# Safety` (called only by the runtime as the method's implementation) and wrap the body in `catch_unwind`. |
| 4 | `cairn:src/lib/outbound/android_fs.rs:31` | §11 FFI error handling | When `getFilesDir` or `getAbsolutePath` throws, the JNI call returns the error but leaves the Java exception pending on the thread. The next JNI call there can abort the app. | On `Err`, call `env.exception_clear()` (after `exception_describe` for the log) before returning. |
| 5 | `zerver/src/lib/outbound/sqlx/auth/mod.rs:342` | behavior | The reuse window remembers a rotation's successor before the transaction commits. If the commit fails, a racing second presentation inside the window gets a token that was rolled back, and its next refresh is `not_found`. It's rare: a failed commit during a two-tab race. | Remember after a successful commit, and keep the losers' blocking re-read, which already waits on the row lock. |
| 6 | `components/src/cover_flow.rs:75-86` | behavior: outside input | The wheel handler takes `strip_units()`, which ignores the delta mode, and steps every 60 px. A line-mode wheel (Firefox, deltaY about 3) needs about 20 notches per printing. The `while` loops also spin on a huge delta, and a NaN delta poisons the sum for good. | Convert lines and pages to pixels, drop non-finite deltas, and compute the step count with one division. |
| 7 | `zwiper/src/lib/inbound/components/held_open.rs:24` | behavior: Dioxus timer | Every close spawns a 250 ms timer that's never cancelled. Close, reopen and close inside 250 ms, and the first timer drops the backdrop early, which cuts the second fade short. | Keep a generation counter (or the `Task`) and ignore or cancel older timers. |
| 8 | `zwiper/src/bin/zwiper.rs:144`, `:147` | behavior | `components::FILTER_CSS` is injected twice, both after `MAIN_CSS`, though its doc says hosts load it before their own sheet. The second copy sits under the comment "The drawer the card details open in." | Keep one copy, placed before the `MAIN_CSS` link, and drop the stray comment. |
| 9 | `orangui:src/os_back.rs:76` | behavior, low | The edge-swipe recognizer and the window fill are keyed by the WKWebView's address. A replacement webview allocated at a freed one's address skips both. | Key by an id the host hands over, or reinstall on every new window. |
| 10 | `cairn:src/lib/inbound/ui/components/counter_list.rs:99`; `cairn:src/lib/inbound/ui/screens/counter.rs:184` | behavior, low | At tap time a failed store read is `unwrap_or_default()`, so the day counts as zero, which skews the goal and celebration logic without a word. | Report the read error as the adjust's own failure does, and skip the tap. |
| 11 | `zwipe-client/src/call.rs:111` | behavior, low: outside input | A negative `Age` header parses and dates the copy in the future. | Parse as `u32`. |

## Docs

| # | Where | Rule | Finding | Fix |
|---|---|---|---|---|
| 12 | `orangui:src/page_kill.rs:353`, `:378` | §8 misleading doc; §9 magic numbers | `cover` says it paints "the app's backdrop", but it hardcodes Gruvbox (`#282828`, `#ebdbb2`) and unnamed layout numbers (64, 32, 70, 140, 48, 17) in a generic kit. | Take the colors from the host, or say they're fixed. Name the layout constants. |
| 13 | `zwipe-core/src/domain/user/models/settings.rs:115` | comment guidelines: no history | "the wire still writes `theme`" narrates a change over time. | "The wire writes `theme`, which 1.15.0 to 1.18.0 parse, and reads both." |
| 14 | `components/src/card_details.rs:310`, `:335` | §8 misleading doc | `show_classification` says the roles show "in the foot", and `footer` says "after the roles". `TagRows` renders in the body. | "in the body, after the facts". |
| 15 | `components/src/card_list_fold.rs:11` | §8 | "Opens open." is garbled. | "Starts open." |
| 16 | `components/src/hover_stack.rs:99` | §8 type docs say what it's for | The public `HoverStack` doc is only "`Copy`; from [`use_hover_stack`]." | Say it holds the one previewed card and its corner, and point to `track` and `leave_held`. |
| 17 | `zwiper/src/lib/inbound/components/mod.rs:45`, `:57` | §8 misleading doc | "Raises zwipe-ui's toasts through the app's toaster." moved from `ui_toast` onto `swipe_confirm`. | Put it back on `ui_toast`, and give `swipe_confirm` its own line. |
| 18 | `zwiper/src/lib/inbound/components/home_hero.rs:95` | comment guidelines: no sermon | "...still reads as a minute old without this." | State it: "The monotonic clock stops during sleep, so elapsed time alone undercounts." |
| 19 | `zwiper-web/src/lib/inbound/chrome/leave_guard.rs:46-52` | §8 no restating the name | `/// The title.`, `/// The body.`, `/// The button that stays.` and `/// The button that leaves.` add nothing to the field names. | Say where each shows, or drop them under an `#[expect(missing_docs, reason)]` on `LeaveCopy`. |

## Naming

| # | Where | Rule | Finding | Fix |
|---|---|---|---|---|
| 20 | `zwipe-client/src/metrics/public_metrics.rs:28` | §5 I/O lookups start with `fetch_` | `public_metrics_dated` (and the existing `public_metrics`) does network I/O. The crate's other lookups are `fetch_deck` and `fetch_settings`. | `fetch_public_metrics_dated`. |
| 21 | `zwipe-ui/src/theme_sync.rs:13` | §5 / §7 ambiguous names | The public enum `Sync` shadows `std::marker::Sync` wherever it's imported. | `ThemeSync` or `Winner`. |
| 22 | `zwipe-ui/src/hint/one_time.rs:57`, `:222` | §5 bool params | `first_deck(wide: bool)` is library API, called as `first_deck(false)`. | An enum (`Layout::Narrow`), or `/* wide */` at the call. |
| 23 | `zwiper-web/src/lib/inbound/cards/swipe_confirm.rs:42`, `:47`, `:53` | §5 bool params | `self.say(false, ..)` and `self.say(true, ..)` pass a bare `success`. | A `Tone` enum. |
| 24 | `zwiper-web/src/lib/inbound/screens/deck/cards/mod.rs:46` (calls in `add.rs:648`, `:669`, `:679` and `remove.rs:413`, `:466`) | §5 bool params | `line.confirm(true, "Removed from deck")`. | The same `Tone` enum. |
| 25 | `zwiper-web/src/lib/inbound/screens/deck/cards/add.rs:646`, `:677` | §5 bool params | `add_card.call((card, true))` passes the `maybe` flag bare. | A `Board` argument, or `/* maybe */`. |
| 26 | `zwiper-web/src/lib/inbound/cards/printing_sheet.rs:216`, `:253`, `:257` | §5 bool params | `step(false)` and `step(true)`. | `step_back` and `step_forward`, or a direction enum. |
| 27 | `components/src/cover_flow.rs:42`, `:62`, `:79`, `:83`, `:144` | §5 bool params | `step(forward: bool)`, the same as #26. | The same fix, shared with #26. |
| 28 | `components/src/hint.rs:168` | §5 model the domain with types | `guide_for_one_time(key: &str)`: every caller holds a `OneTimeHint` and passes `hint.key()`, so the value makes a round trip through a string. | Take `OneTimeHint`. |
| 29 | `zwiper-web/src/lib/domain/landing.rs:31` | §5 don't repeat the module | `landing::landing(..)` stutters. | `landing::decide`. |
| 30 | `zite/src/pages/shared_deck.rs:185` | §9 clarity | `let has_cards = matches!(.., Some(Ok(deck)) if has_cards(deck));` shadows the function inside its own initializer. | Rename the local. |
| 31 | `zerver/src/lib/outbound/sqlx/postgres.rs:134` | §5 constructors | `with_pool` is a constructor from a pool. The `with_` prefix reads as a builder step, like `with_refresh_reuse_window` beside it. | `from_pool`. |

## Style

| # | Where | Rule | Finding | Fix |
|---|---|---|---|---|
| 32 | `zwipe-core/src/domain/user/models/settings.rs:304` | §5 builders | `UpdateSettings::new` is an id plus six optional parameters. Past two optional parameters, the rule is a builder. | `UpdateSettings::builder(id)` with a setter per field. |
| 33 | `zwipe-core/src/domain/user/models/settings.rs:217` | review: messages carry the value | `ParseSwipeConfirmationsError` doesn't name the input it refused. | Carry the string: `unknown swipe confirmations {0:?}`. This changes a log message only. |
| 34 | `zwipe-core/src/domain/deck/models/deck_warning.rs:38` (and `:47`, `:60`) | §4 | `impl Into<String>` is converted, then `sentence_case` copies it again. | Capitalize the owned string in place. |
| 35 | `zwipe-client/src/call.rs:111` | §9 named constants | `"age"` and `"last-modified"` are literals. | `reqwest::header::AGE` and `LAST_MODIFIED`. |
| 36 | `orangui:src/page_kill.rs:451`, `:157` | §13 tests restate constants | The only test asserts `REPEAT_WINDOW == 30s`, and the streak logic in `next_step` is untested. | Give `next_step` an injected `now` and test the three steps. |
| 37 | `orangui:src/lifecycle.rs:143` | §13 | `the_listener_replaces_its_last_copy` asserts substrings of the JS constant. | Drop it, or test the behavior in a page. |
| 38 | `orangui:src/lifecycle.rs:44` | §5 derive eagerly | `AppPhase` lacks `Hash`. | Derive it. |
| 39 | `cairn:src/lib/domain/preferences.rs:428`; `cairn:src/lib/inbound/ui/mod.rs` (`std::time::Duration` x3) | §7 types imported and used unqualified | `chrono::NaiveTime::MIN` and repeated `std::time::Duration` are written out in full. | Import the types. |
| 40 | `zwiper-web/src/lib/inbound/shell/top_nav.rs:63`, `:79` | §4 clone to quiet the borrow checker | `ours` is a second clone of `title`, which already moves into the same effect. | Use `&title`. |
| 41 | `zwiper-web/src/lib/inbound/screens/deck/cards/add.rs:693` | §4 | `Box::new(card.clone())` only because `oracle_id` is read after it. | Read `oracle_id` first, then move `card`. |
| 42 | `zwiper-web/src/lib/inbound/session/cross_tab.rs:50-52` | §9 no `str::replace` templating | `LOCK_JS.replace("__NAME__", ..).replace("__LIMIT__", ..)`. | `format!` over a raw string. |
| 43 | `zwiper-web/src/lib/inbound/cards/swipe_confirm.rs:65-67`, `:80-82` | duplication | `quietly` and `say`'s chip arm repeat the same bump-and-set. | A `set_chip` method. |
| 44 | `zwiper-web/src/lib/inbound/screens/deck/view/ghost.rs:20` | duplication | A private copy of `chrome::ghost::GhostChipRow` with a `trailing` slot. | Give the shared one an optional `trailing`. |
| 45 | `zwiper-web/src/lib/inbound/cards/printing_sheet.rs:29`; `zwiper-web/src/lib/inbound/screens/oracle_tag_dictionary.rs:42` | duplication | `KIT_MODAL_PX = 960` is defined twice. | Define it once, in `chrome::media`. |
| 46 | `zwiper-web/src/lib/inbound/chrome/details_eye.rs:62`, `:72`, `:73`, `:83` | §9 magic numbers | The CSS mirrors (2rem, 38rem, 0.68, 42rem, 3.25rem, 63/88, 26rem, 24rem) are inline. | Named consts that say which `web.css` rule each mirrors. |
| 47 | `zwiper-web/src/lib/inbound/screens/home/mod.rs:43`, `:220`; `zwiper-web/src/lib/inbound/app.rs:13`; `zwiper-web/src/lib/inbound/screens/deck/create.rs:12`; `zwiper-web/src/lib/inbound/screens/deck/edit.rs:19` | §7 free functions through their module | `sync_settings`, `splash_plays_this_load` and `has_unsaved_changes` are called bare. | `settings_sync::sync_settings` and the like. |
| 48 | `zwiper-web/src/lib/inbound/cards/swipe_stack.rs:526` | §13 | `the_gap_is_half_the_exit` asserts `PRESS_GAP_MS == 160`. | Assert the relation (`PRESS_GAP_MS * 2 == EXIT_MS`) or drop it. |
| 49 | `components/src/card_filter/rows.rs:439` | `arithmetic_side_effects`, hidden from clippy in rsx; §9 | `{i * 40}ms, {250 + i * 40}ms` does unchecked arithmetic inside the format string, with bare numbers. | Named stagger and fade consts, saturating, in a `let`. |
| 50 | `components/src/card_filter/rows.rs:109`, `:257` | §4 | Clones the whole `CardQueryBuilder` to read one field. | Read under a scoped guard. |
| 51 | `components/src/card_details.rs:259-284`, `:394-418` | duplication | The Flip and Image button blocks are copied between `CardDetails` and `CardDetailsPanel`. | Private `FlipButton` and `ImageButton` components. |
| 52 | `components/src/card_details.rs:249`, `:440` | §4 | `TagRows { card: card.clone() }` though `card` isn't used after. | Move it. |
| 53 | `components/src/load_failed.rs:7`; `components/src/hint.rs:156` | §5 private by default | `RETRY` and `SEE_GUIDE` are `pub`, but nothing outside components uses them. | `pub(crate)`. |
| 54 | `components/src/site_footer.rs:18` | §9 magic numbers | `"© 2026 Zwipe"` is hardcoded in a shared component. | A named `COPYRIGHT_YEAR`. |
| 55 | `components/src/card_filter/mod.rs:335` | §9 magic numbers | `setTimeout(.., 50)` in a `format!` JS string. | Name the delay. |
| 56 | `components/src/deck_page.rs:60`, `:333`, `:403` | §5 dead public API | No host builds `DeckFilter` or passes `DeckControls { filter }` any more. | Remove `DeckFilter`, `FilterRow` and the prop. |
| 57 | `zite/src/pages/shared_deck.rs:420` | §9 typed let over turbofish | Clones every mainboard card on every render inside `collect::<Vec<_>>()`. | A typed `let`, memoized if it matters. |
| 58 | `zwipe-ui/src/text.rs:8` | duplicate API | A byte-for-byte copy of `zwipe_core::text::sentence_case`, and zwiper imports both. | Delete it and use core's. |
| 59 | `zwipe-ui/src/hint/one_time.rs:566`, `:574` | §1 invalid states | "Swipe up" shows by matching `Picking("background" \| "signature spell")` on `Slot::noun` strings, so renaming a noun changes the hint. `:574` extends a one-item array. | Carry the `Slot`. Use `push` at `:574`. |
| 60 | `zwipe-ui/src/screens/remove_cards/mod.rs:81`, `:461` | §9 redundant logic, stringly typed | The `SWIPE_LINES` retain matches toast text, and `keep_one` already drops a confirm in favor of `NOTHING_LEFT`. | Drop `SWIPE_LINES` and the retain. |
| 61 | `zwipe-ui/src/screens/remove_cards/mod.rs:752`, `:756` | §4 | The held-undo branch clones the card twice because it matches on `&action`. | Match by value. This folds into the fix for #1. |
| 62 | `zwipe-ui/src/screens/home/mod.rs:588`, `:677` | §5 API shape | `retry(&mut self, action: &Action)` rematches the whole enum, with an unreachable `_ => None`. | A private `Panel` enum. |
| 63 | `zwiper/src/lib/inbound/components/navigation/leave_guard.rs:271` | review: don't throw away the cause | A held route that fails to parse silently goes Home and logs nothing. | Log the path and the error. |
| 64 | `zwiper/src/lib/inbound/screens/deck/card/filter/card_filter_sheet.rs:261-573` | §9 repetition | The same 9-line section clear button is written out 15 times. | A `SectionClear` component. |
| 65 | `cairn:src/lib/outbound/sqlite/mod.rs` (restore), `cairn:src/lib/outbound/paths.rs` (`backup_file`) | §9 named constants | The restore and the backup's file name write `"%Y-%m-%d"` and `"%Y-%m-%dT%H:%M:%S"` as literals beside `backup.rs`'s private `DAY_FORMAT` and `AT_FORMAT`. | Make the two consts public and use them. The store's older copies predate the window and are left for a pass of their own. |
| 66 | `cairn:src/lib/outbound/sqlite/mod.rs` (restore) | §7 types imported | `std::collections::BTreeSet` is written in full. | Import it. |
| 67 | `cairn:src/lib/domain/counter/backup.rs` (`validate`, `BackupError::Shape`); `cairn:src/lib/inbound/ui/screens/config.rs` (`write_backup`, `read_backup`) | §6 errors | Errors are carried as `String`, and `Shape` drops serde's error as its source. In an application that's a trade-off: `BackupError` derives `Clone` and `Eq` for its tests, which `serde_json::Error` can't. | Considered, not changed. |

## What's already good

- No panics outside tests in about 13,000 added lines. A poisoned lock is never recovered with `into_inner`: the reuse map, the page-kill streak and the edge-swipe channel all degrade to a safe default and log.
- The iOS page-kill FFI is careful. Every `unsafe` block is one operation with an accurate `// SAFETY:`, the main thread is checked before UIKit is touched, and the deliberate leak of the tap target is explained.
- `RefreshReuse` redacts credentials in `Debug`, bounds its chain walk, and has tests for the window edges, chains and poison.
- The new `Http*` and settings types derive the std traits, and `SwipeConfirmations` has `FromStr`, `Display` and `as_str` round-tripped in tests.
- The screen models (`deck_form`, `pending_settings`, `remove_cards`) keep the logic testable outside the UI, and most of the new behavior lands with tests.

## Where the fixes landed

| PR | Scope | Findings |
|---|---|---|
| zwipe #294 | zwipe-core, zwipe-client, zerver | #5, #11, #13, #20, #31 to #35 |
| zwipe #297 | zwipe-ui, zwiper | #1, #7, #8, #17, #18, #21, #22, #58 to #64 |
| zwipe #299 | zwiper-web | #19, #23 to #26, #29, #40 to #48 |
| zwipe #300 | components, zite | #6, #14 to #16, #27, #28, #30, #49 to #57 |
| orangui #17 | lifecycle, os_back | #2, #9, #37, #38 |
| zwipe #302 | repin orangui after #17 | `flush_loop.rs` reads `use_app_lifecycle`'s `Option` |
| Cairn `main` `7ef3481`, `dd05a70` | Cairn | #4, #10, #39, #65, #66 |

Not fixed here: #3, #12 and #36 (`page_kill.rs`) went to orangui #16, which rewrote the file. #67 is a trade-off, left as it is.

## Fix plan (step 2, after the composer's go)

No behavior change except where a finding is itself a bug (rule 2): the panic becoming an `Option` in #2, and the behavior fixes in #1 and #3 to #11, each called out in its PR. One PR per repo or crate group:

1. zwipe-core, zwipe-client and zerver: #5, #11, #13, #20, #31 to #35. #20 and #32 change public signatures (callers update in the same PR).
2. zwipe-ui and zwiper: #1, #7, #8, #17, #18, #21, #22, #58 to #64.
3. zwiper-web: #19, #23 to #26, #29, #40 to #48.
4. components and zite: #6, #14 to #16, #27, #28, #30, #49 to #57.
5. orangui (PR into orangui `main`): #2, #3, #9, #12, #36 to #38. Then repin zwipe, zite and Cairn.
6. Cairn (PR into Cairn `main`, after the backup work lands): #4, #10, #39.
