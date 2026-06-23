# Project Progress

High-level snapshot of where zwipe stands. See `todo.md` for actionable items.

---

## Infrastructure — ✅ Done

Prod migrated off the home box to a **Hetzner CPX31 VPS** on 2026-06-13 (see entry below).

| Area | Status |
|------|--------|
| Prod host — Hetzner CPX31 VPS (Ubuntu 26.04, PG 18) | ✅ Live (home box retired, kept as rollback) |
| PostgreSQL + zwipe DB | ✅ Live |
| zerver systemd service | ✅ Live, auto-restarts on failure |
| zynergy synergy worker (least-priv DB role) | ✅ Live |
| Cloudflare Tunnel → `api.zwipe.net` | ✅ Live, TLS handled by Cloudflare |
| Self-hosted GitHub Actions runners (on VPS) | ✅ Live, deploy on push to main |
| CI/CD — zerver/zervice auto-deploy | ✅ Live, includes automatic migrations |
| CI/CD — zite → GitHub Pages | ✅ Live |
| Tailscale (local SSH access) | ✅ Configured |
| zervice nightly cron (Scryfall sync) | ✅ On VPS (4am UTC) |
| SQLx offline mode (.sqlx/ committed) | ✅ Configured |
| Database backups (pg_dump → R2, 30-day) | ✅ Nightly cron (5am UTC, on VPS) |

---

## Backend (zerver) — ✅ Feature Complete

| Feature | Status |
|---------|--------|
| JWT auth (access + rotating refresh tokens) | ✅ |
| User registration + email verification | ✅ |
| Password reset (forgot → email → reset) | ✅ |
| Change email / username / password | ✅ |
| Account deletion (`DELETE /api/user`) | ✅ |
| Deck CRUD | ✅ |
| Per-deck card management (add/remove/import) | ✅ |
| Card search (Scryfall data, JSONB filtering) | ✅ |
| Produced mana filter | ✅ |
| Rate limiting (auth + search endpoints) | ✅ |
| Account lockout (5 failures → 30min lock) | ✅ |
| Deck count limit (20/user) + card limit (250/deck) | ✅ |
| Unverified account soft limits (1 deck, 100 cards) | ✅ |
| User preferences (theme, dark mode) | ✅ |
| Transactional email via Resend | ✅ |
| Rolling daily logs + security audit logs | ✅ |
| Binary versioning (health endpoint + startup log) | ✅ |

---

## Web Client (zwipe.net) — ✅ Live

| Page | Status |
|------|--------|
| `/` — landing page | ✅ |
| `/about` | ✅ |
| `/contribute` — GitHub Sponsors | ✅ |
| `/download` — app store pending page | ✅ |
| `/privacy` | ✅ |
| `/verify/:token` — email verification | ✅ |
| `/reset/:token` — password reset form | ✅ |
| Favicon | ✅ |
| Entrance animations, sticky nav, ASCII logo | ✅ |

---

## iOS App (zwiper) — ✅ Feature Complete

| Area | Status |
|------|--------|
| Auth (login, register, forgot password) | ✅ |
| Deck list + deck view | ✅ |
| Card search + add to deck (swipe interface) | ✅ |
| Card image preview modal | ✅ |
| Produced mana filter | ✅ |
| Commander search (debounce + spinner) | ✅ |
| Profile (change email/username/password) | ✅ |
| Account deletion | ✅ |
| Unverified email toast + soft limits | ✅ |
| Preferences screen (9 themes, dark mode) | ✅ |
| Set name on swipe screens | ✅ |
| Clear filter (inline button + clears stack) | ✅ |
| Entrance transitions on all screens | ✅ |
| Toast system (word-wrap, error display) | ✅ |
| App icon (1024×1024 master, full size set) | ✅ |
| App name ("Zwipe" on home screen) | ✅ |
| Full screen integration pass | ✅ |
| Commander eligibility filter + toggle | ✅ |
| Multi-select format legality chips | ✅ |
| Warning action buttons (fix qty, clear commander) | ✅ |
| Per-section clear buttons on filter accordions | ✅ |
| Maybeboard (swipe-up, toggle, move, tri-filter, export/import) | ✅ |
| Partner / Background / Signature Spell fields | ✅ |
| zwipe-core direct dependency (proxy cleanup complete) | ✅ |
| Casing revamp (Title Case headings, sentence-case buttons/labels, backend text as-is) | ✅ |
| Font swap: Cascadia Code → JetBrains Mono @ weight 400 | ✅ |
| Mana value rename (was CMC) in stats + filter labels | ✅ |

---

## App Store Submission — LIVE (2026-06-06)

Zwipe is live on the App Store as **Zwipe TCG**: https://apps.apple.com/us/app/zwipe-tcg/id6761341603

Build 15 cleared review after a metadata scrub for Guideline 4.1(a) Copycats — renamed from "Zwipe MTG" to "Zwipe TCG" and stripped MTG/Magic/Commander/EDH/Planeswalker/Scryfall references from the listing copy. In-app behavior unchanged.

Build 15 shipped over build 14 with: `Email` strict newtype across the workspace (server rejects malformed addresses at construction, matching Resend's accepted shape), fix for resend-verification reading stale email from the JWT instead of the DB profile, and email templates restyled to JetBrains Mono + sentence case.

---

## Android — first build submitted to Play (2026-06-23)

The Android port is **in Google's review queue**: `1.0.9`, **versionCode 3**,
targetSdk 35, signed with a new `zwipe-upload` key (Play App Signing, Google-managed
app key), full rollout to the **Closed testing (Alpha)** track across 176 countries.
Same Rust/Dioxus codebase as iOS; the self-hosted JetBrains Mono fix makes the
Android-WebView block-glyph logo render correctly. Play account verification
(identity / address / phone) all cleared 2026-06-23. **Next gate:** ≥12 testers
opted in for 14 continuous days before Production access (new personal account).
Repeatable build pipeline + the day's gotchas (hardcoded targetSdk 34, burned
versionCode, debug-symbols warning): [`../operations/android/play-store-submission/build-and-submit.md`](../operations/android/play-store-submission/build-and-submit.md).

---

## 1.0.9 — UI consistency pass + new app icon (build 42 submitted 2026-06-23; server live on prod)

iOS **build 42** (version 1.0.9) submitted to review 2026-06-23 with a brand-new app icon (builds 39–41 were app-icon iteration; 42 = the 1.6× keeper). Rides: **new app icon** (the ASCII "Z" mark via the asciier tool — recipe in `operations/ios/appstore_icon_update.md`); **self-hosted JetBrains Mono** (full font bundled, CDN `@import` dropped — fixes the Android-WebView home-screen logo block glyphs, no-op on iOS); **profile rework** (per-field edits → bottom sheets, Delete account behind a `More` sheet, Account/Preferences cards); **deck-view** section subtitles moved inside their carded elements; **deck list** redone as one flowing row with accent stat chips + a warning-yellow card-count chip when a deck is an illegal size; **home flavor card** cached app-wide (1h TTL, stale-while-revalidate); **deck-size rules fixed** for Oathbreaker/Brawl/Historic Brawl/Gladiator; plus "To deck" → "To mainboard", an opaque chart skeleton, and a yellow-leaned Gruvbox text color. Workspace version bumped 1.0.6→1.0.9 (all crates) to keep `CARGO_PKG_VERSION` aligned with the store version for the min-version gate.

**Server + web are already live (2026-06-23):** the push redeployed `zerver` to prod (root reports `version: 1.0.9`, `/health` green — corrected deck-size warnings live) and `zite` to zwipe.net (Gruvbox text tweak). The iOS client is the only piece still in review. Per-change detail in `todo.md`. Android emulation (Pixel_9a) verified this code earlier — JDK gotcha in `operations/android/setup.md`.

> 1.0.6–1.0.8 App Store builds shipped between 1.0.5 and this entry: synergy-ordered suggestions (1.0.6), the mobile look-revamp (1.0.7), and skeleton polish (1.0.8).

---

## Gated merges — wire-format + refresh hardening (2026-06-18)

Two server-side changes that needed the propagation wait landed and deployed: **wire-format RFC3339** (server emits `Z` timestamps; the `wire_time` adapter was deleted from zwipe-core) and **refresh-token hardening** (strict single-use rotation — `FOR UPDATE` + delete check; live concurrency check passed: 4 parallel refreshes → one 200, three 401, replay → 401). `MIN_CLIENT_VERSION` armed at **1.0.5** in prod — the lowest guard-capable floor; not set higher by design (every 1.0.5+ client already carries the Z-parsing and single-flight-refresh fixes).

---

## Production migrated to VPS (2026-06-13)

Prod moved off the home Ubuntu box to a **Hetzner CPX31** (Hillsboro OR, Ubuntu 26.04, PG 18). `api.zwipe.net` now serves from the VPS through a Cloudflare tunnel; the three services run as systemd units (`zerver`, `zynergy` worker, `cloudflared`). CI runners + nightly crons (zervice 4am, backup 5am → R2) moved to the VPS; home crons disabled and the box powered off but intact as the rollback for ~1–2 weeks. Hardened: key-only SSH, ufw deny-all + tailnet-only, CI sudo scoped to `systemctl {start,stop,restart} {zerver,zynergy}`. A backup-restore drill passed first (PG17→18 clean: 115,805 cards / 24 users / 37 decks intact). Full runbook + gotchas in `../plans/vps_migration.md`. *Open follow-ups in `todo.md`: confirm the first unattended crons, repurpose the home box + rotate the still-shared R2 keys.*

---

## Synergy data layer — cache-first (2026-06-11, build 32)

Per-commander synergy/popularity payloads are computed by a separate least-privilege worker (`zynergy` — own DB role, runner, and systemd unit) and cached in Postgres; zerver only reads, never writes. Deck-aware search (`POST /api/deck/{id}/card/search`) excludes in-deck cards and defaults to synergy ordering when no sort is given; the client add-cards screen consumes it and auto-serves suggestions on open (build 32 / 1.0.6). Plan: `../plans/synergy_data_layer.md`. *Data-source strategy: check local memory before extending.*

---

## Post-launch hardening & UX (June 2026, builds 31–34)

- **First-run hints** — `hints_shown` jsonb on users + `PUT /api/user/hint`; six one-time dialogs (login, profile, first deck, deck cards, add/remove swipes) plus a persistent "?" reopener in every screen header.
- **Security notification emails** on email / username / password changes — notifies the *old* address (the one an attacker doesn't control), user values HTML-escaped, fire-and-forget via Resend.
- **Resend-verification throttle** — dedicated limiter (burst 1, then 1/60s per user); client greys the button with a matching 60s countdown + a "Check again" that flips the verified badge in place.
- **Fixes** — missing-auth responses now return 401 (were 500, from the user-keyed rate-limit layer running before the auth extractor); `GET /health` runs the combined server+db check; the "Update required" screen no longer flashes on filter apply (a Dioxus context type-collision, newtyped away).

---

## 1.0.5 — Archidekt Import + Min-Version Gate (2026-06-10, server deployed, build 31 submitted)

**Two features built, merged, and shipped in one day. Server live on prod as v1.0.5; iOS build 31 uploaded via Transporter and submitted as 1.0.5.**

- **Archidekt deck import** (`feat/deck-import-archidekt`) — `POST /api/deck/{deck_id}/import/archidekt` takes a deck URL, fetches Archidekt's open JSON API server-side, resolves every printing by Scryfall UID (`card.uid` == `scryfall_data.id`; name fallback recovers null-oracle reversible printings), and imports into an existing deck with identical semantics to the text importer. Deliberately simplified mid-build: no commander/format sync, no deck creation — just cards onto the selected board. The verified Archidekt `deckFormat` id table is preserved in `context/plans/deck_import.md` for a future opt-in sync.
- **Add/Replace import modes** — both importers carry `mode: ImportMode` (`#[serde(default)]`, absent = Add, so deployed 1.0.4 clients are unaffected). Replace makes the target board exactly match the imported list (board-scoped; an import where nothing resolves never wipes). Import screen gained pinned From/Mode/Board chip rows with per-combination hint text.
- **Min-version gate** (`feat/min-version-gate`) — server-driven force-update kill-switch: public `GET /api/client/min-version` reads `MIN_CLIENT_VERSION` env (`0.0.0` = open, live default; malformed value refuses startup), `zwipe_core::version` does x.y.z compare failing open, zwiper polls in the 60s upkeep loop (first tick at launch) and swaps the router for a blocking "Update required" screen linking to the App Store. Every install ≥1.0.5 is force-updatable; builds ≤1.0.4 ignore it forever, so 1.0.5 itself rides the old propagation wait.
- **API evolution rule documented** (`context/development/api_evolution.md`) — new request fields are always additive + `#[serde(default)]`; server deploys first, client ships second, no gate needed. The min-version gate is reserved for changes that can't be expressed additively.

---

## Card Visibility Fix (2026-06-06, post-launch)

**Backend deployed. iOS 1.0.1 / build 17 submitted for Apple review.**

`Kibo, Uktabi Prince` (Jumpstart 2022 — flagged `promo: true`) and `Wear // Tear` (latest printing was MTGA-only `digital: true`) were importable by exact name but invisible to card search, commander search, and in-deck filtering.

- Backend: `latest_cards` materialized view rewritten so `DISTINCT ON (oracle_id)` prefers paper, non-promo, non-oversized, non-content-warning printings before falling back to most recent release. Migration `20260606120000_latest_cards_prefer_real_printings.sql` also remaps existing `deck_cards` and `decks` references to the new preferred sibling so users' existing decks switch printings on deploy.
- Frontend (zwipe-core): `CardFilterBuilder::default()` dropped `promo: Some(false)` → `None`. `digital: false` stays as a default (paper deck builder shouldn't surface MTGA-only Alchemy cards). All other defaults unchanged.
- Bundled in the same iOS build: toast styling fix (CSS pairing broke when a `cargo update` pulled a newer `dioxus-primitives` commit that dropped default classes). `dioxus-primitives` now pinned to rev `02801f27` to prevent future silent breakage.

iOS 1.0.1 (build 17) replaces build 16 in the review queue. Apple typically clears metadata-stable bugfixes in 24–48h.

---

## User Metrics + Public Marketing Endpoint (2026-06-07, deployed + Build 24 packaged)

**Per-user telemetry, deck-completion tracking, audit log, and a public app-wide stats endpoint surfaced on zwipe.net. Build 24 (1.0.2) packaged for App Store Connect.** Numbers go live for the world the moment a user swipes / searches / creates a deck.

What's in this round:

- **Per-user lifetime counters** (`user_lifetime_counters`) — `swipes_right/left/up/down`, `searches`, `decks_created`, `decks_completed`. Single row per user, hot read path.
- **Daily rollups** (`user_daily_activity`) — one row per (user, UTC day) with the same swipe + search counters. Trend / DAU data without paying event-log storage.
- **Sparse event log** (`user_events`) — `register` (renamed from `signup` 2026-06-09), `deck_created`, `deck_completed`, `first_swipe`. Rare events only; no per-swipe rows.
- **Audit log** (`user_audit_log`) — credential changes (username / email / password). Logs *that* a change happened, not the old value — keeps PII surface near zero.
- **Endpoints** — `POST /api/metrics/usage` (private, IP+user rate-limited, accepts a `HttpUsageBatch`), `GET /api/user/metrics` (private, returns lifetime counters), `GET /api/marketing/stats` (public, sum-aggregates across all users for zwipe.net). Fire-and-forget metric writes via `tokio::spawn` so user request latency is unchanged.
- **Deck completion tracking** — after any deck-card mutation (create / update / delete / import / deck-profile update / clone) the handler reloads the deck, runs `validate_deck`, and if it just became valid stamps `decks.first_completed_at` + emits a `DeckCompleted` event. Idempotent: subsequent invalid→valid transitions don't re-fire.
- **Client-side telemetry buffer** — `zwiper/.../components/telemetry/` keeps four atomic swipe counters + a search counter in memory, flushes every 30s via the existing session upkeeper, drops the batch on HTTP failure (vanity data isn't worth retry plumbing).
- **Public marketing endpoint + CF cache** — `/api/marketing/stats` returns `{cards_swiped, searches, decks_created}` (single `SUM` over `user_lifetime_counters`). Cloudflare Cache Rule `starts_with(http.request.uri.path, "/api/marketing/")` with 2h Edge TTL (CF free-plan minimum). Origin gets one hit per POP per 2 hours.
- **zite stats strip** — three-stat block in the home hero ("Cards swiped · Searches run · Decks created") fetched during SSR via `use_resource`. Hides itself on error. Stats refresh on each GH Pages rebuild (acceptable for vanity; cron rebuild can be added if staleness ever bothers anyone).
- **UTC pool pin** — `PostgresPoolOptions::default()::after_connect` runs `SET TIME ZONE 'UTC'` on every connection. Backstop so the schema's plain `TIMESTAMP` columns are deterministically UTC regardless of cluster/process TZ. Spotted because `user_daily_activity` initial rows landed on a different `CURRENT_DATE` than the local psql session expected. Full migration to `TIMESTAMPTZ` is complete (phases 1-2, shipped 2026-06).

Build train: builds 21-23 (1.0.2, in review), **build 24 (1.0.2 + telemetry, packaged for Transporter)**. Build 24's user-visible delta over Build 23 is essentially zero — all the work this round is backend / silent telemetry. The "Cards swiped" bullet added to the App Store "What's New" reflects the build-23 latency wins that weren't called out.

---

## 1.0.2 Latency Pass (2026-06-07, submitted as build 23)

**iOS 1.0.2 build 23 submitted for Apple review. Full latency optimization round: CF edge caching, server-side compression, HTTP/2 client multiplexing, smaller default page size with prefetch.** End-to-end measurements: `POST /api/card/search` went from `~52ms LOCAL / ~250ms PUBLIC` to `~5ms LOCAL / ~130-180ms PUBLIC` — backend is now sub-frame; PUBLIC time is essentially the CF tunnel hop floor.

What's in build 23 (on top of 1.0.2):

- **Cloudflare edge caching for immutable card endpoints** — 8 GET routes (`/api/card/{id}`, `/{oracle_id}/printings`, `sets`, `types`, `keywords`, `oracle-words`, `artists`, `languages`) moved from `private_routes` to `public_routes` in `zerver/src/lib/inbound/http/routes.rs` with IP-keyed rate limit (60/s burst). Handlers' `AuthenticatedUser` extractors removed. zwiper's API client drops `bearer_auth` on those calls so CF's "don't cache authenticated requests" safety rail no longer triggers. CF Cache Rule with `starts_with(http.request.uri.path, "/api/card/")` + 24h Edge TTL. Verified via `zcripts/latency/cf_cache_verify.sh` — converged to 6/6 HIT once POPs warmed. Cache-hit responses skip the tunnel entirely (~5-10ms).
- **HTTP response compression** — `tower-http`'s `CompressionLayer` added to the Axum stack (`zerver/src/lib/inbound/http/mod.rs`). gzip + brotli via Accept-Encoding negotiation. `/api/card/search` body went 39690b → 16444b on the wire (59% smaller). `/api/deck` body went 3996b → 727b (82% smaller).
- **HTTP/2 client multiplexing** — workspace reqwest gained the `http2` feature. Reqwest auto-negotiates h2 via ALPN with CF, so the 4 parallel `get_card` calls in `deck/card/view.rs` (commander + partner + background + signature spell) now multiplex over a single connection instead of running sequentially.
- **Smaller search pages with prefetch** — `CardFilter::default_limit()` and `CardFilterBuilder::default()` lowered from 100 → 25 in zwipe-core. Swipe stack's `pagination_limit` matched at 25 and `load_more_threshold` tightened from 15 → 5 cards. Compounding win on search: DB query returns 4× fewer rows, serialization is 4× cheaper, then gzip on top. Drove LOCAL search from ~52ms to ~5ms.
- **Roadmap doc** — `context/archive/latency_optimization.md` captures the measurement-driven decision process. `zcripts/latency/probe.sh` and `cf_cache_verify.sh` document how to re-measure.

Build train: build 21 (1.0.2 polish, in review), build 22 (1.0.2 cache routes, replaced before delivery), build 23 (1.0.2 full latency pass, current submission).

---

## 1.0.2 Polish Pass (2026-06-07, submitted)

**iOS 1.0.2 build 21 submitted for Apple review. New gruvbox app icon, polish across filters/render/loading states.**

What's in 1.0.2:

- **In-deck filter fixes** (`filter_cards.rs`) — basic types include/exclude, set include/exclude, "Is commander in <format>", "Is legal in <format>", plus rarity sort tier order (Common < Uncommon < Rare < Mythic < Bonus < Special via derived `Ord`).
- **Card image rendering** — `FlippableCardImage` reworked so card art renders with cleanly rounded corners and bounded sizing across the swipe stack, printing carousel, and image preview. Root cause: wrapper inherited `flex: 1` from `.card-image`/`.carousel-card-image` and stretched in column-flex parents, letterboxing the actual card content and putting the rounded clip on empty space. Fix moves sizing onto the img element (`width: auto; height: auto; max-width/max-height: 100%`, relying on `<img>`'s intrinsic aspect ratio) with per-context max-height caps on the wrapper.
- **Loading skeletons** — deck list, deck view (profile + stats with bordered info-list rendition matching the real `.info-list`), deck cards list, edit deck form, printing sheet, home flavor text.
- **Saving / submitting states** — login shows "Logging in...", register shows "Creating...", profile/preferences/deck edit screens show "Saving..." with Back disabled. Fixed pre-existing race in `login.rs`/`register.rs` where `is_loading.set(false)` ran outside the spawn block, so the loading state never actually appeared.
- **Password show/hide toggle** — single `TextInput` change wires a Show/Hide button onto every password field (login, register, change password, change username, change email password confirm, delete account dialog).
- **AlertDialog backdrop restored** — `dioxus-primitives` deliberately doesn't emit an overlay div for the dim backdrop. Wrapper now renders the `.alert-dialog-overlay` sibling when open.
- **New app icon** — gruvbox tan Z on `#282828`. Source 1024×1024 master flattened to strip alpha, scaled to all required sizes. Process documented in `context/operations/ios/appstore_icon_update.md`.

Build train: build 18 (1.0.2 orphan from prior misclick), build 19 (1.0.1, shipped), build 20 (1.0.2, replaced before delivery), build 21 (1.0.2, current submission). Apple typically clears polish releases in 24–48h with no metadata changes.

---

## DFC Handling (2026-06-06, same day as card visibility fix)

**Front face rendering + flip control. iOS build 19 packaged as 1.0.1 to replace build 17 in the open review queue (since 1.0.1 hasn't published yet, all build numbers attach to the same train). Build 18 was uploaded as 1.0.2 by mistake and is now an orphan in App Store Connect — harmless, can be ignored.**

Double-faced layouts (transform, modal_dfc) store their image URLs inside `card_faces[].image_uris` rather than the top-level `image_uris` that single-faced cards use. Zwiper had zero `card_faces` references anywhere — so `Delver of Secrets`, `Valki, God of Lies`, and every transform/MDFC card rendered as a blank image surface AND was filtered out of search results by a client-side "must have top-level image" filter.

- **zwipe-core**: `ScryfallData::primary_image_url(ImageSize)` and `face_image_url(idx, size)` fall back to `card_faces[face_index].image_uris` when top-level is `None`. Every render site replaced. `face_count()` reports `card_faces.len()` only when all faces have their own image URIs, so split / adventure layouts (single image, no per-face URIs) stay single-faced for rendering purposes.
- **zwiper**: new `FlippableCardImage` component owns face-index state and renders the `<img>` plus a "Flip" squircle button when `face_count() > 1`. Wired into swipe stack (top card only — peeking cards stay plain), printing carousel + single-printing view, image preview modal. Wrapper has `aspect-ratio: 5/7` only when flippable so the button hugs the actual card edge regardless of container size.
- **Meld pieces** continue to render correctly via the existing top-level-image path; flipping to the melded back is out of scope.
