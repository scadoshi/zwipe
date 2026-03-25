# Next Immediate Priorities 🎯

Planned work after completing current tasks.

---

## Testing & Stability

1. **Integration Tests** - Repository tests with real PostgreSQL (longer-term — requires real DB infrastructure)
   - Unit testing phase complete: filter_cards (24 tests), group_cards (15 tests), copy_max (9 tests), quantity, SwipeState (32 tests)
   - Remaining gap: outbound SQLx repositories have no test coverage (integration tests only viable path)

2. **Bug Fixes** - ~~Layout shift after deck creation~~, ~~iOS keyboard push issues~~ (fixed via unified `.screen` layout — see Bugs section for details)
    - ~~Quantity is not built to affect deck profile view screen dashboard metrics and it should~~ — **FIXED** (2026-03-24). `DeckMetrics::from_entries(&[DeckEntry])` replaces `ComputeMetrics` trait; each card counted by its quantity. ViewDeck fetches `Vec<DeckEntry>` instead of discarding quantities.

---

---

## Enhancements

### Deck Composition & Card Management

1. ~~**Deck Import (Text List)**~~ — **DONE** (2026-03-24). Parses Moxfield (`qty name`) and Archidekt (`qtyx name (set) collector# [tags]`) formats. Exact-name batch SQL resolution via CTE dedup. Copy-max clamping (basic lands exempt). Atomic bulk upsert with `ON CONFLICT DO UPDATE`. Import screen with results display (imported + unresolved). Export button on ViewDeck copies deck to clipboard. `ScryfallData::is_basic_land()` helper used across call sites.

2. ~~**Deck Export Screen**~~ — **DONE** (2026-03-24). Dedicated `ExportDeck` screen with readonly textarea + "copy" button with toast feedback. Replaces inline clipboard-copy on ViewDeck.

3. ~~**"Show Lands" Toggle on ViewDeckCard**~~ — **DONE** (2026-03-24). Toggle chip in group-by chip row (right-aligned). Filters lands from displayed groups reactively. Uses `ScryfallData::is_land()`. `ScryfallData::is_spell()` may come later.

2. **Multi-Copy Add Flow** - Right now swiping right always adds exactly 1 copy. For standard decks (CopyMax=4) the user should be able to add up to 4 copies in one action.

   **Scope:**
   - UI: After swipe-right, show a quantity picker (1–CopyMax) before confirming the add, or allow repeat swipe to increment
   - Backend: `AddDeckCard` / `CreateDeckCard` already stores a `Quantity` — the quantity just needs to be driven by user input rather than hardcoded to 1
   - Validation (see CopyMax Enforcement below)

2. ~~**CopyMax Enforcement (Frontend + Backend)**~~ — **DONE** (2026-03-24). Backend: `UpdateDeckCard` guard query enforces copy_max before applying delta. `UpdateDeckProfile` truncates existing card quantities when copy_max becomes more restrictive (single UPDATE in same transaction). Frontend: ViewDeckCard +/- controls respect copy_max with toast feedback. EditDeck shows truncation warning dialog only when actual card quantities exceed the new limit.

3. ~~**Change Quantity in View Deck Screen**~~ — **DONE** (2026-03-24). Inline +/- quantity controls in ViewDeckCard expanded rows. Wired to `UpdateDeckCard` with optimistic updates and rollback on error. Singleton decks show only − (which deletes). Qty column in compact rows, omitted for singleton decks.

4. ~~**Deck Metrics View**~~ — **DONE** (2026-03-23). `DeckMetrics` in deck domain, `ComputeMetrics` trait generic over `IntoIterator<Item = &Card>`. Stats (cards, avg cmc, lands), ASCII mana curve, type/color distributions rendered on ViewDeck screen.

5. **Mana Pip Balance** - Show pips produced vs. pips consumed per color so players can balance their mana base.

   **Concept:** For each color (WUBRG + colorless), compute:
   - **Pips consumed** — count colored mana symbols in `mana_cost` across all nonland cards (e.g. `{W}{W}{U}` → 2 white, 1 blue). Parse from the `mana_cost` string field.
   - **Pips produced** — count land cards whose `produced_mana` field (already present on `ScryfallData`) contains each color.

   Display as a per-color row: `W  ████░░  12 consumed / 8 produced`. Imbalance at a glance — if you're consuming 14 blue pips but only producing 6, you need more blue sources.

   **Implementation notes:**
   - Parse `mana_cost` string (e.g. `"{2}{U}{U}"`) by scanning for `{W}`, `{U}`, `{B}`, `{R}`, `{G}` — simple character-level scan, no regex needed
   - `produced_mana: Option<Colors>` is already on `ScryfallData` — lands that tap for multiple colors (e.g. duals) contribute to all matching buckets
   - Extend `DeckMetrics` with `pip_consumed: [usize; 5]` and `pip_produced: [usize; 5]` (WUBRG indexed) and add to the single-pass computation in `deck_metrics.rs`
   - Render in ViewDeck below colors section

6. **Deck Profile Enhancements (ViewDeck screen)** - Additional deck metadata fields for future.

   **Planned fields:**
   - **Card count** — display total cards in deck on ViewDeck screen
   - **Deck description** — free-text field the user can set (new DB column on `deck_profiles`)
   - **Tags** — user-defined labels for organizing/categorizing decks (e.g., "aggro", "control", "jank")

   **Card count approach:** Don't fetch all cards just to count them. Add a lightweight endpoint (or include count in the `DeckProfile` response) backed by `SELECT COUNT(*)`. ViewDeck already fetches the profile — piggyback the count there.

   **Note:** Commander image was removed from ViewDeck to keep the screen simple and leave room for these future stats. ViewDeck now mirrors the Profile screen layout (label/value rows in `container-sm`).

### Card Filter: Oracle Keywords

4. **Oracle Text Keyword Filter** — Add `oracle_text_contains_any: Option<Vec<String>>` to `CardFilter`, enabling multi-keyword oracle text search. Follows the exact `type_line_contains_any` + `get_card_types` pipeline pattern.

   **Why:** Currently `oracle_text_contains` is a single string. Users want to search cards matching ANY of several keywords (e.g. "destroy" OR "exile" OR "sacrifice") to find removal spells, or "draw" OR "scry" for card advantage. The keyword list is fetched from the database so the frontend can offer autocomplete.

   **Backend — New endpoint: `get_oracle_keywords`**

   Follows `get_card_types` pattern exactly:
   - **Domain model:** New file `zerver/src/lib/domain/card/models/get_oracle_keywords.rs` — `GetOracleKeywordsError` enum (single `Database` variant)
   - **Port trait:** Add `get_oracle_keywords(&self) -> Result<Vec<String>, GetOracleKeywordsError>` to `CardRepository` and `CardService`
   - **Repository SQL** (`outbound/sqlx/card/mod.rs`):
     ```sql
     SELECT DISTINCT LOWER(TRIM(keyword)) AS keyword
     FROM scryfall_data, UNNEST(keywords) AS keyword
     WHERE keywords IS NOT NULL
     ORDER BY keyword ASC
     ```
     Uses the existing `keywords: Option<Vec<String>>` field on `ScryfallData` — these are Scryfall's curated keyword abilities (flying, trample, deathtouch, ward, etc.), not raw oracle text words.
   - **Service:** Passthrough to repo (same as `get_card_types`)
   - **HTTP handler:** New file `handlers/card/get_oracle_keywords.rs` — mirrors `get_card_types.rs` exactly
   - **Route:** `.route("/keywords", get(get_oracle_keywords))` in the `/api/card` nest
   - **Path helper:** `get_oracle_keywords_route() -> String` returning `"api/card/keywords"`

   **Backend — CardFilter extension**

   - **CardFilter field:** `oracle_text_contains_any: Option<Vec<String>>`
   - **Builder setter:** `set_oracle_text_contains_any<I, S>(&mut self, ...)` + `unset_oracle_text_contains_any()`
   - **Builder getter:** `oracle_text_contains_any(&self) -> Option<&[String]>`
   - **CardFilter getter:** Same signature
   - **SQL generation** (`outbound/sqlx/card/mod.rs` search query builder): Loop with `oracle_text ILIKE '%keyword%'` bindings joined by `OR` in parentheses — identical pattern to `type_line_contains_any`
   - **FilterCards** (`filter_cards.rs`): Add client-side matching for in-memory filtering on ViewDeckCard — check if `oracle_text` contains any of the filter strings (case-insensitive)

   **Frontend — Client**

   - New file `zwiper/src/lib/outbound/client/card/get_oracle_keywords.rs`
   - `ClientGetOracleKeywords` trait + impl — GET `/api/card/keywords`, returns `Vec<String>`
   - Register in `client/card/mod.rs`

   **Frontend — Filter component**

   - New file `zwiper/src/lib/inbound/screens/deck/card/filter/keywords.rs`
   - Mirrors `types.rs` "other types" section: resource fetches all keywords, search input with autocomplete, chip-based multi-select with remove buttons
   - Uses `oracle_text_contains_any` getter/setter on `CardFilterBuilder`
   - Register in `filter/mod.rs`
   - Add as new `AccordionItem` in `card/view.rs` filter bottom sheet (between "text" and "types", or after "types")

   **Implementation order:**
   1. Domain model (`get_oracle_keywords.rs`) + port traits
   2. Repository SQL + service passthrough
   3. HTTP handler + route + path
   4. CardFilter field + builder setter/getter + SQL generation + FilterCards
   5. Frontend client
   6. Frontend filter component + accordion registration

   **Critical files:**

   | File | Action |
   |------|--------|
   | `zerver/src/lib/domain/card/models/get_oracle_keywords.rs` | **NEW** |
   | `zerver/src/lib/domain/card/models/mod.rs` | Register module |
   | `zerver/src/lib/domain/card/ports.rs` | Add to both traits |
   | `zerver/src/lib/domain/card/services.rs` | Passthrough |
   | `zerver/src/lib/outbound/sqlx/card/mod.rs` | SQL impl + search filter |
   | `zerver/src/lib/inbound/http/handlers/card/get_oracle_keywords.rs` | **NEW** |
   | `zerver/src/lib/inbound/http/handlers/card/mod.rs` | Register module |
   | `zerver/src/lib/inbound/http/routes.rs` | Add route |
   | `zerver/src/lib/inbound/http/paths.rs` | Add path helper |
   | `zerver/src/lib/domain/card/models/search_card/card_filter/mod.rs` | Add field |
   | `zerver/src/lib/domain/card/models/search_card/card_filter/builder/setters.rs` | Setter + unsetter |
   | `zerver/src/lib/domain/card/models/search_card/card_filter/builder/getters.rs` | Getter |
   | `zerver/src/lib/domain/card/models/search_card/card_filter/builder/mod.rs` | Quick constructor |
   | `zerver/src/lib/domain/card/models/search_card/card_filter/getters.rs` | Getter |
   | `zerver/src/lib/domain/card/models/search_card/filter_cards.rs` | Client-side match |
   | `zwiper/src/lib/outbound/client/card/get_oracle_keywords.rs` | **NEW** |
   | `zwiper/src/lib/outbound/client/card/mod.rs` | Register module |
   | `zwiper/src/lib/inbound/screens/deck/card/filter/keywords.rs` | **NEW** |
   | `zwiper/src/lib/inbound/screens/deck/card/filter/mod.rs` | Register module |
   | `zwiper/src/lib/inbound/screens/deck/card/view.rs` | Add AccordionItem |

### Card Intelligence

5. **Card Keyword Categories (Import Pipeline)** - Tag cards during the Scryfall import with meaningful strategic categories that are hard to derive at query time.

   **Proposed categories:** burn, recursion, flyers, tokens, ramp, removal, counterspells, draw, tutors, board wipes, lifegain

   **Scope:**
   - Add a `keyword_categories: Vec<String>` (or JSONB) column to `card_profiles` (or a separate `card_categories` table)
   - During `zervice` upsert: scan `oracle_text` and `keywords` fields with pattern matching rules per category (e.g., "deals damage" → burn, "flying" keyword → flyers, "create a token" → tokens)
   - Expose as a filter option in `CardFilter` and add a frontend chip multi-select
   - Categories are only as good as the rules — iterate over time

   **Why import-time vs query-time:** Oracle text search is fuzzy; category tagging at import gives indexed, consistent labels the frontend can filter on without complex text queries.

6. **EDHREC Integration** - Synergy scores for the current commander's deck pool.

   **Scope:**
   - Hit EDHREC API (external, not via zervice) with the commander name/id to fetch top synergy cards + scores
   - Store nothing — fetch on demand when user opens AddDeckCard for a commander deck
   - Expose a "EDHREC synergy" sort option in the filter that reorders the card list by synergy score
   - Cards not in EDHREC data sort to the bottom
   - **Note:** Cannot use zervice for this — EDHREC is a read-only external API hit from the frontend client

### UI & Polish

7. **Deck List Screen Redesign** - Better list styling, improved layout with utility bar, visual hierarchy

8. **CardFilter Enhancements (Serve Only Playable Cards)** - Continue refining default CardFilter to exclude non-playable/non-standard cards.

### Pending Improvements
   - ~~**tri-toggle labels**~~ — **DONE**. Labels now read "show / hide / any".
   - ~~**Language filter refinement**~~ — **DONE**. Language chip UI removed from config filter (OracleCards is always used; backend language support preserved for future).

### Set Type Filter (Phase 2)
   - **set_type filter** - Filter by set classification
     - Domain: `SetType` enum or string filter
     - Default: hide `funny`, `memorabilia`, `token` set types
     - Not exposed on frontend initially

### Legality Filter (Phase 3 - Complex)
   - **legality/format filter** - Filter by format legality
     - Uses existing `Legality` and `LegalityKind` enums
     - Requires special UI handling (format + legal status)
     - Deferred - needs design work

9. **Cross-Deck Card Ownership Indicator** - Highlight cards already in other decks when browsing.
    - **Note:** Cards already in the current deck won't be served by the add-card query (filtered server-side), so this is only relevant for cards that appear in *other* decks. Lower priority.

10. **EDHREC Integration** - See Card Intelligence section above.

---

## Infrastructure

1. **Dockerized Backend Dev Environment** - Container for zerver + zervice + postgres so devs don't need to install anything locally

   **Goal:** `docker compose up` spins up the full backend stack ready to develop against

   **Scope:**
   - `Dockerfile.dev` for zerver/zervice (Arch Linux base, Rust toolchain, sqlx-cli)
   - `docker-compose.yml` wiring the app container + postgres container together
   - Postgres container replaces local postgres install (handle migrations on startup)
   - Mount source code as a volume so changes reflect without rebuilding the image
   - `.env` generation handled by compose (no manual setup)
   - Frontend (zwiper) stays native — connects to container via `BACKEND_URL=http://127.0.0.1:3000` unchanged

   **Out of scope:** zwiper/Dioxus (requires GUI libs, stays on host)

   **Companion script:** `zcripts/denv/{platform}/setup-frontend.sh` — frontend-only setup for devs using the Docker backend. Installs Dioxus GUI deps (webkit2gtk, gtk3, etc.), dioxus-cli, and writes `zwiper/.env` pointing at the container. Skips postgres, sqlx-cli, and all backend setup entirely.

---

## Bugs

1. ~~**Layout Shift After Deck Creation**~~ — **FIXED** (2026-03-23)

   **Root cause:** 14 screens used 5 different layout patterns (`position: sticky` on header/footer + `height: 100vh` content divs). This created layouts taller than the viewport, and scroll/positioning state leaked across route changes via Dioxus DOM patching.

   **Fix:** Unified all screens under a single `.screen` fixed-frame layout (`position: fixed; inset: 0` + flexbox). Header and footer are `flex-shrink: 0` items, content fills remaining space with `flex: 1; overflow-y: auto`. No body scroll, no sticky positioning, no per-screen inline layout styles.

2. ~~**iOS Keyboard Pushes Content Down**~~ — **FIXED** (2026-03-23)

   **Root cause:** Same as above — `sticky top-0` + `justify-content: center` + `h-screen` caused layout reflow when iOS keyboard changed the viewport height.

   **Fix:** `position: fixed` on `.screen` is immune to viewport resize from keyboard. Safe-area insets moved from `body` to `.screen` via `env(safe-area-inset-top/bottom)`.
