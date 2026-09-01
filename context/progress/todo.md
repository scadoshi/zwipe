# Todo

**Primary goal: grow the user base — marketing + tester-feedback intake.** (iOS App Store: LIVE. Android Play Store: LIVE (production). Full webapp at zwipe.net: in progress.)

Only open, actionable items live here. When something ships, its outcome moves to
[`overview.md`](overview.md) and leaves this list. Older completed work is archived
at `context/archive/complete_2026_q1.md`.

---

## Next Up

- [x] **1.9.3 LIVE at both stores 2026-08-20** (iOS build 77 / Android versionCode 40). The freeze is over: the next cut is **1.10.0** at build 78 / versionCode 41 (workspace already bumped 2026-09-01; the in-universe feature earned the minor bump). Both artifacts were cut 2026-08-18 from `a36d9e73` and are signed, verified, and sitting in the repo root: `Zwipe.ipa` (build 77) and `zwipe-1.9.3.aab` (versionCode 40). Both numbers are now spent. **Both were rebuilt 2026-08-18** after `cargo update -p h2` (0.4.15 → 0.4.16) cleared RUSTSEC-2026-0258, which the push's Security audit caught; h2 reaches zwiper through reqwest, so the first pair of artifacts was stale. Rebuilding kept build 77 / versionCode 40 because neither had been uploaded yet, and an unuploaded number is not burned. Docs-only commits are fine and do not invalidate them. **Any change to zwiper, zwipe-core, zwipe-components or zerver invalidates the artifacts**, and the right move then is to bump the workspace to 1.9.4 and rebuild both rather than ship a binary whose baked `CARGO_PKG_VERSION` disagrees with the store listing. Do not quietly rebuild 1.9.3 with new code in it: build 77 and versionCode 40 are burned the moment they are uploaded, and Play rejects a reused versionCode outright. Store notes for 1.9.3 are already written in both `form_fields.md` files; a 1.9.4 would need its own. Sequence when 1.9.2 goes live: submit both 1.9.3 artifacts, then move this line to `overview.md`.

- [ ] ~~**CUT 1.9.2**~~ — **DONE 2026-08-17: submitted to both stores** (iOS build 76 / Android versionCode 39). Carries the two Android manifest fixes (the ndk-context crash that survived five releases, and the app silently closing on a system theme change), the back-swipe overlay fixes, the deck list restyle with command-zone art, command-zone art URLs on the wire, per-combination color grouping with mana pips, and the zite work (share-page deal-in, guides search, Panel heroes, 36 guide screenshots). The post-bundle patches are now **one command** — `zcripts/android/patch_bundle.sh` (icons + back handler + manifest). Skipping it silently reships the crash; that checklist is exactly how the bug lived five releases. Build steps: [`../operations/android/play-store/submission/build.md`](../operations/android/play-store/submission/build.md).

- [ ] **Phase 6 — serve on the matured otag signal (data-gated, months out).** The prerequisite **Phase 5S step-3 cleanup shipped 2026-07-25** (legacy `commander_oracle_id` wire + server fallback + client commander resolution all dropped; deck_id is the sole signal key). Server half deploys on next push; the client half rides the next client build. Re-run the pair-depth readiness queries as the user base grows.
- [ ] **Read the anonymous funnel once data accrues.** Anonymous funnel metrics (app-open / register-viewed / register-submitted) have shipped in prod since 1.3.1. When enough sessions accrue, read the funnel with `zcripts` (distinct sessions per kind vs. `user_events.register`) — these numbers gate the sign-in-with-Google decision.
- [ ] **Suggestion signal — Phase 3c (pair-level ranking).** Phases 3a+3b **shipped 2026-07-06** (server 1.3.2): the default synergy ordering now blends base score + pooled net-rate (`added + 0.5·maybed − removed`, shrunk/centered) + per-(deck, day) seeded jitter — different decks serve differently, the same deck stays stable within a day, and crowd favorites drift up as signal accrues. Remaining: the commander-specific pair-level term, gated on pair-depth (baseline 2026-07-06: 0 pairs ≥20 impressions — re-run the readiness queries after the user base grows). Plan: [`../plans/archive/suggestion_signal.md`](../plans/archive/suggestion_signal.md).

---

## Bugs

- [ ] **Store-build eyeball, now against 1.9.3 (live 2026-08-20):** confirm the deck list restyle and back-swipe behave, and reinstall Zwipe from the store on the Pixel, which is still on a debug-signed build Play cannot update. Delete this line once both are done. Early field signal, read 2026-08-20: **zero ndk-context crashes on 1.9.2/1.9.3** with 7 Android users on fixed builds, while 1.9.1 (13 users) was still producing ~10/day through 08-18; the formal 7-day check on 08-25 stands.

- [ ] **One week after 1.9.2 goes LIVE (not submitted), read the crash + error tables and confirm the Android fix held.** **Live at both stores 2026-08-18, so the clock has started: run this on or after 2026-08-25.** 1.9.3 is in review behind it and carries the same manifest patches, so a mid-week 1.9.3 rollout does not reset the window. This is the step that was skipped after 1.7.6 and cost five releases of false confidence, so do not close the bug on the lab result alone. Run both queries — crashes alone can fall because *adoption* fell:

  ```sql
  -- must be zero for ndk-context on 1.9.2+
  SELECT client_version, date(occurred_at) AS day, count(*) AS crashes
  FROM crash_reports WHERE message LIKE '%ndk-context%'
  GROUP BY 1,2 ORDER BY 2 DESC LIMIT 14;

  -- the denominator: are there actually Android users on the new build?
  SELECT client_version, platform, count(DISTINCT user_id) AS users
  FROM refresh_tokens WHERE created_at > now() - interval '7 days'
    AND platform = 'android' GROUP BY 1,2 ORDER BY 3 DESC;
  ```

  Readable over SSH from the work Mac (ask the owner for the address; the metrics suite lives in `zcripts/metrics/`). While in there, also check `errors.sql` for anything new that 1.9.2 introduced. If clean: delete this line and the crash line below. If not clean: the archived plan lists the hypotheses that were never ruled out (a second Activity path we did not find, or the bridge itself) — [`../plans/archive/android_ndk_context_crash.md`](../plans/archive/android_ndk_context_crash.md).

- [ ] **Android ndk-context crash — FIXED AND VERIFIED ON DEVICE 2026-08-17; confirm in the field after 1.9.2 ships, then delete this line.** Root cause was never the resume path: `MainActivity` had no `launchMode`, so an explicit component start (notification tap, another app, an app shortcut, the Play Store's Open button after an update) created a SECOND Activity in the live process, re-running NativeActivity's native init and tripping `assert!(previous.is_none())`. Fixed with `launchMode="singleTask"`; a second bug found the same session (`configChanges` omitted `uiMode`, so a system theme change tore down the Activity and the onDestroy process-kill silently closed the app) fixed by widening configChanges. Both applied by `zcripts/android/manifest.sh` via `patch_bundle.sh`. Verified on a Pixel 6: the launch that panicked is clean five times over, dark-mode toggles no longer kill the app, zero panics. **Field check:** zero ndk-context crashes for 7 days at comparable Android session volume (queries in the archived plan). Evidence: [`../plans/archive/android_ndk_context_crash.md`](../plans/archive/android_ndk_context_crash.md).
- [ ] **Back-swipe overlay fixes — DONE AND VERIFIED ON BOTH PLATFORMS 2026-08-17; ships in 1.9.2, delete this line once released.** Cause was structural: overlays only participate in back handling if they *register* with `OverlayBackStack`, and registration was opt-in, so anything that forgot the hook fell through to `go_back()` and ejected the user. Fixed by registering the shared `BottomSheet` (which alone covers the deck view, deck list, maybeboard, profile and preferences sheets), plus `format_select` (the reported bug; back maps to Cancel), `tag_select`, and `printing_sheet` (hand-rolls its own backdrop, so the shared fix didn't reach it). Verified on iOS by hand and on Android over adb with `KEYCODE_BACK`, including the nested picker->dictionary case and the fall-through case that proves registration didn't over-capture. Untested and lower risk: preferences theme-revert, printing sheet, AlertDialogs. Plan: [`../plans/archive/back_swipe_audit.md`](../plans/archive/back_swipe_audit.md).
- [ ] **Silent zero-results on the add screen (Ebany, Discord 2026-08-23; triaged 2026-09-01).** Synergy defaults on for commander decks and intersects every other filter with the commander's pool, so a set include like "Universes Within" can legitimately return nothing — and the screen shows a bare skeleton that reads as "no cards exist". Agreed fix: toast in `add.rs`'s search Ok branch, one composed message — base "No results for this filter" plus, when synergy is on, "; tap Synergy to turn it off" (the chip is on the screen itself, top right) or "; Synergy is warming up, so all cards were searched" while warming. Built 2026-09-01, awaiting owner eyeball; pointer: [`../plans/in_universe_filter.md`](../plans/in_universe_filter.md) step 1. This is also the prerequisite for ever building sticky/default filters (parked — a sticky filter mystery-empties searches weeks later).

- [ ] **Featured-strip (command zone / MVP) skeleton mismatch + zero-height collapse (owner, Discord 2026-08-23; scoped 2026-09-01).** Two problems: (a) the ghost strip in `view.rs` is a hand-built lookalike that drifted — no `max-width: 11rem` cap so ghosts render wider than real cards, gap 0.75rem vs 0.5rem, wrong margins, always exactly 3 cards for a 1–6 strip, chip much taller than the real role chip; (b) the bigger jump — the skeleton is removed when deck data resolves (`featured_cards.is_empty()` flips), but `FlippableCardImage` starts at `opacity: 0` with `height: auto` and `.deck-featured-image` reserves no aspect-ratio, so the strip collapses to zero height then pops when image bytes land. Fix pattern exists in-repo: the deck-list skeleton reuses live classes so geometry can't drift (`skeletons.rs`, comment says so explicitly), and `.no-image-card` reserves a 5/7 footprint. Reserve aspect-ratio on `.deck-featured-image` + rebuild the ghost from live classes + match the ghost count to the known command-zone size. Sweep in the stray `.skeleton-deck-list-art` width (3.2rem vs real 4rem).

- [ ] **Password reset screen + both email templates polish (owner, Discord 2026-08-23; scoped 2026-09-01).** zwiper's `forgot_password.rs`: label and placeholder both say "Email", success state has no resend affordance (server silently cooldowns, so a second tap shows success with no email sent), no `onsubmit` so keyboard return does nothing. `reset_password.html` + `verify_email.html` (untouched since July): bare "Hi," greeting, the ~90-char font stack repeated inline on every element, no colors set (CTA borrows `currentColor` — dark-mode Gmail unpredictable), no support/Discord link unlike the three account-change templates (which inconsistently hardcode the Discord URL where `email_changed` uses `{support_email}`). Copy content is accurate (15-min / 24-h expiries match server). The set-new-password page is zite `/reset` (title-case "Reset Password" vs sentence case elsewhere). All copy/markup, no logic.

- [ ] **App unresponsive after long backgrounding (owner report 2026-07-30, iOS observed).** Leave the app backgrounded for a long time, return: sometimes the ENTIRE screen is unclickable until force-close + relaunch. Investigation leads, none confirmed: (a) a full-screen element left mounted and intercepting taps (modal backdrop, toast container, an overlay whose dismiss never fired); (b) the WebView's JS event bridge dying after OS memory pressure while the rendered page survives (wry/dioxus eval channel); (c) something in the resume path (visibility flusher, session refresh single-flight) wedging the main loop. Repro is intermittent — next occurrence, note which screen it happened on and whether scrolling still works (scroll-works-but-taps-don't points to (b)); the new crash/error reporting won't catch this class (no panic, no error toast).

Recently resolved (outcomes in [`overview.md`](overview.md)):
the **filter-sheet Reset/Cancel commit bug** (fixed 2026-07-22 `10cf0735` with a
current/staged filter split; Apply is the only commit), the **share-page mana-value
group ordering** (fixed 2026-07-22 `edd46b2e`, contiguous column partition), the
**pre-1.6.0 "connection error" wire break** (fixed by flooring `MIN_CLIENT_VERSION=1.6.0`,
2026-07-13; root cause fully removed 2026-07-14 when the Phase M sunset dropped the
`mechanical_categories` dual-emit), and **app version in session data** (shipped `d1c874fe`,
recorded per-session on the refresh-token row).

Completed fixes are archived to
[`archive/complete_2026_q3.md`](../archive/complete_2026_q3.md) (hashes stay searchable there).

---

## Features — queued (owner 2026-07-11)

- [ ] ~~**Deck status tags (work in progress vs finished)**~~ — **DROPPED (owner, 2026-08-17): the app already derives completeness.** `validate_deck` runs twelve checks (card count, commander required, legality, copy limits, color identity, land/price targets, command-zone validity, sideboard limits) and a deck with no warnings *is* a finished deck. A user-set status tag would be a second, hand-maintained source of truth for something the app already knows and keeps current — and it would drift the moment someone edited a deck without updating the tag. If deck-list filtering by "done vs in progress" is ever wanted, derive it from the warning count rather than storing it.
- [ ] ~~**Deck folders**~~ — **PARKED indefinitely (owner, 2026-08-11: "might never build it")**; the group/filter row above covers the organize-my-list need at the 20-deck cap. Spec preserved at [`../plans/deck_folders.md`](../plans/deck_folders.md); freeform folder names are the one thing filters can't express, so revisit only on real user demand.
- [ ] **Card images on the deck cards screen — minimum SHIPPED 2026-07-30 (rides 1.7.5): `FeaturedCards` strip (command zone role-labeled + MVPs in star order, tap opens the image preview overlay) at the top of the deck cards list, mirroring the zite share row via the shared `FlippableCardImage`. Owner moved it from the deck profile screen to the cards screen.** Still open, deliberately deferred: images on expanded card rows — real design decision, not a drive-by. The always-show-images option for the whole list **shipped 2026-07-30**: `ShowRowArt` defaults on, the "Art" toggle chip sits in the deck cards chip row, and `zwipe-components/src/card_row.rs` renders the thumbnail.
- [x] **Deck share page: full app sections — BUILT + owner-reviewed 2026-08-13.** The share page carries all five deck-view sections (Budget with currency chips + price target, Tags incl. oracle tags, Distributions with Avg P/T, Mana with land target + curve + fulfillment, interactive Draw odds) as collapsible panels between the featured cards and the controls — collapsed by default, several per row. Card groups collapse too (shared `SdCollapsibleGroup`, default open). Whole chart family now shared: `zwipe_components::{DeckCharts, ManaCurve, ManaFulfillment, DrawOdds, ChartLabel}` fed by `DeckMetrics` chart methods (bar/balance/draw-odds math hoisted from the app's view, which deduped onto them). `HttpSharedDeck` gained serde-defaulted `land_target`/`price_target`/`price_target_currency`. The server half is deployed (`get_shared_deck` serves `land_target`), so all that's left is eyeballing a deployed share page with targets showing, then deleting this line.
- [ ] **Import should carry printings, not just card names (owner 2026-08-17).** Text and Archidekt imports currently resolve a name to some printing, so a decklist that specifies a set/collector number loses that choice on the way in. Both formats can express it: Archidekt exports carry set + collector number, and the plain-text convention is `1x Card Name (set) 123` (which the import screen already displays in its own example). Make the importer honor the printing when one is given and fall back to the current behavior when it isn't. Open: what to do when the named printing isn't in `latest_cards` (fall back silently vs surface it in the existing Unresolved list, which already has a per-card reason), and whether export should round-trip printings so a Zwipe export re-imports identically.

- [ ] **In-universe filter + printing-aware set filters — LIVE IN PROD 2026-09-01.** Server + client UI all shipped (`6dc30b48`, `b443b054`, `f4dcd650`, `98d67e41`, `a31819f1`); `zervice_role.sql` re-run on prod (as postgres via stdin redirect, the app role can't transfer matview ownership) and verified by a manual zervice run on v1.10.0: `oou sets overlay: 71 wholly-UB set codes`, refresh ok under the zervice role, all 5 steps ok. The feature is a global preference (`user_preferences.exclude_universes_beyond` + franchise-slug whitelist incl. the exception-only Secret Lair catch-all), applied server-side at serve time so old clients get it too; UI is the Profile's Universes Beyond Show/Hide row + Exceptions chip sheet. Store builds CUT 2026-09-01 from `6b640640`: `Zwipe.ipa` (1.10.0, build 78, Apple Distribution signed, Xcode 26.6/SDK 26.5) and `zwipe-1.10.0.aab` (versionCode 41, targetSdk 36, upload-key signed, all four step-4a patch greps verified) sit in the repo root, both `form_fields.md` files carry the 1.10.0 What's New. Numbers burn on upload; docs-only commits don't invalidate the artifacts, code commits do. Remaining: owner submits both, then move this to overview. Maintenance chore per UB release: top up `universe.rs` FRANCHISES (census query in its module docs).** Set filters run against the one printing `latest_cards` picks, so excluding a set can hide a card outright — Sol Ring's pick is a Secret Lair Drop row today, so an SLD exclude removes Sol Ring from search (live bug, third printing-shadowing incident after the prefer-real-printings and prefer-english migrations). Fix: per-oracle `printing_set_names TEXT[]` aggregate in the view (include = "has a printing in", exclude = "every printing excluded"), an in-universe ORDER BY tiebreaker on the pick, and then an "exclude Universes Beyond" chip that reads the pick directly. OOU detection = `security_stamp = 'triangle'` + a hand-maintained set-code list in zwipe-core (spm/tla/tle are stampless — owner accepts the per-UB-release chore). Steps 2–4 are server-only and deploy alone; the chip forces 1.9.4. Ebany's other asks are parked: sticky/default filters (until the zero-results toast exists) and the earliest-print setting (declined; the pick preference addresses the real complaint). Full design: [`../plans/in_universe_filter.md`](../plans/in_universe_filter.md).

- [ ] **Deck composition targets (big, needs MVP scoping).** (owner, Discord, 2026-07-23) Like the land target, but for everything a card can be: set per-deck target counts for a card role (10 ramp), a card type (30 creatures), or cards matching an arbitrary filter, with goal-vs-actual display. Needs functionality + MVP scoping before any build: storage shape (deck_profile vs new table), which axes ship first (roles/types before filter-based), editing UI, and whether targets feed warnings. Skipped for now by owner call.
- [ ] **Mana pip-count filter (investigate).** (owner idea, Discord, 2026-07-21) Let players filter the card pool by the exact count of colored pips of a given color in the mana cost — e.g. "has 2 blue pips and 1 red pip." Per-color pip counts are derivable from `mana_cost`. Open: UI (per-color count steppers? which colors shown?), match semantics (exact vs min/max), and whether it stacks with the existing color/mana filters. Not specced.

---

## Web App — Ship Full App via Zite at zwipe.net

Build the full deck builder into zite so `zwipe.net` serves both marketing pages (logged out) and the authenticated app experience (logged in). See `architecture/decisions.md` for rationale.

### Wasm Build Blockers

Zwiper doesn't compile to `wasm32-unknown-unknown` yet. Two issues (discovered 2026-04-06):

1. **`getrandom` needs `wasm_js` feature** — `getrandom` 0.4+ requires explicit `features = ["wasm_js"]` for wasm32 targets. Zite already has this. Zwiper needs it too, but it goes in `zwiper/Cargo.toml` (NOT the workspace root — virtual manifests can't have `[target]` sections).

2. **`tokio` pulls in `mio`, which doesn't compile to wasm32** — Tokio's full runtime uses OS-level I/O via mio, which has no wasm support. Zwiper uses tokio only for timers (`tokio::time::sleep` / `interval`), spread across 10 files as of 2026-08-18. Get the current list rather than trusting a written-down one:

   ```sh
   grep -rl "tokio::time" zwiper/src
   ```

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

- [ ] **Discord testimonials — parked pending permission asks (owner call 2026-08-10).** The site's review panel now carries "Shipped in x.y.z" receipt tags; two Discord quotes would strengthen it if their authors consent (ask in-channel first — Discord isn't published the way App Store reviews are): Ebany's 500-card verification ("no discernable performance change... handles 500 cards quite well" → Shipped in 1.7.0, request-to-ship same day) and hopeful_boy's beginner angle ("I've loved the app so far as a beginner deck builder", no tag). Mark provenance ("From the Discord") if added. Future: the companion-slot thread becomes a tagged Caed quote the day companions ship.
- [ ] **Marketing posts — keep the cadence going.**
  - **Wave 1 — 2026-06-25 (Day 333):** build-in-public text posts to X, Reddit, Bluesky (1.1.0 features).
  - **Wave 2 — 2026-06-28 (Day 336): the swipe-demo video.** A vertical 9:16 screen-recording (Tinder-for-Magic hook: swipe a commander → build by swiping → filter → import → finished deck), captions + an end card built from `context/marketing/video_end_card.html`. Posted natively to **Instagram** (new `@scadoshi`), **YouTube Shorts**, **X**, **Bluesky**, and **Reddit r/EDH** (video + pinned dev comment, feedback-framed). **TikTok pending** — the old 8-yr personal account was compromised (password breach; rogue sessions cleared, no third-party apps, recovering via phone+email, needs new pw + 2FA before use); lean toward a fresh branded handle. Post-copy templates were drafted then deleted (kept out of the repo).
  - r/EDH and r/magicTCG *posts* are karma/age-gated (comment first, build toward posting); reuse the r/EDH title + pinned comment for r/custommagic, r/SideProject, r/buildinpublic.
- **Business cards — reference for reprints** (task done; card at `context/marketing/business_card.html`, `4d2c6606`). Standard playing-card stock is **2.5×3.5in (63×88mm)**. For a true card feel use a card printer (MakePlayingCards/MPC, PrinterStudio, Ad Magic, or a local shop with 300–350gsm + matte/linen) — order a custom card / "custom card game" SKU. Export: open the HTML, print to PDF at 100% (no scaling), "background graphics" ON; add 1/8in bleed if the printer wants it. QR holds down to ~0.8in (current is well above).

---

## Monetization

- [ ] **TCGplayer affiliate**: status and next steps live in [`../product/affiliate/tcgplayer.md`](../product/affiliate/tcgplayer.md); check there rather than here. When approved: wire the tracking ID into `tcgplayer_url()` (`zwiper/.../outbound/buy_links.rs`) — zero UI change — then add per-card **"Buy ↗"** links (currently buy is whole-deck only).
- [ ] **Card Kingdom affiliate** — **no public self-serve program**; it's a direct-outreach partnership. Email CK when ready (cite the Archidekt `?partner=` precedent). `cardkingdom_url()` stays untracked until then.

---

## Web/Zite Polish

- [ ] **About page (`/about`) visual overhaul — larger redesign wanted.** A partial alignment pass landed 2026-07-21 (`8c873e4e`, `f0fcae6d`, `4990b0a0`): five-crate diagram with `zwipe-components`, the enrichment card rewritten for community oracle tags (roles derived from otag subtrees, not the retired heuristic), refreshed test counts, and the whole thing brought toward the app's tag/chip grammar (colored theme chips cycling accent 1–3 / success / warning / error, single-line wrapping header, tech stacks as chip rows, linkified imports). Owner still wants a fuller visual redesign of this section. Open bits from the pass: single-label subtitles (Scryfall "external service", PostgreSQL "primary datastore", the two foundation bands) → chips for full consistency; and the tagline comma (moot now the tagline is gone).
- [x] **Favicon with a background color — REGENERATED 2026-08-05, deploys with next push.** All six assets (`favicon.ico` 16/32/48/64, `favicon-16x16/32x32.png`, `icon-180/192/512.png`) recomposited from `zite/assets/favicon-no-background/` onto solid `#282828` (the Android adaptive-launcher bg). Google recrawls favicons on its own schedule — check the "zwipe" SERP icon in ~a week, then delete this line.
- [x] **Contribute page: mirror the portfolio site's version — DONE 2026-08-05, deploys next push.** zite's `/contribute` rebuilt on the shared `zwipe_components::Panel` cards (the delta vs the portfolio was hand-rolled divs vs Panels — same three options/URLs already matched), portfolio card copy adopted, Zwipe-specific intro kept. Delete this line after a look at the deployed page.
- [ ] **Tiny, ride along with the next zite change:** `zite/src/pages/guides/mod.rs` doc comment still says "19 articles" (the registry holds 21 — commander-maybeboard and one more landed since), and the 2026-08 commit note's "36 images across 19 guides" count is equally stale. Comment-only fix.
- [ ] **Keep zwipe.net in sync as the app grows.** The guides knowledge base shipped (20 guides under `/guides` carrying 36 screenshots, sitemap + per-guide `Article` JSON-LD landed 2026-07-08). No committed appetite for the demand-first SEO guides ("best mobile MTG deck builder", etc.) — leave them optional. The standing task is just to update the site (guides, feature pages, screenshots) as the app becomes more feature-rich. (SEO-guides plan archived at [`../archive/seo_guides.md`](../archive/seo_guides.md).)

---

## Synergy & Popularity Data

The cache-first synergy layer shipped, and so did its first two consumers:
**synergy scores** (the Synergy chip on the add screen, guide: `synergy`) and
**popularity data** (commanders serve in EDHREC-popularity order — see
`swipe_select.rs`, and the in-app hint says "Most-played cards come first").
Outcomes in [`overview.md`](overview.md). What's left:

- [ ] Salt score, display per card and aggregate per deck, filtering and sorting on card search
- [ ] Evaluate further data (themes, combos, etc.) as the layer matures

---

## Maintenance

- [x] **Orphaned otag-description slugs — CLEANED 2026-08-18; confirm on the next nightly, then delete this line.** The WARN had grown 12 → 20 (a tagger rename pass, not new tags): the whole `hand-neutral`/`hand-positive`/`hand-negative` trio was replaced by a `hand-size-*` family that splits on *maximum hand size* rather than card-advantage direction, so that text was dropped rather than moved. All 20 authored entries removed from `ORACLE_TAG_DESCRIPTIONS`; every successor (`untracked-indefinite-effect`, `phasing-matters`, `typal-serpent`, `your-sacrifice-matters`, the `hand-size-*` family) already had authored copy. Verified against the live catalog: all 4,383 remaining authored slugs exist, so the WARN should print nothing. Two silent dead references found in the same sweep, neither of which is warn-checked the way `ROLE_TAG_OVERRIDES` is: `CATEGORY_ROOTS` still listed the retired `hand-positive` under `card_advantage` (a no-op, the umbrella `card-advantage` root already covers it) and `NOISE_ORACLE_TAG_SLUGS` still hid `hand-neutral`. **Behavior change to watch:** `ROLE_TAG_OVERRIDES`' dangling `synergy-sacrifice` was remapped to `your-sacrifice-matters`, which carries 118 cards where the dead slug carried none, so the `sacrifice` role gains cards on the next `zervice` run. Blank descriptions went 75 → 70 in the same change (see below).

- [x] **Every oracle tag now has our description except one — DONE 2026-08-18, ships on the next zerver deploy.** Authored the whole remaining tail in one pass: 138 tags (69 that still carried Scryfall's own copy, 69 that were blank), taking `ORACLE_TAG_DESCRIPTIONS` from 4,383 to **4,521 of the catalog's 4,522**. The only holdout is `nanni`: one card (Xanathar, Guild Kingpin), no parent, no children, and no derivable meaning, so any copy would be invention. It stays blank on purpose, and the nightly coverage line is what will surface it. This also removes the last of Scryfall's copy from the catalog, which matters beyond voice: eleven of its descriptions carried markdown cross-links with relative Tagger URLs (`[dual land](/tags/card/dual-land)`) that would have rendered as literal brackets in-app if the tagger ever correlated one of those tags to a card. No parser or sanitizer was added; the text is simply all ours now.

  Run with the `Workflow` tool via [`../development/runbooks/otag_authoring_workflow.js`](../development/runbooks/otag_authoring_workflow.js) (sonnet drafts, opus verifies against real oracle text): 28 agents, 14 chunks, 33 accurate / 87 minor / 18 wrong, meaning the verify stage overrode 18 drafts it judged inaccurate. **The script needed a fix first, and it is worth knowing before the next batch:** its grounding query inner-joined `card_oracle_tags`, so a tag with no cards of its own came back empty and the drafter had nothing to work from. That was 137 of these 138, because what was left were umbrella nodes (`recursion-land`, `typal-creature`) and cycle roots whose members live on child tags. Grounding is now hierarchy-aware, passing each tag's parents, children, and cards sampled from the tag plus its direct children, which is what let `cycle-fetchland` be written off actual fetchlands. Chunk size moved 7 to 10 to suit the lighter per-tag payload.
- [x] **Otag description coverage line — SHIPPED 2026-07-30.** The nightly sync now logs `oracle tag descriptions: X/Y have one (A authored, B blank)` after the overlay. Its first run corrected the books: **4,348 authored** (not the ~1,100 the bulk-authoring todo claimed), 78 blank, ~157 unauthored total — the authoring project is ~96% done.

- [x] **Zervice least privilege — SHIPPED in full 2026-07-29.** Insert-time token prune (expired + cap) in zerver; zervice CardService-only on minimal `ZerviceConfig` + `.env.zervice`; scoped `zervice` Postgres role live (card catalog + matview ownership only, user tables DENIED — verified, full sync green as the role). Zerver deliberately stays on the permissive `zwipe` owner role (split idea → backlog). Plan archived: [`../archive/zervice_least_privilege.md`](../archive/zervice_least_privilege.md).
- [x] **Zervice dead-man's switch — FULLY SET UP 2026-08-06; confirm the first real ping, then move to overview.** Code half `aa831cf8` (all-steps-ok run GETs `HEALTHCHECK_PING_URL`, 10s timeout, failed ping only warns, unset = silent dev runs); owner half done same day: "Zervice Nightly" check on healthchecks.io (cron `0 4 * * *` UTC, grace 2h, email notify) and the URL placed in `.env.zervice`. The `OnFailure=` email covers "ran and failed"; this covers "never ran at all" (the 2026-08-05 cron ghost's blind spot). Arms itself at the first nightly AFTER the next zerver deploy — confirm "healthcheck ping: ok" in that night's log and a green check. Known wrinkle: the check went live during the 2026-08-06 GitHub Actions outage, so if no deploy lands before the next 04:00 UTC nightly, ONE false "down" email arrives ~06:00 UTC (self-heals the first armed night; owner may pause the check until the deploy instead).
- [x] **Rate-limit copy fragments error dedupe — FIXED 2026-08-05 (`ffeaaffa`).** Every rate limiter now serves stable bucketed copy ("try again in a minute" / "in a few minutes"); the live countdown moved to the `Retry-After` header, so `client_errors` dedupe keys stay whole.
- [x] **Android target API level — DONE 2026-07-24.** vc30 (1.7.3) built and submitted with **targetSdk 36** (runbook `build.md` updated to match), clearing the 2026-08-31 Play deadline. Verify the Play Console warning disappears once vc30 is live, then delete this line.
- [ ] **UUID v4 → v7 everywhere (HORIZON, when time allows).** (owner, 2026-08-05) Prerequisite-ordered: Postgres 16→18 with tests green, THEN a one-time migration regenerating existing IDs as backdated v7s (Scryfall card ids exempt; FK lockstep; share-link + mass-logout landmines), THEN `now_v7()` / `DEFAULT uuidv7()` at the mint sites. Plan: [`../plans/uuid_v7_migration.md`](../plans/uuid_v7_migration.md).
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `ff8f4b13`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
