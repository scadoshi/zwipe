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
| CI/CD — zerver/zervice auto-deploy | ✅ Live |
| CI/CD — zweb → GitHub Pages | ✅ Live |
| Tailscale (local SSH access) | ✅ Configured |
| zervice nightly cron (Scryfall sync) | ✅ Configured |
| SQLx offline mode (.sqlx/ committed) | ✅ Configured |

---

## Backend (zerver) — ✅ Feature Complete for MVP

| Feature | Status |
|---------|--------|
| JWT auth (access + rotating refresh tokens) | ✅ |
| User registration + email verification | ✅ |
| Password reset (forgot → email → reset) | ✅ |
| Change email / username / password | ✅ |
| Deck CRUD | ✅ |
| Per-deck card management (add/remove/import) | ✅ |
| Card search (Scryfall data, JSONB filtering) | ✅ |
| Rate limiting (auth + search endpoints) | ✅ |
| Deck count limit (20/user) + card limit (250/deck) | ✅ |
| Transactional email via Resend | ✅ |
| Rolling daily logs | ✅ |
| Account deletion (`DELETE /api/user`) | ❌ Required for App Store |

---

## Web Client (zwipe.net) — ✅ Live

| Page | Status |
|------|--------|
| `/` — landing page | ✅ |
| `/about` | ✅ |
| `/contribute` — Stripe, BMaC, GitHub Sponsors | ✅ |
| `/privacy` | ✅ |
| `/verify/:token` — email verification | ✅ |
| `/reset/:token` — password reset form | ✅ |

---

## iOS App (zwiper) — In Progress

| Area | Status |
|------|--------|
| Auth (login, register, forgot password) | ✅ |
| Deck list + deck view | ✅ |
| Card search + add to deck | ✅ |
| Profile (change email/username/password) | ✅ |
| Email verification badge + resend | 🔲 Planned |
| Account deletion | ❌ Required for App Store |
| App icon (1024×1024, 180×180, 120×120) | ❌ Required for App Store |
| App name fix (`name = "Zwipe"` in Dioxus.toml) | ❌ Required for App Store |
| Full screen integration pass (on-device) | 🔲 Planned |

---

## App Store Submission — Blocked

Remaining hard blockers before submission:
1. **Account deletion** — Apple guideline 5.1.1, mandatory
2. **App icon** — current icons are web favicons, rejected by iOS
3. **App name** — shows "Main" on home screen, fix in Dioxus.toml
4. **Distribution cert + App Store provisioning profile** — dev cert only right now
5. **App Store Connect** — app listing, screenshots, description, privacy policy URL

---

## Post-Launch Priorities (after App Store approval)

### Database Backups — High Priority
The server is the single source of truth. If the machine dies before backups are set up,
all user data is gone. Plan: nightly `pg_dump` compressed and uploaded offsite.

- **Tool**: `pg_dump` → gzip
- **Destination**: Backblaze B2 (cheapest S3-compatible option, ~$0.006/GB/month) via
  `rclone` — or AWS S3/Cloudflare R2
- **Schedule**: nightly cron alongside zervice
- **Retention**: 7–30 days of rolling backups
- **Restore runbook**: document full restore steps in `ops/server.md`

See `todo.md` → Database Backups for the rough script sketch.

### User Metrics
Start simple — don't reach for Mixpanel/Amplitude until you know what questions to ask.

- **Web traffic**: Plausible or Fathom (privacy-friendly, no GDPR/cookie banner headache)
- **API activity**: structured logs already exist — add a `user_events` table for key
  actions (registration, deck created, card added) that can be queried directly
- **Dashboard**: query the DB directly to start; build reporting later if needed

### In-App Feedback
A "Send Feedback" button that composes an email to a support address is enough to start.
Don't over-engineer until there are users generating feedback volume.

### Patch Discipline
The App Store review cycle is 1–3 days per iOS submission. Backend patches ship in
minutes via CI/CD. That asymmetry shapes everything:

- Keep the iOS client **defensive** — handle unexpected server responses gracefully so
  the server can be patched without forcing an app update
- **Never edit existing migration files** — always add a new migration forward
- **Semantic versioning**: `MAJOR.MINOR.PATCH` — bump PATCH for bug fixes, MINOR for
  new features, MAJOR for breaking changes
- **Deprecate before removing**: leave old endpoints alive for at least one app version
  cycle before pulling them
- **API versioning**: don't add `/v2/` preemptively — only version when you have an
  actual breaking change and need both versions live simultaneously
- **Breaking change checklist**: before removing or changing an endpoint signature,
  check what version of zwiper is in the wild and whether old clients will break
