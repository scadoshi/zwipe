# Todo

**Primary goal: Get Zwipe publicly available on the App Store.**

---

## Server Migration (2026-03-27)

✅ Complete. Ubuntu Server (Intel i5, 32GB RAM, x86_64) running zerver, zervice, PostgreSQL, cloudflared, CI/CD runner. Full setup guide: `ops/server.md`.

### Remaining minor issues

- [ ] Empty card shape persists in mobile application — needs to be bigger
- [ ] Clicking clear filter still doesn't clear out the cards currently in the stack. Upon entrance into the add.rs screen filter should be checked and if empty clear cards and if has filter it needs to immediately run in case they changed the filter on another screen! (cards now persist across navigation when filter unchanged — `82c67f0`, but clear-filter behavior still needs work)
- [ ] When I sort by random on remove screen and click refresh it returns to the start of the list without randomizing again it should re-apply the sorting filter causing refresh in this instance to continually randomize the card stack showing

---

## App Store Submission

### 1. Fix App Name

✅ Fixed (`8d03fb7`). Renamed binary from `main.rs` to `zwipe.rs` so iOS reads "Zwipe" instead of "Main". `Dioxus.toml` already had `[application] name = "Zwipe"`. **Needs device confirmation** — rebuild and check home screen.

### 2. Account Deletion (App Store Required)

✅ Done (`af7fd87`, `70a7042`).

### 3. iOS App Icon

Current `zwiper/assets/favicon/` icons are web favicons — iOS ignores them.

**Hard requirement:** 1024×1024 PNG (no alpha) for App Store listing. Other sizes (180, 120, 87, 80, 60, 40) improve on-device appearance but won't block submission.

Closest existing asset: `zwiper/assets/favicon/android-chrome-512x512.png` (only 512×512). Need a clean 1024×1024 master.

Add to `zwiper/Dioxus.toml`:
```toml
[bundle]
icon = ["assets/icons/icon-1024.png"]
```

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
- [ ] Full screen integration pass — walk every screen on device

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

Add a version string to `zerver` and `zervice` that prints on startup — makes it immediately obvious after a manual or CI deploy whether the new binary is live.

- Add `version` to workspace `Cargo.toml` (e.g. `0.1.0`)
- Print version on startup using `env!("CARGO_PKG_VERSION")`
- Expose on health endpoint: `GET /` already returns a `version` field — verify it uses `CARGO_PKG_VERSION` and not a hardcoded string

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

The server runs PostgreSQL locally. Need a periodic backup job that gets data off the machine.

- **Tool**: `pg_dump`
- **Schedule**: nightly cron
- **Destination**: S3/R2, Backblaze B2, or remote SSH
- **Retention**: TBD
- **Restore runbook**: TBD

The database is the only critical stateful data not replicated elsewhere.

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (250+ tests). Remaining gap: outbound adapters have no coverage.

---

## Project Structure Doc

Add a `context/architecture/structure.md` walking through the full directory tree — useful for onboarding and AI context.
