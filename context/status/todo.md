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

## Android — Near Submission Ready

Android build compiles and runs. Remaining polish before Play Store submission:

- [ ] Card images show white corners — the white is baked into the image data from Scryfall (white-bordered card editions). iOS clips correctly via WKWebView; Android WebView does not honor `overflow: hidden` + `border-radius` on `object-fit: contain` images. Tried: `overflow: hidden` on img, wrapper div with `border-radius` + `overflow: hidden`, `-webkit-mask-image` hack. None work on Android WebView. Options: crop with `object-fit: cover` (loses card edges), mask SVG overlay, or accept as-is for black-bordered cards (majority) and revisit for white-bordered.
- [ ] Swipe gesture doesn't tilt the card — cards should rotate slightly during drag like they do on iOS
- [ ] Lock screen orientation to portrait — need `android:screenOrientation="portrait"` on main activity. Dioxus may support this via `[android.raw.manifest]` or activity-level config. Test on Pixel once available.

---

## UX — Future

- [ ] Rethink the util bar — consider removing it and placing buttons in more natural-feeling locations per screen
- [ ] Token counting chart on deck view screen

---

## EDHREC Integration

- [ ] Salt score data import — requires scraping/syncing from EDHREC (no public API)
- [ ] Salt score display per card and aggregate per deck
- [ ] Salt score filtering and sorting on card search
- [ ] Popularity / synergy suggestions (future)

---

## Format-Aware Commander Querying

`is_commander` was removed from the database (was a persisted boolean on `card_profiles`). Commander eligibility will be computed in-memory via query filters. Each format has different rules for what constitutes a valid commander:

- [ ] **Commander / Duel / PreDH** — legendary creature, legendary vehicle/spacecraft with P/T, or "can be your commander" oracle text
- [ ] **Brawl / Standard Brawl / Historic Brawl** — legendary creature OR legendary planeswalker
- [ ] **Pauper Commander** — uncommon creature
- [ ] **Oathbreaker** — planeswalker (+ signature spell concept, future work)
- [ ] Wire format-aware filtering into commander search on create/edit screens (currently serves all cards)
- [ ] Validate commander selection against format rules and surface as deck warning if mismatched

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (269 tests). Remaining gap: outbound adapters have no coverage.

---

## Domain Extraction into `zwipe-core`

**Goal:** `zwipe-core` becomes the single source of truth for all domain types and API contract types. No feature flags — everything in core is meant to be shared.

**Why:** Today `zwiper` depends on `zerver` (with `default-features = false`) just to reuse domain types and HTTP request/response models. This pulls ~17 transitive deps zwiper doesn't need and creates a backwards dependency (client → server). After extraction:

```
zwiper ──→ zwipe-core ←── zerver
zweb  ──→ zwipe-core
```

**Migrate incrementally** — one module at a time, as each module is touched. Do not attempt a single large migration.

**Done:**
- [x] Password validation + common password dictionary
- [x] Add zweb to Cargo workspace (unified lockfile, workspace lints)
- [x] Content moderation (`ContainsBadWord` trait + ban lists)
- [x] `EmailAddress` re-export from `email_address` crate
- [x] Newtypes: `Username`, `DeckName`, `Quantity`, `UpdateQuantity`
- [x] User domain: `User`, `UserPreferences`, `UpdatePreferences`, `GetUser`
- [x] Deck domain: `Format`, `DeckProfile`, `DeckCard`, `DeckWarning`, all request types
- [x] Organize `requests/` subdirectories for user and deck modules
- [x] Remove redundant custom SQLx `Type/Encode/Decode` impls for `Format` (see `architecture/decisions.md`)

**Next:**
- [ ] Remove remaining custom SQLx impls on card domain types (Rarity, Colors, Legalities, Prices, CardFaces, AllParts, ImageUris) — replace with `Json<T>` wrapper and `String` + `TryFrom` in adapter layer
- [ ] Card domain types → zwipe-core (blocked until SQLx impls are removed)
- [ ] `Deck`, `DeckEntry` aggregate → zwipe-core (blocked on Card types)
- [ ] `validate_deck()` → zwipe-core (blocked on Card types)
- [ ] Auth domain types (`Session`, token models)
- [ ] API contract types (HTTP request/response structs)

---

## Project Structure Doc

Add a `context/architecture/structure.md` walking through the full directory tree — useful for onboarding and AI context.

---

## Maintenance

- **GitHub Actions Node.js 20 deprecation** — forced to Node.js 24 on June 2, 2026. All workflows already on latest major versions. No action needed — monitor for v5 releases.

---

## Recently Completed

### Component Extraction & Deck Enhancements (2026-04-02)

- [x] Extract deck view into profile, stats, and charts components (`995dc9e8`)
- [x] Add deck price stats with currency selection chips — USD/EUR/TIX (`9959dc16`)
- [x] Add buy deck links for TCGplayer and CardKingdom (`28326c10`)
- [x] Extract shared CardFilterSheet component from add, view, and remove screens (`9e0c6044`)
- [x] Extract CardInfoDisplay, CardSkeleton, DeckFormFields components (`249a6ed9`)
- [x] Move extracted components into components/ directories (`0a5e6bf9`)
- [x] Unify SwipeAction across add and remove screens, move to components (`2491c043`)
- [x] Add deck tokens endpoint and display on deck cards screen (`b8026582`)

### zweb & README Updates (2026-04-01)

- [x] Split download page into separate iOS and Android store pages (`b6acebe8`)
- [x] Add zwipe-core to README tech stack and architecture (`519305b6`)

### Shared Password Validation Crate (2026-04-01)

- [x] Extract password validation + common password dictionary into `zwipe-core` crate
- [x] Wire into zerver and zweb, delete duplicated code

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
- [x] Download page for app store pending status (`5e38f78`), split into separate iOS/Android pages (`b6acebe8`)
- [x] Favicon (`5026d4b`)

### Infrastructure

- [x] Database backups: nightly pg_dump → Cloudflare R2, 30-day retention (`6e93dd2`, `c72e361`)
- [x] Automatic migrations in deploy pipeline (`7393c6d`)
- [x] Binary versioning: startup logs + workspace version (`7041918`)
- [x] CI/CD both pipelines live
