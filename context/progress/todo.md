# Todo

**Primary goal: grow the user base — marketing + tester-feedback intake.** (iOS App Store: LIVE. Android Play Store: LIVE (production). Full webapp at zwipe.net: in progress.)

Only open, actionable items live here. When something ships, its outcome moves to
[`overview.md`](overview.md) and leaves this list. Older completed work is archived
at `context/archive/complete_2026_q1.md`.

---

## Next Up

- [ ] **Post-Phase-5 canary watch (through ~2026-08-14).** The PATCH-only server merged + deployed 2026-08-12 (PR #24) with the version gate at 1.7.5. Watch `zcripts/metrics/errors.sql` for a day or two: a stray PUT from a not-yet-relaunched old client now surfaces as a 405 client-error row (expected to be zero at current DAU). Full outcome in `overview.md` 2026-08-12; plans archived (`../plans/archive/patch_idempotent_updates.md`, `../plans/archive/cut_1_7_5.md`).

- [ ] **Client error + crash reporting — final step: prod verification once 1.7.5 is live.** Server half deployed 2026-07-29; the client half (all 73 toast sites + panic-hook crash capture) is IN REVIEW as 1.7.5; data-safety labels were already updated with 1.7.4 (1.7.5 adds no new collection). Remaining: (1) confirm the next 04:0x UTC nightly ran 5/5 — doubling as the single-banner check after the 2026-08-05 cron-ghost removal (`grep -c "zervice running v" $LOG_DIR/zervice.YYYY-MM-DD.log` = 1); (2) once 1.7.5 is live, the prod black-box checks per the plan's Verification section (force a 422 → `client_errors` row; debug panic → exactly one `crash_reports` row across two relaunches). Plan: [`../plans/client-error-reporting.md`](../plans/client-error-reporting.md). Note: the pipeline already scored its first real catch on 1.7.4 data (the ndk-context crash, Bugs below).

- [ ] **Phase 6 — serve on the matured otag signal (data-gated, months out).** The prerequisite **Phase 5S step-3 cleanup shipped 2026-07-25** (legacy `commander_oracle_id` wire + server fallback + client commander resolution all dropped; deck_id is the sole signal key). Server half deploys on next push; the client half rides the next client build. Re-run the pair-depth readiness queries as the user base grows.
- [ ] **Read the anonymous funnel once data accrues.** Anonymous funnel metrics (app-open / register-viewed / register-submitted) have shipped in prod since 1.3.1. When enough sessions accrue, read the funnel with `zcripts` (distinct sessions per kind vs. `user_events.register`) — these numbers gate the sign-in-with-Google decision.
- [ ] **Suggestion signal — Phase 3c (pair-level ranking).** Phases 3a+3b **shipped 2026-07-06** (server 1.3.2): the default synergy ordering now blends base score + pooled net-rate (`added + 0.5·maybed − removed`, shrunk/centered) + per-(deck, day) seeded jitter — different decks serve differently, the same deck stays stable within a day, and crowd favorites drift up as signal accrues. Remaining: the commander-specific pair-level term, gated on pair-depth (baseline 2026-07-06: 0 pairs ≥20 impressions — re-run the readiness queries after the user base grows). Plan: [`../plans/archive/suggestion_signal.md`](../plans/archive/suggestion_signal.md).

---

## Bugs

- [ ] **Arcane Signet name-search miss — FIXED 2026-08-12, awaiting prod deploy + zervice re-run (owner).** Root cause: `latest_cards`' dedup never considered language, so a newer foreign printing shadowed every English one and the default `language=en` filter hid the card from search entirely — 268 cards affected (hoc's Dwarvish spoilers, soa's Japanese-only reprints). Migration `20260812210000_latest_cards_prefer_english.sql` adds English preference to the dedup ordering + remaps deck references off foreign printing ids (June's rebuild pattern); verified locally 268 → 0. **Standing guard for this bug class** (a dedup dimension silently shadowing cards): after any odd "card missing from search" report, run `SELECT count(*) FROM latest_cards lc WHERE lc.lang <> 'en' AND EXISTS (SELECT 1 FROM scryfall_data sd WHERE sd.oracle_id = lc.oracle_id AND sd.lang = 'en')` (should be 0) and eyeball the reported card's `latest_cards` row for which dedup dimension went wrong. Delete this line once deployed + spot-checked on prod.

- [ ] **Verify the Android resume-crash fix in the field (1.7.6+ live since 2026-08-10).** The onDestroy process-kill shipped untested-on-device by owner call; the verify is the crash reporter — the ndk-context panic site should be silent for vc33+ sessions (it fired 10 rows on 1.7.4). Check after a week of 1.7.6+ adoption; full history in `overview.md` 2026-08-12. Note: the iOS backgrounding-unresponsive bug below is likely the same trigger and is NOT covered by this fix.

- [ ] **App unresponsive after long backgrounding (owner report 2026-07-30, iOS observed).** Leave the app backgrounded for a long time, return: sometimes the ENTIRE screen is unclickable until force-close + relaunch. Investigation leads, none confirmed: (a) a full-screen element left mounted and intercepting taps (modal backdrop, toast container, an overlay whose dismiss never fired); (b) the WebView's JS event bridge dying after OS memory pressure while the rendered page survives (wry/dioxus eval channel); (c) something in the resume path (visibility flusher, session refresh single-flight) wedging the main loop. Repro is intermittent — next occurrence, note which screen it happened on and whether scrolling still works (scroll-works-but-taps-don't points to (b)); the new crash/error reporting won't catch this class (no panic, no error toast).

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

- [ ] **Share page: featured commander/MVP cards deal in like the app (owner 2026-08-13).** Replace the featured strip's skeleton ghosts on zwipe.net's shared deck page with the app's entrance instead: cards animate in as they load, using the deck cards screen's `deck-featured-deal` idiom (dealt from above with rotate + nth-child stagger, `backwards` fill, the ~0.65s cubic-bezier ease in `zwiper/assets/main.css`). Port the keyframes to `zite/assets/style.css` and drop the `sd-featured` skeleton block in `zite/src/pages/shared_deck.rs` (the rest of the page's skeleton stays). zite-only, no train dependency — can deploy anytime.

- [x] **Collapsible deck cards groups — BUILT 2026-08-12, rides 1.8.1.** Every list section (group-by groups, Tokens, Lands, Maybeboard, Sideboard) gains a tappable header with the card-row disclosure arrow (down open, sideways collapsed); rows stay mounted and hide via CSS so expanded-card state survives. Command-zone single-row headers deliberately stay static. Ephemeral per visit. HR-in-groups fix rode the same day (deck list headers drop border-bottom via `.deck-group-header`).
- [x] **Deck list + deck cards skeletons — UPDATED 2026-08-12, rides 1.8.1.** Both now mirror the live layouts: deck list gains Group by / Show chip-row ghosts; deck cards gains identity header + tag chips, three featured-card image ghosts, quick add bar, chip rows, and art thumbnails on every row ghost.
- [ ] **Commander maybeboard — SHIPPED with 1.9.0, SUBMITTED both stores 2026-08-13 (iOS build 74 / Android vc36). Delete this line when 1.9.0 is live on both stores and the maybeboard works against prod.** Per-user "maybe this commander" list, fully committed and reviewed (server: migration + 4 endpoints, 4 integration tests; client: the full commander-hub screen — details in the spec and overview). Server + zite halves need the owner's push/deploy BEFORE the store builds go live, or the app's maybeboard calls 404. Full spec: [`../plans/commander_maybeboard.md`](../plans/commander_maybeboard.md) — archive it alongside deleting this line.
- [x] **Deck list group + filter — BUILT 2026-08-11, ships in 1.8.1 (1.8.0's build 72/vc34 submissions were superseded before release; 1.8.1 build 73/vc35 in review at both stores since 2026-08-12 night — move to overview when it ships, together with the two entries below).** Group by (None/Format/Color/Tag sections) + Show filter row (color pips, tag chips) on the deck list, plus the one-time HINT_DECK_LIST tip. First train under the any-feature-bumps-minor convention (`development/versioning.md`). If folders ever get built, the section rendering here is most of their UI.
- [ ] ~~**Deck folders**~~ — **PARKED indefinitely (owner, 2026-08-11: "might never build it")**; the group/filter row above covers the organize-my-list need at the 20-deck cap. Spec preserved at [`../plans/deck_folders.md`](../plans/deck_folders.md); freeform folder names are the one thing filters can't express, so revisit only on real user demand.
- [ ] **Oracle tags (otags) — HORIZON, big.** Ingest Scryfall's community-maintained functional tags (hundreds; daily `zervice` sync → `card_otags`), let players select strategy otags per deck (reconciled with deck tags), show the distribution, and use them as a new algorithmic serving axis (commander + otags, MVP otags, non-EDH formats via color-identity + otags + swipe data). Community-accurate replacement/complement for our heuristic `mechanical_categories`. Full vision + open research questions in [`../plans/otags.md`](../plans/otags.md).
- [ ] **Card images on the deck cards screen — minimum SHIPPED 2026-07-30 (rides 1.7.5): `FeaturedCards` strip (command zone role-labeled + MVPs in star order, tap opens the image preview overlay) at the top of the deck cards list, mirroring the zite share row via the shared `FlippableCardImage`. Owner moved it from the deck profile screen to the cards screen.** Still open, deliberately deferred: images on expanded card rows, or an always-show-images option for the whole list — real design decision, not a drive-by.
- [x] **Deck share page: full app sections — BUILT + owner-reviewed 2026-08-13 (needs a server deploy for the target fields; otherwise zite-only).** The share page carries all five deck-view sections (Budget with currency chips + price target, Tags incl. oracle tags, Distributions with Avg P/T, Mana with land target + curve + fulfillment, interactive Draw odds) as collapsible panels between the featured cards and the controls — collapsed by default, several per row. Card groups collapse too (shared `SdCollapsibleGroup`, default open). Whole chart family now shared: `zwipe_components::{DeckCharts, ManaCurve, ManaFulfillment, DrawOdds, ChartLabel}` fed by `DeckMetrics` chart methods (bar/balance/draw-odds math hoisted from the app's view, which deduped onto them). `HttpSharedDeck` gained serde-defaulted `land_target`/`price_target`/`price_target_currency`. Delete this line after eyeballing a deployed share page with targets showing.
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
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
