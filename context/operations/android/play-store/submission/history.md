# Android release history

Per-release build log. Build recipe is in [build.md](build.md).

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
