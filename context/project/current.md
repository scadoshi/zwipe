# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: 2026-03-24. Oracle keyword filter, then pip counter, then hosting.

**Current Focus**: Oracle text keyword filter (full-stack pipeline).

**Recent Achievements**:
- **Deck Import/Export**: Full import pipeline (Moxfield + Archidekt format parsing, batch exact-name SQL resolution, copy-max clamping, atomic bulk upsert). Dedicated export screen with clipboard copy + toast.
- **Commander Guard**: `CreateDeckCard` rejects adding the deck's commander. Import silently skips it.
- **Show Lands Toggle**: Chip toggle on ViewDeckCard filters lands from card groups reactively. `ScryfallData::is_land()` + `is_basic_land()` helpers.
- **Clippy Clean**: All clippy warnings resolved (test indexing/unwrap allows, tri-toggle function pointer suppression at module level).

---

## Remaining Before Hosting

1. **Oracle Text Keyword Filter** (in progress) — `oracle_text_contains_any` on CardFilter, `get_oracle_keywords` endpoint, frontend chip-based multi-select. Full plan in `next.md`.

2. **Mana Pip Counter** — Pips consumed (from `mana_cost`) vs produced (from `produced_mana`) per WUBRG color. Single-pass extension to `DeckMetrics`. Rendered on ViewDeck below colors section. Already specced in `next.md`.

3. **Hosting** — Deploy zerver + postgres. Frontend stays as local/desktop app for now.

---

## Post-Hosting Backlog

- **AI Card Categorization** — Batch-classify 35k cards via Claude API during `zervice` sync. Store category tags (burn, ramp, removal, draw, etc.) as jsonb on `card_profiles`. Expose as filter + grouper on frontend. Rule-based approach breaks down due to oracle text variance — AI is the right tool.
- **Multi-Copy Add Flow** — Quantity picker on swipe-right for standard decks.
- **EDHREC Integration** — External API, complex, deferred.
