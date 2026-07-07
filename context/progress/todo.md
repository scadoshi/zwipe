# Todo

**Primary goal: grow the user base — Android launch + marketing.** (iOS App Store: LIVE. Full webapp at zwipe.net: in progress.)

Only open, actionable items live here. When something ships, its outcome moves to
[`overview.md`](overview.md) and leaves this list. Older completed work is archived
at `context/archive/complete_2026_q1.md`.

---

## Next Up

- [ ] **Track 1.3.1 review (anonymous funnel metrics)** — iOS **build 60** + Android **vc21** submitted to both stores 2026-07-05, in review. Server deployed the same day (two additive migrations; first deploy attempt failed on a stale crate-local `zerver/.sqlx/` shadowing the workspace offline data — removed, and the workflow now verifies `.sqlx` against the migrated schema before building; see `operations/infrastructure/cicd.md`). Once clients roll out and data accrues, read the funnel with `zcripts` (distinct sessions per kind vs. `user_events.register`) — these numbers gate the sign-in-with-Google decision.
- [ ] **Track 1.3.0 review on both stores** — iOS **build 59** + Android **versionCode 20** submitted 2026-07-03 (server skip/unskip endpoints deployed to prod first). 1.3.0 supersedes the withdrawn 1.2.3 and folds in: swipe memory (FR #11) now **per-swipe durable**, **per-deck stack memory**, the CardStack refactor, the profile **About section**, image/skeleton polish, and the **filter-intent + Reset** pass (sort/synergy-only searches serve, Reset returns each screen to its default, accurate filter dot, filter sheet collapses on close). Build progression: 57/18 → 58/19 (About) → 59/20 (filter/Reset). Once live, 1.3.0 becomes the floor for `MIN_CLIENT_VERSION` gating; mark FR #11 shipped. Per-change detail in `overview.md`. Marketing video plans (`marketing/plans/`) are refreshed for this build.
- [ ] **Privacy follow-ups for per-user collection.** The policy text shipped 2026-07-02 (`b1ee1b11`, discloses per-account activity + deck skip memory). Remaining owner passes: update the App Store privacy "nutrition label" + Play data-safety form to reflect per-account analytics, and send the policy-change notification email to users.
- [ ] **Suggestion signal — Phase 3c (pair-level ranking).** Phases 3a+3b **shipped 2026-07-06** (server 1.3.2): the default synergy ordering now blends base score + pooled net-rate (`added + 0.5·maybed − removed`, shrunk/centered) + per-(deck, day) seeded jitter — different decks serve differently, the same deck stays stable within a day, and crowd favorites drift up as signal accrues. Remaining: the commander-specific pair-level term, gated on pair-depth (baseline 2026-07-06: 0 pairs ≥20 impressions — re-run the readiness queries after the user base grows). Plan: [`../plans/suggestion_signal.md`](../plans/suggestion_signal.md).
- [ ] **Draw-odds — Phase 4 (premium gating).** Phases 1–3 shipped in 1.2.0 (engine + deck-view section + per-turn `<- / ->`). Remaining: decide which depth is free vs premium, and optionally add the play/draw toggle (turn 1 = 7 on the play vs 8 on the draw). Client-only. (Plan doc removed once Phases 1–3 shipped; only this gating decision remains.)

**The next big three (set 2026-06-11, in order):**
1. **Android — get the clock ticking.** Confirm the Play account's closed-testing requirement and start the 14-day clock immediately; polish items ride alongside (see Android section below).
2. **Marketing — get users.** Business cards for LGSs, Reddit/X posts (see Marketing & Discovery below). The first wave doubles as the Android closed-test tester pool.
3. **VPS — stabilize.** Cutover is done (see `overview.md`); remaining follow-ups are the two items below.

- [ ] **Refine the launcher/app logo (deferred from 1.1.1).** 1.1.1 shipped a *functional* Android icon — the Z was repadded into the adaptive-icon safe zone (`zwiper/assets/favicon/icon-1024-android.png`, Z ≈ 47% of canvas) so the circular mask stops clipping it, but the design wasn't polished. Revisit the actual logo/icon art (and consider whether iOS/web want the same treatment). The mechanism + safe-zone math are in `operations/android/play-store-submission/build-and-submit.md` (1.1.1 history entry).
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
  queue). Pipeline + gotchas captured in `operations/android/play-store-submission/build-and-submit.md`;
  listing copy in `.../form_fields.md`.
- [ ] **Recruit ≥12 closed testers + run the 14-day clock.** New personal accounts
  need **12 testers opted in for 14 *continuous* days** before applying for
  production access. Track the live opted-in count **in Play Console** — the clock
  starts only when 12 are opted in *simultaneously*, not when invites are sent.
  - **Setup done 2026-06-24:** public Google Group **`zwipers`**
    (`groups.google.com/g/zwipers`) created and attached to the Alpha closed-testing
    track (Testers → Google Groups); self-opt-in verified ("You are a tester").
    Tester flow is 3 steps: **join group → opt in**
    (`play.google.com/apps/testing/com.scadoshi.zwipe`) **→ install**
    (`play.google.com/store/apps/details?id=com.scadoshi.zwipe`). A bare opt-in/store
    link does nothing for non-members — the group join is what makes someone eligible.
  - **Recruiting progress 2026-06-24:** 3 personal-network testers emailed (still
    need to confirm they actually completed opt-in). Started **reciprocal
    comment-swapping on r/AndroidTesting** (test theirs ↔ they test yours) — note
    our own *posts* there are **karma/age-gated** (so are the MTG subs r/EDH /
    r/magicTCG), but *comments* aren't, so swap via comments now + build karma toward
    posting later. Un-gated bulk channel to try next: **Discord/Telegram
    closed-testing exchange servers** (search "Android closed testing" on disboard.org).
  - **Open before driving strangers in:** verify `zwipers` is joinable from a
    **logged-out/incognito** session — a non-public group silently blocks every
    opt-in (saw this exact failure on two other devs' groups: "Content unavailable" /
    "no permission" = their join setting wasn't "Anyone can join"). Then chase the 3
    emailed testers and keep swapping toward 12.

Android build compiles and runs — **emulator-confirmed on Pixel_9a 2026-06-22** (login → decks → swipe all working against prod). Build gotcha: must point `JAVA_HOME` at Android Studio's bundled JDK 21, not the system-default JDK 26 — see `operations/android/setup.md`. The home-screen ASCII logo (block glyphs) now renders correctly via the self-hosted font (ships in 1.0.9). None of the below blocked the closed-testing submission — they're polish for a future build:

- [ ] Card images show white corners — the white is baked into the image data from Scryfall (white-bordered card editions). iOS clips correctly via WKWebView; Android WebView does not honor `overflow: hidden` + `border-radius` on `object-fit: contain` images. Tried: `overflow: hidden` on img, wrapper div with `border-radius` + `overflow: hidden`, `-webkit-mask-image` hack. None work on Android WebView. Options: crop with `object-fit: cover` (loses card edges), mask SVG overlay, or accept as-is for black-bordered cards (majority) and revisit for white-bordered.
- [ ] Swipe gesture doesn't tilt the card — cards should rotate slightly during drag like they do on iOS
- [ ] Lock screen orientation to portrait — need `android:screenOrientation="portrait"` on main activity. Dioxus may support this via `[android.raw.manifest]` or activity-level config. Test on Pixel once available.

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

- [ ] **Guides SEO leftovers.** The guides knowledge base shipped (12 guides live under `/guides`), which retired the SEO-guides plan (now [`../archive/seo_guides.md`](../archive/seo_guides.md)). Still unshipped from it: (1) **guide routes are missing from `zite/build.rs` ROUTES**, so none of the guide pages land in `sitemap.xml` — five-line fix, do this one; (2) Article/HowTo JSON-LD per guide; (3) demand-first MTG-topic guides ("best mobile MTG deck builder", "how to build a Commander deck on your phone") that ride search volume rather than app screens.
- [ ] **Increase `Z` logo size on zwipe.net** — current ASCII logo reads small; bump scale or font-size on the landing hero.

**SEO batch shipped 2026-06-30 (zite):** OG/Twitter share image (`context/marketing/og_default.html` → `zite/assets/og-default.png`, the tag was pointing at a missing file), keyword-rich home `<title>` + a semantic `<h1>`, `MobileApplication` JSON-LD (4.8/4 rating — bump when it moves), App Store testimonials strip, a "Free on iOS & Android. No ads." line, and a `build.rs`-generated `sitemap.xml` (now includes `/download/ios`). `Dioxus.toml` fallback title cased to "Zwipe".
- [ ] **Mobile testing pass on zwipe.net** — verify landing, about, privacy, verify/reset token pages on iOS Safari + Android Chrome. Check the sticky nav and entrance animations under narrow viewports.
- [ ] **Everforest theme review** — possibly too green; sample real card art against it and consider desaturating the background or shifting the accent.

---

## zwiper — Open Follow-ups

- [ ] **SQL-vs-predicate parity test** — build a `CardCriteria` with every field populated and assert that the backend SQL adapter and `CardCriteria::matches` accept/reject the same fixture set (needs a test Postgres). Would catch the "field exists end-to-end except in the frontend predicate" pattern that hid the commander and legality bugs (fixed in 1.0.2). *Narrowed 2026-07-02 by the CardFilter split:* the predicate now lives on the same `CardCriteria` type the query flattens (wire round-trip is tested), but `matches` still mirrors the SQL by hand — this fixture parity check is the remaining gap.
- [ ] **Investigate Delver not showing in existing deck after MDFC fix landed** — observed 2026-06-06 on an airplane (slow wifi may be a factor): a deck containing `Delver of Secrets` returned zero rows from `SELECT ... WHERE scryfall_data.name ILIKE 'delver of secrets'`. Card and combination definitely existed before. Fresh deck testing worked fine, so probably state-specific. Could be the migration's deck-card remap pointed the old reference at a row that no longer exists, OR the connection was timing out and the join silently returned empty. Worth re-running the query off airplane wifi first; if it still returns zero, dig into the remap audit.

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

- [ ] **Integration tests — PLANNED 2026-07-06, full plan at [`../plans/integration_tests/`](../plans/integration_tests/overview.md).** External audit confirmed the gap: zero coverage on sqlx repos, HTTP handlers, and most domain services; no CI test run at all. Decisions settled: `#[sqlx::test]` harness (fresh DB per test), both HTTP-level and repo-level coverage, plus a GitHub Actions test workflow (non-gating on deploy). Build in slices: harness + auth flow first, then deck lifecycle, then card serving (replaces the throwaway band-shuffle dev harness with permanent regression tests).

---

## Maintenance

- **sqlx 0.8 → 0.9** — major bump. 0.9 has breaking changes around type mappings and connection options. Needs a dedicated branch where the integration tests run against a real Postgres before merge.
- **keyring 3 → 4** (zwiper) — major bump. Used for iOS Keychain on `apple-native`. Needs on-device test before merging; don't ship blind.
- **GitHub Actions Node.js 20 deprecation** — forced to Node.js 24 on June 2, 2026. All workflows already on latest major versions. No action needed — monitor for v5 releases.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
