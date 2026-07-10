# Todo

**Primary goal: grow the user base — production launch + marketing + tester-feedback intake.** (iOS App Store: LIVE. Android: closed testing LIVE ~400 testers, production access next. Full webapp at zwipe.net: in progress.)

Only open, actionable items live here. When something ships, its outcome moves to
[`overview.md`](overview.md) and leaves this list. Older completed work is archived
at `context/archive/complete_2026_q1.md`.

---

## Next Up

- [ ] **1.4.0 LIVE on the App Store (2026-07-08); track the Android side** — the
  feature batch: Zwipe-select popularity ordering (client leg), commander-select
  signal ingest, partner autofill, Deck MVPs phase 1, deck share links. Server
  halves (three additive migrations) deployed to prod first, verified against
  live clients. Store copy staged in both `form_fields.md` files; What's New:
  commander picks lead with the community's most-built commanders, fresh order
  daily; partners that name each other pair automatically; star your deck's
  MVPs; share any deck with a link. iOS cleared review and is distributed;
  confirm the vc22 closed-testing rollout on Play.

- [ ] **Next store build carries the 2026-07-08 zwiper polish batch** (all on
  main; ships as **1.4.1** — or **1.5.0** if new functionality lands first,
  per the owner's versioning rule 2026-07-08): land-target auto-filter leak fix, skeleton
  realignment (deck list tag tiles, full edit-deck form, collapsed stat
  sections, spinner-free search), expanded card-row emphasis + DFC front-face
  mana cost (via the shared `CardRow`), `card-action-*` class rename, shared
  site constants (About/store links now debug-gated via
  `zwipe_core::domain::site`).
- [ ] **Track 1.3.1 review (anonymous funnel metrics)** — iOS **build 60** + Android **vc21** submitted to both stores 2026-07-05, in review. Server deployed the same day (two additive migrations; first deploy attempt failed on a stale crate-local `zerver/.sqlx/` shadowing the workspace offline data — removed, and the workflow now verifies `.sqlx` against the migrated schema before building; see `operations/infrastructure/cicd.md`). Once clients roll out and data accrues, read the funnel with `zcripts` (distinct sessions per kind vs. `user_events.register`) — these numbers gate the sign-in-with-Google decision.
- [ ] **Track 1.3.0 review on both stores** — iOS **build 59** + Android **versionCode 20** submitted 2026-07-03 (server skip/unskip endpoints deployed to prod first). 1.3.0 supersedes the withdrawn 1.2.3 and folds in: swipe memory (FR #11) now **per-swipe durable**, **per-deck stack memory**, the CardStack refactor, the profile **About section**, image/skeleton polish, and the **filter-intent + Reset** pass (sort/synergy-only searches serve, Reset returns each screen to its default, accurate filter dot, filter sheet collapses on close). Build progression: 57/18 → 58/19 (About) → 59/20 (filter/Reset). Once live, 1.3.0 becomes the floor for `MIN_CLIENT_VERSION` gating; mark FR #11 shipped. Per-change detail in `overview.md`. Marketing video plans (`marketing/plans/`) are refreshed for this build.
- [ ] **Privacy follow-ups for per-user collection.** The policy text shipped 2026-07-02 (`b1ee1b11`, discloses per-account activity + deck skip memory). Remaining owner passes: update the App Store privacy "nutrition label" + Play data-safety form to reflect per-account analytics, and send the policy-change notification email to users.
- [ ] **Suggestion signal — Phase 3c (pair-level ranking).** Phases 3a+3b **shipped 2026-07-06** (server 1.3.2): the default synergy ordering now blends base score + pooled net-rate (`added + 0.5·maybed − removed`, shrunk/centered) + per-(deck, day) seeded jitter — different decks serve differently, the same deck stays stable within a day, and crowd favorites drift up as signal accrues. Remaining: the commander-specific pair-level term, gated on pair-depth (baseline 2026-07-06: 0 pairs ≥20 impressions — re-run the readiness queries after the user base grows). Plan: [`../plans/suggestion_signal.md`](../plans/suggestion_signal.md).
- [x] **Draw-odds — DONE 2026-07-09 (shipped FREE).** The full feature (hypergeometric engine + deck-view section + per-turn `<- / ->`) shipped in 1.2.0; the free-vs-premium question is resolved to **fully free** (owner call 2026-07-09) — no gating.

**The next big three (set 2026-06-11):**
1. ~~**Android — get the clock ticking.**~~ **DONE 2026-07-09** — ~400 active closed testers (hired testing service + organic); the 12-tester / 14-continuous-day requirement is satisfied. Next: apply for production access + intake tester feedback (see Android section).
2. **Marketing — get users.** Business cards for LGSs, Reddit/X posts (see Marketing & Discovery below).
3. **VPS — stabilize.** Cutover is done (see `overview.md`); remaining follow-ups are the two items below.

- [x] **Launcher/app logo — DONE 2026-07-09.** The icon reads well on both stores (owner call); no further polish needed. Safe-zone mechanism + history in `operations/android/play-store/submission/history.md` (1.1.1 entry).
- [ ] **Verify the VPS crons fired.** First unattended run was 2026-06-14 (zervice 4am + backup 5am UTC). Check: `ssh root@100.114.251.8` (or `zerver` alias), then `tail /var/log/zwipe/zervice-cron.log /var/log/zwipe/backup.log` and `rclone lsl r2:zwipe-backups/ | tail -3` — a recent dated dump should be present. If the zervice log is empty, recheck the `SHELL=/bin/bash` line (the dash-vs-bash trap from `operations/infrastructure/server.md`).
- [ ] **Repurpose home box + rotate R2 keys — after ~1–2 clean VPS weeks.** Home is powered off but intact as the rollback (boot → flip `api.zwipe.net` CNAME to home tunnel `70ba169b-…` → `systemctl start zerver`). Once the VPS has a clean run, rebuild home as a gaming box (a different Linux distro) — that reinstall wipes the old prod secrets on its disk. Then rotate the still-shared R2 keys (JWT/DB/Resend are already fresh on the VPS, so only R2 carries over from home). Closing the rollback window = remove old home tunnel `70ba169b-…` + old Tailscale device.

---

## Android — Near Submission Ready (PRIORITIZED 2026-06-10: ship before premium)

Decision (2026-06-10): grow the user base before monetizing — Android ships
before the premium tier. Rationale: the port is days of polish away (Dioxus),
premium revenue scales with users, MTG pods are mixed-device so adoption
spreads through playgroups, and shipping Android first means premium later
launches on both stores in one pass (StoreKit + Play Billing designed together).

- [x] **Play identity verification — DONE 2026-06-23.** Account created + $25 paid
  2026-06-11 (personal account; ID 5194812603818548859), identity + address +
  phone all cleared.
- [x] **First Android build submitted to Closed testing — 2026-06-23.** `1.0.9`
  versionCode `3`, targetSdk 35, signed with the new `zwipe-upload` key, full
  rollout to the Alpha closed track across 176 countries (in Google's review
  queue). Pipeline + gotchas captured in `operations/android/play-store/submission/build.md`;
  listing copy in `.../form_fields.md`.
- [x] **Closed-testing clock — DONE 2026-07-09.** **~400 active testers** on the
  Alpha closed track (hired testing service + organic), well past the 12-tester /
  14-*continuous*-day requirement for production access. The Google Group `zwipers`
  flow worked (**join group → opt in → install**; a bare opt-in/store link does
  nothing for non-members — the group join is what makes someone eligible). Play
  App Signing enrolled, `com.scadoshi.zwipe` live to testers across 176 countries.
- [ ] **Apply for production access + full launch.** 14-day cycle **confirmed
  complete 2026-07-09** by the QA partner (Teekam Suthar / 12testers); the
  production questionnaire is in hand (Zwipe-tailored answers drafted in
  [`../operations/android/play-store/submission/production_access.md`](../operations/android/play-store/submission/production_access.md)).
  Submit the production-access application in the Play Console (~72h review), then
  promote a build to the Production track.
  (Coordinate with review state: 1.4.0 / vc22 is the live closed-testing build;
  the next build 1.4.1 adds back-swipe + filter persistence + the polish batch.)
- [ ] **Intake tester feedback → `feature_requests.md`.** ~400 testers + hired
  testers are generating suggestions; triage them into the weighted request queue
  ([`feature_requests.md`](feature_requests.md)) and surface anything actionable
  into this list. This is the near-term driver now that the launch gate is cleared.

Android build compiles and runs — **emulator-confirmed on Pixel_9a 2026-06-22** (login → decks → swipe all working against prod). Build gotcha: must point `JAVA_HOME` at Android Studio's bundled JDK 21, not the system-default JDK 26 — see `operations/android/setup.md`. The home-screen ASCII logo (block glyphs) now renders correctly via the self-hosted font (ships in 1.0.9). None of the below blocked the closed-testing submission — they're polish for a future build:

- [x] Swipe gesture card tilt — CONFIRMED WORKING 2026-07-09 (cards rotate during drag on Android, same as iOS; the earlier note was stale).
- [x] Android tap flash — FIXED 2026-07-09: `-webkit-tap-highlight-color: transparent` on the universal selector in `main.css` kills Android WebView's default blue tap highlight (iOS never showed it). Emulator-verified; rides the next build.
- [x] Lock screen orientation to portrait — CONFIRMED WORKING 2026-07-09 (verified on a physical Android device; already portrait-locked, no change needed).
- [x] **Edge back-swipe navigation (Android + iOS) — SHIPPED-READY 2026-07-09** (rides the next store build). The OS back intent now drives the router's `go_back()` (or exits at a root screen) instead of the old broken behavior (Android closed the app; iOS did nothing). iOS: a custom `UIScreenEdgePanGestureRecognizer` (objc2) → tokio channel → `go_back` (instant; the WKWebView-history approach was ~3s and got dropped). Android: a `MainActivity` patch (`zcripts/android/back_handler.sh`, re-applied after `dx bundle` — see play-store/submission/build.md step 1c) registers an `OnBackPressedCallback` catching both gesture + button, dispatches `zwipe:back` → Rust `go_back`/JNI `finish`. Both verified (iPhone + Pixel_9a emulator). Plan/record: [`../plans/back_swipe_gesture.md`](../plans/back_swipe_gesture.md).

---

## Web App — Ship Full App via Zite at zwipe.net

Build the full deck builder into zite so `zwipe.net` serves both marketing pages (logged out) and the authenticated app experience (logged in). See `architecture/decisions.md` for rationale.

### Wasm Build Blockers

Zwiper doesn't compile to `wasm32-unknown-unknown` yet. Two issues (discovered 2026-04-06):

1. **`getrandom` needs `wasm_js` feature** — `getrandom` 0.4+ requires explicit `features = ["wasm_js"]` for wasm32 targets. Zite already has this. Zwiper needs it too, but it goes in `zwiper/Cargo.toml` (NOT the workspace root — virtual manifests can't have `[target]` sections).

2. **`tokio` pulls in `mio`, which doesn't compile to wasm32** — Tokio's full runtime uses OS-level I/O via mio, which has no wasm support. Zwiper uses tokio in 4 places, all for timers:
   - `zwiper/src/lib/inbound/screens/profile/components/delete_account_dialog.rs` — `tokio::time::sleep`
   - `zwiper/src/lib/inbound/components/auth/session_upkeep.rs` — `tokio::time::interval`
   - `zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs` — `tokio::time::sleep`
   - `zwiper/src/lib/inbound/screens/deck/card/components/image_preview.rs` — `tokio::time::sleep`

   Options for wasm-compatible timers:
   - `gloo_timers::future::sleep` for wasm, `tokio::time::sleep` for native (behind `#[cfg]`)
   - `dioxus-sdk-time` (already a dependency) if it provides cross-platform timers
   - `web_sys::setTimeout` wrapped in a future

   Tokio itself should be gated behind non-web features in zwiper's `Cargo.toml`, or the timer calls need platform abstraction.

### Build the App into Zite

Once wasm compiles, build the authenticated experience into zite:

- [ ] Resolve wasm build blockers (getrandom feature + tokio/mio platform abstraction)
- [ ] Add login/register screens to zite
- [ ] Add authenticated routes: deck list, deck view, card search/swipe, profile, preferences
- [ ] Dual input for card selection: swipe gestures for mobile browsers, arrow buttons for desktop
- [ ] Add `zwipe.net` to zerver's `ALLOWED_ORIGINS` for CORS
- [ ] Session storage for web (localStorage or similar — no keyring on web)
- [ ] Test full auth flow: register, verify email, login, refresh token rotation
- [ ] Test deck CRUD, card search, card add/remove via both swipe and arrow buttons
- [ ] Rework `/download` page — still useful for iOS users, but less central

### Architecture Notes

- **Single domain**: `zwipe.net` — no subdomain split. Marketing and app coexist.
- **Security posture unchanged**: Same JWT auth, rate limiting, account lockout. Browser is just another API client.
- **Ship both**: Webapp ships first for immediate reach. iOS submits to App Store in parallel.
- **Reuse**: zite already depends on `zwipe-core`. Domain types, validation, and shared CSS (`shared/themes.css`) are ready.

---

## Marketing & Discovery

- [ ] **Marketing posts — keep the cadence going.**
  - **Wave 1 — 2026-06-25 (Day 333):** build-in-public text posts to X, Reddit, Bluesky (1.1.0 features).
  - **Wave 2 — 2026-06-28 (Day 336): the swipe-demo video.** A vertical 9:16 screen-recording (Tinder-for-Magic hook: swipe a commander → build by swiping → filter → import → finished deck), captions + an end card built from `context/marketing/video_end_card.html`. Posted natively to **Instagram** (new `@scadoshi`), **YouTube Shorts**, **X**, **Bluesky**, and **Reddit r/EDH** (video + pinned dev comment, feedback-framed). **TikTok pending** — the old 8-yr personal account was compromised (password breach; rogue sessions cleared, no third-party apps, recovering via phone+email, needs new pw + 2FA before use); lean toward a fresh branded handle. Post-copy templates were drafted then deleted (kept out of the repo).
  - r/EDH and r/magicTCG *posts* are karma/age-gated (comment first, build toward posting); reuse the r/EDH title + pinned comment for r/custommagic, r/SideProject, r/buildinpublic.
- [ ] **Business cards for LGSs** (noted 2026-06-10, designed 2026-06-12; **PRINTED + distributing as of 2026-06-28** — cards out on local bulletin boards + 1 LGS so far, working on store-owner relationships; ongoing) — self-contained HTML/CSS mockup at `context/marketing/business_card.html` (committed `cebe5d91`). It's a real MTG-style **creature card** (not a token): 2.5×3.5in, gruvbox by default with a dark-theme picker, front + back shown together. Front = title bar with WUBRG mana cost (real mana-font symbols, toggle for coded `{W}` text), theme-tinted **QR** (defaults to zwipe.net; App Store option) as the art, type line `Artifact Creature — Mobile App` with the pixel-Z as the set symbol, oracle text with keyworded features (**ETB swipe** left-red/right-green, **Synergy**, **Import**, **Free** — "Free forever. No ads.", deliberately no "subscriptions" so premium stays open), `*/*` P/T, selectable flavor (default banger: "Every great deck begins with a single swipe."). Back = full Zwipe wordmark + `zwipe.net`, nothing else. Oracle text auto-fits its box. Flat (no gradients). **Next: keep distributing — more LGSs + bulletin boards, build store-owner relationships** (see printing notes below). Ask store owners before leaving stacks; commander-night crowds are the target.
  - **Printing notes**: standard playing-card stock is **2.5×3.5in (63×88mm)**. For a true card feel use a card printer (MakePlayingCards/MPC, PrinterStudio, Ad Magic, or a local print shop with 300–350gsm + matte/linen finish) — order a custom card or "custom card game" SKU. For cheap-and-fast, a local shop can do business-card stock but it's thinner. Export: open the HTML, print to PDF at 100% (no scaling) with "background graphics" ON; add 1/8in bleed if the printer wants it. QR scannability holds down to ~0.8in — current QR is well above that.

---

## Monetization

- [ ] **TCGplayer affiliate** — application submitted 2026-06-23, **In Review** on Impact (impact.com). When approved: wire the tracking ID into `tcgplayer_url()` (`zwiper/.../outbound/buy_links.rs`) — zero UI change — then add per-card **"Buy ↗"** links (currently buy is whole-deck only).
- [ ] **Card Kingdom affiliate** — **no public self-serve program**; it's a direct-outreach partnership. Email CK when ready (cite the Archidekt `?partner=` precedent). `cardkingdom_url()` stays untracked until then.
- Detail + saved signup copy: `context/product/affiliate/tcgplayer.md`.

---

## Web/Zite Polish

- [ ] **Guides SEO leftovers.** The guides knowledge base shipped (12 guides live under `/guides`), which retired the SEO-guides plan (now [`../archive/seo_guides.md`](../archive/seo_guides.md)). Still unshipped from it: (1) ~~guide routes missing from `zite/build.rs` ROUTES~~ **DONE 2026-07-08** — `/guides` index + all 13 guide pages now emit into `sitemap.xml` via a `GUIDE_SLUGS` loop in `build.rs`; (2) ~~Article/HowTo JSON-LD per guide~~ **DONE 2026-07-08** (each guide page emits `Article` JSON-LD from its title/summary/category); (3) demand-first MTG-topic guides ("best mobile MTG deck builder", "how to build a Commander deck on your phone") that ride search volume rather than app screens.
- [x] **Increase `Z` logo size on zwipe.net** — skipped 2026-07-08; current size reads fine, no change wanted.

**SEO batch shipped 2026-06-30 (zite):** OG/Twitter share image (`context/marketing/og_default.html` → `zite/assets/og-default.png`, the tag was pointing at a missing file), keyword-rich home `<title>` + a semantic `<h1>`, `MobileApplication` JSON-LD (4.8/4 rating — bump when it moves), App Store testimonials strip, a "Free on iOS & Android. No ads." line, and a `build.rs`-generated `sitemap.xml` (now includes `/download/ios`). `Dioxus.toml` fallback title cased to "Zwipe".
- [ ] **Mobile testing pass on zwipe.net** — verify landing, about, privacy, verify/reset token pages on iOS Safari + Android Chrome. Check the sticky nav and entrance animations under narrow viewports.

---

## zwiper — Open Follow-ups

- [x] **SQL-vs-predicate parity test — DONE 2026-07-10.** `zerver/tests/card_filter_parity.rs`: seeds a 16-card universe (built on the integration-test fixture), then for **55 criteria** (every branch of `CardCriteria::matches` except the 3 documented server-only pool flags — `is_partner`/`is_background`/`is_signature_spell`) asserts `{cards SQL search_cards returns} == {cards matches() keeps}`. Unblocked by the new `#[sqlx::test]` harness. Immediately earned its keep: caught a fixture-accuracy bug (rarity was stored as the long word `"rare"` where production stores the short code `"R"` via `to_short_name()` — reads masked it since `Rarity::try_from` accepts both, but the SQL rarity filters compare against short codes). Extended the `card()` fixture with `flavor_text`/`artist`/`lang`/`digital`/`oversized`/`promo`/`content_warning`/`token`/`categories` to exercise the full battery.
- [x] **Delver not showing in existing deck — RESOLVED (confirmed 2026-07-08).** The 2026-06-06 airplane observation never reproduced afterward; card serves correctly in existing decks today. Most likely the slow connection returning an empty join at the time.

---

## Synergy & Popularity Data

The cache-first synergy layer shipped (see `overview.md`); these are the consumers built on top of it.

- [ ] Synergy scores — surface cards with high synergy to the deck's commander
- [ ] Popularity data — most-played cards for a given commander
- [ ] Salt score, display per card and aggregate per deck, filtering and sorting on card search
- [ ] Evaluate further data (themes, combos, etc.) as the layer matures

---

## Mechanical Category — Heuristic Refinement

Phases 1+2 shipped (see archive). ~73% classification rate today; refinement targets are below. Layers 2+3 (AI classifier + fine-tuned model) tracked in `backlog.md`.

**Testing approach (owner idea 2026-07-09):** assemble a set of known special-case / edge cards, run each through the heuristic classifier, and have a **cheap model (Haiku)** grade whether the classification is right. Where Haiku flags a miss, **escalate that card to a stronger model** to propose the heuristic/regex fix. Turns "audit a sample by hand" into a semi-automated find-and-fix loop; do this before hand-tuning the individual patterns below.

- [ ] Add more test cases for edge cases and false positives/negatives
- [ ] Audit a sample of classified cards per category to find misclassifications
- [ ] Lands should NOT be classified as ramp (fixed: removed `type_line.contains("land")` from ramp fallback) — verify still holds
- [ ] Tune regex proximity windows (e.g. blink regex was too narrow, widened to 80 chars)
- [ ] Consider additional ramp patterns (e.g. treasure token creators, rituals like Dark Ritual)
- [ ] Consider additional removal patterns (e.g. "exile target" with qualifiers, fight mechanics)
- [ ] Burn heuristic excludes creatures — should it include creatures with ETB damage?
- [ ] Stax heuristic may false-positive on cards that say "can't" in reminder text

---

## Testing

- [x] **Integration tests — CORE COMPLETE (2026-07-09).** `#[sqlx::test]` harness driving the real Axum router via `tower::oneshot`. **All 6 slices shipped** — harness + auth flows (3), CI (a gating `test` job in each deploy `needs:`, `Test` PR-only), deck-profile lifecycle (5) + a 404-IDOR fix, the `card()`/`seed_cards()` **fixture builder**, deck-card ops (3), card serving/search (6), [repo] card tests (synergy/exclude/NULL-oracle/rollup — 4), health (1), metrics (usage→`commander_card_signal`, anonymous kinds — 4), user (change username/email/password re-auth, delete cascade — 5), auth edges (verify/reset via captured email, refresh single-use, lockout 429 — 5); **36 integration tests green**, clippy clean. **Optional backlog** (none blocking) tracked in [`../plans/integration-tests/overview.md`](../plans/integration-tests/overview.md): deck-aware serve suppression + land auto-stop, band-boundary shuffle + clone card-copy, `user_week_signal` rows, last-active debounce, preferences/hint, card-metadata DISTINCT endpoints, deck share/tokens + public share page, archidekt import. Full resume context (how to run, harness map, gotchas, remaining slices + endpoint coverage map) at [`../plans/integration-tests/overview.md`](../plans/integration-tests/overview.md) → "▶ Resume here".

---

## Maintenance

- **sqlx 0.8 → 0.9** — major bump. 0.9 has breaking changes around type mappings and connection options. Needs a dedicated branch where the integration tests run against a real Postgres before merge.
- **keyring 3 → 4** (zwiper) — major bump. Used for iOS Keychain on `apple-native`. Needs on-device test before merging; don't ship blind.
- [x] **GitHub Actions Node.js 20 deprecation — DONE 2026-07-09.** Bumped the offenders to Node-24 majors: `checkout@v4→v7`, `cache@v4→v6`, `deploy-pages@v4→v5`, `upload-pages-artifact@v3→v5` (its internal `upload-artifact` was the last transitive offender). Verified live: all three workflow runs now annotation-free.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.

---

## Ad-hoc issues (noted 2026-07-07)

- [x] **Skeleton alignment (zite + zwiper) — DONE 2026-07-08.** Rule settled: consistency is **per screen**, not global. zite's shared-deck skeleton is all plain ghosts (translucent boxes, no chrome, no inner structure); the app's skeletons stay element-like and were updated to mirror current screens — deck list gets 3 varied tag-chip tiles, edit-deck covers the full form (name/format/commander inputs, tags box, power level, other tags, land + price target), deck view shows Stats expanded + 3 collapsed section headers, and the add-screen search shows the plain skeleton (spinner removed).
- [x] **zwiper: land-target auto-filter leak — FIXED 2026-07-08.** The Add screen's exit paths (Back button + the Search→Maybeboard source switch) checked `is_empty_ignoring_deck_context()` when deciding whether to clear the auto-populated defaults, which doesn't count the automatic Land exclusion — so at land target the filter never read as default and leaked into Cards/Maybeboard. Both now use `is_empty_ignoring_deck_context_and_auto_lands(lands_at_target())`; sim-verified (lands visible in Cards, no stray "Filter is active" toast).
- [x] **zwiper: independent, per-screen, per-deck filter persistence — SHIPPED-READY 2026-07-09, sim-verified** (plan: [`../plans/filter_persistence.md`](../plans/filter_persistence.md)). `FilterStore` maps `(FilterScope, deck_id) → CardQueryBuilder` (in-memory, unbounded); each screen owns a local filter signal seeded from the store, provided as context (filter modules unchanged via provider shadowing), parked on leave. Add's two sources are separate scopes; the Back-button clear-if-default and source-switch clear/reapply hacks are gone; a restored Add filter keeps the user's synergy toggle and matches its parked stack. Owner-tested on sim; rides the next store build.
- [x] **zite shared-deck: featured-row roles are tags.** DONE 2026-07-08 (`907757a8`): `sd-cz-role` is now a color-coded tag (command zone accent-2, MVP accent-3).
- [x] **Review CSS class semantics across the frontend (zwiper + zite).** DONE 2026-07-08 in two passes: `qty-*` → `card-action-*` rename (dropped dead `.qty-btn-remove`), then the `card-row-*`/`card-detail-*` family duplication dissolved entirely by moving `CardRow` + its CSS into `zwipe-components` (each app keeps only its genuine deviations). See `plans/zwipe_components.md`.
- [x] **Consistent base-URL handling — DONE 2026-07-08.** One shared `zwipe_core::domain::site` module (`WEB_BASE`/`API_BASE` debug-gated to local dev servers, `SUPPORT_EMAIL`, `DISCORD_URL`); zwiper's three `WEB_DOMAIN` consts + duplicated support/discord consts and zite's four consts all resolve there (zite re-exports from crate root, so page imports were untouched); zerver's env defaults reference the core consts (env still overrides). Exceptions, documented in the module: zwiper `BACKEND_URL` stays `.env`-driven (device testing needs LAN IPs) and `zite/build.rs` keeps one mirrored literal (build scripts can't import the lib).
