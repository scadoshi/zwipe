# Todo

**Primary goal: grow the user base — marketing + tester-feedback intake.** (iOS App Store: LIVE. Android Play Store: LIVE (production). Full webapp at zwipe.net: in progress.)

Only open, actionable items live here. When something ships, its outcome moves to
[`overview.md`](overview.md) and leaves this list. Older completed work is archived
at `context/archive/complete_2026_q1.md`.

---

## Next Up

- [ ] **1.7.5 rollout → version gate → Phase 5 cleanup.** In review at both stores (submitted 2026-08-05, iOS build 70 / Android vc32). Once approved AND fully rolled out (Play staged rollout to 100%): raise `MIN_CLIENT_VERSION=1.7.5` on the server, watch a quiet day or two for stray PUT / legacy-Opdate traffic (`zcripts/errors.sql` is the canary), then the one-commit Phase 5 cleanup: drop the seven PUT registrations, the delta types, the legacy Opdate decode arms, and make explicit `null` on non-clearable fields a 422 (owner-decided). Closes [`../plans/patch_idempotent_updates.md`](../plans/patch_idempotent_updates.md) and [`../plans/cut_1_7_5.md`](../plans/cut_1_7_5.md).

- [ ] **Client error + crash reporting — final step: prod verification once 1.7.5 is live.** Server half deployed 2026-07-29; the client half (all 73 toast sites + panic-hook crash capture) is IN REVIEW as 1.7.5; data-safety labels were already updated with 1.7.4 (1.7.5 adds no new collection). Remaining: (1) confirm the next 04:0x UTC nightly ran 5/5 — doubling as the single-banner check after the 2026-08-05 cron-ghost removal (`grep -c "zervice running v" $LOG_DIR/zervice.YYYY-MM-DD.log` = 1); (2) once 1.7.5 is live, the prod black-box checks per the plan's Verification section (force a 422 → `client_errors` row; debug panic → exactly one `crash_reports` row across two relaunches). Plan: [`../plans/client-error-reporting.md`](../plans/client-error-reporting.md). Note: the pipeline already scored its first real catch on 1.7.4 data (the ndk-context crash, Bugs below).

- [ ] **Phase 6 — serve on the matured otag signal (data-gated, months out).** The prerequisite **Phase 5S step-3 cleanup shipped 2026-07-25** (legacy `commander_oracle_id` wire + server fallback + client commander resolution all dropped; deck_id is the sole signal key). Server half deploys on next push; the client half rides the next client build. Re-run the pair-depth readiness queries as the user base grows.
- [ ] **Read the anonymous funnel once data accrues.** Anonymous funnel metrics (app-open / register-viewed / register-submitted) have shipped in prod since 1.3.1. When enough sessions accrue, read the funnel with `zcripts` (distinct sessions per kind vs. `user_events.register`) — these numbers gate the sign-in-with-Google decision.
- [ ] **Suggestion signal — Phase 3c (pair-level ranking).** Phases 3a+3b **shipped 2026-07-06** (server 1.3.2): the default synergy ordering now blends base score + pooled net-rate (`added + 0.5·maybed − removed`, shrunk/centered) + per-(deck, day) seeded jitter — different decks serve differently, the same deck stays stable within a day, and crowd favorites drift up as signal accrues. Remaining: the commander-specific pair-level term, gated on pair-depth (baseline 2026-07-06: 0 pairs ≥20 impressions — re-run the readiness queries after the user base grows). Plan: [`../plans/archive/suggestion_signal.md`](../plans/archive/suggestion_signal.md).

---

## Bugs

- [x] **Android: crash on resume after backgrounding — FIXED 2026-08-05, rides 1.7.6.** (ndk-context double-init, field data 2026-08-02/03 on 1.7.4 — the crash reporter's first real catch: 10 rows, one panic site, one user's app dying on every resume for two days.) Root cause confirmed: the OS destroys the Activity but keeps the process (also happens after a root-screen `finish()`); the next `onCreate` re-runs wry's native init and trips `ndk-context`'s already-initialized assert. **Fix: `MainActivity.onDestroy` now kills the process** (in `zcripts/android/back_handler.sh`'s template, applied at every Android bundle) — every reopen is a clean cold start instead of a crash. Upstream research (2026-08-05): tao 0.34.4+ emits `WindowEvent::Destroyed` and by 0.34.8 releases the ndk context on destroy — but dioxus has no webview-recreation path, so a `cargo update -p tao` alone would trade the panic for undefined recreation behavior; the process-kill stays correct either way. Owner call 2026-08-06: no pre-cut device test — the fix is trusted as built (test-device access is a hassle, and a follow-up version ships fast if the field says otherwise); the crash-reporter rows for this panic site are the post-release verify. Likely the Android sibling of the iOS backgrounding-unresponsive bug below (same trigger; iOS wedges instead of panicking — the process-kill does NOT cover iOS).

- [ ] **App unresponsive after long backgrounding (owner report 2026-07-30, iOS observed).** Leave the app backgrounded for a long time, return: sometimes the ENTIRE screen is unclickable until force-close + relaunch. Investigation leads, none confirmed: (a) a full-screen element left mounted and intercepting taps (modal backdrop, toast container, an overlay whose dismiss never fired); (b) the WebView's JS event bridge dying after OS memory pressure while the rendered page survives (wry/dioxus eval channel); (c) something in the resume path (visibility flusher, session refresh single-flight) wedging the main loop. Repro is intermittent — next occurrence, note which screen it happened on and whether scrolling still works (scroll-works-but-taps-don't points to (b)); the new crash/error reporting won't catch this class (no panic, no error toast).

- [x] **Android: card swipe animation clips back to start — FIXED 2026-07-30 (`e480fcae`), rides 1.7.5.** Root cause was two-layered: the exit keyframes hardcoded `from { translate(0,0) }`, and the release position was destroyed (`reset()`) before the exit overlay was even created. Now `SwipeState.release_delta` is captured pre-reset and seeds the keyframes via `--exit-from-x/y/rot` custom properties. Same commit: swipe thresholds raised (60→90px, flick 1.5→2.5 px/ms, flick travel minimum 10→32px) and an easeOutBack return-to-center bounce. Shipped in 1.7.5 (submitted 2026-08-05); delete this line once the store build is confirmed good on-device.

Recently resolved (outcomes in [`overview.md`](overview.md)):
the **filter-sheet Reset/Cancel commit bug** (fixed 2026-07-22 `db5562b4` with a
current/staged filter split; Apply is the only commit), the **share-page mana-value
group ordering** (fixed 2026-07-22 `bfc10309`, contiguous column partition), the
**pre-1.6.0 "connection error" wire break** (fixed by flooring `MIN_CLIENT_VERSION=1.6.0`,
2026-07-13; root cause fully removed 2026-07-14 when the Phase M sunset dropped the
`mechanical_categories` dual-emit), and **app version in session data** (shipped `ce8abcad`,
recorded per-session on the refresh-token row).

Completed fixes are archived to
[`archive/complete_2026_q3.md`](../archive/complete_2026_q3.md) (hashes stay searchable there).

---

## Features — queued (owner 2026-07-11)

- [x] **Global undo — DONE 2026-08-05 (`a7763093`), device-verified same day, rides 1.7.6.** One per-deck mutation stack across every deck-mutation door (deck cards, quick add, add/remove screen swipes, promotes, board moves, command-zone printing swaps); gesture undo reconciles through the store so the two undo paths can't double-reverse; import clears the stack (disclosed in the import hint). All nine run-sheet scenarios passed on a real phone against prod. Plan archived: [`../plans/archive/global_undo.md`](../plans/archive/global_undo.md). Move to overview when 1.7.6 ships.
- [x] **Featured flavor — server + zwipe.net halves DONE, prod-verified 2026-08-06; the app half rides 1.7.6. Move to overview when 1.7.6 ships.** The zite element lives under the home demo gallery; the two bugs that plagued it are root-caused and fixed (full story in `overview.md` 2026-08-06): the trapped overlay was DOM nesting (hoisted to the page top level per the shared-deck pattern), and the never-rotating card was CF edge caching (origin now sends `Cache-Control` counting down to the UTC hour, `bb7266b1`; owner confirmed the CF rules respect origin TTL, purged the edge cache post-deploy, rotation verified). Late same day the CF cache rule was hardened further (owner): Edge TTL = "use cache-control header if present, BYPASS if not" and Browser TTL = respect origin — origin is now the sole caching authority, so unheadered endpoints (`/api/client/min-version`, anything future) bypass instead of inheriting a default TTL; the whole silently-cached class is dead. Nothing on the rule caches until the pending zerver deploy ships the headers. Spoof fallback is debug-only (`2198b2a1`). Plan archived: [`../plans/archive/featured_flavor.md`](../plans/archive/featured_flavor.md).
- [x] **Clear (×) button on every floating-results search bar — DONE 2026-08-05, rides 1.7.6.** All 14 inputs across the 7 search-float hosts (filter bars' includes + excludes, all four command-zone pickers) got the show-when-non-empty × that clears the text and closes the results. Plan archived: [`../plans/archive/search_bar_clear_buttons.md`](../plans/archive/search_bar_clear_buttons.md).

- [x] **Quick add should search past skips.** (user report — Collin, 2026-08-10) Skipping a card while swiping suppresses it from ALL deck-aware search, including quick add's name search, so a skipped card comes up "no results" in that deck until Clear skips. Owner call: typing a card's name into quick add is explicit intent — quick add should return skipped cards; the swipe pile keeps respecting suppressions (Clear skips stays the pile's escape hatch). **BUILT 2026-08-10, both halves.** Server: `CardQuery.include_skipped` (serde-default false, synergy-flag pattern) + builder setter/getter, and the suppression `NOT EXISTS` clause in the deck-aware search now skips when the flag is set — deploys with next push, harmless until a client sends it. Client: `quick_add.rs` sets the flag (rides 1.7.6; against pre-flag prod servers the field is ignored and skips stay hidden, no error). Note the flag also resurfaces REMOVED cards (suppressions cover both) — owner-accepted. No unskip-on-add step: removals suppress too, so a stale skip row matches existing semantics. Move to overview when 1.7.6 ships.
- [ ] **Commander shortlist / dedicated commander-swiping area** — "save for later" while swiping commanders. **Feature request** ([`../plans/commander_shortlist.md`](../plans/commander_shortlist.md)): recommend a dedicated Commanders browse space with a per-user shortlist + "start a deck with this," decoupled from the deck-creation picker (kills the "where did it go?" of an in-flow up-swipe). Open decisions: storage (server vs local), placement, commander scope. Not specced.
- [x] **Deck list group + filter — BUILT 2026-08-11, rides 1.7.7.** The small version of deck folders (owner call: folders PARKED, may never build — plan stays at [`../plans/deck_folders.md`](../plans/deck_folders.md) if demand appears). Deck list gains the deck-cards chip-row grammar: single-select "Group by:" (None/Format/Color/Tag — tag grouping shows a deck under each of its tags, color sections render identity pips) and a "Show:" filter row seeded from the decks themselves (All + color pips with contains-all semantics + deck/descriptive tag chips, OR within tags). Client-only over the already-loaded profiles; ephemeral per visit (20-deck cap makes persistence pointless). If folders ever get built, the section rendering here is most of their UI.
- [ ] ~~**Deck folders**~~ — **PARKED indefinitely (owner, 2026-08-11: "might never build it")**; the group/filter row above covers the organize-my-list need at the 20-deck cap. Spec preserved at [`../plans/deck_folders.md`](../plans/deck_folders.md); freeform folder names are the one thing filters can't express, so revisit only on real user demand.
- [ ] **Oracle tags (otags) — HORIZON, big.** Ingest Scryfall's community-maintained functional tags (hundreds; daily `zervice` sync → `card_otags`), let players select strategy otags per deck (reconciled with deck tags), show the distribution, and use them as a new algorithmic serving axis (commander + otags, MVP otags, non-EDH formats via color-identity + otags + swipe data). Community-accurate replacement/complement for our heuristic `mechanical_categories`. Full vision + open research questions in [`../plans/otags.md`](../plans/otags.md).
- [ ] **Card images on the deck cards screen — minimum SHIPPED 2026-07-30 (rides 1.7.5): `FeaturedCards` strip (command zone role-labeled + MVPs in star order, tap opens the image preview overlay) at the top of the deck cards list, mirroring the zite share row via the shared `FlippableCardImage`. Owner moved it from the deck profile screen to the cards screen.** Still open, deliberately deferred: images on expanded card rows, or an always-show-images option for the whole list — real design decision, not a drive-by.
- [ ] **Deck share screen: add charts (investigate).** (owner, 2026-07-22) Bring the deck stats charts (distributions etc.) onto the zwipe.net deck share screen to make it more capable. Not specced.
- [ ] **Deck composition targets (big, needs MVP scoping).** (owner, Discord, 2026-07-23) Like the land target, but for everything a card can be: set per-deck target counts for a card role (10 ramp), a card type (30 creatures), or cards matching an arbitrary filter, with goal-vs-actual display. Needs functionality + MVP scoping before any build: storage shape (deck_profile vs new table), which axes ship first (roles/types before filter-based), editing UI, and whether targets feed warnings. Skipped for now by owner call.
- [ ] **Mana pip-count filter (investigate).** (owner idea, Discord, 2026-07-21) Let players filter the card pool by the exact count of colored pips of a given color in the mana cost — e.g. "has 2 blue pips and 1 red pip." Per-color pip counts are derivable from `mana_cost`. Open: UI (per-color count steppers? which colors shown?), match semantics (exact vs min/max), and whether it stacks with the existing color/mana filters. Not specced.

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

- [ ] **Discord testimonials — parked pending permission asks (owner call 2026-08-10).** The site's review panel now carries "Shipped in x.y.z" receipt tags; two Discord quotes would strengthen it if their authors consent (ask in-channel first — Discord isn't published the way App Store reviews are): Ebany's 500-card verification ("no discernable performance change... handles 500 cards quite well" → Shipped in 1.7.0, request-to-ship same day) and hopeful_boy's beginner angle ("I've loved the app so far as a beginner deck builder", no tag). Mark provenance ("From the Discord") if added. Future: the companion-slot thread becomes a tagged Caed quote the day companions ship.
- [ ] **Marketing posts — keep the cadence going.**
  - **Wave 1 — 2026-06-25 (Day 333):** build-in-public text posts to X, Reddit, Bluesky (1.1.0 features).
  - **Wave 2 — 2026-06-28 (Day 336): the swipe-demo video.** A vertical 9:16 screen-recording (Tinder-for-Magic hook: swipe a commander → build by swiping → filter → import → finished deck), captions + an end card built from `context/marketing/video_end_card.html`. Posted natively to **Instagram** (new `@scadoshi`), **YouTube Shorts**, **X**, **Bluesky**, and **Reddit r/EDH** (video + pinned dev comment, feedback-framed). **TikTok pending** — the old 8-yr personal account was compromised (password breach; rogue sessions cleared, no third-party apps, recovering via phone+email, needs new pw + 2FA before use); lean toward a fresh branded handle. Post-copy templates were drafted then deleted (kept out of the repo).
  - r/EDH and r/magicTCG *posts* are karma/age-gated (comment first, build toward posting); reuse the r/EDH title + pinned comment for r/custommagic, r/SideProject, r/buildinpublic.
- **Business cards — reference for reprints** (task done; card at `context/marketing/business_card.html`, `cebe5d91`). Standard playing-card stock is **2.5×3.5in (63×88mm)**. For a true card feel use a card printer (MakePlayingCards/MPC, PrinterStudio, Ad Magic, or a local shop with 300–350gsm + matte/linen) — order a custom card / "custom card game" SKU. Export: open the HTML, print to PDF at 100% (no scaling), "background graphics" ON; add 1/8in bleed if the printer wants it. QR holds down to ~0.8in (current is well above).

---

## Monetization

- [ ] **TCGplayer affiliate** — application submitted 2026-06-23, **In Review** on Impact (impact.com). When approved: wire the tracking ID into `tcgplayer_url()` (`zwiper/.../outbound/buy_links.rs`) — zero UI change — then add per-card **"Buy ↗"** links (currently buy is whole-deck only).
- [ ] **Card Kingdom affiliate** — **no public self-serve program**; it's a direct-outreach partnership. Email CK when ready (cite the Archidekt `?partner=` precedent). `cardkingdom_url()` stays untracked until then.
- Detail + saved signup copy: `context/product/affiliate/tcgplayer.md`.

---

## Web/Zite Polish

- [ ] **About page (`/about`) visual overhaul — larger redesign wanted.** A partial alignment pass landed 2026-07-21 (`51f69d72`, `a007ac9f`, `c33bf479`): five-crate diagram with `zwipe-components`, the enrichment card rewritten for community oracle tags (roles derived from otag subtrees, not the retired heuristic), refreshed test counts, and the whole thing brought toward the app's tag/chip grammar (colored theme chips cycling accent 1–3 / success / warning / error, single-line wrapping header, tech stacks as chip rows, linkified imports). Owner still wants a fuller visual redesign of this section. Open bits from the pass: single-label subtitles (Scryfall "external service", PostgreSQL "primary datastore", the two foundation bands) → chips for full consistency; and the tagline comma (moot now the tagline is gone).
- [x] **Favicon with a background color — REGENERATED 2026-08-05, deploys with next push.** All six assets (`favicon.ico` 16/32/48/64, `favicon-16x16/32x32.png`, `icon-180/192/512.png`) recomposited from `zite/assets/favicon-no-background/` onto solid `#282828` (the Android adaptive-launcher bg). Google recrawls favicons on its own schedule — check the "zwipe" SERP icon in ~a week, then delete this line.
- [x] **Contribute page: mirror the portfolio site's version — DONE 2026-08-05, deploys next push.** zite's `/contribute` rebuilt on the shared `zwipe_components::Panel` cards (the delta vs the portfolio was hand-rolled divs vs Panels — same three options/URLs already matched), portfolio card copy adopted, Zwipe-specific intro kept. Delete this line after a look at the deployed page.
- [ ] **Keep zwipe.net in sync as the app grows.** The guides knowledge base shipped (12 guides under `/guides`, sitemap + per-guide `Article` JSON-LD landed 2026-07-08). No committed appetite for the demand-first SEO guides ("best mobile MTG deck builder", etc.) — leave them optional. The standing task is just to update the site (guides, feature pages, screenshots) as the app becomes more feature-rich. (SEO-guides plan archived at [`../archive/seo_guides.md`](../archive/seo_guides.md).)

---

## Synergy & Popularity Data

The cache-first synergy layer shipped (see `overview.md`); these are the consumers built on top of it.

- [ ] Synergy scores — surface cards with high synergy to the deck's commander
- [ ] Popularity data — most-played cards for a given commander
- [ ] Salt score, display per card and aggregate per deck, filtering and sorting on card search
- [ ] Evaluate further data (themes, combos, etc.) as the layer matures

---

## Maintenance

- [ ] **Orphaned otag-description slugs — PARKED, now at 12 (was 9; recounted 2026-08-06 against the live tagger); clean when the list reaches ~dozens.** Nightly zervice WARNs: `ORACLE_TAG_DESCRIPTIONS references unknown oracle-tag slugs` — the original nine (`personal-text`, `synergy-sacrifice`, `variable-effect-same-ability`, `serpent-like`, `static-effect-in-graveyard`, `relaxed-commander-restriction`, `cycle-clb-monument`, `hate-phasing`, `synergy-phasing`) plus tagger churn since: `hate-sacrifice`, `hunter-trigger`, and `tron` (authored 2026-08-06 in the closeout batch, then found retired from the live tagger the same day — its text stays in the const in case a successor slug appears). `ROLE_TAG_OVERRIDES` still references `synergy-sacrifice`. Benign — orphaned entries match nothing. Cleanup procedure when triggered: check the Tagger for each slug's successor (renames get their authored text moved; retired tags get deleted), and remap `synergy-sacrifice` in `ROLE_TAG_OVERRIDES` (`outbound/sqlx/card/helpers/derive_categories.rs`) the same way. Descriptions live in `outbound/sqlx/card/helpers/oracle_tag_descriptions.rs`; runbook at `development/runbooks/`.
- [x] **Otag description coverage line — SHIPPED 2026-07-30.** The nightly sync now logs `oracle tag descriptions: X/Y have one (A authored, B blank)` after the overlay. Its first run corrected the books: **4,348 authored** (not the ~1,100 the bulk-authoring todo claimed), 78 blank, ~157 unauthored total — the authoring project is ~96% done.

- [x] **Zervice least privilege — SHIPPED in full 2026-07-29.** Insert-time token prune (expired + cap) in zerver; zervice CardService-only on minimal `ZerviceConfig` + `.env.zervice`; scoped `zervice` Postgres role live (card catalog + matview ownership only, user tables DENIED — verified, full sync green as the role). Zerver deliberately stays on the permissive `zwipe` owner role (split idea → backlog). Plan archived: [`../archive/zervice_least_privilege.md`](../archive/zervice_least_privilege.md).
- [x] **Zervice dead-man's switch — FULLY SET UP 2026-08-06; confirm the first real ping, then move to overview.** Code half `1f613c57` (all-steps-ok run GETs `HEALTHCHECK_PING_URL`, 10s timeout, failed ping only warns, unset = silent dev runs); owner half done same day: "Zervice Nightly" check on healthchecks.io (cron `0 4 * * *` UTC, grace 2h, email notify) and the URL placed in `.env.zervice`. The `OnFailure=` email covers "ran and failed"; this covers "never ran at all" (the 2026-08-05 cron ghost's blind spot). Arms itself at the first nightly AFTER the next zerver deploy — confirm "healthcheck ping: ok" in that night's log and a green check. Known wrinkle: the check went live during the 2026-08-06 GitHub Actions outage, so if no deploy lands before the next 04:00 UTC nightly, ONE false "down" email arrives ~06:00 UTC (self-heals the first armed night; owner may pause the check until the deploy instead).
- [x] **Rate-limit copy fragments error dedupe — FIXED 2026-08-05 (`f43bdd3a`).** Every rate limiter now serves stable bucketed copy ("try again in a minute" / "in a few minutes"); the live countdown moved to the `Retry-After` header, so `client_errors` dedupe keys stay whole.
- [x] **Android target API level — DONE 2026-07-24.** vc30 (1.7.3) built and submitted with **targetSdk 36** (runbook `build.md` updated to match), clearing the 2026-08-31 Play deadline. Verify the Play Console warning disappears once vc30 is live, then delete this line.
- [ ] **UUID v4 → v7 everywhere (HORIZON, when time allows).** (owner, 2026-08-05) Prerequisite-ordered: Postgres 16→18 with tests green, THEN a one-time migration regenerating existing IDs as backdated v7s (Scryfall card ids exempt; FK lockstep; share-link + mass-logout landmines), THEN `now_v7()` / `DEFAULT uuidv7()` at the mint sites. Plan: [`../plans/uuid_v7_migration.md`](../plans/uuid_v7_migration.md).
- [x] **keyring 3 → 4 — DONE 2026-08-06 (`a499ab27`), rides 1.7.6.** 4.x split the crate into `keyring-core` + per-store crates; our low-level `default_credential_builder()` path was dropped, so `session.rs` + `theme_store.rs` moved to the wrapper's `Entry` API (byte secrets kept). The logout risk was retired properly: a scratch binary linking BOTH versions (write with 3, read with 4, and reverse) proved on the real macOS keychain that the (service, account) item mapping is identical — existing sessions survive the upgrade. Optional device sanity on the next dev build: sign in on the current store build, install a 1.7.6 dev build over it, session should persist. Delete this line when 1.7.6 ships.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
