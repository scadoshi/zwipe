# Todo

**Primary goal: Submit Zwipe to the App Store.**

---

## App Store Submission — Ready

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

## Android

- [ ] Card images showing white corners (border-radius not clipping properly)
- [ ] Refresh with filters set doesn't reset card index — image/stats stick on old card

---

## UX — Skeleton Loading

- [x] Add screen empty state: skeleton placeholder with ghost bars for price, set, artist (`beba43a`)
- [ ] Deck card list: skeleton rows while cards load

---

## Rate Limiting

✅ All critical endpoints covered. Per-user rate limiting on private routes (`0e9e8be`).

- Forgot password, reset password — IP-level governor
- Change password/username/email, delete user — burst 2, then 1 req/30min (keyed by user ID)
- Card search — burst 20, then 1 req/10s (keyed by user ID)
- General private routes — burst 500, 1 req/600ms (keyed by user ID)

---

## GitHub Actions Node.js 20 Deprecation

Actions running on Node.js 20 will be **forced to Node.js 24** starting **June 2, 2026**.
Bump `actions/checkout@v4` and `actions/cache@v4` in both `deploy-zerver.yml` and `deploy-zweb.yml` before then.

---

## Extract Password Validation into a Shared Crate

`zweb/src/pages/reset.rs` duplicates the password policy from `zerver`. Low priority — duplication is fine for now. See full notes in git history (`46c0f68`).

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (269 tests). Remaining gap: outbound adapters have no coverage.

---

## Project Structure Doc

Add a `context/architecture/structure.md` walking through the full directory tree — useful for onboarding and AI context.

---

## Recently Completed

### Per-User Rate Limiting (2026-03-30)

- [x] Custom `UserIdKeyExtractor` for tower_governor, keys private routes by JWT user ID instead of IP (`0e9e8be`)

### Community & Web (2026-03-30)

- [x] Discord server setup (Zwipers) with channel structure
- [x] Discord invite link added to zweb nav (`63a7a3d`)
- [x] GitHub webhook integration for #change-log

### Card Info Text Clipping Fix (2026-03-30)

- [x] Fix card-info text clipping on smaller screens by using flex layout (`017cdd0`)

### Zervice Rewrite (2026-03-29)

- [x] Rewrite zervice as run-once binary, flatten bin layout, add file logging (`ac8d8e1`)
- [x] Add server version to health check responses (`468c456`)
- [x] Add password rotation guide to server ops docs (`d8b0491`)

### App Store Submission Attempts (2026-03-27–28)

- [x] Add missing Info.plist keys to Dioxus.toml (`85a90f1`)
- [x] Document App Store submission errors and post-build patching (`f0bb7e1`)
- [x] Document beta Xcode rejection and Apple Support ticket (`fa49916`)
- [x] Update app icon with centered Z design (`72fdbb9`)
- [x] Add PWA icons and apple-touch-icon to zweb (`62d3b0c`)

### User Preferences & Themes (2026-03-28)

- [x] Domain layer, migration, repository (`28a9f98`)
- [x] HTTP handlers, routes, domain tests (`e507199`)
- [x] Preferences embedded in JWT claims and session response (`825980f`)
- [x] Theme system with 9 themes, preferences screen, UI polish (`7ab15a3`)
- [x] Set name on swipe screens, alphabetized theme list (`0f431ab`)
- [x] is_commander filter, rename from is_valid_commander (`80fe75b`)

### Card Filtering & Search

- [x] Produced mana card filter with frontend chip UI (`40cdb65`)
- [x] Commander search burst size increase (`657456b`)
- [x] Commander search debounce fix + loading spinner (`b8f66c5`)
- [x] Fix commander search UI: consistent no-results, dropdown animation (`1893e56`)

### UX Polish

- [x] Entrance transitions on all screens (`d456e10`, `f153b23`)
- [x] Inline submission errors → toasts (`d456e10`, `a1bff36`)
- [x] Card image preview modal on deck card list (`89368ce`)
- [x] Clear filter: clears card stack + inline clear button (`bc46b5f`, `a198d60`)
- [x] Toast word-wrap: prefer word boundaries (`67eadf1`)
- [x] Unverified email toast on login + soft limits (`cf9071c`)
- [x] Show hello and verify toasts on home screen for all flows (`fbc74c2`)
- [x] Full screen integration pass

### zweb

- [x] Design alignment — entrance animations, CSS tokens, spinner (`24704b8`)
- [x] Nav: ASCII z logo, sticky on scroll, animation on click (`351fff5`, `79f7914`, `241bf48`)
- [x] Download page for app store pending status (`5e38f78`)
- [x] Favicon (`5026d4b`)

### Infrastructure

- [x] Database backups: nightly pg_dump → Cloudflare R2, 30-day retention (`6e93dd2`, `c72e361`)
- [x] Automatic migrations in deploy pipeline (`7393c6d`)
- [x] Binary versioning: startup logs + workspace version (`7041918`)
- [x] CI/CD both pipelines live
