# Completed Work — 2026 Q1 + early Q2

Archived from `context/status/todo.md` on 2026-05-27. Everything below is shipped. Kept here so commit hashes stay searchable.

---

## Clone Deck — Complete

Full-stack deck duplication: `POST /decks/{id}/clone`, copies profile + all entries in a single transaction, clone button on deck view with editable name prompt.

Commander eligibility is computed via query filters, not persisted.

### Phase 1: Commander Search Filter + Validation

- [x] Add `is_commander_in_format: Option<Format>` to `CardFilter` / `CardFilterBuilder` (zwipe-core) (`2db7751a`)
- [x] Implement eligibility logic — pure function per format rules (zwipe-core) (`2db7751a`)
- [x] Backend: apply filter in SQL query (`2db7751a`)
- [x] Frontend: commander eligibility chips in format filter section (`48e5087e`)
- [x] Frontend: commander filter toggle on create/edit screens, format-first layout (`a6fe3a50`)
- [x] Multi-select on format legality filter chips (`f37177ed`)
- [x] `validate_deck()`: warning if selected commander is not valid for format (`2db7751a`)

### Phase 2: Partner, Background, and Oathbreaker Support

- [x] Add `partner_commander_id`, `background_id`, `signature_spell_id` columns to decks table (`503eed11`)
- [x] DeckProfile, DatabaseDeckProfile, HTTP contracts, request types updated (`503eed11`)
- [x] Partner search filter (`is_partner`), background filter (`is_background`), signature spell filter (`is_signature_spell`) on CardFilter (`503eed11`)
- [x] `partner_kind()`, `are_valid_partners()`, background/signature spell eligibility functions (`503eed11`)
- [x] validate_deck: partner validity, background validity, signature spell validity, mutual exclusivity, color identity union (`503eed11`)
- [x] Frontend: partner, background, signature spell fields on create/edit with conditional visibility (`c8b6a219`)
- [x] Oathbreaker label refinements (`c8b6a219`)
- [x] Card view: partner pinned in commander group as "commanders", background in own group (`6a1b1198`)
- [x] All four command zone cards filtered alongside deck entries (`6a1b1198`)
- [x] Partner and background search threshold lowered to 1 character (`6a1b1198`)

---

## Maybeboard — Complete

All 5 phases shipped. See `context/plans/maybeboard-phase*.md` for original plans.

- [x] Add `maybeboard: bool` column to `deck_cards`, exclude from metrics/validation/card_count (`224ad721`)
- [x] Up-swipe adds to maybeboard on add and remove screens, undo support (`0aed2c1b`)
- [x] Deck card view: maybeboard toggle, "to deck"/"to maybeboard" move buttons (`9e8548b3`)
- [x] Remove screen: tri-state maybeboard filter (no/yes/any) (`afbb1c12`)
- [x] Export toggle "include maybeboard", import `// Maybeboard` header, buy links exclude by default (`689f9b22`)

---

## Sideboard — Complete

All phases shipped: Board enum, migration, validation, deck view UI, remove screen, export/import.

- [x] `Board` enum (`Deck`, `Maybeboard`, `Sideboard`) replacing `maybeboard: bool` across domain (`e0e53fe1`)
- [x] Migration from `maybeboard` boolean to `board` text column (`5ab38460`)
- [x] Sideboard toggle chip, section rendering, move buttons, BoardFilter (`b88cb37a`)
- [x] Format-specific sideboard validation, export/import with `// Sideboard` header (`b88cb37a`)

---

## Mechanical Category — Phases 1+2 Complete

Multi-tag strategic role system for cards (Ramp, Draw, Removal, etc.). 24 categories defined. ~73% classification rate from heuristics (79k / 108k cards). Heuristic refinement is now tracked in `status/todo.md`; Layers 2 + 3 are tracked in `status/backlog.md`.

**Schema + Domain:**
- [x] `MechanicalCategory` enum (24 variants) with `to_short_name()`, `display_name()`, serde, TryFrom (`a717165c`)
- [x] `mechanical_categories: JSONB` on `card_profiles` table with GIN index (`a717165c`)
- [x] `CardProfile.mechanical_categories: Vec<MechanicalCategory>` (`a717165c`)
- [x] `classify_by_heuristics()` pure function with regex patterns for all 24 categories (`a717165c`)
- [x] Batched post-sync classification in zervice (1000 cards/batch) (`a717165c`)
- [x] `--recategorize` / `-rc` flag for full reclassification (`a717165c`)
- [x] Renamed SyncMetrics → ZerviceMetrics, DB table → zervice_metrics (`a717165c`)

**Filtering + Grouping:**
- [x] `CardFilter`: `mechanical_categories_contains_any/all` with `?|`/`@>` SQL operators
- [x] Client-side filtering via `card.card_profile.mechanical_categories`
- [x] `GroupByOption::Category` — multi-bucket grouping (card appears in every matching group)
- [x] `DeckMetrics.mechanical_category_counts` — breakdown per category, sorted by count desc

**Frontend:**
- [x] Category filter section in CardFilterSheet (24-chip multi-select with any/all toggle)
- [x] "category" grouping chip on deck card view
- [x] Category distribution horizontal bar chart in deck stats

---

## Deck View Polish — Complete

Small UX improvements to the deck view screen and related flows.

- [x] Update "average price per card" label → "avg card price" (`abed4658`)
- [x] Toast "card removed" on quantity-decrement-to-zero (`e844395e`)
- [x] Toast "card removed" on warning remove button (`e844395e`)
- [x] "fix to N" button on copy limit warnings (`e844395e`)
- [x] "clear" button on invalid commander warning — sends update_deck_profile to clear commander (`e844395e`)
- [x] WarningAction enum (FixQuantity, ClearCommander, Remove) on DeckWarning (`e844395e`)
- [x] Card count in deck stats includes commander/partner/background/spell (`e844395e`)
- [x] Clear filter button on filter groups — per-section clear buttons on accordion headers (`81be0bac`)
- [x] Fix: `is_commander_in_format` alone now counts as non-empty filter (`81be0bac`)
- [x] Fix: remove screen deck load failure (`81be0bac`)
- [x] "command zone" show toggle on deck card view (`f1f73ac2`)
- [x] Clear commander on format change to prevent stale selections (`8e9eb432`)
- [x] Toast on import completion with chip-bubble result styling (`a915631f`)
- [x] Replace format chip grid with typeahead input on create/edit deck screens (`9dce9d20`)
- [x] Fix import card limit check double-counting upserted cards, split verified/unverified error messages (`46ee3b85`)
- [x] Sort maybeboard, sideboard, and tokens on deck card view — sorting now persists across all board sections (`3073707b`)
- [x] Add source selector to add screen: "search" (API) vs "maybeboard" (swipe through maybeboard to promote to deck) with smart filter defaults per mode (`e68c9e68`)
- [x] Lowercase filter chip display for set, artist, keywords, oracle words (`3f5cb034`)
- [x] Punctuation-insensitive text search — `strip_punctuation()` utility, trim at build() time, `regexp_replace` on server SQL for name, oracle text, type line, flavor text (`7266fd3a`)
- [x] 13 exclusion filter fields (not-contains + excludes) across full stack with include/exclude UI toggles (`6a4b1ce9`)
- [x] Switch 6 metadata queries (types, keywords, oracle words, artists, sets, languages) from `scryfall_data` (110k) to `latest_cards` (35k) — oracle words drops from ~1.5s to under 1s (`6a4b1ce9`)
- [x] Add `tracing::info!` logging to all 33 client methods in zwiper (`6a4b1ce9`)

---

## Theme Audit & Color System — Complete

Originally 9 themes, now 15 (including 3 colorblind-accessible).

- [x] Define Zwipe color scheme — slate blue-grey bg, magenta off-white text, muted blue accents (`28559812`)
- [x] Audit CSS variable usage — semantic consistency across all screens (`28559812`)
- [x] Add 3-variable accent system (`--accent-primary/secondary/tertiary`) to every theme (`28559812`)
- [x] Normalize contrast ratios across all themes — text-muted/text-subtle adjusted (`28559812`)
- [x] Fix shadow syntax, remove selection color inversions, border semantic audit (`34fbb716`)
- [x] Add colorblind accessible themes: protanopia, deuteranopia, tritanopia (dark/light) (`4d8929bd`)
- [x] Add monokai, one dark, solarized themes (dark/light) (`a9d76086`)
- [x] Move ThemeConfig from zwiper to zwipe-core for cross-crate sharing (`39c235c4`)
- [x] Add all 15 dark themes to zite with live theme picker in footer (`39c235c4`)
- [x] Sync zite colors with app defaults, remove color-inverting hover states (`39c235c4`)
- [x] Extract shared `themes.css` — single source of truth, copied into both projects via `build.rs` at compile time, gitignored as build artifacts
- [x] Add light mode toggle to zite theme picker — dark/light button appears for themes that support it (`40ac8b19`)
- [x] Update zite content: home, ios, android pages reflect 15 themes with dark/light modes (`40ac8b19`)
- [x] Match hero store buttons to nav store link hover style (`801c484e`)
- [x] Full visual test — every theme on every screen

---

## Multi-Printing — Complete

Carousel UI with swipe-to-browse, page dots, save/close header, and command zone printing selection.

- [x] Switch Scryfall sync from `oracle_cards` to `default_cards` (~110k+ cards including tokens) (`590a8b46`)
- [x] Add `oracle_id UUID NOT NULL` to `deck_cards`, unique constraint `(deck_id, oracle_id)` (`590a8b46`)
- [x] All SQL queries, domain models, HTTP contracts, import flow, frontend add/remove updated (`590a8b46`)
- [x] Printing carousel with snap-to-page, edge bounce, page dots, info row (`7b3b4aa7`)
- [x] Save/close header — save appears only after swiping to a different printing (`7b3b4aa7`)
- [x] Refactored PrintingSheet to generic `on_save` callback (`5a3336c1`)
- [x] Command zone printing selection — commander, partner, background, signature spell (`5a3336c1`)
- [x] Oracle ID audit — all card identity comparisons use oracle_id (`d1e6cb2c`)
- [x] Fix carousel dots invisible (wrong CSS variable), add centered scrolling dots with edge fade (`4a9a0a39`)
- [x] Add artist to printing sheet info, reuse CardInfoDisplay component (`65b83966`, `4a9a0a39`)
- [x] Printing saved/discarded toasts, fix printing info height jumping (`5bd7d629`)

---

## Search Query Performance — Complete

See `context/plans/search-performance.md`.

- [x] `pg_trgm` extension + GIN trigram indexes on `name`, `oracle_text`, `type_line`
- [x] Replace `color_identity_within` power-set OR explosion with single `<@` operator + GIN index
- [x] Eliminate double table read (subsumed by materialized view)
- [x] `latest_cards` materialized view — pre-deduplicated to latest printing per oracle_id, refreshed by zervice after sync + classification

---

## Add Screen — Default Color Identity Filter — Complete

Pre-populate the color identity filter to the commander's colors when the deck's format enforces color identity. See `context/plans/add-screen-color-identity-default.md`.

- [x] Extend `is_empty_ignoring_deck_context()` to also ignore `color_identity_within`
- [x] Resolve commander + partner + background color identity union on mount
- [x] Auto-set `color_identity_within` if not already set
- [x] Re-apply color identity default on filter clear (alongside legality re-apply)
- [x] Cache resolved colors in `deck_color_identity` signal
- [x] Clear default filters on back navigation so view/remove screens start fresh
- [x] Add `Eq, PartialOrd, Ord, Hash` to `Color` enum (WUBRG ordering)

---

## Domain Extraction into `zwipe-core` — Complete

`zwipe-core` is the single source of truth for all shared types. Proxy re-export cleanup complete — ~35 proxy files deleted, ~200 import rewrites across zerver and zwiper. `zwipe-core` is a direct dependency of both zwiper and zite. Zerver only owns server-specific code (error types, Password/HashedPassword, JwtSecret, ports, services, database adapters). See `architecture/decisions.md` for the full rationale and purity rules.

---

## Project Structure Doc — Complete

`context/architecture/structure.md` — full directory tree, crate dependency graph, database schema, key patterns.

---

## Per-User Rate Limiting (2026-03-30)

- [x] Custom `UserIdKeyExtractor` for tower_governor, keys private routes by JWT user ID instead of IP (`c3a1b85c`). Private routes governor configs in `inbound/http/routes.rs` are all keyed by user ID; pre-auth routes remain IP-keyed (deliberate — no user_id exists yet). Credential-stuffing layer keyed by submitted email tracked in `backlog.md`.

---

## Sub-feature timeline (2026-04-01 to 2026-04-05)

### Theme Audit & Zite Theme Handling (2026-04-05)

- [x] Theme audit phases 1-3: fix variable bugs, normalize contrast ratios, add accent system, rework Zwipe default identity (`28559812`)
- [x] Theme phases 4-5: fix shadow syntax, remove selection color inversions, border semantic audit (`34fbb716`)
- [x] Add colorblind accessible themes: protanopia, deuteranopia, tritanopia (dark/light) (`4d8929bd`)
- [x] Add monokai, one dark, solarized themes (dark/light), register in ALLOWED_THEMES (`a9d76086`)
- [x] Mute zwipe accent-tertiary to softer gold palette (`eeff85af`)
- [x] Move ThemeConfig from zwiper/domain/theme.rs to zwipe-core/domain/user/models/theme.rs (`39c235c4`)
- [x] Update 6 zwiper import sites to use shared ThemeConfig from zwipe-core (`39c235c4`)
- [x] Sync zite CSS with app's new Zwipe default identity — bg, text, border, accent variables (`39c235c4`)
- [x] Add all 15 dark theme CSS classes to zite (no light variants — dark-only website aesthetic) (`39c235c4`)
- [x] Add live theme picker in zite footer using ALLOWED_THEMES from zwipe-core (`39c235c4`)
- [x] Apply theme class to body via eval for full-page CSS variable propagation (`39c235c4`)
- [x] Remove color-inverting hover states on zite buttons — no more bg fill + dark text flip (`39c235c4`)
- [x] Extract `shared/themes.css` — single source of truth for all theme CSS variables, copied into both projects via `build.rs` at compile time, gitignored as build artifacts
- [x] Add light mode toggle to zite theme picker (`40ac8b19`)
- [x] Update zite home, ios, android page content — 15 themes, sideboard, multi-printing (`40ac8b19`)
- [x] Match hero store buttons to nav store link hover style — accent text/border at rest, text-primary on hover (`801c484e`)

### Deck View Polish + Printing Sheet + Carousel Dots (2026-04-05)

- [x] Replace format chip grid with typeahead input on create/edit deck screens (`9dce9d20`)
- [x] Import completion toasts with chip-bubble result styling (`a915631f`)
- [x] Fix import card limit check double-counting upserted cards, split verified/unverified error messages (`46ee3b85`)
- [x] Add artist to printing sheet info, reuse CardInfoDisplay (deduplicate `printing_info`) (`65b83966`, `4a9a0a39`)
- [x] Fix carousel dots invisible — `--color-text` CSS variable didn't exist, replaced with `--text-primary` (`4a9a0a39`)
- [x] Carousel dots: centered scrolling track with edge fade mask, `flex-shrink: 0` to prevent squishing (`4a9a0a39`)
- [x] Printing saved/discarded toasts, fix printing info height jumping between printings (`5bd7d629`)
- [x] Fix card info height jumping on add/remove swipe screens when prices or artist absent (`5665f94b`)
- [x] Fix qty change collapsing expanded card, fix board filter showing deck when only sideboard/maybeboard selected (`84540091`)
- [x] Bottom-sheet carousel image max-height reduced to 38vh to fit with 4-line card info (`4a9a0a39`)

### Search Query Performance (2026-04-05)

- [x] `pg_trgm` extension + GIN trigram indexes on `name`, `oracle_text`, `type_line`
- [x] Replace `color_identity_within` power-set (up to 31 OR clauses) with single `<@` operator
- [x] `latest_cards` materialized view: `DISTINCT ON` pre-deduplication (~35k rows vs 110k), refreshed by zervice after sync + classification
- [x] Both `search_scryfall_data` and `find_cards_by_exact_names` rewritten to query the view
- [x] Trigram + GIN indexes on the view for ILIKE and color identity searches

### Add Screen Default Color Identity Filter (2026-04-05)

- [x] `is_empty_ignoring_deck_context()` also ignores `color_identity_within`
- [x] Resolve commander + partner + background color identity union on mount, auto-set filter
- [x] Re-apply color identity on filter clear, cache in `deck_color_identity` signal
- [x] Clear default filters on back navigation so view/remove screens start fresh
- [x] `Color` enum: added `Eq, PartialOrd, Ord, Hash` derives (WUBRG ordering)

### Sideboard Data Model + UI (2026-04-04)

- [x] Board enum (Deck, Maybeboard, Sideboard) replacing maybeboard bool across domain (`e0e53fe1`)
- [x] Migration from maybeboard boolean to board text column (`5ab38460`)
- [x] Sideboard toggle chip, section rendering, move buttons, BoardFilter (`b88cb37a`)

### Oracle ID Audit + Printing Carousel + Command Zone Printing (2026-04-04)

- [x] Printing carousel: snap-to-page swipe, edge bounce, page dots, info row, save/close header (`7b3b4aa7`)
- [x] Unified bottom sheet layout with util-bar footers and header labels (`7b3b4aa7`)
- [x] Command zone printing selection: generic on_save callback, printing button on commander/partner/bg/spell (`5a3336c1`)
- [x] Oracle ID audit: all card identity comparisons resolve to oracle_id instead of scryfall_data_id (`d1e6cb2c`)

### Zite Content Refresh + CSS Migration (2026-04-04)

- [x] Update home, about, ios, android, privacy page content to reflect shipped features (`16073ab9`)
- [x] Convert all px measurements to rem for accessible scaling (`5b522d6d`)

### Mechanical Category System (2026-04-04)

- [x] MechanicalCategory enum (24 variants), JSONB column with GIN index, classify_by_heuristics() (`a717165c`)
- [x] Batched post-sync classification in zervice, --recategorize flag (`a717165c`)
- [x] CardFilter: mechanical_categories_contains_any/all, GroupByOption::Category, DeckMetrics.mechanical_category_counts (`4b809ebd`)
- [x] Frontend: 24-chip category filter section, category grouping chip, category bar chart in stats (`4b809ebd`)

### Multi-Printing Phase 1+2 (2026-04-04)

- [x] Switch sync from oracle_cards to default_cards (~110k+ cards, tokens included) (`590a8b46`)
- [x] Add oracle_id to deck_cards with UNIQUE(deck_id, oracle_id) constraint (`590a8b46`)
- [x] All SQL queries updated (create, get, update RETURNING, bulk import ON CONFLICT) (`590a8b46`)
- [x] Domain model, HTTP contracts, CreateDeckCard request, handler — oracle_id plumbed through (`590a8b46`)
- [x] Import flow deduplicates by oracle_id instead of scryfall_data_id (`590a8b46`)
- [x] Frontend add screen tracks oracle_id in exclusion set, prevents adding different printings of same card (`590a8b46`)

### Partner, Background & Signature Spell (2026-04-04)

- [x] Database columns, DeckProfile, HTTP contracts, request types, repository queries (`503eed11`)
- [x] CardFilter: is_partner, is_background, is_signature_spell with SQL filters (`503eed11`)
- [x] Eligibility functions: partner_kind, are_valid_partners, background/spell validation (`503eed11`)
- [x] validate_deck: partner, background, signature spell, mutual exclusivity, color identity union (`503eed11`)
- [x] Frontend: conditional field visibility on create/edit, oathbreaker labels (`c8b6a219`)
- [x] Card view: partner in commander group, background in own group, all command zone filtered (`6a1b1198`)
- [x] Partner/background search threshold lowered to 1 char (`6a1b1198`)

### Small Bugs + Filter UX (2026-04-04)

- [x] Fix commander filter empty check — is_commander_in_format counts as non-empty (`81be0bac`)
- [x] Per-section clear buttons on filter accordion headers (`81be0bac`)
- [x] Fix remove screen deck load failure (`81be0bac`)

### Deck View Polish + UX Fixes (2026-04-04)

- [x] WarningAction enum: FixQuantity, ClearCommander, Remove — per-warning action buttons (`e844395e`)
- [x] "fix to N" on copy limit, "clear" on invalid commander, "card removed" toasts (`e844395e`)
- [x] Card count includes commander/partner/background/spell in stats (`e844395e`)
- [x] Rename Optdate → Opdate across codebase (`f2f7981d`)
- [x] Command zone show toggle on deck card view (`f1f73ac2`)
- [x] Clear commander on format change to prevent stale selections (`8e9eb432`)
- [x] Architecture structure doc: `context/architecture/structure.md`

### Maybeboard (2026-04-04)

- [x] Add `maybeboard: bool` to deck card pipeline — migration, model, metrics/validation exclusion, card_count filter (`224ad721`)
- [x] Swipe-up to maybeboard on add and remove screens with undo support, card-exit-up animation (`0aed2c1b`)
- [x] Deck card view: maybeboard toggle, section rendering, "to deck"/"to maybeboard" move buttons via update_deck_card (`9e8548b3`)
- [x] Remove screen: tri-state maybeboard filter (no/yes/any) in config section (`afbb1c12`)
- [x] Export toggle "include maybeboard" with `// Maybeboard` header, import detects header, buy links exclude maybeboard with toggle (`689f9b22`)

### Rename zweb → zite (2026-04-04)

- [x] Rename web client crate from zweb to zite — directory, Cargo.toml, workflow, all docs (`4c7d1126`)

### Commander Filter System (2026-04-04)

- [x] Add `is_commander_in_format` filter to CardFilter with per-format eligibility rules (`2db7751a`)
- [x] Commander eligibility chips in format filter section (`48e5087e`)
- [x] Commander filter toggle on create/edit screens with format-first layout (`a6fe3a50`)
- [x] Multi-select on format legality filter chips (`f37177ed`)
- [x] Update avg price label (`abed4658`)

### Proxy Re-export Cleanup (2026-04-04)

- [x] Remove logo and moderation proxy modules from zerver (`87eb024b`)
- [x] Remove HTTP paths and helpers proxy files from zerver (`5a894a0a`)
- [x] Clean up auth domain proxy re-exports in zerver (`2455edfc`)
- [x] Migrate zwiper Session imports from zerver proxy to zwipe-core (`b3fb4b77`)
- [x] Clean up deck domain proxy re-exports in zerver and zwiper (`5cff3b21`)
- [x] Migrate zwiper card imports from zerver proxy to zwipe-core (`bd20e9e9`)
- [x] Clean up user domain proxy re-exports (`06f6480b`)
- [x] Clean up card domain proxy re-exports in zerver and zwiper (`dd62c153`)
- [x] Downgrade handler pub use to use, migrate zwiper Http imports to zwipe-core (`b84f1453`)
- [x] Add zwipe-core as direct dependency of zwiper — frontend no longer routes domain types through zerver

### zwipe-core Domain Extraction (2026-04-02)

- [x] Extract newtypes + moderation into zwipe-core (`17525966`)
- [x] Extract User, UserPreferences, GetUser (`db7dec53`)
- [x] Extract deck + deck_card domain types (`5962d409`)
- [x] Document SQLx adapter pattern decision (`b340d7d7`)
- [x] Replace custom SQLx impls with DatabaseScryfallData adapter (`5452554a`)
- [x] Extract Card, CardProfile, ScryfallData + all nested types (`f1a473a4`)
- [x] Extract CardFilter, search types (`6320d30b`)
- [x] Extract Deck/DeckEntry aggregate, validate_deck, DeckMetrics (`ff816b27`)
- [x] Add models/ directories to zwipe-core modules (`2dd7ffa6`)
- [x] Separate requests/ from models/ in zerver auth and card (`d3073314`)
- [x] Extract Session, AccessToken, RefreshToken, Jwt (`26304041`)
- [x] Extract logo module to zwipe-core (`92cd2fc6`)
- [x] Extract HTTP contract types, paths, ApiError, Opdate (`df07bdef`)

### Component Extraction & Deck Enhancements (2026-04-02)

- [x] Extract deck view into profile, stats, and charts components (`c107ea10`)
- [x] Add deck price stats with currency selection chips — USD/EUR/TIX (`36e4083c`)
- [x] Add buy deck links for TCGplayer and CardKingdom (`166837ec`)
- [x] Extract shared CardFilterSheet component from add, view, and remove screens (`f1b0c5f4`)
- [x] Extract CardInfoDisplay, CardSkeleton, DeckFormFields components (`e9fda4a0`)
- [x] Move extracted components into components/ directories (`de3d2cc1`)
- [x] Unify SwipeAction across add and remove screens, move to components (`ab0659d5`)
- [x] Add deck tokens endpoint and display on deck cards screen (`1f429682`)

### zite & README Updates (2026-04-01)

- [x] Split download page into separate iOS and Android store pages (`c7210d52`)
- [x] Add zwipe-core to README tech stack and architecture (`0eaf3a2d`)

### Shared Password Validation Crate (2026-04-01)

- [x] Extract password validation + common password dictionary into `zwipe-core` crate
- [x] Wire into zerver and zite, delete duplicated code

### Community & Web (2026-03-30)

- [x] Discord server setup (Zwipers) with channel structure
- [x] Discord invite link added to zite nav (`b9e108aa`)
- [x] GitHub webhook integration for #change-log

### Card Info Text Clipping Fix (2026-03-30)

- [x] Fix card-info text clipping on smaller screens by using flex layout (`073ccff8`)

### Zervice Rewrite (2026-03-29)

- [x] Rewrite zervice as run-once binary, flatten bin layout, add file logging (`4a42a86a`)
- [x] Add server version to health check responses (`6c73269d`)
- [x] Add password rotation guide to server ops docs (`8b5cb7cd`)

### App Store Submission Attempts (2026-03-27–28)

- [x] Add missing Info.plist keys to Dioxus.toml (`b16732ea`)
- [x] Document App Store submission errors and post-build patching (`81c27daf`)
- [x] Document beta Xcode rejection and Apple Support ticket (`77e50c18`)
- [x] Update app icon with centered Z design (`f9546de9`)
- [x] Add PWA icons and apple-touch-icon to zite (`a0b5aeb0`)

### User Preferences & Themes (2026-03-28)

- [x] Domain layer, migration, repository (`9bdc3b5b`)
- [x] HTTP handlers, routes, domain tests (`9f6c8ff5`)
- [x] Preferences embedded in JWT claims and session response (`695b5fb8`)
- [x] Theme system with 9 themes, preferences screen, UI polish (`854b8efa`)
- [x] Set name on swipe screens, alphabetized theme list (`8b0184f9`)
- [x] is_commander filter, rename from is_valid_commander (`f9d9590f`)

### Card Filtering & Search

- [x] Produced mana card filter with frontend chip UI (`a7c80f16`)
- [x] Commander search burst size increase (`a60a0f0f`)
- [x] Commander search debounce fix + loading spinner (`3fde409f`)
- [x] Fix commander search UI: consistent no-results, dropdown animation (`94930cf2`)

### UX Polish

- [x] Entrance transitions on all screens (`37cdcf94`, `07a12d84`)
- [x] Inline submission errors → toasts (`37cdcf94`, `2347121d`)
- [x] Card image preview modal on deck card list (`0bca21ee`)
- [x] Clear filter: clears card stack + inline clear button (`465fe6cf`, `514f1d57`)
- [x] Toast word-wrap: prefer word boundaries (`af879f4f`)
- [x] Unverified email toast on login + soft limits (`7e754eec`)
- [x] Show hello and verify toasts on home screen for all flows (`c576b93a`)
- [x] Full screen integration pass

### zite (early)

- [x] Design alignment — entrance animations, CSS tokens, spinner (`8ef981f8`)
- [x] Nav: ASCII z logo, sticky on scroll, animation on click (`a094ab9b`, `aa5d2712`, `a7cbc7bd`)
- [x] Download page for app store pending status (`9e2ff793`), split into separate iOS/Android pages (`c7210d52`)
- [x] Favicon (`c797d8e5`)

### Infrastructure

- [x] Database backups: nightly pg_dump → Cloudflare R2, 30-day retention (`5933f2f4`, `cdb7c62d`)
- [x] Automatic migrations in deploy pipeline (`f491d196`)
- [x] Binary versioning: startup logs + workspace version (`30fbc128`)
- [x] CI/CD both pipelines live
