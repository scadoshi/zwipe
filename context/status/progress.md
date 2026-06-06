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

## DFC Handling (2026-06-06, same day as card visibility fix)

**Front face rendering + flip control. iOS 1.0.2 / build 18 packaged for submission.**

Double-faced layouts (transform, modal_dfc) store their image URLs inside `card_faces[].image_uris` rather than the top-level `image_uris` that single-faced cards use. Zwiper had zero `card_faces` references anywhere — so `Delver of Secrets`, `Valki, God of Lies`, and every transform/MDFC card rendered as a blank image surface AND was filtered out of search results by a client-side "must have top-level image" filter.

- **zwipe-core**: `ScryfallData::primary_image_url(ImageSize)` and `face_image_url(idx, size)` fall back to `card_faces[face_index].image_uris` when top-level is `None`. Every render site replaced. `face_count()` reports `card_faces.len()` only when all faces have their own image URIs, so split / adventure layouts (single image, no per-face URIs) stay single-faced for rendering purposes.
- **zwiper**: new `FlippableCardImage` component owns face-index state and renders the `<img>` plus a "Flip" squircle button when `face_count() > 1`. Wired into swipe stack (top card only — peeking cards stay plain), printing carousel + single-printing view, image preview modal. Wrapper has `aspect-ratio: 5/7` only when flippable so the button hugs the actual card edge regardless of container size.
- **Meld pieces** continue to render correctly via the existing top-level-image path; flipping to the melded back is out of scope.
