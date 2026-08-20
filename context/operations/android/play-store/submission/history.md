# Android release history

Per-release build log. Build recipe is in [build.md](build.md).

- **2026-08-18 — `1.9.3`, versionCode `40`** (the correctness train, built
  ahead of 1.9.2 clearing review so it could go the moment that slot freed.
  **Submitted to Play 2026-08-18, RELEASED 2026-08-20** (iOS same day, so the
  stores realigned after briefly being out of step during 1.9.2's review). Server half: **MVP steering** (deck-MVPs phase
  3) gives cards whose `card_roles` overlap the deck's mainboard MVPs a flat
  `W_STEER = 0.12` lift on the synergy serve, riding the existing
  `DeckServeContext` seam beside `deck_oracle_tags` so it is dormant for decks
  with no MVPs and reverts exactly at 0.0; flat rather than the plan's overlap
  count because three MVPs frequently share one role, which a tally would lift
  above commander synergy. **Migration `20260818120000`** adds
  `deck_cards_mvp_mainboard_only`, closing a hole where the import upsert set
  `board` on conflict without clearing `mvp_at`, stranding a star on a card off
  the mainboard where the podium cap could not see it (prod had no violations).
  Client half: sign-in no longer applies registration policy to a typed
  password or username, which would have locked out existing accounts on any
  future tightening; changing an email now updates the session's verification
  state and prompts to verify; `BACKEND_URL` is validated at build time rather
  than panicking on first launch; and an in-app copy sweep corrected help that
  promised behavior the code lacked, most notably the MVP hint claiming
  suggestion steering that had never been built. Site/docs, deploy-only: all 20
  guides corrected against the app, a false password-blocklist claim removed
  from /about, sitemap drift now fails the build, and oracle-tag descriptions
  finished at 4,521 of 4,522. Built on the work Mac, then rebuilt the same day
  against h2 0.4.16 (RUSTSEC-2026-0258, reached via reqwest) keeping versionCode
  40 since nothing had been uploaded; `patch_bundle.sh` applied and all four
  step-4a greps verified against the signed AAB both times. iOS counterpart
  build 77.)

- **2026-08-17 — `1.9.2`, versionCode `39`** (**released 2026-08-18**; the reliability + deck-list
  train. The `ndk-context` crash was root-caused and fixed after surviving five
  releases: `MainActivity` carried no `launchMode`, so an explicit component
  start — a notification, another app, the Play Store's Open button — created a
  second Activity in the live process and NativeActivity's native init ran
  twice, tripping `assert!(previous.is_none())`. `launchMode="singleTask"`
  fixes it. A second bug surfaced the same session: `configChanges` omitted
  `uiMode`, so a system dark/light switch recreated the Activity, reached
  `onDestroy`, and the 1.7.6 process-kill silently closed the app. Both are
  manifest patches, now applied by `zcripts/android/patch_bundle.sh` alongside
  the icons and back handler — one command, because the manifest gap survived
  as an unrun checklist item. Back-swipe overlays fixed as a class: the shared
  `BottomSheet` now registers with `OverlayBackStack` on every sheet's behalf,
  plus format_select, tag_select and printing_sheet. Deck list restyled into
  one contained list of card-style rows led by command-zone art, with
  collapsible groups; `DeckProfile` gained additive art URLs. Card color
  grouping splits per color combination with mana-pip headers. Verified on a
  Pixel 6 before cutting: the crash repro is silent and back unwinds one
  overlay at a time. Built per the recipe on the work Mac and **submitted for review
  2026-08-17**. iOS counterpart build 76.)

- **2026-08-14 — `1.9.1`, versionCode `37`** (the keyword-quality train:
  reminder-definition sweep for 2026-set/crossover/Arena mechanics — 26 real
  definitions grounded in the cards' own reminder text, Blight/Prepared/
  Vivid/Start your engines! among them; reminders now served from
  `/api/card/keyword-reminders` with the compiled table as offline fallback,
  so future sweeps land on deploy; maybeboard otag chips reveal descriptions;
  spacing riders. Replaces vc36's in-review release — users update 1.8.1 →
  1.9.1, so the release notes combine both trains. Built per the recipe on
  the work Mac. iOS counterpart build 75.)

- **2026-08-13 — `1.9.0`, versionCode `36`** (commander maybeboard: up-swipe
  during any commander Zwipe-select saves to a per-user list; the new screen
  under the Decks More sheet has card rows with art + Art toggle, in-row
  Printing/Create deck/Remove, quick add with floating chips, Show pips + the
  shared filter sheet, its own Swipe overlay (right-swipe seeds a new deck,
  saves excluded from the pile), and a clear-all dialog. Create-deck commander
  seeding. Server riders: commander_maybeboard migration + 4 endpoints
  (add/remove/list/clear, cap 50). Fixes the collapsible-grid width blowout
  (expanded rows in grouped lists clipped at the viewport — shipped in 1.8.1)
  and unifies the 0.5rem element rhythm app-wide. 1.8.1 (vc35) went live
  before this submission, so 1.9.0 is a normal update on top of it. Built per
  the recipe on the work Mac. iOS counterpart build 74.)

- **2026-08-12 — `1.8.1`, versionCode `35`** (the Discord-todo polish train:
  collapsible deck-cards groups with the eased card-row arrow, skeleton
  principle rework (static chrome renders real, in-place ghosts), featured
  cards dealing in from above, all five color pips always on the deck list,
  header underline removed; server rider: the latest_cards English-preference
  migration. Replaces vc34's still-in-review release — users update 1.7.6 →
  1.8.1. Built per the recipe on the work Mac, warm caches. Manifest verified
  targetSdk 36 / vc35 / 1.8.1. iOS counterpart build 73.)

- **2026-08-12 — `1.8.0`, versionCode `34`** (deck list Group by + Show chip rows
  (sections by format/color/tag, color-pip and tag filters), the deck-list
  one-time tip, pinned import/export consoles with edge-fade scrolling, bolder
  chip-row labels. First train under the any-feature-bumps-minor convention,
  `development/versioning.md`. Built per the recipe on the work Mac; the
  aarch64-linux-android target cache was wiped first (stale `~/Work` asset paths
  baked in from the pre-move vc33 build). Manifest verified targetSdk 36 / vc34 /
  1.8.0. iOS counterpart build 72.)

- **2026-08-10 — `1.7.6`, versionCode `33`** (global undo across every deck-mutation
  door with import resetting history, the Android resume-crash fix — `onDestroy`
  process kill so reopening after OS-killed background is a clean cold start, the
  featured flavor home element (hourly shared card), clear (×) buttons on all search
  inputs, quick add searching past skips via the new `include_skipped` flag, and the
  keyring 3→4 migration — Android unaffected at runtime (sessions are a private-storage
  file) but first Android compile of the keyring-4 graph. Built per the recipe on the
  **work Mac's first Android build** (fresh toolchain 2026-08-10: NDK 28.2.13676358,
  platforms 36/36.1, build-tools 36.0.0, Gradle 9.1 via JBR 21): `dx bundle` →
  `launcher_icons.sh` → `back_handler.sh` → gradle patch (compileSdk 36 / targetSdk 36
  / versionCode 33) → `gradlew :app:bundleRelease` → jarsigner (JBR's — the
  `/usr/bin` stub wants a system JDK). Manifest verified targetSdk 36 / vc33 / 1.7.6.
  iOS counterpart build 71.)

- **2026-08-04 — `1.7.5`, versionCode `32`** (the 1.7.5 feature train: deck-cards undo
  (adds, removals, coalesced qty bursts, board moves, printing swaps; per-deck stacks
  survive navigation), quick-add search bar, deck identity header, floating
  type-to-search results, featured card images + row art with the Art chip, empty
  Maybe/Side board gray-out, swipe feel pass (exit-from-release animation, raised
  thresholds, return bounce), quantity tap debounce, and the PATCH migration client
  half — PATCH verb on all seven update endpoints, absolute-quantity deck-card body,
  clean Opdate wire form; server dual-accept halves were deployed + verified on prod
  before this build). Built per the recipe: `dx bundle` → `launcher_icons.sh` →
  `back_handler.sh` → gradle patch (compileSdk 36 / targetSdk 36 / versionCode 32,
  versionName 1.7.5) → `gradlew :app:bundleRelease` → jarsigner. Artifact
  `zwipe-1.7.5.aab`, signed + `jar verified`, targetSdk 36 + versionCode 32 confirmed
  via bundletool. Emulator smoke test skipped (owner tests the release on-device;
  runbook step demoted to optional this release). iOS counterpart build 70.
- **2026-07-30 — `1.7.4`, versionCode `31`** (client error + crash reporting — anonymous,
  first-party: handled errors ride the usage batch with screen/component/action
  breadcrumbs and client-side dedupe, crashes exactly-once via panic-hook disk file →
  unauthed `/api/metrics/crash`; field-verified on a real device before submission.
  Server half deployed 2026-07-29 with the `UpkeepService` 90-day retention prunes.
  The iOS-only photo-save crash fix is NOT claimed in the Play notes). Built per the
  recipe: `dx bundle` → `launcher_icons.sh` → `back_handler.sh` → gradle patch
  (compileSdk 36 / targetSdk 36 / versionCode 31, versionName 1.7.4) →
  `gradlew :app:bundleRelease` → jarsigner. Artifact `zwipe-1.7.4.aab`, signed +
  `jar verified`, targetSdk 36 + versionCode 31 confirmed via bundletool. Data safety
  form updated: Crash logs + Diagnostics + Other app performance data, and App
  interactions added under App activity (previously undeclared). iOS counterpart
  build 69.
- **2026-07-24 — `1.7.3`, versionCode `30`** (filter sheet current/staged split with Apply
  as the sole commit and Reset/Cancel staging + restoring with toasts, plus the maybeboard
  empty-Apply fix; average power/toughness in the deck's Distributions; shared ranked
  otag search in core, exact > slug/label > description, across the deck selector, card
  filter, and dictionary; tag definitions tap-to-reveal in the swipe-screen card details
  dialog; guide-card tags recolored per tag on zite). Built per the recipe: `dx bundle` →
  `launcher_icons.sh` → `back_handler.sh` → gradle patch (compileSdk 36 / **targetSdk 36**
  / versionCode 30, versionName 1.7.3) → `gradlew :app:bundleRelease` → jarsigner.
  Artifact `zwipe-1.7.3.aab`, signed + `jar verified`, targetSdk 36 + versionCode 30
  confirmed via bundletool. **First targetSdk 36 build — clears the Play "update your
  target API level" deadline (2026-08-31).** iOS counterpart build 68 (Xcode 26.6).
  Server side (sqlx 0.9, share-page ordering fix) deployed 2026-07-23 ahead of the
  clients.

- **2026-07-20 — `1.7.2`, versionCode `29`** (deck-view polish: lands moved to their own
  pinned bottom section with the Show-Lands toggle removed and hiding via the card filter;
  the card filter now covers the Maybeboard and Sideboard, not just the main deck; the
  Category grouping renamed Card role; tag-row stacking + flavor-pill fixes; tap-outside-
  to-dismiss on dialogs; a filter Cancel button; the Avatar bending keyword reminders
  rewritten to the real mechanics; and an in-app guide overhaul with a new oracle-tag
  examples guide). Built per the recipe: `dx bundle` → `launcher_icons.sh` →
  `back_handler.sh` → gradle patch (compileSdk 36 / targetSdk 35 / versionCode 29,
  versionName 1.7.2) → `gradlew :app:bundleRelease` → jarsigner. Artifact `zwipe-1.7.2.aab`,
  signed + `jar verified`, targetSdk 35 + versionCode 29 confirmed. iOS counterpart build
  67. Server: shared-deck endpoint now returns tokens (additive `HttpSharedDeck.tokens`, no
  migration; deserialized only by zite, safe for old clients).

- **2026-07-17 — `1.7.1`, versionCode `28`** (oracle-tag definitions revealed inline in
  the expanded card row, with Examples/Use straight from the dictionary and a dictionary
  link in the card filter; image-less cards rendered as a text card instead of hidden;
  card-details restyle that opens at the top; back-aware overlays so the OS back closes
  the top overlay first; hidden scrollbars; zite shared-deck pulls tag descriptions too).
  Built per the recipe: `dx bundle` → `launcher_icons.sh` → `back_handler.sh` → gradle
  patch (compileSdk 36 / targetSdk 35 / versionCode 28, versionName 1.7.1) → `gradlew
  :app:bundleRelease` → jarsigner. Artifact `zwipe-1.7.1.aab`, signed + `jar verified`,
  targetSdk 35 + versionCode 28 confirmed. iOS counterpart build 66. Client-only release
  (no server/migration changes).

- **2026-07-14 — `1.7.0`, versionCode `27`** (oracle-tag dictionary, unified catalog
  cache, 1,100 authored tag descriptions, Phase M sunset `mechanical_categories` →
  `card_roles` + Phase 5S `deck_id`-driven signal, per-deck cap raised to 500 across all
  boards). Built per the recipe: `dx bundle` → `launcher_icons.sh` → `back_handler.sh` →
  gradle patch (compileSdk 36 / targetSdk 35 / versionCode 27, versionName 1.7.0) →
  `gradlew :app:bundleRelease` → jarsigner. Artifact `zwipe-1.7.0-vc27.aab`, signed +
  `jar verified`, targetSdk 35 + versionCode 27 confirmed. Submitted for review
  2026-07-14. iOS counterpart build 65. Server 1.7.0 pushed first (card_roles column
  migration + deck_id signal + deck-cap change).

- **2026-07-12 — `1.6.0`, versionCode `26`** (folds the Lands row Budget->Mana
  deck-view move into 1.6.0; no other client changes). Built per the recipe:
  `dx bundle` → `launcher_icons.sh` → `back_handler.sh` → gradle patch (compileSdk 36 /
  targetSdk 35 / versionCode 26) → `gradlew :app:bundleRelease` → jarsigner. Artifact
  `zwipe-1.6.0-vc26.aab`, signed + `jar verified`, targetSdk 35 confirmed. Smoke test
  skipped (UI-only row move). iOS counterpart build 64. Supersedes vc25 (submitted to
  Production review earlier the same day).

- **2026-07-12 — `1.6.0`, versionCode `25`** (card roles + oracle tags, deck-view reorg
  into Profile/Budget/Tags, shared CardDetails + flippable card images across app and
  site, 31 themes (17 new) + theme persistence, in-app changelog). Built per the recipe:
  `dx bundle` → `launcher_icons.sh` → `back_handler.sh` → gradle patch (compileSdk 36 /
  targetSdk 35 / versionCode 25) → `gradlew :app:bundleRelease` → jarsigner. Artifact
  `zwipe-1.6.0-vc25.aab`, signed + `jar verified`, targetSdk 35 confirmed. Release
  smoke-test on Pixel_9a: launched clean (no libmain/R8 crash, layout clear of the
  system bars). iOS counterpart build 63. **Submitted to Production for review
  2026-07-12.**

- **2026-07-11 — Production launch submitted for review** (Play Console Submission 21,
  "Production" track, **all countries**). Promoted the `1.5.0` / vc24 build from closed
  testing to Production. Gotcha: the Production track starts with **no countries** — set
  its own list via Test and release → Production → Countries/regions (separate from the
  176 on closed testing; not on the release page, not in the bundle). Now awaiting Google
  review, then live on Play.

- **2026-07-09 — `1.5.0`, versionCode `24`** (edge back-swipe navigation, per-screen
  per-deck filter persistence, session-platform tracking on the refresh-token row,
  Android tap-highlight fix, and the CardRow/skeleton polish batch). Built per the
  recipe — `dx bundle` → `launcher_icons.sh` → `back_handler.sh` (the new
  back-navigation patch, step 1c) → gradle patch (compileSdk 36 / targetSdk 35 /
  versionCode 24) → `gradlew :app:bundleRelease` → jarsigner (0600 scratchpad
  password, deleted after). Artifact `zwipe-1.5.0-vc24.aab`, signed + `jar
  verified`. **vc23 was burned** — Play rejected it ("Version code 23 has already
  been used"), so bumped to 24 (just re-patch versionCode + re-run gradle/jarsigner,
  no full rebuild). Server halves (session-platform additive migration) deployed to
  prod first. iOS counterpart: build 62 (submitted to review 2026-07-09).
  *R8/edge-to-edge smoke test still to run — first thing to check if back-swipe
  misbehaves on a tester device (R8 could strip the OnBackPressedCallback path).*

- **2026-07-07 — `1.4.0`, versionCode `22`** (feature batch: commander picks now
  lead with the community's most-built commanders in a fresh daily order
  (Zwipe-select popularity ordering + wildcard deep slice), partners that name
  each other auto-pair, Deck MVPs phase 1 (star up to three cards per deck), and
  share-a-deck public links from the More sheet; commander-select signal ingest
  ships dormant. First minor bump since 1.3.x; workspace version bumped 1.3.2 →
  1.4.0). Built per this recipe — `dx bundle` → `launcher_icons.sh` → gradle
  patch (compileSdk 36 / targetSdk 35 / versionCode 22) → `gradlew
  :app:bundleRelease` → jarsigner (0600 scratchpad password, deleted after).
  Artifact `zwipe-1.4.0-vc22.aab`, signed + `jar verified`. **R8/edge-to-edge
  smoke test run this round** (Pixel_9a): app launches clean (no
  `libmain.so`/R8 crash, no FATAL), login + bottom action bar clear of the
  status/nav bars. iOS counterpart: build 61. Server halves (commander
  popularity endpoint already live 2026-07-07; commander-select-signal + Deck
  MVPs additive migrations) deploy to prod before rollout.

- **2026-07-05 — `1.3.1`, versionCode `21`** (pre-auth funnel telemetry: the
  client posts anonymous session events — app_opened, register_viewed,
  register_submitted — to the new `/api/metrics/anonymous` endpoint; plus the
  server-side AppState type-erasure refactor, no behavior change). Built per
  this recipe — `dx bundle` → `launcher_icons.sh` → gradle patch (compileSdk 36 /
  targetSdk 35 / versionCode 21) → `gradlew :app:bundleRelease` → jarsigner
  (0600 scratchpad password, deleted after). Artifact `zwipe-1.3.1-vc21.aab`,
  signed + `jar verified`. iOS counterpart: build 60. Server (anonymous_events +
  daily-activity BIGINT migrations) must deploy to prod before rollout.
  *R8/edge-to-edge emulator smoke test skipped again.*

- **2026-07-03 — `1.3.0`, versionCode `20`** (filter-intent + Reset batch:
  sort/synergy-only searches now serve, `Reset` returns each screen to its
  default view, the filter dot tracks any real filter or sort, and the filter
  sheet collapses its sections on close. Supersedes vc19). Built per this recipe
  (gradle patch versionCode 20). Artifact `zwipe-1.3.0-vc20.aab`, signed +
  `jar verified`, uploaded to the Alpha track. iOS counterpart: build 59. No
  server change. *R8/edge-to-edge emulator smoke test skipped again.*

- **2026-07-02 — `1.3.0`, versionCode `19`** (per-swipe durable skips via the
  new skip/unskip endpoints; per-deck add-stack memory with MRU parking;
  CardStack refactor across the three swipe screens; image/skeleton ease-ins +
  swipe-layout spacing; stack cap 1000 → 500; profile About section with the
  website link. Supersedes 1.2.3/vc17, withdrawn from review — release notes
  folded into 1.3.0). Built per this recipe — `dx bundle` → `launcher_icons.sh`
  → gradle patch (compileSdk 36 / targetSdk 35 / versionCode 19) →
  `gradlew :app:bundleRelease` → jarsigner (0600 scratchpad password, deleted
  after). Artifact `zwipe-1.3.0-vc19.aab`, signed + `jar verified`, uploaded to
  the Alpha closed-testing track. **vc18 was built and submitted first, then
  superseded by vc19** (added the About section) before review completed. iOS
  counterpart: build 58. Server (skip endpoints, no migration) deployed to prod
  first. *R8/edge-to-edge emulator smoke test skipped again — first suspect if a
  tester device misbehaves.*

- **2026-06-23 — first Android build (`1.0.9`).** targetSdk 35 (compiled against
  API 36.1), signed with the new `zwipe-upload` key, R8 + edge-to-edge smoke-tested
  clean on Pixel_9a. **versionCode `1` burned** by an initial targetSdk-34 upload
  (rejected for the API-35 rule but still consumed the code); `2` uploaded then
  superseded by **`3`**, which shipped — Closed testing (Alpha), 176 countries,
  with the harmless native-debug-symbols warning. *Lessons: a rejected/superseded
  upload still burns its versionCode (always bump); the debug-symbols warning is
  unavoidable with dx's prebuilt Rust lib (see the native-debug-symbols note in
  [publish.md](publish.md)).*

- **2026-06-23 — `1.0.10`** (update-screen redesign + external-link arrows;
  first coordinated release run alongside iOS build 44). targetSdk 35, signed with
  `zwipe-upload`. **versionCode `4` burned** by an upload attempt, **`5` shipped**
  to the Alpha track. *Lesson: keep every closed-test release on the **same Alpha
  track** so the 12-tester / 14-day clock accumulates — don't create a new track
  per version (Play won't let you delete the stray ones, only rename them).*

- **2026-06-23 — `1.0.10` refresh, versionCode `6`** (commander-search "Searching…"
  indicator — the debounce-feedback fix). Same versionName `1.0.10` (no app-version
  bump); only the versionCode increments. Artifact `zwipe-1.0.10-build6.aab`,
  **submitted to the Alpha track 2026-06-23**. iOS counterpart: build 45
  (submitted to Apple review the same day).

- **2026-06-25 — `1.1.0`, versionCode `8`** (Zwipe-select, deck tags, keyword
  hinter, expanded card detail — first minor bump). Two Android-only fixes rode
  here: **session persistence** (keyring has no Android backend → was using its
  in-memory mock, so sessions died on restart; now a JSON file in internal storage
  via JNI — see `zwiper/src/lib/outbound/session.rs`) and the **real launcher icon**
  (step 1b — dx ships its default droid). versionCode **`7` was built + smoke-tested
  but never uploaded**, then a one-line metrics fix (record the SwipeSelect select
  swipe) bumped it to **`8`, submitted to the Alpha track**. Artifact
  `zwipe-1.1.0.aab`. iOS counterpart: build 48. *Lesson: an unuploaded versionCode
  can be reused — 7 was never sent to Play, so 8 is the next real number after 6.*

- **2026-06-26 — `1.1.1`, versionCode `9`** (in-app help button, import/export hints,
  the `mailto` OS-open fix). Artifact `zwipe-1.1.1.aab`, signed + R8/edge-to-edge
  smoke-tested clean on Pixel_9a, rolled out to the Alpha track. iOS counterpart:
  build 49. **Launcher-icon lesson:** the full-bleed Z (`icon-1024.png`) was getting
  its edges sliced by the adaptive-icon **circular mask** — adaptive icons are a
  108dp canvas but only the inner ~66dp is the guaranteed-visible safe zone, and a
  wide logo like the Z has bars at the very top/bottom of its bbox that land outside
  the circle. Fix: a separate **padded** source `icon-1024-android.png` (Z ≈ 47% of
  the canvas, centered, generous `#282828` padding) wired into `launcher_icons.sh`;
  iOS/web keep the full-bleed `icon-1024.png` (square icons aren't masked). Verify a
  candidate by simulating the mask: crop the foreground to the center 66.6% and
  circle-mask it before rebuilding. *Logo design polish still deferred (see `todo.md`).*

- **2026-06-28 — `1.1.2`, versionCode `10`** (filter-control consistency pass on the
  card-swipe screens). iOS counterpart: build 50.

- **2026-06-28 — `1.1.3`, versionCode `11`** (media-day release: card names while
  swiping, deck-form overhaul, expanded tags + format/power pickers, in-app privacy
  policy, under-field validation). Built per this recipe — `dx bundle` →
  `launcher_icons.sh` → gradle patch (compileSdk 36 / targetSdk 35 / versionCode 11)
  → `gradlew :app:bundleRelease` → jarsigner (password via a 0600 scratchpad file,
  deleted after). Artifact `zwipe-1.1.3.aab`, signed + `jar verified`, uploaded to the
  Alpha closed-testing track. iOS counterpart: build 51. *R8/edge-to-edge emulator
  smoke test was skipped this round (same build path as prior releases) — first
  suspect if a tester device misbehaves.*

- **2026-06-30 — `1.1.4`, versionCode `14`** (bottom-sheet flash fix + clone-nav
  fix, over an initial 52/vc12 rebuild). Artifact `zwipe-1.1.4-vc14.aab`. iOS
  counterpart: build 53.

- **2026-07-02 — `1.2.3`, versionCode `17`** (swipe memory: per-deck skip/removal
  suppressions with server-side filtering + Clear skips in the deck More sheet;
  alphabetical deck lists; profile System/version row; email-verification row
  rework; updated privacy policy. 1.2.2 skipped, versionCode 16 shipped as 1.2.1).
  Built per this recipe — `dx bundle` → `launcher_icons.sh` → gradle patch
  (compileSdk 36 / targetSdk 35 / versionCode 17) → `gradlew :app:bundleRelease` →
  jarsigner (0600 scratchpad password, deleted after). Artifact `zwipe-1.2.3.aab`,
  signed + `jar verified`, uploaded to the Alpha closed-testing track. iOS
  counterpart: build 56. Server (swipe-memory migration) deployed to prod first.
  *R8/edge-to-edge emulator smoke test skipped again — first suspect if a tester
  device misbehaves.*

- **2026-07-01 — `1.2.1`, versionCode `16`** (card rules dialog + launch-flash
  fix). Built per this recipe, published to the Alpha closed-testing track.
  iOS counterpart: build 55 (uploaded but held behind 1.2.0, ultimately
  superseded by build 56 — 1.2.1 never went to iOS review).

- **2026-06-30 — `1.2.0`, versionCode `15`** (first minor bump since 1.1.0:
  hypergeometric draw-odds, Synergy on/off toggle, power level + other-tags,
  deck tags 85→117, include/exclude filter guard, PDH commander fix, `edhrec_rank`
  index, proliferate→Counters, create/edit top-scroll fix). Built per this recipe —
  `dx bundle` → `launcher_icons.sh` → gradle patch (compileSdk 36 / targetSdk 35 /
  versionCode 15) → `gradlew :app:bundleRelease` → jarsigner (0600 scratchpad
  password, deleted after). Artifact `zwipe-1.2.0.aab`, signed + `jar verified`,
  uploaded to the Alpha closed-testing track. iOS counterpart: build 54. Server
  batch (additive migrations) deployed to prod first. *R8/edge-to-edge emulator
  smoke test skipped again — first suspect if a tester device misbehaves.*
