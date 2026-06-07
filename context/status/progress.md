# Project Progress

High-level snapshot of where zwipe stands. See `todo.md` for actionable items.

---

## Infrastructure — ✅ Done

| Area | Status |
|------|--------|
| Ubuntu Server (i5, 32GB, x86_64) | ✅ Live |
| PostgreSQL + zwipe DB | ✅ Live |
| zerver systemd service | ✅ Live, auto-restarts on failure |
| Cloudflare Tunnel → `api.zwipe.net` | ✅ Live, TLS handled by Cloudflare |
| Self-hosted GitHub Actions runner | ✅ Live, deploys on push to main |
| CI/CD — zerver/zervice auto-deploy | ✅ Live, includes automatic migrations |
| CI/CD — zite → GitHub Pages | ✅ Live |
| Tailscale (local SSH access) | ✅ Configured |
| zervice nightly cron (Scryfall sync) | ✅ Configured |
| SQLx offline mode (.sqlx/ committed) | ✅ Configured |
| Database backups (pg_dump → R2, 30-day) | ✅ Nightly cron |

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

## Card Visibility Fix (2026-06-06, post-launch)

**Backend deployed. iOS 1.0.1 / build 17 submitted for Apple review.**

`Kibo, Uktabi Prince` (Jumpstart 2022 — flagged `promo: true`) and `Wear // Tear` (latest printing was MTGA-only `digital: true`) were importable by exact name but invisible to card search, commander search, and in-deck filtering.

- Backend: `latest_cards` materialized view rewritten so `DISTINCT ON (oracle_id)` prefers paper, non-promo, non-oversized, non-content-warning printings before falling back to most recent release. Migration `20260606120000_latest_cards_prefer_real_printings.sql` also remaps existing `deck_cards` and `decks` references to the new preferred sibling so users' existing decks switch printings on deploy.
- Frontend (zwipe-core): `CardFilterBuilder::default()` dropped `promo: Some(false)` → `None`. `digital: false` stays as a default (paper deck builder shouldn't surface MTGA-only Alchemy cards). All other defaults unchanged.
- Bundled in the same iOS build: toast styling fix (CSS pairing broke when a `cargo update` pulled a newer `dioxus-primitives` commit that dropped default classes). `dioxus-primitives` now pinned to rev `02801f27` to prevent future silent breakage.

iOS 1.0.1 (build 17) replaces build 16 in the review queue. Apple typically clears metadata-stable bugfixes in 24–48h.

---

## 1.0.2 Latency Pass (2026-06-07, submitted as build 23)

**iOS 1.0.2 build 23 submitted for Apple review. Full latency optimization round: CF edge caching, server-side compression, HTTP/2 client multiplexing, smaller default page size with prefetch.** End-to-end measurements: `POST /api/card/search` went from `~52ms LOCAL / ~250ms PUBLIC` to `~5ms LOCAL / ~130-180ms PUBLIC` — backend is now sub-frame; PUBLIC time is essentially the CF tunnel hop floor.

What's in build 23 (on top of 1.0.2):

- **Cloudflare edge caching for immutable card endpoints** — 8 GET routes (`/api/card/{id}`, `/{oracle_id}/printings`, `sets`, `types`, `keywords`, `oracle-words`, `artists`, `languages`) moved from `private_routes` to `public_routes` in `zerver/src/lib/inbound/http/routes.rs` with IP-keyed rate limit (60/s burst). Handlers' `AuthenticatedUser` extractors removed. zwiper's API client drops `bearer_auth` on those calls so CF's "don't cache authenticated requests" safety rail no longer triggers. CF Cache Rule with `starts_with(http.request.uri.path, "/api/card/")` + 24h Edge TTL. Verified via `zcripts/latency/cf_cache_verify.sh` — converged to 6/6 HIT once POPs warmed. Cache-hit responses skip the tunnel entirely (~5-10ms).
- **HTTP response compression** — `tower-http`'s `CompressionLayer` added to the Axum stack (`zerver/src/lib/inbound/http/mod.rs`). gzip + brotli via Accept-Encoding negotiation. `/api/card/search` body went 39690b → 16444b on the wire (59% smaller). `/api/deck` body went 3996b → 727b (82% smaller).
- **HTTP/2 client multiplexing** — workspace reqwest gained the `http2` feature. Reqwest auto-negotiates h2 via ALPN with CF, so the 4 parallel `get_card` calls in `deck/card/view.rs` (commander + partner + background + signature spell) now multiplex over a single connection instead of running sequentially.
- **Smaller search pages with prefetch** — `CardFilter::default_limit()` and `CardFilterBuilder::default()` lowered from 100 → 25 in zwipe-core. Swipe stack's `pagination_limit` matched at 25 and `load_more_threshold` tightened from 15 → 5 cards. Compounding win on search: DB query returns 4× fewer rows, serialization is 4× cheaper, then gzip on top. Drove LOCAL search from ~52ms to ~5ms.
- **Roadmap doc** — `context/ops/latency-optimization.md` captures the measurement-driven decision process. `zcripts/latency/probe.sh` and `cf_cache_verify.sh` document how to re-measure.

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
- **New app icon** — gruvbox tan Z on `#282828`. Source 1024×1024 master flattened to strip alpha, scaled to all required sizes. Process documented in `context/ops/ios/appstore-icon-update.md`.

Build train: build 18 (1.0.2 orphan from prior misclick), build 19 (1.0.1, shipped), build 20 (1.0.2, replaced before delivery), build 21 (1.0.2, current submission). Apple typically clears polish releases in 24–48h with no metadata changes.

---

## DFC Handling (2026-06-06, same day as card visibility fix)

**Front face rendering + flip control. iOS build 19 packaged as 1.0.1 to replace build 17 in the open review queue (since 1.0.1 hasn't published yet, all build numbers attach to the same train). Build 18 was uploaded as 1.0.2 by mistake and is now an orphan in App Store Connect — harmless, can be ignored.**

Double-faced layouts (transform, modal_dfc) store their image URLs inside `card_faces[].image_uris` rather than the top-level `image_uris` that single-faced cards use. Zwiper had zero `card_faces` references anywhere — so `Delver of Secrets`, `Valki, God of Lies`, and every transform/MDFC card rendered as a blank image surface AND was filtered out of search results by a client-side "must have top-level image" filter.

- **zwipe-core**: `ScryfallData::primary_image_url(ImageSize)` and `face_image_url(idx, size)` fall back to `card_faces[face_index].image_uris` when top-level is `None`. Every render site replaced. `face_count()` reports `card_faces.len()` only when all faces have their own image URIs, so split / adventure layouts (single image, no per-face URIs) stay single-faced for rendering purposes.
- **zwiper**: new `FlippableCardImage` component owns face-index state and renders the `<img>` plus a "Flip" squircle button when `face_count() > 1`. Wired into swipe stack (top card only — peeking cards stay plain), printing carousel + single-printing view, image preview modal. Wrapper has `aspect-ratio: 5/7` only when flippable so the button hugs the actual card edge regardless of container size.
- **Meld pieces** continue to render correctly via the existing top-level-image path; flipping to the melded back is out of scope.
