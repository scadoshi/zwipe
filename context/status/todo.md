# Todo

**Primary goal: Submit Zwipe to the App Store.**

All core features, infrastructure, security, and UX polish are complete. The app is ready for submission.

---

## App Store Submission — Ready

### ✅ Completed Prerequisites

| Requirement | Status | Commit |
|-------------|--------|--------|
| Account deletion (`DELETE /api/user`) | ✅ | `af7fd87`, `70a7042` |
| App icon (1024×1024 master, full size set) | ✅ | `5cf5d79` |
| App name fix (binary → "Zwipe") | ✅ | `8d03fb7` |
| Privacy policy (zwipe.net/privacy) | ✅ | Live |
| zwipe.net web client | ✅ | Live |
| Download page (app store pending) | ✅ | `5e38f78` |
| Favicon on zweb | ✅ | `5026d4b` |

### Remaining Steps

1. **App Store Connect Setup**
   - [appstoreconnect.apple.com](https://appstoreconnect.apple.com) — sign in with Apple ID
   - Create app: Bundle ID `com.scadoshi.zwipe`, name "Zwipe", English
   - Fill out: description, keywords (MTG, Magic the Gathering, deck builder, commander), screenshots (6.7" iPhone required), privacy policy URL, support URL, age rating 4+, category: Games > Card Games

2. **Build for Distribution**
   - Distribution certificate (Apple Distribution)
   - App Store provisioning profile
   - Archive and upload via `xcrun altool` or Transporter

3. **Submit**
   - Export compliance: no encryption beyond HTTPS — answer No
   - Submit for review — typical 1–3 days

---

## Recently Completed

### User Preferences & Themes (2026-03-28)

Full-stack feature: database-backed user preferences with 9 color themes and light/dark mode.

- [x] Domain layer, migration, repository (`28a9f98`)
- [x] HTTP handlers, routes, domain tests (`e507199`)
- [x] Preferences embedded in JWT claims and session response (`825980f`)
- [x] Theme system with 9 themes, preferences screen, UI polish (`7ab15a3`)
- [x] Set name on swipe screens, alphabetized theme list (`0f431ab`)

### Card Filtering Improvements

- [x] Produced mana card filter with frontend chip UI (`40cdb65`)
- [x] Commander search burst size increase (`657456b`)
- [x] Commander search debounce fix + loading spinner (`b8f66c5`)

### UX Polish

- [x] Entrance transitions on all screens (`d456e10`, `f153b23`)
- [x] Inline submission errors → toasts (`d456e10`, `a1bff36`)
- [x] Lowercase rate limiting error messages (`027c6e8`)
- [x] Card image preview modal on deck card list (`89368ce`)
- [x] Clear filter: clears card stack + inline clear button (`bc46b5f`, `a198d60`)
- [x] Random sort refresh re-shuffles (`2cd61b2`)
- [x] Remove "show" from show image button on deck entries (`c669e02`)
- [x] Toast word-wrap: prefer word boundaries (`67eadf1`)
- [x] Unverified email toast on login + soft limits (`cf9071c`)
- [x] iOS app icons: full set from 1024×1024 master (`5cf5d79`)
- [x] Binary versioning: startup logs + workspace version (`7041918`)
- [x] Full screen integration pass

### zweb Polish

- [x] Design alignment — entrance animations, CSS tokens, spinner, dimmer borders (`24704b8`)
- [x] Reset password mobile UX (`15585ae`)
- [x] Nav: ASCII z logo, sticky on scroll, logo animation on click (`351fff5`, `79f7914`, `241bf48`)
- [x] Logo line-height tightened (`8deb2b2`)
- [x] Smooth scroll to top on Z logo click (`faf4f82`)
- [x] Download page for app store pending status (`5e38f78`)
- [x] Favicon (`5026d4b`)

### CI/CD

- [x] Automatic migrations in deploy pipeline (`7393c6d`)
- [x] Fix migration `DATABASE_URL` export via `set -a` (`5e5d2a3`)

### Infrastructure

- [x] Database backups: nightly `pg_dump` → Cloudflare R2 via `rclone`, 30-day retention (`6e93dd2`, `c72e361`)
- [x] Server migration to Ubuntu (Intel i5, 32GB) — complete (`ops/server.md`)

---

## Rate Limiting

✅ All critical endpoints covered.

- Forgot password, reset password — IP-level governor
- Change password/username/email, delete user — burst 2, then 1 req/30min
- Card search — burst 5, then 1 req/10s
- Future: key by authenticated user ID instead of IP for per-user fairness

---

## Limits

✅ Done. Verified: 20 decks, 250 cards/deck. Unverified: 1 deck, 100 cards/deck (`cf9071c`). Limit selected via `email_verified` JWT claim — no extra DB query.

---

## CI/CD

✅ Both pipelines live. See `ops/cicd.md`.

---

## GitHub Actions Node.js 20 Deprecation

Actions running on Node.js 20 will be **forced to Node.js 24** starting **June 2, 2026**.
Bump `actions/checkout@v4` and `actions/cache@v4` in both `deploy-zerver.yml` and `deploy-zweb.yml` before then.

---

## Extract Password Validation into a Shared Crate

`zweb/src/pages/reset.rs` duplicates the password policy from `zerver`. Low priority — duplication is fine for now. See full notes in git history (`46c0f68`).

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (250+ tests). Remaining gap: outbound adapters have no coverage.

---

## Project Structure Doc

Add a `context/architecture/structure.md` walking through the full directory tree — useful for onboarding and AI context.
