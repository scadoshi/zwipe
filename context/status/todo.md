# Todo

**Primary goal: Get Zwipe publicly available on the App Store.**

---

## Server Migration (2026-03-27)

✅ Complete. Ubuntu Server (Intel i5, 32GB RAM, x86_64) running zerver, zervice, PostgreSQL, cloudflared, CI/CD runner. Full setup guide: `ops/server.md`.

### Remaining minor issues

- [x] Clear filter now clears card stack + inline clear button added (`bc46b5f`, `a198d60`)
- [x] Random sort refresh now re-shuffles (`2cd61b2`)
- [x] Empty card shape sizing fixed

---

## App Store Submission

### 1. Fix App Name

✅ Fixed (`8d03fb7`). Renamed binary from `main.rs` to `zwipe.rs` so iOS reads "Zwipe" instead of "Main". `Dioxus.toml` already had `[application] name = "Zwipe"`. **Needs device confirmation** — rebuild and check home screen.

### 2. Account Deletion (App Store Required)

✅ Done (`af7fd87`, `70a7042`).

### 3. iOS App Icon

✅ Done (`5cf5d79`). Full icon set generated from 1024×1024 master (no alpha). All sizes in `zwiper/assets/favicon/icon-*.png`. Webmanifest updated.

### 4. zwipe.net Web Client

✅ Live at https://zwipe.net.

### 5. App Store Connect Setup

1. [appstoreconnect.apple.com](https://appstoreconnect.apple.com) — sign in with Apple ID
2. Create app: Bundle ID `com.scadoshi.zwipe`, name "Zwipe", English
3. Fill out: description, keywords (MTG, Magic the Gathering, deck builder, commander), screenshots (6.7" iPhone required), privacy policy URL, support URL, age rating 4+, category: Games > Card Games

### 6. Build for Distribution

Current build uses Development profile. App Store requires:
- Distribution certificate (Apple Distribution)
- App Store provisioning profile
- Archive and upload via `xcrun altool` or Transporter

### 7. Submit

- Export compliance: no encryption beyond HTTPS — answer No
- Submit for review — typical 1–3 days

---

## UX Polish

- [x] Entrance transitions on all screens (`d456e10`, `f153b23`)
- [x] Migrate inline submission errors to toasts (`d456e10`, `a1bff36`)
- [x] Lowercase rate limiting error messages (`027c6e8`)
- [x] Card image preview modal on deck card list (`89368ce`)
- [x] zweb design alignment — entrance animations, CSS tokens, spinner, dimmer borders (`24704b8`)
- [x] zweb reset password mobile UX (`15585ae`)
- [x] zweb nav: replace text brand with ASCII z logo, sticky on scroll, re-triggers logo animation on click (`351fff5`, `79f7914`, `241bf48`)
- [x] zweb logo line-height tightened so block characters stack flush (`8deb2b2`)
- [x] Unverified email toast on login + soft limits for unverified accounts (`cf9071c`)
- [x] Toast word-wrap: prefer word boundaries over mid-word breaks (`67eadf1`)
- [x] iOS app icons: full set from 1024×1024 master (`5cf5d79`)
- [x] Clear filter clears card stack + inline clear button (`bc46b5f`, `a198d60`)
- [x] Random sort refresh re-shuffles (`2cd61b2`)
- [x] Binary versioning: startup logs + workspace version (`7041918`)
- [x] Full screen integration pass

---

## Rate Limiting

✅ All critical endpoints covered. See below for details.

- Forgot password, reset password — IP-level governor
- Change password/username/email, delete user — burst 2, then 1 req/30min
- Card search — burst 5, then 1 req/10s
- Future: key by authenticated user ID instead of IP for per-user fairness

---

## Limits

✅ Done. Verified: 20 decks, 250 cards/deck. Unverified: 1 deck, 100 cards/deck (`cf9071c`). Limit selected via `email_verified` JWT claim — no extra DB query.

---

## Binary Versioning

✅ Done (`7041918`). Workspace-level version in root `Cargo.toml`. All binaries (zerver, zervice, zwiper) print version on startup via `tracing::info!`. Health endpoint (`GET /`) returns version field.

---

## CI/CD

✅ Both pipelines live. See `ops/cicd.md`.

---

## GitHub Actions Node.js 20 Deprecation

Actions running on Node.js 20 will be **forced to Node.js 24** starting **June 2, 2026**.
Bump `actions/checkout@v4` and `actions/cache@v4` in both `deploy-zerver.yml` and `deploy-zweb.yml` before then.

---

## Donate Button

✅ Done. GitHub Sponsors application pending approval.

---

## Extract Password Validation into a Shared Crate

`zweb/src/pages/reset.rs` duplicates the password policy from `zerver`. Low priority — duplication is fine for now. See full notes in git history (`46c0f68`).

---

## Database Backups

✅ Done. Nightly `pg_dump` → Cloudflare R2 via `rclone`. Cron at 5am daily. 30-day lifecycle retention. Full runbook: `ops/backups.md`.

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (250+ tests). Remaining gap: outbound adapters have no coverage.

---

## Project Structure Doc

Add a `context/architecture/structure.md` walking through the full directory tree — useful for onboarding and AI context.
