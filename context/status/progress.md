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

## App Store Submission — Ready to Submit

All hard blockers resolved. Remaining steps are administrative:
1. App Store Connect listing (description, screenshots, keywords)
2. Distribution certificate + provisioning profile
3. Archive, upload, submit for review
