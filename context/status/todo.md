# Todo

**Primary goal: Submit Zwipe to the App Store.**

---

## App Store Submission — Waiting on Apple

Submission steps are ready but blocked on Apple Support ticket (filed 2026-03-28, no response as of 2026-04-02). Revisit week of 2026-04-07.

1. **App Store Connect Setup** — Bundle ID `com.scadoshi.zwipe`, name "Zwipe"
2. **Build for Distribution** — Distribution certificate + App Store provisioning profile
3. **Submit** — Export compliance: no encryption beyond HTTPS

---

## Android — Near Submission Ready

Android build compiles and runs. Remaining polish before Play Store submission:

- [ ] Card images show white corners — the white is baked into the image data from Scryfall (white-bordered card editions). iOS clips correctly via WKWebView; Android WebView does not honor `overflow: hidden` + `border-radius` on `object-fit: contain` images. Tried: `overflow: hidden` on img, wrapper div with `border-radius` + `overflow: hidden`, `-webkit-mask-image` hack. None work on Android WebView. Options: crop with `object-fit: cover` (loses card edges), mask SVG overlay, or accept as-is for black-bordered cards (majority) and revisit for white-bordered.
- [ ] Swipe gesture doesn't tilt the card — cards should rotate slightly during drag like they do on iOS
- [ ] Lock screen orientation to portrait — need `android:screenOrientation="portrait"` on main activity. Dioxus may support this via `[android.raw.manifest]` or activity-level config. Test on Pixel once available.

---

## EDHREC Integration

Closed API — must request access at edhrec.com/api. Full scope pending what they expose.

- [ ] Request API access
- [ ] Salt score import, display per card and aggregate per deck, filtering and sorting on card search
- [ ] Synergy scores — surface cards with high synergy to the deck's commander
- [ ] Popularity data — most-played cards for a given commander
- [ ] Evaluate other EDHREC data (themes, combos, etc.) once API access granted

---

## Commander System

Commander eligibility is computed via query filters, not persisted. Three phases, each building on the last.

### Phase 1: Commander Search Filter + Validation — Complete

- [x] Add `is_commander_in_format: Option<Format>` to `CardFilter` / `CardFilterBuilder` (zwipe-core) (`3e74f8a0`)
- [x] Implement eligibility logic — pure function per format rules (zwipe-core) (`3e74f8a0`)
- [x] Backend: apply filter in SQL query (`3e74f8a0`)
- [x] Frontend: commander eligibility chips in format filter section (`be93812b`)
- [x] Frontend: commander filter toggle on create/edit screens, format-first layout (`3e3fadce`)
- [x] Multi-select on format legality filter chips (`f751cbd4`)
- [x] `validate_deck()`: warning if selected commander is not valid for format (`3e74f8a0`)

### Phase 2: Partner, Background, and Oathbreaker Support — Complete

- [x] Add `partner_commander_id`, `background_id`, `signature_spell_id` columns to decks table (`e08a218c`)
- [x] DeckProfile, DatabaseDeckProfile, HTTP contracts, request types updated (`e08a218c`)
- [x] Partner search filter (`is_partner`), background filter (`is_background`), signature spell filter (`is_signature_spell`) on CardFilter (`e08a218c`)
- [x] `partner_kind()`, `are_valid_partners()`, background/signature spell eligibility functions (`e08a218c`)
- [x] validate_deck: partner validity, background validity, signature spell validity, mutual exclusivity, color identity union (`e08a218c`)
- [x] Frontend: partner, background, signature spell fields on create/edit with conditional visibility (`eb1a68d9`)
- [x] Oathbreaker label refinements (`eb1a68d9`)
- [x] Card view: partner pinned in commander group as "commanders", background in own group (`1bf5e7a7`)
- [x] All four command zone cards filtered alongside deck entries (`1bf5e7a7`)
- [x] Partner and background search threshold lowered to 1 character (`1bf5e7a7`)

### Phase 3: Zwipe for Commander (UX Enhancement)

A dedicated swiping flow for commander selection. Future work — only build if users want it.

**Concept:**
- [ ] On create/edit screen, when format has a commander, show "Zwipe for Commander" button
- [ ] Opens the swiping interface pre-filtered to valid commanders for the selected format
- [ ] User can adjust filters (colors, mana cost, set, etc.) and swipe through candidates
- [ ] First swipe-right sets the commander and returns to the create/edit screen
- [ ] Format filter defaults to deck's format but user can change it
- [ ] Works on both create and edit screens

---

## Maybeboard — Complete

All 5 phases shipped. See `context/plans/maybeboard-phase*.md` for original plans.

- [x] Add `maybeboard: bool` column to `deck_cards`, exclude from metrics/validation/card_count (`75502526`)
- [x] Up-swipe adds to maybeboard on add and remove screens, undo support (`c41dae16`)
- [x] Deck card view: maybeboard toggle, "to deck"/"to maybeboard" move buttons (`cb6db5c3`)
- [x] Remove screen: tri-state maybeboard filter (no/yes/any) (`e6016305`)
- [x] Export toggle "include maybeboard", import `// Maybeboard` header, buy links exclude by default (`15c38980`)

---

## Sideboard

Add sideboard support alongside maybeboard. A card lives in exactly one state: **deck** (active), **maybeboard** (considering), or **sideboard** (game-ready swaps between matches). Mirrors maybeboard implementation but with format-specific rules.

**Data model:**
- [ ] Migrate from `maybeboard: bool` to a `board` enum column (`deck`, `maybeboard`, `sideboard`) — or add `sideboard: bool` alongside `maybeboard` with a constraint that both can't be true
- [ ] Sideboard cards excluded from deck metrics, validation, and card_count (same as maybeboard)
- [ ] Format-specific sideboard validation: 15-card limit for 60-card formats (Standard, Modern, Legacy, Vintage), no sideboard for Commander/Brawl/Oathbreaker

**Swipe screens:**
- [ ] All four swipe directions are taken (left=skip, right=add/remove, up=maybeboard, down=undo) — need a different UX for sideboard assignment (long-press menu? button in expanded card info? post-add dialog?)
- [ ] Remove screen: sideboard cards accessible via tri-state filter (deck/maybeboard/sideboard/all)

**Deck view screen:**
- [ ] "sideboard" toggle chip alongside "maybeboard" chip
- [ ] Sideboard section rendered between maybeboard and commander
- [ ] "to sideboard" / "to deck" move buttons in CardRow expanded view

**Remove screen:**
- [ ] Extend MaybeboardFilter to include Sideboard variant
- [ ] Swipe behavior: swipe right removes from sideboard (same delete endpoint)

**Export/Import:**
- [ ] Export: `// Sideboard` header (standard format recognized by Moxfield, Archidekt, MTGO)
- [ ] Import: detect `// Sideboard` header, tag cards accordingly

---

## Mechanical Category

Multi-tag strategic role system for cards (Ramp, Draw, Removal, etc.). Cards can have multiple categories. Taxonomy: 24 categories defined. See `context/plans/mechanical-category.md` for full plan.

**Taxonomy (finalized):** Ramp, Draw, Removal, Wipe, Counterspell, Protection, Evasion, Finisher, Tokens, Lifegain, Blink, Recursion, Mill, Burn, Drain, Pump, Anthem, Counters, Copy, Sacrifice, Stax, Untap, Tutor, Graveyard Hate

**Schema + Domain:**
- [ ] `MechanicalCategory` enum (24 variants) in zwipe-core
- [ ] `mechanical_categories: JSONB` on `card_profiles` table (GIN indexed) — update existing migration
- [ ] `CardProfile.mechanical_categories: Vec<MechanicalCategory>`

**Classification (three layers):**
- [ ] Layer 1: Runtime oracle text heuristics — `classify_by_heuristics()` pure function, ~70-80% accuracy, runs during Scryfall sync
- [ ] Layer 2: AI classification client — standalone binary, reads DB in batches, LLM-classifies, writes tags back. Corrects heuristic errors. ~90-95% accuracy
- [ ] Layer 3 (future): Fine-tuned lightweight model trained on Layer 2's corrected data. ~95-99% accuracy. Embeddable in sync pipeline

**Filtering + Grouping:**
- [ ] `CardFilter`: `mechanical_categories_contains_any/all` with `?|`/`@>` SQL operators
- [ ] `GroupByOption::Category` — multi-bucket grouping (card appears in every matching group)
- [ ] `DeckMetrics.category_counts` — breakdown per category

**Frontend:**
- [ ] Category filter section in CardFilterSheet (chip-based multi-select)
- [ ] "category" grouping chip on deck card view
- [ ] Category breakdown in deck stats

---

## Deck View Polish

Small UX improvements to the deck view screen and related flows.

- [x] Update "average price per card" label → "avg card price" (`e16bd01d`)
- [x] Toast "card removed" on quantity-decrement-to-zero (`bd860bce`)
- [x] Toast "card removed" on warning remove button (`bd860bce`)
- [x] "fix to N" button on copy limit warnings (`bd860bce`)
- [x] "clear" button on invalid commander warning — sends update_deck_profile to clear commander (`bd860bce`)
- [x] WarningAction enum (FixQuantity, ClearCommander, Remove) on DeckWarning (`bd860bce`)
- [x] Card count in deck stats includes commander/partner/background/spell (`bd860bce`)
- [x] Clear filter button on filter groups — per-section clear buttons on accordion headers (`0e381a53`)
- [x] Fix: `is_commander_in_format` alone now counts as non-empty filter (`0e381a53`)
- [x] Fix: remove screen deck load failure (`0e381a53`)
- [x] "command zone" show toggle on deck card view (`36da3374`)
- [x] Clear commander on format change to prevent stale selections (`128bbeea`)

---

## Theme Audit & Color System

Full audit of all 9 themes to make the app more colorful and ensure visual consistency.

**Steps:**
1. **Define Zwipe color scheme** — select distinct, vibrant colors for the default theme
2. **Audit CSS variable usage** — map which variables apply where across the app (backgrounds, borders, text, accents, buttons). Ensure semantic consistency: `--primary-border` should mean the same thing on every screen
3. **Ensure contrast consistency** — every theme must meet the same contrast ratios (e.g., text on background, border against background). Document the target contrast rules
4. **Per-theme color pass** — go through each of the 9 themes, ensure they have ample color variety within their flavor (not just 2 shades of one hue). Adjust to meet contrast rules from step 3
5. **Full visual test** — test every theme on every screen, tweak as needed

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (308+ tests, ~100 in zwipe-core). Remaining gap: outbound adapters have no coverage.

---

## Domain Extraction into `zwipe-core`

**Complete.** `zwipe-core` is the single source of truth for all shared types. Proxy re-export cleanup complete — ~35 proxy files deleted, ~200 import rewrites across zerver and zwiper. `zwipe-core` is a direct dependency of both zwiper and zite. Zerver only owns server-specific code (error types, Password/HashedPassword, JwtSecret, ports, services, database adapters). See `architecture/decisions.md` for the full rationale and purity rules.

---

## Project Structure Doc — Complete

`context/architecture/structure.md` — full directory tree, crate dependency graph, database schema, key patterns.

---

## Low Priority

- [ ] **Token cards missing from database** — Some token cards referenced in `all_parts` don't exist in `scryfall_data` (e.g. Whirler Virtuoso's Thopter token). Fix: switch Scryfall sync from `oracle_cards` to `default_cards` to include tokens and all printings. Search already deduplicates via `ROW_NUMBER() OVER (PARTITION BY oracle_id ORDER BY released_at DESC)`.

- [ ] **Multi-printing support + oracle_id deck constraint** — Importing `default_cards` introduces multiple printings per card. Deck constraint must change:
  - `deck_cards` unique constraint switches from `(deck_id, scryfall_data_id)` to `(deck_id, oracle_id)` — one logical card per deck regardless of printing
  - `deck_cards` keeps `scryfall_data_id` as FK (points to the selected printing)
  - Add `oracle_id` column to `deck_cards` (denormalized from scryfall_data for constraint enforcement)
  - Default printing: latest by `released_at DESC` — same logic as search dedup
  - Future: printing selector UI so users can pick preferred art/set
  - Requires updating create_deck_card, import, and bulk upsert to resolve oracle_id before insert

---

## Maintenance

- **GitHub Actions Node.js 20 deprecation** — forced to Node.js 24 on June 2, 2026. All workflows already on latest major versions. No action needed — monitor for v5 releases.

---

## Recently Completed

### Partner, Background & Signature Spell (2026-04-04)

- [x] Database columns, DeckProfile, HTTP contracts, request types, repository queries (`e08a218c`)
- [x] CardFilter: is_partner, is_background, is_signature_spell with SQL filters (`e08a218c`)
- [x] Eligibility functions: partner_kind, are_valid_partners, background/spell validation (`e08a218c`)
- [x] validate_deck: partner, background, signature spell, mutual exclusivity, color identity union (`e08a218c`)
- [x] Frontend: conditional field visibility on create/edit, oathbreaker labels (`eb1a68d9`)
- [x] Card view: partner in commander group, background in own group, all command zone filtered (`1bf5e7a7`)
- [x] Partner/background search threshold lowered to 1 char (`1bf5e7a7`)

### Small Bugs + Filter UX (2026-04-04)

- [x] Fix commander filter empty check — is_commander_in_format counts as non-empty (`0e381a53`)
- [x] Per-section clear buttons on filter accordion headers (`0e381a53`)
- [x] Fix remove screen deck load failure (`0e381a53`)

### Deck View Polish + UX Fixes (2026-04-04)

- [x] WarningAction enum: FixQuantity, ClearCommander, Remove — per-warning action buttons (`bd860bce`)
- [x] "fix to N" on copy limit, "clear" on invalid commander, "card removed" toasts (`bd860bce`)
- [x] Card count includes commander/partner/background/spell in stats (`bd860bce`)
- [x] Rename Optdate → Opdate across codebase (`cfd19ab7`)
- [x] Command zone show toggle on deck card view (`36da3374`)
- [x] Clear commander on format change to prevent stale selections (`128bbeea`)
- [x] Architecture structure doc: `context/architecture/structure.md`

### Maybeboard (2026-04-04)

- [x] Add `maybeboard: bool` to deck card pipeline — migration, model, metrics/validation exclusion, card_count filter (`75502526`)
- [x] Swipe-up to maybeboard on add and remove screens with undo support, card-exit-up animation (`c41dae16`)
- [x] Deck card view: maybeboard toggle, section rendering, "to deck"/"to maybeboard" move buttons via update_deck_card (`cb6db5c3`)
- [x] Remove screen: tri-state maybeboard filter (no/yes/any) in config section (`e6016305`)
- [x] Export toggle "include maybeboard" with `// Maybeboard` header, import detects header, buy links exclude maybeboard with toggle (`15c38980`)

### Rename zweb → zite (2026-04-04)

- [x] Rename web client crate from zweb to zite — directory, Cargo.toml, workflow, all docs (`2b11fd3b`)

### Commander Filter System (2026-04-04)

- [x] Add `is_commander_in_format` filter to CardFilter with per-format eligibility rules (`3e74f8a0`)
- [x] Commander eligibility chips in format filter section (`be93812b`)
- [x] Commander filter toggle on create/edit screens with format-first layout (`3e3fadce`)
- [x] Multi-select on format legality filter chips (`f751cbd4`)
- [x] Update avg price label (`e16bd01d`)

### Proxy Re-export Cleanup (2026-04-04)

- [x] Remove logo and moderation proxy modules from zerver (`02e01bda`)
- [x] Remove HTTP paths and helpers proxy files from zerver (`05ce80c1`)
- [x] Clean up auth domain proxy re-exports in zerver (`27286090`)
- [x] Migrate zwiper Session imports from zerver proxy to zwipe-core (`845be76f`)
- [x] Clean up deck domain proxy re-exports in zerver and zwiper (`d5395b22`)
- [x] Migrate zwiper card imports from zerver proxy to zwipe-core (`e3c72218`)
- [x] Clean up user domain proxy re-exports (`b2903f2e`)
- [x] Clean up card domain proxy re-exports in zerver and zwiper (`da5fbc36`)
- [x] Downgrade handler pub use to use, migrate zwiper Http imports to zwipe-core (`75fa3208`)
- [x] Add zwipe-core as direct dependency of zwiper — frontend no longer routes domain types through zerver

### zwipe-core Domain Extraction (2026-04-02)

- [x] Extract newtypes + moderation into zwipe-core (`6d75e675`)
- [x] Extract User, UserPreferences, GetUser (`2b4201d7`)
- [x] Extract deck + deck_card domain types (`b8dc8836`)
- [x] Document SQLx adapter pattern decision (`8d3ea8eb`)
- [x] Replace custom SQLx impls with DatabaseScryfallData adapter (`a9618e4b`)
- [x] Extract Card, CardProfile, ScryfallData + all nested types (`7dc2e487`)
- [x] Extract CardFilter, search types (`75670892`)
- [x] Extract Deck/DeckEntry aggregate, validate_deck, DeckMetrics (`98982af3`)
- [x] Add models/ directories to zwipe-core modules (`7ef56603`)
- [x] Separate requests/ from models/ in zerver auth and card (`d71ef8e8`)
- [x] Extract Session, AccessToken, RefreshToken, Jwt (`38617714`)
- [x] Extract logo module to zwipe-core (`32fc23ba`)
- [x] Extract HTTP contract types, paths, ApiError, Opdate (`fab717c1`)

### Component Extraction & Deck Enhancements (2026-04-02)

- [x] Extract deck view into profile, stats, and charts components (`995dc9e8`)
- [x] Add deck price stats with currency selection chips — USD/EUR/TIX (`9959dc16`)
- [x] Add buy deck links for TCGplayer and CardKingdom (`28326c10`)
- [x] Extract shared CardFilterSheet component from add, view, and remove screens (`9e0c6044`)
- [x] Extract CardInfoDisplay, CardSkeleton, DeckFormFields components (`249a6ed9`)
- [x] Move extracted components into components/ directories (`0a5e6bf9`)
- [x] Unify SwipeAction across add and remove screens, move to components (`2491c043`)
- [x] Add deck tokens endpoint and display on deck cards screen (`b8026582`)

### zite & README Updates (2026-04-01)

- [x] Split download page into separate iOS and Android store pages (`b6acebe8`)
- [x] Add zwipe-core to README tech stack and architecture (`519305b6`)

### Shared Password Validation Crate (2026-04-01)

- [x] Extract password validation + common password dictionary into `zwipe-core` crate
- [x] Wire into zerver and zite, delete duplicated code

### Per-User Rate Limiting (2026-03-30)

- [x] Custom `UserIdKeyExtractor` for tower_governor, keys private routes by JWT user ID instead of IP (`0e9e8be`)

### Community & Web (2026-03-30)

- [x] Discord server setup (Zwipers) with channel structure
- [x] Discord invite link added to zite nav (`63a7a3d`)
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
- [x] Add PWA icons and apple-touch-icon to zite (`62d3b0c`)

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

### zite

- [x] Design alignment — entrance animations, CSS tokens, spinner (`24704b8`)
- [x] Nav: ASCII z logo, sticky on scroll, animation on click (`351fff5`, `79f7914`, `241bf48`)
- [x] Download page for app store pending status (`5e38f78`), split into separate iOS/Android pages (`b6acebe8`)
- [x] Favicon (`5026d4b`)

### Infrastructure

- [x] Database backups: nightly pg_dump → Cloudflare R2, 30-day retention (`6e93dd2`, `c72e361`)
- [x] Automatic migrations in deploy pipeline (`7393c6d`)
- [x] Binary versioning: startup logs + workspace version (`7041918`)
- [x] CI/CD both pipelines live
