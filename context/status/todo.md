# Todo

**Primary goal: Ship Zwipe as a webapp at zwipe.net. (iOS App Store: LIVE as of 2026-06-06. Android: next.)**

Completed work archived at `context/archive/complete-2026-Q1.md` (swept 2026-05-27).

---

## Next Up

- [ ] **Synergy data layer (cache-first)** — PRIORITIZED 2026-06-10: build right after the Android closed-test clock starts (fills the 14-day wait). Lazy per-commander fill on first demand via a queue table, Postgres cache, weekly background refresh, graceful degradation to cached data. Consumed as an internal ranking signal (smart stack / suggestions). **The worker is BUILT and verified (2026-06-11)** — zwipe-side contract (one migration + read path, zerver writes nothing): `context/plans/synergy-data-layer.md`. Product framing: `context/product/premium/smart-stack.md`. *AI assistants: check local memory for additional context on this item before starting; if nothing is found, ask Scotty.*
- [x] **Moxfield import (phase 2) — DENIED, shelved** (2026-06-10) — Moxfield support (Rob) replied same day: policy is no API access for sites/apps that offer deckbuilding. They plan a scoped deck-export endpoint for such services eventually (no ETA; will be announced on their help pages) and invited a re-request once it's live. Until then Moxfield users use the text-paste importer. Watch item moved to `backlog.md`; see `context/plans/deck-import.md`.
- [ ] **Backup restore drill** (written 2026-06-10, not yet run) — prove the nightly R2 dumps actually restore: pull latest dump, restore into a scratch DB, verify counts/freshness, optionally boot zerver against it. ~20 minutes, zero prod risk, runnable on the Mac. Full steps + checkpoints: `context/plans/backup-restore-drill.md`.
- [ ] **Migrate prod off the home server to a VPS** (planned 2026-06-10, DEFERRED — at ~20 users the home server is fine; revisit when users grow meaningfully, premium revenue starts, or the home server's retirement becomes real). Ready-to-run plan with options, phased tunnel-CNAME cutover, rollback, and decommission checklist: `context/plans/vps-migration.md`.
- [ ] **Track 1.0.5 (build 31) App Store review + propagation** — submitted 2026-06-10; carries Archidekt import and the min-version gate. Once approved and propagated, every install ≥1.0.5 is force-updatable via `MIN_CLIENT_VERSION`.
- [ ] **Deck-aware card search → client adoption** — server side BUILT + live-tested 2026-06-11 on `feat/synergy-data-layer`: `POST /api/deck/{deck_id}/card/search` excludes in-deck cards (all boards + profile slots) and defaults to synergy ordering when no explicit sort; old `/api/card/search` untouched for existing clients. Remaining: zwiper add-cards screen consumes the new endpoint and auto-serves on open (see `context/plans/synergy-data-layer.md`).
- [x] **Missing-auth responses return 500, should be 401** (fixed 2026-06-11) — the 500 came from the user-id-keyed `tower_governor` layer (it runs before the auth extractor; `UnableToExtractKey` mapped to 500 by default). Added an `error_handler` remapping that one case to 401 on all four user-keyed limiters, delegating 429/everything else to the library default; also moved the `AuthenticatedUser` extractor's missing-header rejection 400→401. Live-tested: unauth private routes → 401, authed → 200, rate limiting still 429.
- [x] **`/health` path drifted** (fixed 2026-06-11) — `GET /health` now runs the combined server+database check (`/health/server` and `/health/database` unchanged); added `health_route()` helper in core paths. `GET /` still returns status+version.
- [ ] **Verification screen: resend throttle + refresh** (noted 2026-06-11; investigated 2026-06-11) — **server throttle is NOT meaningfully in place.** `POST /api/auth/resend-verification` (`routes.rs:315`) requires auth but sits under the broad `private_config` governor (burst 500, then 1/600ms per user) — a DDoS backstop, not an email-abuse cooldown; a logged-in user could fire hundreds of verification emails. Fix: wrap that one route in its own `GovernorLayer` (add the `unauthorized_on_missing_key` error_handler for consistency) keyed per user — a dedicated config like burst 2, then 1 per ~2-3 min (reusing `sensitive_config`'s 1/30min is server-safe but harsh for a legit "didn't get it, resend"). Then the client gets: a resend button with a visible cooldown (UX over the real server limit) and a refresh affordance to re-check verification status without leaving the screen (pure client work, no server change).
- [ ] **Security notification emails on profile changes** (noted 2026-06-10) — email the user when their username, password, or email changes, to surface account takeovers. Email changes notify the OLD address (the new one is what an attacker controls); password/username changes notify the current address. Server-side only via Resend (no client work, no app release). Consider including a "wasn't you? reset your password / revoke sessions" line.
- [x] **Min-version gate client build (1.0.5)** (2026-06-10) — built per `context/plans/min-version-gate.md` and shipped: public `/api/client/min-version` driven by `MIN_CLIENT_VERSION` env (live on prod at `0.0.0` = open), `zwipe_core::version` compare (fails open), 60s client poll, blocking "Update required" screen. In build 31.
- [x] **Deck migration from Archidekt** (2026-06-10) — `POST /api/deck/{deck_id}/import/archidekt` imports an Archidekt URL's cards into an existing deck by Scryfall printing UID (name fallback), identical semantics to text import. Both importers gained Add/Replace `ImportMode` (board-scoped replace) and the import screen got From/Mode/Board chips. Server deployed + smoke-tested on prod; client in build 31. See `context/plans/deck-import.md`.
- [x] **Reach out to recommander.cards dev** (2026-06-10) — sent friendly intro to Michael Celani (GamesfreakSA) via Discord + X, pitching a coffee chat about Zwipe consuming Recommander's recommendation API (good-ergonomics client + good-recs API), with traffic + credit driven back to him. Awaiting reply. See `backlog.md` High Priority for talking points.

---

## Pending Gated Merges — server flips waiting on iOS propagation (~2026-06-23)

Two prepared branches sit on origin, each a server-side flip that would break
older iOS clients. **Merge only after ASC Analytics → App Versions shows the
gating build at ~100% of active installs.** Both can land together once 1.0.4 (build 30)
has propagated (1.0.4 implies Build 25's gate too). 1.0.5 (build 31, submitted
2026-06-10) supersedes 30 and carries the min-version gate — once 1.0.5 is the
floor, stragglers on these flips can be force-updated via `MIN_CLIENT_VERSION`
instead of waited out.

| Branch | What it does | Gate | Plan |
|---|---|---|---|
| `feat/wire-format-rfc3339` | Server emits RFC3339 `Z` timestamps, deletes `wire_time` adapter from zwipe-core | Build 25+ (1.0.3) propagated | `context/plans/timestamptz-migration.md` (phase 2) |
| `feat/refresh-token-hardening` | Strict single-use refresh rotation (`FOR UPDATE` + delete check) | 1.0.4 (build 30, submitted 2026-06-09: single-flight refresh + new icon + toast copy) propagated | `context/plans/refresh-token-hardening.md` |

Per-branch verification and rollback steps live in the linked plans. Merging
either triggers an automatic zerver deploy via CI.

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

## App Store Submission — LIVE 🎉 (2026-06-06)

**Zwipe is live on the App Store as "Zwipe TCG":** https://apps.apple.com/us/app/zwipe-tcg/id6761341603

Build 15 cleared review after the metadata scrub. The path there:
- Rejected 2026-06-03 under Guideline 4.1(a) Copycats — metadata referenced Magic: The Gathering without authorization from Wizards of the Coast. Apple's complaint was scoped to **metadata** (app name, description, keywords, screenshots), not in-app behavior.
- Resubmitted 2026-06-04 after scrub: renamed "Zwipe MTG" → **"Zwipe TCG"**, removed all MTG/Magic/Commander/EDH/Planeswalker/Scryfall references from the listing, reworded as generic "trading card game deck builder". Screenshots still showed MTG card art but it passed anyway.

Details:
- Bundle ID `com.scadoshi.zwipe`, App Store name "Zwipe TCG"
- Distribution certificate + App Store provisioning profile in place
- Export compliance: no encryption beyond HTTPS
- zite: iOS "App Store" nav link now points straight to the live listing; the old pending `/download/ios` page was removed

Standing risk: WotC could still C&D the app for trademark/copyright at any time. Their [Fan Content Policy](https://company.wizards.com/en/legal/fancontentpolicy) explicitly excludes applications. Long-term risk; independent of the App Store.

---

## Post-Launch Polish (Live on App Store)

Momentum work now that Zwipe TCG is live. Roughly ordered by user-visible impact.

### Marketing & Discovery
- [x] **Refresh App Store imagery** (2026-06-06) — screenshots recaptured from build 19 (1.0.1) on iPhone 11 Pro Max simulator (1242×2688px). 12 screenshots in `context/ops/ios/app-store-submission/`.
- [x] **Update App Store icon to gruvbox** (2026-06-07) — new 1024×1024 master, gruvbox tan Z on `#282828`, alpha stripped, sized to 40/60/80/87/120/180/1024. Shipped in 1.0.2 build 21. Repack process documented in `context/ops/ios/appstore-icon-update.md`.
- [ ] **Marketing posts** — Reddit (r/EDH, r/magicTCG, r/CompetitiveEDH) and X. Lead with the swipe demo video, link the App Store listing.
- [ ] **Business cards for LGSs** (noted 2026-06-10) — design + print cards to leave at local game stores. Pixel-art Z branding, App Store link (QR code to https://apps.apple.com/us/app/zwipe-tcg/id6761341603 or zwipe.net), one-line hook ("Swipe right on your next deck" or similar). Ask store owners first; commander night crowds are the target audience.
- [ ] **First-run tutorial toasts** (decided 2026-06-10 — was "Tutorial?", resolved by real user feedback: first questions are "where do I go?", "how do I add cards?", "will it suggest cards?", "how do filters work?") — lightweight, contextual hint toasts surfaced at key moments rather than a guided flow: on first swipe screen "Swipe right to add, left to skip, up for maybeboard"; on remove screen "Swipe right to remove"; first deck view points at filters; etc. Dismissible, never block input, reuse the existing toast layer. Persistence: a **`hints_shown` jsonb on the user profile** — generic and flexible by design, e.g. `{"swipe_basics": true, "remove_swipe": true}`. Client checks the map before showing a hint and reports back when one is shown; new hints are added dynamically by introducing a new key, no schema migration, no new columns, and old clients ignore keys they don't know (additive per `context/dev/api_evolution.md`). Server-side so it survives reinstalls and follows the account across devices.

### Web/Zite Polish
- [ ] **Increase `Z` logo size on zwipe.net** — current ASCII logo reads small; bump scale or font-size on the landing hero.
- [ ] **Mobile testing pass on zwipe.net** — verify landing, about, privacy, verify/reset token pages on iOS Safari + Android Chrome. Check the sticky nav and entrance animations under narrow viewports.
- [ ] **Everforest theme review** — possibly too green; sample real card art against it and consider desaturating the background or shifting the accent.

### UX Regressions (zwiper)
- [x] **Show password eye icon** (2026-06-07) — Show/Hide text button wired into `TextInput` so every password field (login, register, change password, change username, change email confirm, delete account) gets it automatically. Shipped in 1.0.2.
- [x] **Alert dialog dark overlay missing** (2026-06-07) — confirmed via reading the upstream `dioxus-primitives` source: `AlertDialogRoot` deliberately does *not* emit an overlay (docs say "you can use it to create a backdrop for the dialog if needed"). Our wrapper now renders `<div class="alert-dialog-overlay" data-state="open">` as a sibling when open. Shipped in 1.0.2.
- [x] **Card filter broken in deck screens** (2026-06-07) — exhaustive per-filter pass found four distinct bugs in `zwipe-core/src/domain/card/models/search_card/filter_cards.rs` (the in-deck predicate the deck cards / remove cards screens run against the loaded deck — backend SQL search unaffected). Plus a rarity sort fix.
  - **Basic types include/exclude** — `card_type_contains_any`, `card_type_contains_all`, `card_type_excludes_any` compared a lowercased `type_line` against `CardType::to_string()` (capitalized: `"Creature"`), so `"legendary creature — dragon".contains("Creature")` was `false`. Include hid everything; exclude excluded nothing. Added `.to_lowercase()` to each `ct.to_string()`.
  - **Set filter include/exclude** — predicate compared against `sd.set` (the lowercase 3-letter code, e.g. `"mh2"`), but the UI (`extract_sets`) and backend `/sets` endpoint both send `set_name` (e.g. `"Modern Horizons 2"`). Switched predicate to `sd.set_name`. Fixed stale setter doc that still said `"MH2", "ONE"`.
  - **"Is commander in <format>"** — predicate field/getter/setter/UI all existed and the backend SQL adapter honored it, but `filter_cards.rs` had **zero** references to `is_commander_in_format`. Toggle did nothing on deck screens. Added predicate using existing `commander_eligibility::is_valid_commander(card, format)` helper.
  - **"Is legal in <format>"** — same shape as commander: `legalities_contains_any` set everywhere, backend SQL honored it (`legalities->>format IN ('legal', 'restricted')` OR-joined per format), `filter_cards.rs` ignored it. Added predicate mirroring backend semantics — card passes if any chosen format is `Legal` or `Restricted`, parsing format strings via `Format::try_from`.
  - **Rarity sort alphabetical** — `OrderByOption::Rarity` compared `to_long_name()` strings, giving `Common → Mythic → Rare → Uncommon`. Derived `Ord, PartialOrd, Eq` on `Rarity` (variant declaration is already in tier order so discriminant comparison gives Common < Uncommon < Rare < Mythic < Bonus < Special). Both sort sites updated to compare enum values directly. Added a `do not reorder` comment on the enum.
- [ ] **Round-trip filter coverage test** (follow-up from the audit above) — build a `CardFilter` with every field populated and assert that both the backend SQL adapter and `filter_cards.rs::filter_by` accept/reject the same fixture set. Would catch the "field exists end-to-end except in the frontend predicate" pattern that hid the commander and legality bugs for an unknown amount of time. Currently `filter_cards.rs` mirrors backend logic by hand, with no compile-time linkage to the `CardFilter` field list.
- [x] **Deck metrics skeleton state** (2026-06-07) — replaced the spinner/blank flash with skeletons across deck list, deck view (profile + stats with bordered info-list rendition), deck cards list, edit deck form, printing sheet, and home flavor text. Stats skeleton uses a shared `SkeletonInfoList` component that mirrors the real `.info-list` border, radius, padding, and per-row dividers. Shipped in 1.0.2.

### Card Rendering Bugs
- [x] **Cards missing from search** (2026-06-06) — `Kibo, Uktabi Prince` and `Wear // Tear` were invisible to card/commander search and in-deck filtering. Two root causes, both fixed: (1) `latest_cards` materialized view's `DISTINCT ON ... ORDER BY released_at DESC` picked digital-only and promo-flagged printings whenever they were the most recent — the new ORDER BY prefers paper, non-promo, non-oversized, non-content-warning rows first. (2) `CardFilterBuilder::default()` hardcoded `promo: Some(false)`, which over-filtered Jumpstart, Secret Lair, and UB-bonus printings where many cards only exist in promo form — relaxed to `None`. Migration `20260606120000_latest_cards_prefer_real_printings.sql` also remaps existing `deck_cards.scryfall_data_id` and `decks.{commander,partner_commander,background,signature_spell}_id` references so existing decks switch to the preferred printing on deploy.
- [x] **DFC handling — render front face for transform/MDFC + add flipper for all double-faced layouts** (2026-06-06) — landed in two commits:
  - **Step 1 — front face renders**: `ScryfallData::primary_image_url(ImageSize)` in zwipe-core now falls back from top-level `image_uris` to `card_faces[0].image_uris` so transform/MDFC cards surface in search and on every image render site. Also fixed the client-side filter in `add.rs` that was dropping cards with no top-level `image_uris` (the actual reason `Delver of Secrets` never reached the renderer despite the backend returning it).
  - **Step 2 — flipper**: new `FlippableCardImage` component (`zwiper/.../components/flippable_card_image.rs`) wraps `<img>` + a "Flip" squircle button. Wraps every image surface: swipe stack (top card only, peeking cards stay plain), printing carousel + single-printing view, image preview modal. Wrapper gets `aspect-ratio: 5/7` only when flippable so the button hugs the card edge at every call site; single-faced cards keep natural sizing.
  - **Meld handling not added** — meld pieces (Urza, Lord Protector et al.) already render correctly because each piece is a separate single-faced Scryfall row. Their "back" (the melded result, e.g. Urza, Planeswalker) is a separate scryfall row, not in `card_faces`. Cross-piece flipping via `all_parts` could surface that, but it's a different feature and not in scope for the current fix.
- [ ] **Investigate Delver not showing in existing deck after MDFC fix landed** — observed 2026-06-06 on an airplane (slow wifi may be a factor): a deck containing `Delver of Secrets` returned zero rows from `SELECT users.username, decks.name, scryfall_data.name FROM users JOIN decks ... JOIN deck_cards ... JOIN scryfall_data WHERE scryfall_data.name ILIKE 'delver of secrets'`. Card and combination definitely existed before. Fresh deck testing worked fine, so probably state-specific. Could be the migration's deck-card remap pointed the old reference at a row that no longer exists, OR the connection was timing out and the join silently returned empty. Worth re-running the query off airplane wifi first; if it still returns zero, dig into the remap audit.

### Infrastructure (Reactive)
- [x] **Cloudflare edge caching for immutable card endpoints** (2026-06-07) — probed via `zcripts/latency/probe.sh`: backend healthy (~52ms LOCAL search, 110k+ cards via materialized view + GIN trigram), tunnel floor ~125ms. Redis would buy ~50ms on 250ms — not worth complexity. CF edge caching does. Backend: moved 8 GET card metadata routes (`/api/card/{id}`, `/{oracle_id}/printings`, `sets`, `types`, `keywords`, `oracle-words`, `artists`, `languages`) from `private_routes` to `public_routes` with IP-keyed rate limit (60/s burst); `POST /api/card/search` stays private. Frontend: dropped `bearer_auth` + `session` param on all 8 client methods so CF doesn't `BYPASS` cache for authenticated requests. CF dashboard: one Cache Rule with `starts_with(http.request.uri.path, "/api/card/")`, Eligible for cache, Ignore origin Cache-Control, Edge TTL 24h. Verified end-to-end via `zcripts/latency/cf_cache_verify.sh` — converged to 6/6 HIT across POPs after warmup. Shipped in zerver `5666a86b` and zwiper `a721e413`; iOS client ships in 1.0.2 build 22. Roadmap doc: `context/ops/latency-optimization.md`.
- [x] **HTTP response compression** (2026-06-07) — `tower-http`'s `CompressionLayer` added to the public stack in `zerver/src/lib/inbound/http/mod.rs`. gzip+brotli enabled via `tower-http` features. `/api/card/search` body 39690b → 16444b on the wire (59% smaller); `/api/deck` body 3996b → 727b (82% smaller). Verified via `--compressed` curl in `zcripts/latency/probe.sh`.
- [x] **HTTP/2 client multiplexing** (2026-06-07) — workspace reqwest gained `http2` feature. Reqwest auto-negotiates h2 via ALPN with CF. Benefits burst flows like `deck/card/view.rs` (4 parallel `get_card` calls for command zone). Ships in iOS 1.0.2 build 23.
- [x] **Lower default search page size** (2026-06-07) — `CardFilter::default_limit()` 100 → 25 in zwipe-core; swipe stack's `pagination_limit` matched at 25, `load_more_threshold` 15 → 5. Compounding win: DB query returns 4× fewer rows, serialization is 4× cheaper, then gzip on top. LOCAL search dropped 52ms → 5ms. Ships in iOS 1.0.2 build 23.
- [ ] **Home server may struggle under marketing load** — current host is a single Ubuntu box behind Cloudflare Tunnel. If real users surface latency or 5xx spikes beyond what edge caching absorbs, evaluate migrating zerver to a cheap VPS in a major POP (Hetzner/Fly). Don't pre-optimize — only act on observed pain.
- [ ] **Migrate TIMESTAMP columns to TIMESTAMPTZ** — every timestamp column in the schema is plain `TIMESTAMP` (no time zone), so values stored = whatever the writing session's `TIMEZONE` resolves to. Today the SQLx pool pins `SET TIME ZONE 'UTC'` on every connection (`zerver/src/lib/outbound/sqlx/postgres.rs::after_connect`) as a backstop, so writes land canonically UTC, but the type system doesn't enforce it. Plan and execution captured in [context/ops/timestamptz-migration.md](../ops/timestamptz-migration.md). Schedule after metrics PR burns in for a week; not coupled to any feature work.
- [x] **Public stats strip on zwipe.net** (2026-06-07) — shipped on `zite-metrics-dashboard` branch. `StatsStrip` component in `zite/src/components/stats_strip.rs` fetches `GET /api/marketing/stats` during SSR and renders `Cards swiped · Searches run · Decks created` under the tagline in JetBrains Mono. CF Cache Rule live at 2h Edge TTL (free-plan minimum) covering `/api/marketing/*`. Numbers refresh on each GH Pages rebuild — acceptable for vanity; if staleness matters later, scheduled rebuild cron is the next move.

---

## Android — Near Submission Ready (PRIORITIZED 2026-06-10: ship before premium)

Decision (2026-06-10): grow the user base before monetizing — Android ships
before the premium tier. Rationale: the port is days of polish away (Dioxus),
premium revenue scales with users, MTG pods are mixed-device so adoption
spreads through playgroups, and shipping Android first means premium later
launches on both stores in one pass (StoreKit + Play Billing designed together).

- [ ] **FIRST: check Google Play account type / closed-testing requirement** —
  personal developer accounts created after late 2023 must run a closed test
  with 12 testers for 14 continuous days before production publishing is
  allowed. If it applies, start the closed test IMMEDIATELY so the 14-day
  clock runs while the polish items below are finished (LGS regulars + Reddit
  are the easy tester pool). Started 2026-06-10 (account check in progress).

Android build compiles and runs. Remaining polish before Play Store submission:

- [ ] Card images show white corners — the white is baked into the image data from Scryfall (white-bordered card editions). iOS clips correctly via WKWebView; Android WebView does not honor `overflow: hidden` + `border-radius` on `object-fit: contain` images. Tried: `overflow: hidden` on img, wrapper div with `border-radius` + `overflow: hidden`, `-webkit-mask-image` hack. None work on Android WebView. Options: crop with `object-fit: cover` (loses card edges), mask SVG overlay, or accept as-is for black-bordered cards (majority) and revisit for white-bordered.
- [ ] Swipe gesture doesn't tilt the card — cards should rotate slightly during drag like they do on iOS
- [ ] Lock screen orientation to portrait — need `android:screenOrientation="portrait"` on main activity. Dioxus may support this via `[android.raw.manifest]` or activity-level config. Test on Pixel once available.

---

## Synergy & Popularity Data

Per-commander synergy/popularity layer — the cache-first item at the top of
Next Up is the foundation; these are the consumers once it exists.

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

## Zwipe for Commander (Phase 3, UX Enhancement)

A dedicated swiping flow for commander selection. Future work — only build if users want it.

- [ ] On create/edit screen, when format has a commander, show "Zwipe for Commander" button
- [ ] Opens the swiping interface pre-filtered to valid commanders for the selected format
- [ ] User can adjust filters (colors, mana cost, set, etc.) and swipe through candidates
- [ ] First swipe-right sets the commander and returns to the create/edit screen
- [ ] Format filter defaults to deck's format but user can change it
- [ ] Works on both create and edit screens

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (308+ tests, ~100 in zwipe-core). Remaining gap: outbound adapters have no coverage.

---

## Maintenance

- **sqlx 0.8 → 0.9** — major bump. 0.9 has breaking changes around type mappings and connection options. Needs a dedicated branch where the integration tests run against a real Postgres before merge.
- **keyring 3 → 4** (zwiper) — major bump. Used for iOS Keychain on `apple-native`. Needs on-device test before merging; don't ship blind.
- **GitHub Actions Node.js 20 deprecation** — forced to Node.js 24 on June 2, 2026. All workflows already on latest major versions. No action needed — monitor for v5 releases.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
